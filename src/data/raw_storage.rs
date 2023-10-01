use crate::data::objects::IdType;
use crate::data::objects_traits::{MysqlObject, StorableRaw};
use std::any::Any;
use std::collections::HashMap;

#[derive(Default)]
pub struct Storage {
    data: HashMap<&'static str, HashMap<IdType, Box<dyn Any + Sync + Send>>>,
}

impl Storage {
    pub fn update<T: MysqlObject + StorableRaw + Sync + Send + 'static>(&mut self, obj: T) {
        self.data
            .entry(T::table())
            .or_insert(HashMap::new())
            .insert(obj.get_id(), Box::new(obj));
    }
    pub fn delete<T: MysqlObject + StorableRaw + Sync + Send + 'static>(&mut self, obj: T) {
        self.data
            .entry(T::table())
            .or_insert(HashMap::new())
            .remove(&obj.get_id());
    }

    pub fn get<T: StorableRaw + MysqlObject + 'static>(&self, id: IdType) -> &T {
        self.data
            .get(T::table())
            .unwrap()
            .get(&id)
            .unwrap()
            .downcast_ref::<T>()
            .unwrap()
    }

    pub fn try_get<T: StorableRaw + MysqlObject + 'static>(&self, id: IdType) -> Option<&T> {
        match self.data.get(T::table()) {
            Some(objects) => match objects.get(&id) {
                Some(obj) => obj.downcast_ref::<T>(),
                None => None,
            },
            None => None,
        }
    }
}
