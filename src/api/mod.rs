pub mod common;
pub mod pet;
pub mod store;
pub mod user;

use finchers::{Endpoint, Handler};
use error::Error;
use petstore::Petstore;

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
    use api::common::*;

    impl IntoResponse for super::Response {
        fn into_response(self) -> HyperResponse {
            use super::Response::*;
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
    type Result = Result<Option<Self::Item>, Self::Error>;

    fn call(&self, request: Request) -> Self::Result {
        use self::Request::*;
        match request {
            Pet(pet) => self.call(pet).map(|r| r.map(Response::Pet)),
            Store(store) => self.call(store).map(|r| r.map(Response::Store)),
            User(user) => self.call(user).map(|r| r.map(Response::User)),
        }.map_err(Into::into)
    }
}
