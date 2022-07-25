/*
use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::Serialize;

pub enum Error {
    WrongCredentialsError,
    JWTTokenError,
    JWTTokenCreationError,
    NoAuthHeaderError,
    InvalidAuthHeaderError,
    NoPermissionsError
}

#[derive(Serialize, Debug)]
struct ErrorResponse {
    message: String,
    status: String
}

pub async fn handle_error(err: Error) -> HttpResponseBuilder {
    match err {
        Error::InvalidAuthHeaderError | Error::NoAuthHeaderError | Error::JWTTokenError => {
            HttpResponse::NonAuthoritativeInformation()
        },
        Error::JWTTokenCreationError => {
            HttpResponse::InternalServerError()
        },
        Error::NoPermissionsError => {
            HttpResponse::Forbidden()
        },
        Error::WrongCredentialsError => {
            HttpResponse::BadRequest()
        }
    }
}*/