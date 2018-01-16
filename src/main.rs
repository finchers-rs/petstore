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

use finchers::{Application, Endpoint, EndpointServiceExt};
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

    #[derive(Debug, Deserialize)]
    struct FindByStatusParam {
        #[serde(deserialize_with = "from_csv")]
        status: Vec<model::Status>,
    }

    #[derive(Debug, Deserialize)]
    struct UpdatePetViaFormParam {
        name: Option<String>,
        status: Option<model::Status>,
    }

    let pet_api = choice![
        post("pet").with(json_body::<model::Pet>()).map(|_| Dummy),
        put("pet").with(json_body::<model::Pet>()).map(|_| Dummy),
        get("pet/findByStatus")
            .with(queries_req::<FindByStatusParam>())
            .map(|_| Dummy),
        get("pet").with(path::<u64>()).map(|_| Dummy),
        post("pet")
            .with(path::<u64>())
            .join(form_body::<UpdatePetViaFormParam>())
            .map(|_| Dummy),
        delete("pet").with(path::<u64>()).map(|_| Dummy),
    ];

    let store_api = choice![
        get("store/inventory").map(|_| Dummy),
        post("store/order")
            .with(json_body::<model::Order>())
            .map(|_| Dummy),
        get("store/order").with(path::<u64>()).map(|_| Dummy),
        delete("store/order").with(path::<u64>()).map(|_| Dummy),
    ];

    let user_api = choice![
        post("user").with(json_body::<model::User>()).map(|_| Dummy),
        post("user/createWithArray")
            .with(json_body::<Vec<model::User>>())
            .map(|_| Dummy),
        get("user").with(path::<String>()).map(|_| Dummy),
        put("user")
            .with(path::<String>())
            .join(json_body::<model::User>())
            .map(|_| Dummy),
        delete("user").with(path::<String>()).map(|_| Dummy),
    ];

    endpoint("v2").with(choice![pet_api, store_api, user_api,])
}

fn main() {
    let endpoint = build_endpoint();
    let service = endpoint.with_responder(JsonResponder::<Response>::default());
    Application::from_service(service).run();
}
