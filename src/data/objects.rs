use crate::data::mysql_cdc_converter::convert;
use crate::data::objects_traits::{MysqlObject, StorableRaw};
use crate::data::slave::FieldMapping;
use mysql::prelude::FromRow;

use std::collections::HashMap;

pub type IdType = i32;

#[derive(Debug, Default, Clone, FromRow, MysqlObject, StorableRaw)]
pub struct Campaign {
    pub id: IdType,
    pub name: String,
    pub package_id: IdType,
}

impl Campaign {
    pub fn html_debug(&self) -> String {
        format!(
            r#"id={}</br>name={}</br>package_id=<a href="/admin/store/package/{}">{}</a>"#,
            self.id, self.name, self.package_id, self.package_id
        )
    }
}

#[derive(Debug, Default, Clone, FromRow, MysqlObject, StorableRaw)]
pub struct Package {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, Default, Clone, FromRow, MysqlObject, StorableRaw)]
pub struct Pad {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, Default, Clone, FromRow, MysqlObject, StorableRaw)]
pub struct PadRelation {
    pub id: IdType,
    pub pad_id: IdType,
    pub parent_pad_id: IdType,
}

#[derive(Debug, Default, Clone, FromRow, MysqlObject, StorableRaw)]
pub struct TargetingPad {
    pub id: IdType,
    pub object_id: IdType,
    pub object_type: String,
    pub positive: bool,
}
