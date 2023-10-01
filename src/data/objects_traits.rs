use crate::data::slave::FieldMapping;
use crate::data::store;

use crate::data::objects;
use mysql_cdc::events::row_events::row_data::RowData;
use std::collections::HashMap;

pub trait MysqlObject {
    fn table<'life>() -> &'life str
    where
        Self: Sized;
    fn from_slave(row_data: &RowData, fields_map: &HashMap<String, FieldMapping>) -> Self
    where
        Self: Sized;
}

pub trait StorableRaw {
    fn get_id(&self) -> objects::IdType;
}

pub trait Storable {
    fn insert(self, store: &mut store::Store);
    fn update(self, store: &mut store::Store, old: Option<Self>)
    where
        Self: Sized;
    fn delete(self, store: &mut store::Store);
}
