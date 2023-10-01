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
use std::mem;
use std::mem::swap;
use std::ops::{Add, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread::{sleep, JoinHandle};
use std::{thread, time};

pub type UpdaterPtr = Arc<RwLock<Updater>>;
pub struct Updater {
    pub conf: config::Updater,
    pub stop_flag: Arc<AtomicBool>,
    #[allow(dead_code)]
    engine: Arc<RwLock<engine::Engine>>,
    #[allow(dead_code)]
    read_store: Arc<RwLock<Store>>,
    write_store: Arc<RwLock<Store>>,
    slave: Option<JoinHandle<()>>,
    cron: Option<JoinHandle<()>>,
    index_iteration: u64,
}

#[derive(Debug)]
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

        let updater_ptr = Arc::new(RwLock::new(Updater {
            conf: conf.clone(),
            stop_flag: stop_flag.clone(),
            engine,
            read_store: Arc::new(RwLock::new(Store::default())),
            write_store: Arc::new(RwLock::new(Store::default())),
            slave: None,
            cron: None,
            index_iteration: 0,
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

    #[stime("info")]
    fn update_stores(&mut self) {
        let mut write_store = self.write_store.write().unwrap();
        write_store.rebuild_index(self.index_iteration.add(1));
        // slave is still blocked for write access because workers still read from old store
        swap(&mut self.write_store, &mut self.read_store);
        self.engine
            .write()
            .unwrap()
            .set_new_store(&self.write_store);
    }
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
            let mut locked = updater.write().unwrap();
            locked.update_stores();
            last_swap_ts = loop_start_ts;
        }
        sleep(time::Duration::from_secs(1));
    }
}

pub fn apply_to_store<T: Storable + Debug>(
    updater: &UpdaterPtr,
    obj: T,
    old_obj: Option<T>,
    ev_type: EventType,
) {
    let store_ptr = updater.write().unwrap().write_store.clone();
    let mut store_locked = store_ptr.write().unwrap();
    log::debug!(
        "apply_to_store: action={:?}, old={:?}, old_obj={:?}",
        ev_type,
        obj,
        old_obj
    );
    match ev_type {
        EventType::INSERT => {
            obj.insert(store_locked.deref_mut());
        }
        EventType::UPDATE => {
            obj.update(store_locked.deref_mut(), old_obj);
        }
        EventType::DELETE => {
            obj.delete(store_locked.deref_mut());
        }
    }
    // store.add(&obj);
    // self.raw_data.get<>(&13);
    // obj.apply_to_store(self);
}
