use crate::data::store;
use mysql::prelude::FromRow;
pub type IdType = i32;

pub trait MysqlObject {
    fn table<'life>() -> &'life str;
    fn from_slave() -> Self;
}

pub trait Storable {
    fn insert(self, store: &mut store::Store);
    fn update(self, store: &mut store::Store, old: Option<Self>)
    where
        Self: Sized;
    fn delete(self, store: &mut store::Store);
}

#[derive(Debug, Default, MysqlObject, Clone, FromRow)]
pub struct Campaign {
    pub id: IdType,
    pub name: String,
    pub package_id: IdType,
}

#[derive(Debug, MysqlObject, Default, Clone, FromRow)]
pub struct Package {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, MysqlObject, Default, Clone, FromRow)]
pub struct Pad {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, MysqlObject, Default, Clone, FromRow)]
pub struct PadRelation {
    pub id: IdType,
    pub pad_id: IdType,
    pub parent_pad_id: IdType,
}

#[derive(Debug, MysqlObject, Default, Clone, FromRow)]
pub struct TargetingPad {
    pub id: IdType,
    pub object_id: IdType,
    pub object_type: String,
    pub positive: bool,
}
