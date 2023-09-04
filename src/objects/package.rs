use crate::objects::CreateFromSlave;

#[derive(Debug, CreateFromSlave)]
pub struct Package {
    pub id: i32,
    pub object_id: i32,
    pub object_type: String
}