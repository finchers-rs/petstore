use finchers::endpoint::prelude::*;
use finchers_json::json_body;

use model::{Order, OrderId};

pub fn api() -> impl Endpoint<Item = String> + 'static {
    let order_id = path::<OrderId>();
    let order_body = json_body::<Order>();

    endpoint("store").with(choice![
        get("inventory").map(|_| format!("")),
        endpoint("order").with(choice![
            post(order_body).map(|order| format!("{:?}", order)),
            get(order_id).map(|id| format!("{:?}", id)),
            delete(order_id).map(|id| format!("{:?}", id)),
        ]),
    ])
}
