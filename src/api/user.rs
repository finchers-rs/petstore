use finchers::endpoint::prelude::*;
use finchers_json::json_body;

use model::{User, Username};

pub fn api() -> impl Endpoint<Item = String> + 'static {
    let username = path::<Username>();
    let user_body = json_body::<User>();

    endpoint("user").with(choice![
        post(user_body).map(|user| format!("{:?}", user)),
        post("createWithArray")
            .with(json_body::<Vec<User>>())
            .map(|users| format!("{:?}", users)),
        get(username).map(|username| format!("{:?}", username)),
        put((username, user_body)).map(|(username, user)| format!("{:?}", (username, user))),
        delete(username).map(|username| format!("{:?}", username)),
    ])
}
