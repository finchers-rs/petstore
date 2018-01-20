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

mod api;
mod common;
mod db;
mod model;

use finchers::service::FinchersService;
use finchers::responder::DefaultResponder;
use futures::{Future, Stream};
use hyper::server::{Http, NewService};
use tokio_core::reactor::Core;

use db::PetstoreDb;

fn main() {
    let db = PetstoreDb::new();

    let service = FinchersService::new(
        api::endpoint(),
        api::Petstore::new(db),
        DefaultResponder::default(),
    );

    run_service(move || Ok(service.clone()));
}

fn run_service<S>(new_service: S)
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + 'static,
{
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
