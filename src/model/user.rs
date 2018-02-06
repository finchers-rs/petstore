define_id! {
    UserId: u64,
    Username: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: Option<UserId>,
    username: Option<Username>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    phone: Option<String>,
    user_status: Option<u32>,
}
