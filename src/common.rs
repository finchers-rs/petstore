use serde::Serialize;
use serde_json;
use finchers::http::header;

pub use finchers::http::{IntoResponse, Response as HyperResponse, StatusCode};

pub fn json_response<T: Serialize>(content: &T) -> HyperResponse {
    let body = serde_json::to_vec(&content).unwrap();
    HyperResponse::new()
        .with_header(header::ContentType::json())
        .with_header(header::ContentLength(body.len() as u64))
        .with_body(body)
}

pub fn no_content() -> HyperResponse {
    HyperResponse::new()
        .with_status(StatusCode::NoContent)
        .with_header(header::ContentLength(0))
}

pub fn no_route() -> HyperResponse {
    HyperResponse::new()
        .with_status(StatusCode::NotFound)
        .with_header(header::ContentLength(0))
}
