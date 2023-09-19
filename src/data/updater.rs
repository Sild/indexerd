extern crate mysql;
use crate::config;
use crate::data::slave::run_slave;
use crate::data::store::{Storable, Store};
use crate::engine::Engine;
use crate::helpers;
use mysql::*;
use std::error::Error;
use std::fmt::Debug;
use std::mem::swap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;
use std::sync::{Arc, RwLockWriteGuard};
use std::thread;
use std::thread::JoinHandle;

pub struct Updater {
    conf: config::Updater,
    read_store: Arc<RwLock<Store>>,
    write_store: Arc<RwLock<Store>>,
    engine: Engine,
    slave: JoinHandle<()>,
}

impl Updater {
    pub fn new(
        conf: &config::Updater,
        engine: Engine,
        shutdown: &Arc<AtomicBool>,
    ) -> Result<Self, Box<dyn Error>> {
        // let url = "mysql://root:password@localhost:3307/db_name";
        // let pool = Pool::new(url)?;
        //
        // let mut conn = pool.get_conn()?;
        let conf_th = conf.clone();
        let shutdown_th = shutdown.clone();
        let slave = thread::Builder::new()
            .name(String::from("slave"))
            .spawn(move || {
                if let Err(err) = helpers::bind_thread(1) {
                    log::error!(
                        "Fail to bind {} to core {} with err={:?}",
                        thread::current().name().unwrap_or("noname"),
                        0,
                        err
                    );
                }
                run_slave(&conf_th, shutdown_th);
            })
            .expect("fail to run slave thread");

        let updater = Updater {
            conf: conf.clone(),
            read_store: Arc::new(RwLock::new(Store::default())),
            write_store: Arc::new(RwLock::new(Store::default())),
            engine,
            slave,
        };
        Ok(updater)
    }

    pub fn shutdown(&mut self) {
        self.engine.shutdown();
    }

    fn swap_stores(&mut self) {
        {
            let mut store_lock = self.write_store.write().unwrap();
            store_lock.deref_mut().build_aci();
        }
        self.engine.set_new_store(&self.write_store);
        swap(&mut self.read_store, &mut self.write_store);
        let _store_lock = self.write_store.write().unwrap();
    }
}

#[allow(dead_code)]
pub fn insert_to_store<T: Debug>(store: &mut Store, obj: T)
where
    Store: Storable<T>,
{
    log::trace!("dm call insert for {:?}", obj);
    store.add(&obj);
    // self.raw_data.get<>(&13);
    // obj.apply_to_store(self);
}
