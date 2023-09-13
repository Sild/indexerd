#[derive(Debug, Default)]
pub struct ActiveCampaignIndex {
    pub id: i32,
    pub name: String,
}

#[derive(Default)]
pub struct IndexParams {
    pub pad_id: i32,
    pub age: i32,
}

#[derive(Default)]
pub struct IndexResult {}

impl ActiveCampaignIndex {
    pub fn _get(_params: &IndexParams) -> IndexResult {
        IndexResult::default()
    }
}
