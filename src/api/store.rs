use finchers::{Endpoint, Handler};
use futures::Future;

use model::{Inventory, Order};
use super::{Error, Petstore};

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    GetInventory,
    AddOrder(Order),
    DeleteOrder(u64),
    FindOrder(u64),
}

#[derive(Debug)]
pub enum Response {
    TheInventory(Inventory),
    TheOrder(Option<Order>),
    OrderCreated(u64),
    OrderDeleted(bool),
}

use self::Request::*;
use self::Response::*;

mod imp {
    use super::*;
    use common::*;

    impl IntoResponse for super::Response {
        fn into_response(self) -> HyperResponse {
            match self {
                TheInventory(inventory) => json_response(&inventory),
                TheOrder(order) => order.map_or_else(no_route, |o| json_response(&o)),
                OrderCreated(id) => json_response(&id).with_status(StatusCode::Created),
                OrderDeleted(deleted) => json_response(&deleted),
            }
        }
    }
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + Clone + 'static {
    use finchers::endpoint::prelude::*;
    use finchers::endpoint::ok;
    use finchers::contrib::json::json_body;

    endpoint("store").with(choice![
        get("inventory").with(ok::<_, Error>(GetInventory)),
        endpoint::<_, _, Error>("order").with(choice![
            post(json_body()).map(AddOrder),
            delete(path()).map(DeleteOrder),
            get(path()).map(FindOrder),
        ]),
    ])
}

impl Handler<Request> for Petstore {
    type Item = Response;
    type Error = Error;
    type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        match request {
            GetInventory => self.db.get_inventory().map(TheInventory).into(),
            AddOrder(order) => self.db.add_order(order).map(OrderCreated).into(),
            DeleteOrder(id) => self.db.delete_order(id).map(OrderDeleted).into(),
            FindOrder(id) => self.db.find_order(id).map(TheOrder).into(),
        }
    }
}
