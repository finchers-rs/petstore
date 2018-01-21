use finchers::{Endpoint, Handler};

use error::Error;
use model::{Inventory, Order};
use petstore::Petstore;

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

use self::Request::*;
use self::Response::*;

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
    type Result = Result<Option<Self::Item>, Self::Error>;

    fn call(&self, request: Request) -> Self::Result {
        match request {
            GetInventory => self.get_inventory()
                .map_err(Error::database)
                .map(|i| Some(TheInventory(i))),
            AddOrder(order) => self.add_order(order)
                .map_err(Error::database)
                .map(|id| Some(OrderCreated(id))),
            DeleteOrder(id) => self.delete_order(id)
                .map_err(Error::database)
                .map(|deleted| Some(OrderDeleted(deleted))),
            FindOrder(id) => self.find_order(id)
                .map_err(Error::database)
                .map(|o| o.map(TheOrder)),
        }
    }
}
