use std::collections::HashMap;

use crate::objects::{Campaign, IdType, Package, Pad};
use crate::store::aci::ActiveCampaignIndex;

#[derive(Default)]
struct RawDataStorage {
    campaigns: HashMap<IdType, Campaign>,
    package: HashMap<IdType, Package>,
    pads: HashMap<IdType, Pad>,
}

pub trait Storable<T> {
    fn add(&mut self, obj: &T);
}

#[derive(Default)]
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
    pub fn get<T>(&self, id: &IdType) -> Option<&Campaign> {
        return self.campaigns.get(id);
    }
}

impl Store {}
