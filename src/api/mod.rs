mod pet;
mod store;
mod user;

use finchers::endpoint::prelude::*;

pub fn api() -> impl Endpoint<Item = String> + 'static {
    endpoint("v2").with(choice![
        self::pet::api(),
        self::store::api(),
        self::user::api(),
    ])
}
