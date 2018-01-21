#![feature(conservative_impl_trait)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod api;
pub mod error;
pub mod petstore;
pub mod model;

pub use petstore::Petstore;
