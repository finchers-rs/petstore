use finchers::{Endpoint, Handler};
use error::EndpointError;
use model::{Inventory, Order};
use petstore::{Petstore, PetstoreError};
use self::Request::*;
use self::Response::*;

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
    TheOrder(Order),
    OrderCreated(u64),
    OrderDeleted(bool),
}

mod imp {
    use super::*;
    use api::common::*;

    impl IntoResponse for Response {
        fn into_response(self) -> HyperResponse {
            match self {
                TheInventory(inventory) => json_response(&inventory),
                TheOrder(order) => json_response(&order),
                OrderCreated(id) => json_response(&id).with_status(StatusCode::Created),
                OrderDeleted(deleted) => json_response(&deleted),
            }
        }
    }
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = EndpointError> + Clone + 'static {
    use finchers::endpoint::prelude::*;
    use finchers::endpoint::ok;
    use finchers_json::json_body;

    endpoint("store").with(choice![
        get("inventory").with(ok(GetInventory)),
        endpoint("order").with(choice![
            post(json_body().from_err()).map(AddOrder),
            delete(path()).map(DeleteOrder),
            get(path()).map(FindOrder),
        ]),
    ])
}

impl Handler<Request> for Petstore {
    type Item = Response;
    type Error = PetstoreError;
    type Result = Result<Option<Self::Item>, Self::Error>;

    fn call(&self, request: Request) -> Self::Result {
        match request {
            GetInventory => self.get_inventory().map(|i| Some(TheInventory(i))),
            AddOrder(order) => self.add_order(order).map(|id| Some(OrderCreated(id))),
            DeleteOrder(id) => self.delete_order(id)
                .map(|deleted| Some(OrderDeleted(deleted))),
            FindOrder(id) => self.find_order(id).map(|o| o.map(TheOrder)),
        }
    }
}
