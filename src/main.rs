#![feature(conservative_impl_trait)]
#![warn(missing_debug_implementations)]

extern crate chrono;
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

mod model;
mod api;

use finchers::service::{backend, EndpointServiceExt, Server};
use finchers_json::JsonResponder;

fn main() {
    let endpoint = api::api();
    let service = endpoint.with_responder(JsonResponder::<String>::default());
    Server::from_service(service).run(backend::default());
}
