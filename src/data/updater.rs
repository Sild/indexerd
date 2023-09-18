use std::error::Error;
use crate::data::store::{Storable, Store};
use std::fmt::Debug;
use std::sync::RwLock;
use std::sync::Arc;

pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_username: String,
    pub db_password: String,
    pub db_name: String,
    pub swap_interval: u32,
}

#[derive(Default)]
pub struct Updater {
    read_store: Arc<RwLock<Store>>,
    write_store: Arc<RwLock<Store>>,
    // objects_pool: Vec<Any>,
}

impl Updater {
    pub fn new(conf: &Updater) -> Result<Self, Box<dyn Error>> {
        let read_store = init_store();
        let write_store = read_store.clone();
        let updater = Updater{
            read_store: Arc::new(RwLock::new(read_store)),
            write_store: Arc::new(RwLock::new(write_store)),
        };
        Ok(updater)
    }

    #![allow(dead_code)]
    pub fn insert<T: Debug>(&mut self, obj: T)
    where
        Store: Storable<T>,
    {
        log::trace!("dm call insert for {:?}", obj);
        self.write_store.add(&obj);
        // self.raw_data.get<>(&13);
        // obj.apply_to_store(self);
    }

    // fn swapStores(&mut self) {
    //     self.read_store, self.write_store = self.write_store, self.read_store;
    // }
}

fn init_store() -> Store {

}
