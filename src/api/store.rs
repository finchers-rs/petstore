use finchers::contrib::json::Json;
use finchers::endpoint::{body, ok, path, Endpoint};
use finchers::endpoint::method::{delete, get, post};
use finchers::handler::Handler;
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
    use self::Request::*;

    endpoint![
        get("store/inventory").with(ok(GetInventory)),
        post("store/order")
            .with(body().map_err(Error::endpoint))
            .map(|Json(order)| AddOrder(order)),
        delete("store/order")
            .with(path().map_err(Error::endpoint))
            .map(|id| DeleteOrder(id)),
        get("store/order")
            .with(path().map_err(Error::endpoint))
            .map(|id| FindOrder(id)),
    ]
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
