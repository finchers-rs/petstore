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
pub enum PetstoreResponse {
    ThePet(Option<Pet>),
    PetCreated(u64),
    Pets(Vec<Pet>),
    PetDeleted,

    TheInventory(Inventory),
    TheOrder(Option<Order>),
    OrderCreated(u64),
    OrderDeleted(bool),

    UserCreated(String),
    UsersCreated(Vec<String>),
    TheUser(Option<User>),
    UserDeleted,
}

pub use self::PetstoreResponse::*;

impl IntoResponse for PetstoreResponse {
    fn into_response(self) -> Response {
        match self {
            ThePet(pet) => pet.map_or_else(no_route, |p| json_response(&p)),
            PetCreated(id) => json_response(&id).with_status(StatusCode::Created),
            Pets(id) => json_response(&id),
            PetDeleted => no_content(),

            TheInventory(inventory) => json_response(&inventory),
            TheOrder(order) => order.map_or_else(no_route, |o| json_response(&o)),
            OrderCreated(id) => json_response(&id).with_status(StatusCode::Created),
            OrderDeleted(deleted) => json_response(&deleted),

            UserCreated(username) => json_response(&username).with_status(StatusCode::Created),
            UsersCreated(usernames) => json_response(&usernames).with_status(StatusCode::Created),
            TheUser(user) => user.map_or_else(no_route, |u| json_response(&u)),
            UserDeleted => no_content(),
        }
    }
}

fn json_response<T: Serialize>(content: &T) -> Response {
    let body = serde_json::to_vec(&content).unwrap();
    Response::new()
        .with_header(header::ContentType::json())
        .with_header(header::ContentLength(body.len() as u64))
        .with_body(body)
}

fn no_content() -> Response {
    Response::new()
        .with_status(StatusCode::NoContent)
        .with_header(header::ContentLength(0))
}

fn no_route() -> Response {
    Response::new()
        .with_status(StatusCode::NotFound)
        .with_header(header::ContentLength(0))
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

impl PetstoreHandler {
    fn add_users(&self, users: Vec<User>) -> impl Future<Item = Vec<String>, Error = DbError> {
        use futures::future::join_all;
        let db = self.db.clone();
        join_all(users.into_iter().map(move |new_user| db.add_user(new_user)))
    }
}

impl Handler<Request> for PetstoreHandler {
    type Item = PetstoreResponse;
    type Error = Error;
    type Future = PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Future {
        match request {
            GetPet(id) => self.db.get_pet(id).map(ThePet).into(),
            AddPet(pet) => self.db.add_pet(pet).map(PetCreated).into(),
            UpdatePet(pet) => self.db.update_pet(pet).map(|pet| ThePet(Some(pet))).into(),
            DeletePet(id) => self.db.delete_pet(id).map(|_| PetDeleted).into(),
            FindPetsByStatuses(param) => self.db.get_pets_by_status(param.status).map(Pets).into(),
            FindPetsByTags(param) => self.db.find_pets_by_tag(param.tags).map(Pets).into(),
            UpdatePetViaForm(id, param) => self.db
                .update_pet_name_status(id, param.name, param.status)
                .map(|pet| ThePet(Some(pet)))
                .into(),

            GetInventory => self.db.get_inventory().map(TheInventory).into(),
            AddOrder(order) => self.db.add_order(order).map(OrderCreated).into(),
            DeleteOrder(id) => self.db.delete_order(id).map(OrderDeleted).into(),
            FindOrder(id) => self.db.find_order(id).map(TheOrder).into(),

            AddUser(new_user) => self.db.add_user(new_user).map(UserCreated).into(),
            AddUsersViaList(users) => self.add_users(users).map(UsersCreated).into(),
            DeleteUser(name) => self.db.delete_user(name).map(|_| UserDeleted).into(),
            GetUser(name) => self.db.get_user(name).map(TheUser).into(),
            UpdateUser(user) => self.db
                .update_user(user)
                .map(|user| TheUser(Some(user)))
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
