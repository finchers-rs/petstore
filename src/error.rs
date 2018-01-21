use std::error::Error as StdError;
use finchers::http::{header, IntoResponse, Response, StatusCode};
use petstore::PetstoreError;

#[derive(Debug, From)]
pub enum Error {
    Endpoint(EndpointError),
    Petstore(PetstoreError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Endpoint(e) => e.into_response(),
            Error::Petstore(e) => Response::new()
                .with_status(StatusCode::InternalServerError)
                .with_body(e.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct EndpointError(Box<StdError + 'static>);

impl<E: StdError + 'static> From<E> for EndpointError {
    fn from(err: E) -> Self {
        EndpointError(Box::new(err))
    }
}

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        let body = self.0.to_string();
        Response::new()
            .with_status(StatusCode::BadRequest)
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }
}

impl IntoResponse for PetstoreError {
    fn into_response(self) -> Response {
        let body = self.to_string();
        Response::new()
            .with_status(StatusCode::InternalServerError)
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }
}
