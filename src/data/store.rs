use std::collections::HashMap;

use crate::data::aci::ActiveCampaignIndex;
use crate::data::objects::{Campaign, IdType, Package, Pad};
use crate::data::objects::{PadRelation, TargetingPad};
use crate::data::objects_traits::{MysqlObject, Storable, StorableRaw};

#[derive(Default)]
struct RawDataStorage {
    data: HashMap<&'static str, HashMap<IdType, Box<dyn StorableRaw + Sync + Send>>>,
}

impl RawDataStorage {
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
}

#[derive(Default)]
pub struct Store {
    raw_data: RawDataStorage,
    _aci: ActiveCampaignIndex,
    pub name: String,
}

impl Store {
    #[allow(dead_code)]
    pub fn build_aci(&mut self) {}
}

impl Storable for Campaign {
    fn insert(self, store: &mut Store) {
        store.raw_data.update(self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        store.raw_data.update(self);
    }
    fn delete(self, store: &mut Store) {
        store.raw_data.delete(self);
    }
}

impl Storable for Package {
    fn insert(self, store: &mut Store) {
        store.raw_data.update(self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        store.raw_data.update(self);
    }
    fn delete(self, store: &mut Store) {
        store.raw_data.delete(self);
    }
}

impl Storable for Pad {
    fn insert(self, store: &mut Store) {
        store.raw_data.update(self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        store.raw_data.update(self);
    }
    fn delete(self, store: &mut Store) {
        store.raw_data.delete(self);
    }
}

impl Storable for PadRelation {
    fn insert(self, store: &mut Store) {
        store.raw_data.update(self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        store.raw_data.update(self);
    }
    fn delete(self, store: &mut Store) {
        store.raw_data.delete(self);
    }
}

impl Storable for TargetingPad {
    fn insert(self, store: &mut Store) {
        store.raw_data.update(self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        store.raw_data.update(self);
    }
    fn delete(self, store: &mut Store) {
        store.raw_data.delete(self);
    }
}
