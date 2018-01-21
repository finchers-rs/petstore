#![feature(conservative_impl_trait)]

#[macro_use]
extern crate finchers;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;
mod error;
mod petstore;
mod model;

pub mod api;
pub use db::PetstoreDb;
pub use petstore::Petstore;
