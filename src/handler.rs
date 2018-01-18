use finchers::Handler;
use finchers::http::{header, IntoResponse, Response, StatusCode};
use futures::{Future, Poll};
use serde::Serialize;
use serde_json;

use model::*;
use db::{DbError, PetstoreDb};
use endpoint::Request;
use endpoint::Request::*;
use error::Error;

#[derive(Debug)]
pub struct PetstoreResponse {
    pub status: StatusCode,
    pub content: Option<PetstoreResponseContent>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum PetstoreResponseContent {
    ThePet(Option<Pet>),
    PetId(u64),
    Pets(Vec<Pet>),

    TheInventory(Inventory),
    OrderId(u64),
    TheOrder(Option<Order>),
    OrderDeleted(bool),

    Username(String),
    Usernames(Vec<String>),
    TheUser(Option<User>),
}

pub use self::PetstoreResponseContent::*;

impl From<PetstoreResponseContent> for PetstoreResponse {
    fn from(content: PetstoreResponseContent) -> Self {
        PetstoreResponse {
            status: StatusCode::Ok,
            content: Some(content),
        }
    }
}

impl PetstoreResponse {
    pub fn created(content: PetstoreResponseContent) -> Self {
        PetstoreResponse {
            status: StatusCode::Created,
            content: Some(content),
        }
    }

    pub fn no_content() -> Self {
        PetstoreResponse {
            status: StatusCode::NoContent,
            content: None,
        }
    }
}

impl IntoResponse for PetstoreResponse {
    fn into_response(self) -> Response {
        let PetstoreResponse { status, content } = self;
        match content {
            Some(content) => {
                use handler::PetstoreResponseContent::*;
                match content {
                    ThePet(pet) => pet.map_or_else(no_route, |p| respond_json(&p).with_status(status)),
                    PetId(id) => respond_json(&id).with_status(status),
                    Pets(id) => respond_json(&id).with_status(status),

                    TheInventory(inventory) => respond_json(&inventory).with_status(status),
                    OrderId(id) => respond_json(&id).with_status(status),
                    TheOrder(order) => order.map_or_else(no_route, |o| respond_json(&o).with_status(status)),
                    OrderDeleted(deleted) => respond_json(&deleted).with_status(status),

                    Username(username) => respond_json(&username).with_status(status),
                    Usernames(usernames) => respond_json(&usernames).with_status(status),
                    TheUser(user) => user.map_or_else(no_route, |u| respond_json(&u).with_status(status)),
                }
            }
            None => Response::new()
                .with_status(StatusCode::NoContent)
                .with_header(header::ContentLength(0)),
        }
    }
}

fn respond_json<T: Serialize>(content: &T) -> Response {
    let body = serde_json::to_vec(&content).unwrap();
    Response::new()
        .with_header(header::ContentType::json())
        .with_header(header::ContentLength(body.len() as u64))
        .with_body(body)
}

fn no_route() -> Response {
    Response::new().with_status(StatusCode::NotFound)
}

#[derive(Debug, Clone)]
pub struct PetstoreHandler {
    db: PetstoreDb,
}

impl PetstoreHandler {
    pub fn new(db: PetstoreDb) -> Self {
        PetstoreHandler { db }
    }
}

impl Handler<Request> for PetstoreHandler {
    type Item = PetstoreResponse;
    type Error = Error;
    type Future = PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Future {
        match request {
            GetPet(id) => self.db.get_pet(id).map(|pet| ThePet(pet).into()).into(),
            AddPet(pet) => self.db
                .add_pet(pet)
                .map(|id| PetstoreResponse::created(PetId(id)))
                .into(),
            UpdatePet(pet) => self.db
                .update_pet(pet)
                .map(|pet| ThePet(Some(pet)).into())
                .into(),
            DeletePet(id) => self.db
                .delete_pet(id)
                .map(|_| PetstoreResponse::no_content())
                .into(),
            FindPetsByStatuses(param) => self.db
                .get_pets_by_status(param.status)
                .map(|pets| Pets(pets).into())
                .into(),
            FindPetsByTags(param) => self.db
                .find_pets_by_tag(param.tags)
                .map(|pets| Pets(pets).into())
                .into(),
            UpdatePetViaForm(id, param) => self.db
                .update_pet_name_status(id, param.name, param.status)
                .map(|pet| ThePet(Some(pet)).into())
                .into(),

            GetInventory => self.db
                .get_inventory()
                .map(|inventory| TheInventory(inventory).into())
                .into(),
            AddOrder(order) => self.db.add_order(order).map(|id| OrderId(id).into()).into(),
            DeleteOrder(id) => self.db
                .delete_order(id)
                .map(|deleted| OrderDeleted(deleted).into())
                .into(),
            FindOrder(id) => self.db
                .find_order(id)
                .map(|order| TheOrder(order).into())
                .into(),

            AddUser(new_user) => self.db
                .add_user(new_user)
                .map(|username| Username(username).into())
                .into(),
            AddUsersViaList(users) => ::futures::future::join_all(users.into_iter().map({
                let db = self.db.clone();
                move |new_user| db.add_user(new_user)
            })).map(|usernames| PetstoreResponse::created(Usernames(usernames)))
                .into(),
            DeleteUser(name) => self.db
                .delete_user(name)
                .map(|_| PetstoreResponse::no_content())
                .into(),
            GetUser(name) => self.db
                .get_user(name)
                .map(|user| TheUser(user).into())
                .into(),
            UpdateUser(user) => self.db
                .update_user(user)
                .map(|user| TheUser(Some(user)).into())
                .into(),
        }
    }
}

pub struct PetstoreHandlerFuture(Box<Future<Item = PetstoreResponse, Error = DbError>>);

impl<F> From<F> for PetstoreHandlerFuture
where
    F: Future<Item = PetstoreResponse, Error = DbError> + 'static,
{
    fn from(f: F) -> Self {
        PetstoreHandlerFuture(Box::new(f))
    }
}

impl Future for PetstoreHandlerFuture {
    type Item = PetstoreResponse;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map_err(Error::database)
    }
}
