use crate::store::store::{Storable, Store};
use std::fmt::Debug;

#[derive(Default)]
pub struct DataManager {
    _read_store: Store,
    write_store: Store,
    // objects_pool: Vec<Any>,
}

impl DataManager {
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
