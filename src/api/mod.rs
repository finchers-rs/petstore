pub mod pet;
pub mod store;
pub mod user;
pub mod error;

use finchers::{Endpoint, Handler};
use finchers::http::{header, IntoResponse, Response, StatusCode};
use futures::{Future, Poll};

use model::*;
use db::{DbError, PetstoreDb};
use self::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Pet(pet::Request),
    Store(store::Request),
    User(user::Request),
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + 'static {
    endpoint![
        pet::endpoint().map(Request::Pet),
        store::endpoint().map(Request::Store),
        user::endpoint().map(Request::User),
    ]
}

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

mod imp {
    use super::*;
    use serde::Serialize;
    use serde_json;

    impl IntoResponse for PetstoreResponse {
        fn into_response(self) -> Response {
            use super::PetstoreResponse::*;
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
}

#[derive(Debug, Clone)]
pub struct Petstore {
    db: PetstoreDb,
}

impl Petstore {
    pub fn new(db: PetstoreDb) -> Self {
        Petstore { db }
    }
}

impl Petstore {
    fn add_users(&self, users: Vec<User>) -> impl Future<Item = Vec<String>, Error = DbError> {
        use futures::future::join_all;
        let db = self.db.clone();
        join_all(users.into_iter().map(move |new_user| db.add_user(new_user)))
    }
}

impl Handler<Request> for Petstore {
    type Item = PetstoreResponse;
    type Error = Error;
    type Future = PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Future {
        use self::Request::*;
        match request {
            Pet(pet) => self.call(pet),
            Store(store) => self.call(store),
            User(user) => self.call(user),
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
