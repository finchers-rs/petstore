use super::{Category, Tag};

define_id!(PetId: u64);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pet {
    id: Option<PetId>,
    category: Option<Category>,
    name: String,
    photo_urls: Vec<String>,
    tags: Option<Vec<Tag>>,
    status: Option<Status>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Available,
    Pending,
    Sold,
}
