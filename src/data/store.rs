use std::collections::HashMap;

use crate::data::aci::ActiveCampaignIndex;
use crate::data::objects::Storable;
use crate::data::objects::{Campaign, IdType, Package, Pad};

#[derive(Default, Clone)]
struct RawDataStorage {
    campaigns: HashMap<IdType, Campaign>,
    packages: HashMap<IdType, Package>,
    pads: HashMap<IdType, Pad>,
}

#[derive(Default, Clone)]
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
        log::debug!("insert campaign: {:?}", self);
        store.raw_data.campaigns.insert(self.id, self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        log::debug!("update campaign: {:?}", self);
        store.raw_data.campaigns.insert(self.id, self);
    }
    fn delete(self, store: &mut Store) {
        log::debug!("delete campaign: {:?}", self);
        store.raw_data.campaigns.insert(self.id, self);
    }
}

impl Storable for Package {
    fn insert(self, store: &mut Store) {
        log::debug!("insert package: {:?}", self);
        store.raw_data.packages.insert(self.id, self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        log::debug!("update package: {:?}", self);
        store.raw_data.packages.insert(self.id, self);
    }
    fn delete(self, store: &mut Store) {
        log::debug!("delete package: {:?}", self);
        store.raw_data.packages.insert(self.id, self);
    }
}

impl Storable for Pad {
    fn insert(self, store: &mut Store) {
        log::debug!("insert pad: {:?}", self);
        store.raw_data.pads.insert(self.id, self);
    }
    fn update(self, store: &mut Store, _old: Option<Self>) {
        log::debug!("update pad: {:?}", self);
        store.raw_data.pads.insert(self.id, self);
    }
    fn delete(self, store: &mut Store) {
        log::debug!("delete pad: {:?}", self);
        store.raw_data.pads.insert(self.id, self);
    }
}

impl RawDataStorage {
    #![allow(dead_code)]
    pub fn get(&self, id: &IdType) -> Option<&Campaign> {
        return self.campaigns.get(id);
    }
}
