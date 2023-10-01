use crate::data::aci::ActiveCampaignIndex;
use crate::data::objects::{Campaign, Package, Pad};
use crate::data::objects::{PadRelation, TargetingPad};
use crate::data::objects_traits::Storable;
use crate::data::raw_storage;
use crate::helpers;

#[derive(Default)]
pub struct IndexStat {
    iteration: u64,
    rebuild_start_ts: u64,
    rebuild_end_ts: u64,
    activation_ts: u64, // the time when store became read-only last time
}
#[derive(Default)]
pub struct Store {
    pub id: String, // just to identify it somehow
    raw_data: raw_storage::Storage,
    _aci: ActiveCampaignIndex,
    index_stat: IndexStat,
}

impl Store {
    pub fn rebuild_index(&mut self, iteration: u64) {
        self.index_stat.iteration = iteration;
        self.index_stat.rebuild_start_ts = helpers::time::cur_ts();
        self.index_stat.rebuild_end_ts = helpers::time::cur_ts();
    }
    pub fn get_store_stat(&self) -> &IndexStat {
        &self.index_stat
    }
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
