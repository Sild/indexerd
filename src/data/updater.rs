use crate::config;
use crate::data::objects::Storable;
use crate::data::store::Store;
use crate::data::{select, slave};
use crate::engine;
use crate::helpers;
use logging_timer::stime;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use std::error::Error;
use std::mem::swap;
use std::ops::DerefMut;
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
    pub write_store: Arc<RwLock<Store>>,
    slave: Option<JoinHandle<()>>,
    cron: Option<JoinHandle<()>>,
}

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
        }));

        let last_gtid = select::get_master_gtid(&conf.db)?;
        log::info!("Got last gtid: {:?}", last_gtid);

        select::init(&updater_ptr)?;

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
        {
            let mut store_lock = self.write_store.write().unwrap();
            store_lock.deref_mut().build_aci();
        }
        let mut locked_engine = self.engine.write().unwrap();
        locked_engine.set_new_store(&self.write_store);
        swap(&mut self.read_store, &mut self.write_store);
        let _store_lock = self.write_store.write().unwrap();
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
        // let mut updater = match updater.try_read() {
        //     Ok(locked) => locked,
        //     Err(_) => continue,
        // };
        if (last_swap_ts) < loop_start_ts {
            // updater.as.update_stores();
            last_swap_ts = loop_start_ts;
        } else {
            // log::debug!("skip update_stores becase of interval");
        }
        // match new_conf {
        //     Some(c) => updater
        //         .read()
        //         .unwrap()
        //         .engine
        //         .update_config(c.db.clone()),
        //     _ => {}
        // }
        sleep(time::Duration::from_secs(1));
    }
}

pub fn apply_to_store<T: Storable>(
    updater: &UpdaterPtr,
    obj: T,
    old_obj: Option<T>,
    ev_type: EventType,
) {
    let store_ptr = updater.write().unwrap().write_store.clone();
    let mut store_locked = store_ptr.write().unwrap();
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
