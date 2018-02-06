define_id!(CategoryId: u64);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: Option<CategoryId>,
    name: Option<String>,
}
