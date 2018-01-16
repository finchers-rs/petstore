use finchers::responder::{Error, Responder};
use finchers::http::{header, Response, StatusCode};
use serde::Serialize;
use serde_json;

use endpoint::EndpointError;
use handler::PetstoreResponse;
use db::DbError;

#[derive(Debug, Clone)]
pub struct PetstoreResponder {
    _priv: (),
}

impl PetstoreResponder {
    pub fn new() -> Self {
        PetstoreResponder { _priv: () }
    }
}

impl PetstoreResponder {
    fn respond_item(&self, item: PetstoreResponse) -> Response {
        let PetstoreResponse { status, content } = item;
        match content {
            Some(content) => {
                use handler::PetstoreResponseContent::*;
                match content {
                    ThePet(pet) => pet.map_or_else(
                        || self.no_route(),
                        |p| self.respond_json(&p).with_status(status),
                    ),
                    PetId(id) => self.respond_json(&id).with_status(status),
                    Pets(id) => self.respond_json(&id).with_status(status),

                    TheInventory(inventory) => self.respond_json(&inventory).with_status(status),
                    OrderId(id) => self.respond_json(&id).with_status(status),
                    TheOrder(order) => order.map_or_else(
                        || self.no_route(),
                        |o| self.respond_json(&o).with_status(status),
                    ),
                    OrderDeleted(deleted) => self.respond_json(&deleted).with_status(status),

                    Username(username) => self.respond_json(&username).with_status(status),
                    Usernames(usernames) => self.respond_json(&usernames).with_status(status),
                    TheUser(user) => user.map_or_else(
                        || self.no_route(),
                        |u| self.respond_json(&u).with_status(status),
                    ),
                }
            }
            None => Response::new()
                .with_status(StatusCode::NoContent)
                .with_header(header::ContentLength(0)),
        }
    }

    fn respond_endpoint_error(&self, err: EndpointError) -> Response {
        let body = err.to_string();
        Response::new()
            .with_status(StatusCode::BadRequest)
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }

    fn respond_handler_error(&self, err: DbError) -> Response {
        Response::new()
            .with_status(StatusCode::InternalServerError)
            .with_body(err.to_string())
    }

    fn respond_json<T: Serialize>(&self, content: &T) -> Response {
        let body = serde_json::to_vec(&content).unwrap();
        Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }

    fn no_route(&self) -> Response {
        Response::new().with_status(StatusCode::NotFound)
    }
}

impl Responder<PetstoreResponse, EndpointError, DbError> for PetstoreResponder {
    type Response = Response;

    fn respond(&self, input: Result<PetstoreResponse, Error<EndpointError, DbError>>) -> Self::Response {
        match input {
            Ok(item) => self.respond_item(item),
            Err(Error::NoRoute) => self.no_route(),
            Err(Error::Endpoint(e)) => self.respond_endpoint_error(e),
            Err(Error::Handler(e)) => self.respond_handler_error(e),
        }
    }
}
