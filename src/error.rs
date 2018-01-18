use std::error::Error as StdError;
use finchers::http::{header, IntoResponse, Response, StatusCode};
use db::DbError;

#[derive(Debug)]
pub enum Error {
    Endpoint(Box<StdError + 'static>),
    Database(DbError),
}

impl Error {
    pub fn endpoint<E: StdError + 'static>(err: E) -> Self {
        Error::Endpoint(Box::new(err))
    }

    pub fn database(err: DbError) -> Self {
        Error::Database(err)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Endpoint(e) => {
                let body = e.to_string();
                Response::new()
                    .with_status(StatusCode::BadRequest)
                    .with_header(header::ContentType::plaintext())
                    .with_header(header::ContentLength(body.len() as u64))
                    .with_body(body)
            }
            Error::Database(e) => Response::new()
                .with_status(StatusCode::InternalServerError)
                .with_body(e.to_string()),
        }
    }
}
