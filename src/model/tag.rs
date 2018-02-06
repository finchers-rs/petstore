define_id!(TagId: u64);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    id: Option<TagId>,
    name: Option<String>,
}
