use crate::config;
use crate::data::objects_traits::Storable;
use crate::data::store::Store;
use crate::data::{select, slave};
use crate::engine;
use crate::helpers;

use logging_timer::stime;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{Add, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread::{sleep, JoinHandle};
use std::{thread, time};

pub type UpdaterPtr = Arc<RwLock<Updater>>;
pub type SlaveUpdateFunc = Box<dyn FnOnce(&mut Store) + Send + Sync>;

pub struct Updater {
    pub conf: config::Updater,
    pub stop_flag: Arc<AtomicBool>,
    engine: Arc<RwLock<engine::Engine>>,
    read_store: Arc<RwLock<Store>>,
    write_store: Arc<RwLock<Store>>,
    slave: Option<JoinHandle<()>>,
    cron: Option<JoinHandle<()>>,
    index_iteration: u64,
    slave_updates: Vec<SlaveUpdateFunc>,
}

#[derive(Debug, Clone)]
pub enum EventType {
    INSERT,
    #[allow(dead_code)]
    UPDATE,
    #[allow(dead_code)]
    DELETE,
}

impl Updater {
    pub fn new(
        conf: &config::Updater,
        engine: Arc<RwLock<engine::Engine>>,
    ) -> Result<Arc<RwLock<Updater>>, Box<dyn Error>> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let mut store_first = Store::default();
        store_first.id = String::from("first");
        let mut store_second = Store::default();
        store_second.id = String::from("second");
        let updater_ptr = Arc::new(RwLock::new(Updater {
            conf: conf.clone(),
            stop_flag: stop_flag.clone(),
            engine,
            read_store: Arc::new(RwLock::new(store_first)),
            write_store: Arc::new(RwLock::new(store_second)),
            slave: None,
            cron: None,
            index_iteration: 0,
            slave_updates: Vec::new(),
        }));

        let last_gtid = select::get_master_gtid(&conf.db)?;
        log::info!("Got last gtid: {:?}", last_gtid);

        select::init(&updater_ptr, &conf.db)?;

        let slave = run_slave(updater_ptr.clone(), last_gtid);
        let cron = run_cron(updater_ptr.clone());
        if let Ok(mut updater) = updater_ptr.write() {
            updater.slave = Some(slave);
            updater.cron = Some(cron);
        } else {
            return Err("fail to assign threads".into());
        }
        Ok(updater_ptr)
    }

    #[stime("info")]
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        let _ = self.slave.take().unwrap().join();
        let _ = self.cron.take().unwrap().join();
    }
}

#[stime("info")]
fn swap_stores(updater: &Arc<RwLock<Updater>>) {
    let mut updater_w = updater.write().unwrap();
    {
        let mut write_store_w = updater_w.write_store.write().unwrap();
        write_store_w.rebuild_index(updater_w.index_iteration.add(1));
    }

    let tmp = updater_w.write_store.clone();
    updater_w.write_store = updater_w.read_store.clone();
    updater_w.read_store = tmp;
    updater_w
        .engine
        .write()
        .unwrap()
        .set_new_store(&updater_w.read_store);
    //
    let slave_updates = updater_w.slave_updates.drain(..).collect::<Vec<_>>();
    // we can't access write_store until all workers switch to the new one
    {
        let mut write_store_w = updater_w.write_store.write().unwrap();
        log::debug!("apply {} events from slave_updates...", slave_updates.len());
        for update_func in slave_updates.into_iter() {
            update_func(write_store_w.deref_mut());
        }
    }

    log::info!(
        "swap is done. read_store.id={}, write_store.id={}",
        updater_w.read_store.read().unwrap().id,
        updater_w.write_store.read().unwrap().id
    );
}

fn run_slave(updater: Arc<RwLock<Updater>>, gtid: Option<GtidSet>) -> JoinHandle<()> {
    thread::Builder::new()
        .name(String::from("slave"))
        .spawn(move || {
            helpers::bind_thread(1);
            slave::slave_loop(updater, gtid);
        })
        .expect("fail to run slave thread")
}

fn run_cron(updater: Arc<RwLock<Updater>>) -> JoinHandle<()> {
    thread::Builder::new()
        .name(String::from("cron"))
        .spawn(move || {
            helpers::bind_thread(1);
            cron_loop(updater);
        })
        .expect("fail to run cron thread")
}

#[stime("info")]
fn cron_loop(updater: Arc<RwLock<Updater>>) {
    let stop_flag = updater.read().unwrap().stop_flag.clone();
    let mut stop_checker = helpers::StopChecker::new(stop_flag);
    let mut last_swap_ts = 0;

    while !stop_checker.is_time() {
        let loop_start_ts = helpers::time::cur_ts();

        if loop_start_ts > last_swap_ts + 5 {
            swap_stores(&updater);
            last_swap_ts = loop_start_ts;
        }
        sleep(time::Duration::from_secs(1));
    }
}

pub fn apply_to_store<T: Storable + Clone + Debug + Sync + Send + 'static>(
    updater: &UpdaterPtr,
    obj: T,
    old_obj: Option<T>,
    ev_type: EventType,
) {
    log::debug!(
        "apply_to_store: action={:?}, old={:?}, old_obj={:?}",
        ev_type,
        obj,
        old_obj
    );

    let apply_func = move |store: &mut Store| {
        match ev_type.clone() {
            EventType::INSERT => {
                obj.clone().insert(store);
            }
            EventType::UPDATE => {
                obj.clone().update(store, old_obj);
            }
            EventType::DELETE => {
                obj.clone().delete(store);
            }
        };
    };

    let mut updater = updater.write().unwrap();
    updater.slave_updates.push(Box::new(apply_func.clone()));

    let mut store_locked = updater.write_store.write().unwrap();
    apply_func(store_locked.deref_mut());
}
