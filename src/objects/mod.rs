pub trait MysqlObject {
    fn table<'life>() -> &'life str;
    fn from_select();
    fn from_slave();
}
pub mod campaign;