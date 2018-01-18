#![feature(conservative_impl_trait)]

#[macro_use]
extern crate finchers;
extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

mod model;
mod db;
mod endpoint;
mod error;
mod handler;

use finchers::service::FinchersService;
use finchers::responder::DefaultResponder;
use std::rc::Rc;
use futures::{Future, Stream};
use hyper::server::Http;
use tokio_core::reactor::Core;

use db::PetstoreDb;
use endpoint::petstore_endpoint;
use handler::PetstoreHandler;

fn main() {
    let db = PetstoreDb::new();
    let service = FinchersService::new(
        Rc::new(petstore_endpoint()),
        PetstoreHandler::new(db),
        DefaultResponder::default(),
    );
    let new_service = move || Ok(service.clone());

    let mut core = Core::new().unwrap();
    let mut http = Http::new();
    http.pipeline(true);

    let addr = "0.0.0.0:4000".parse().unwrap();
    println!("Serving on listen address {}...", addr);
    let serves = http.serve_addr_handle(&addr, &core.handle(), new_service)
        .unwrap()
        .for_each(|conn| conn.map(|_| ()));
    core.run(serves).unwrap();
}
