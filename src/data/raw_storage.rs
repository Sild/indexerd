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
            .or_default()
            .insert(obj.get_id(), Box::new(obj));
    }
    pub fn delete<T: MysqlObject + StorableRaw + Sync + Send + 'static>(&mut self, obj: T) {
        self.data
            .entry(T::table())
            .or_default()
            .remove(&obj.get_id());
    }

    #[allow(dead_code)]
    pub fn get<T: StorableRaw + MysqlObject + 'static>(&self, id: IdType) -> &T {
        self.data
            .get(T::table())
            .unwrap()
            .get(&id)
            .unwrap()
            .downcast_ref::<T>()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn try_get<T: StorableRaw + MysqlObject + 'static>(&self, id: IdType) -> Option<&T> {
        match self.data.get(T::table()) {
            Some(objects) => match objects.get(&id) {
                Some(obj) => obj.downcast_ref::<T>(),
                None => None,
            },
            None => None,
        }
    }

    pub fn list<T: MysqlObject>(&self) -> Vec<IdType> {
        match self.data.get(T::table()) {
            Some(objects) => Vec::from_iter(objects.iter().map(|(x, _)| *x).collect::<Vec<_>>()),

            None => Vec::default(),
        }
    }
}
