pub type IdType = i32;
pub trait MysqlObject {
    fn table<'life>() -> &'life str;
    fn from_select() -> Self;
    fn from_slave() -> Self;
}

#[derive(Debug, Default, MysqlObject, Clone)]
pub struct Campaign {
    pub id: IdType,
    pub name: String,
    pub package_id: IdType,
}

#[derive(Debug, MysqlObject, Default, Clone)]
pub struct Package {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, MysqlObject, Default, Clone)]
pub struct Pad {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, MysqlObject, Default)]
pub struct PadRelation {
    pub id: IdType,
    pub pad_id: IdType,
    pub parent_pad_id: IdType,
}

#[derive(Debug, MysqlObject, Default)]
pub struct TargetingPad {
    pub id: IdType,
    pub object_id: IdType,
    pub object_type: String,
    pub positive: bool,
}
