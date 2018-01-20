pub mod pet;
pub mod store;
pub mod user;
pub mod error;
pub mod petstore;

use finchers::{Endpoint, Handler};
use futures::Future;

use self::error::Error;
use self::petstore::Petstore;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Pet(pet::Request),
    Store(store::Request),
    User(user::Request),
}

#[derive(Debug)]
pub enum Response {
    Pet(pet::Response),
    Store(store::Response),
    User(user::Response),
}

mod imp {
    use common::*;
    use super::Response::*;

    impl IntoResponse for super::Response {
        fn into_response(self) -> HyperResponse {
            match self {
                Pet(pet) => pet.into_response(),
                Store(store) => store.into_response(),
                User(user) => user.into_response(),
            }
        }
    }
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + Clone + 'static {
    choice![
        pet::endpoint().map(Request::Pet),
        store::endpoint().map(Request::Store),
        user::endpoint().map(Request::User),
    ]
}

impl Handler<Request> for Petstore {
    type Item = Response;
    type Error = Error;
    type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        match request {
            Request::Pet(pet) => self.call(pet),
            Request::Store(store) => self.call(store),
            Request::User(user) => self.call(user),
        }
    }
}
