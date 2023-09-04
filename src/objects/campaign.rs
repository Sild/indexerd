use crate::objects::MysqlObject;

#[derive(Debug, MysqlObject)]
pub struct Campaign {
    pub id: i32,
    pub name: String,
    pub package_id: i32,
}

impl Campaign {
    pub fn table<'life>() -> &'life str {
        "campaign"
    }
}
