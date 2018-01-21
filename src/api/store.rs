use finchers::{Endpoint, Handler};

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
    TheOrder(Order),
    OrderCreated(u64),
    OrderDeleted(bool),
}

mod imp {
    use api::common::*;

    impl IntoResponse for super::Response {
        fn into_response(self) -> HyperResponse {
            use super::Response::*;
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
    use self::Request::*;

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
        use self::Request::*;
        use self::Response::*;
        match request {
            GetInventory => self.db
                .get_inventory()
                .map_err(Error::database)
                .map(|i| Some(TheInventory(i))),
            AddOrder(order) => self.db
                .add_order(order)
                .map_err(Error::database)
                .map(|id| Some(OrderCreated(id))),
            DeleteOrder(id) => self.db
                .delete_order(id)
                .map_err(Error::database)
                .map(|deleted| Some(OrderDeleted(deleted))),
            FindOrder(id) => self.db
                .find_order(id)
                .map_err(Error::database)
                .map(|o| o.map(TheOrder)),
        }
    }
}
