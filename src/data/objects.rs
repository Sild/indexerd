pub type IdType = i32;
pub trait MysqlObject {
    fn table<'life>() -> &'life str;
    fn from_select() -> Self;
    fn from_slave() -> Self;
    fn select_all_req() -> String;
}

#[derive(Debug, Default, MysqlObject, Clone)]
#[diesel(table_name = campaign)]
pub struct Campaign {
    pub id: IdType,
    pub name: String,
    pub package_id: IdType,
}

#[derive(Debug, MysqlObject, Default, Clone)]
#[diesel(table_name = package)]
pub struct Package {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, MysqlObject, Default, Clone)]
#[diesel(table_name = pad)]
pub struct Pad {
    pub id: IdType,
    pub name: String,
}

#[derive(Debug, MysqlObject, Default)]
#[diesel(table_name = pad_relation)]
pub struct PadRelation {
    pub id: IdType,
    pub pad_id: IdType,
    pub parent_pad_id: IdType,
}

#[derive(Debug, MysqlObject, Default)]
#[diesel(table_name = targeting_pad)]
pub struct TargetingPad {
    pub id: IdType,
    pub object_id: IdType,
    pub object_type: String,
    pub positive: bool,
}
