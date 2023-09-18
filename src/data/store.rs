use std::collections::HashMap;

use crate::data::aci::ActiveCampaignIndex;
use crate::data::objects::{Campaign, IdType, Package, Pad};

#[derive(Default, Clone)]
struct RawDataStorage {
    campaigns: HashMap<IdType, Campaign>,
    package: HashMap<IdType, Package>,
    pads: HashMap<IdType, Pad>,
}

pub trait Storable<T> {
    fn add(&mut self, obj: &T);
}

#[derive(Default, Clone)]
pub struct Store {
    raw_data: RawDataStorage,
    _aci: ActiveCampaignIndex,
    pub name: String,
}

impl Storable<Campaign> for Store {
    fn add(&mut self, obj: &Campaign) {
        log::trace!("got new object: {:?}", obj);
        self.raw_data.campaigns.insert(obj.id, obj.clone());
    }
}

impl Storable<Package> for Store {
    fn add(&mut self, obj: &Package) {
        log::trace!("got new object: {:?}", obj);
        self.raw_data.package.insert(obj.id, obj.clone());
    }
}

impl Storable<Pad> for Store {
    fn add(&mut self, obj: &Pad) {
        log::trace!("got new object: {:?}", obj);
        self.raw_data.pads.insert(obj.id, obj.clone());
    }
}

impl RawDataStorage {
    #![allow(dead_code)]
    pub fn get(&self, id: &IdType) -> Option<&Campaign> {
        return self.campaigns.get(id);
    }
}
