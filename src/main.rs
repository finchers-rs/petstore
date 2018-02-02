#![feature(conservative_impl_trait)]

#[allow(unused_imports)]
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate finchers;
extern crate finchers_json;
extern crate finchers_urlencoded;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate serde;

use finchers::{Endpoint, EndpointServiceExt};
use finchers::application::{Application};
use finchers_json::JsonResponder;
use Response::*;

mod model {
    pub type OrderId = u64;
    pub type UserId = u64;
    pub type CategoryId = u64;
    pub type TagId = u64;
    pub type PetId = u64;

    #[derive(Debug, Deserialize)]
    pub struct Order {
        id: OrderId,
        #[serde(rename = "petId")]
        pet_id: PetId,
        quantity: u32,
        #[serde(rename = "shipDate")]
        ship_date: String,
        status: OrderStatus,
        complete: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct User {
        id: UserId,
        username: String,
        #[serde(rename = "firstName")]
        first_name: String,
        #[serde(rename = "lastName")]
        last_name: String,
        email: String,
        password: String,
        phone: String,
        #[serde(rename = "userStatus")]
        user_status: u32,
    }

    #[derive(Debug, Deserialize)]
    pub struct Category {
        id: CategoryId,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Tag {
        id: TagId,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Pet {
        id: PetId,
        category: Category,
        name: String,
        #[serde(rename = "photoUrls")]
        photo_urls: Vec<String>,
        tags: Vec<Tag>,
        status: Status,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum OrderStatus {
        Placed,
        Approved,
        Delivered,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Status {
        Available,
        Pending,
        Sold,
    }
}

#[derive(Debug, Serialize, HttpStatus)]
#[serde(untagged)]
pub enum Response {
    Dummy,
}

pub fn build_endpoint() -> impl Endpoint<Item = Response> + 'static {
    use finchers::endpoint::prelude::*;
    use finchers_json::json_body;
    use finchers_urlencoded::{form_body, from_csv, queries_req};
    use model::*;

    #[derive(Debug, Deserialize)]
    struct FindByStatusParam {
        #[serde(deserialize_with = "from_csv")]
        status: Vec<Status>,
    }

    #[derive(Debug, Deserialize)]
    struct UpdatePetViaFormParam {
        name: Option<String>,
        status: Option<Status>,
    }

    let pet_id = path::<PetId>();
    let pet_body = json_body::<Pet>();
    let pet_api = endpoint("pet").with(choice![
        post(pet_body).map(|_| Dummy),
        put(pet_body).map(|_| Dummy),
        get("findByStatus")
            .with(queries_req::<FindByStatusParam>())
            .map(|_| Dummy),
        get(pet_id).map(|_| Dummy),
        post((pet_id, form_body::<UpdatePetViaFormParam>())).map(|_| Dummy),
        delete(pet_id).map(|_| Dummy),
    ]);

    let order_id = path::<OrderId>();
    let order_body = json_body::<Order>();
    let store_api = endpoint("store").with(choice![
        get("inventory").map(|_| Dummy),
        endpoint("order").with(choice![
            post(order_body).map(|_| Dummy),
            get(order_id).map(|_| Dummy),
            delete(order_id).map(|_| Dummy),
        ]),
    ]);

    let username = path::<String>();
    let user_body = json_body::<User>();
    let user_api = endpoint("user").with(choice![
        post(user_body).map(|_| Dummy),
        post("createWithArray")
            .with(json_body::<Vec<User>>())
            .map(|_| Dummy),
        get(username).map(|_| Dummy),
        put((username, user_body)).map(|_| Dummy),
        delete(username).map(|_| Dummy),
    ]);

    endpoint("v2").with(choice![pet_api, store_api, user_api,])
}

fn main() {
    let endpoint = build_endpoint();
    let service = endpoint.with_responder(JsonResponder::<Response>::default());
    Application::from_service(service).run();
}
