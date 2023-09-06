use std::fmt::Debug;
use crate::store::store::{Storable, Store};
use crate::objects::MysqlObject;

#[derive(Default)]
pub struct DataManager {
    read_store: Store,
    write_store: Store,
    // objects_pool: Vec<Any>,
}

impl DataManager {
    pub fn insert<T: Debug>(& mut self, obj: T)  where Store: Storable<T> {
        println!("dm call insert for {:?}", obj);
        self.write_store.add(&obj);
        // self.raw_data.get<>(&13);
        // obj.apply_to_store(self);
    }

    // fn swapStores(&mut self) {
    //     self.read_store, self.write_store = self.write_store, self.read_store;
    // }
}