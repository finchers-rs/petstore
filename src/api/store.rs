use finchers::{Endpoint, Handler};
use futures::Future;

use model::Order;
use super::{Error, Petstore};

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    GetInventory,
    AddOrder(Order),
    DeleteOrder(u64),
    FindOrder(u64),
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + Clone + 'static {
    use finchers::endpoint::prelude::*;
    use finchers::endpoint::ok;
    use finchers::contrib::json::json_body;
    use self::Request::*;

    endpoint("store").with(choice![
        get("inventory").with(ok::<_, Error>(GetInventory)),
        endpoint::<_, _, Error>("order")
            .with(choice![
                post(json_body()).map(AddOrder),
                delete(path()).map(DeleteOrder),
                get(path()).map(FindOrder),
            ]),
    ])
}

impl Handler<Request> for Petstore {
    type Item = super::PetstoreResponse;
    type Error = Error;
    type Future = super::PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Future {
        use self::Request::*;
        use super::PetstoreResponse::*;
        match request {
            GetInventory => self.db.get_inventory().map(TheInventory).into(),
            AddOrder(order) => self.db.add_order(order).map(OrderCreated).into(),
            DeleteOrder(id) => self.db.delete_order(id).map(OrderDeleted).into(),
            FindOrder(id) => self.db.find_order(id).map(TheOrder).into(),
        }
    }
}
