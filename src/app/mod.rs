use std::{sync::Arc, fmt::Display, num::ParseIntError};
use actix_web::{ResponseError, HttpResponse};
use diesel::{r2d2::{Pool, ConnectionManager}, PgConnection};
use r2d2_redis::{RedisConnectionManager, redis::RedisError};

pub struct AppState{
    pub psql_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    pub redis_pool: Arc<Pool<RedisConnectionManager>>
}

impl Clone for AppState{
    fn clone(&self) -> Self {
        Self { psql_pool: self.psql_pool.clone(), redis_pool: self.redis_pool.clone() }
    }
}

#[derive(Debug)]
pub enum AppError{
    UnauthorizedError,
    InternalServerError,
    BadRequest,
    Forbidden
}

impl Display for AppError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            AppError::UnauthorizedError => f.write_str("Unauthorized"),
            AppError::InternalServerError => f.write_str("Internal server error"),
            AppError::BadRequest => f.write_str("Bad request"),
            AppError::Forbidden => f.write_str("Forbidden"),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::UnauthorizedError => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::InternalServerError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Forbidden => actix_web::http::StatusCode::FORBIDDEN
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        HttpResponse::new(self.status_code())
    }
}
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::InvalidCString(_) => AppError::BadRequest,
            diesel::result::Error::DatabaseError(_, _) => AppError::UnauthorizedError,
            diesel::result::Error::NotFound => AppError::UnauthorizedError,
            diesel::result::Error::QueryBuilderError(_) => AppError::BadRequest,
            diesel::result::Error::DeserializationError(_) => AppError::BadRequest,
            diesel::result::Error::SerializationError(_) => AppError::InternalServerError,
            diesel::result::Error::RollbackTransaction => AppError::InternalServerError,
            diesel::result::Error::AlreadyInTransaction => AppError::InternalServerError,
            _ => AppError::InternalServerError,
        }
    }
}
impl From<RedisError> for AppError{
    fn from(err: RedisError) -> Self {
        match err.kind() {
            r2d2_redis::redis::ErrorKind::ResponseError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::AuthenticationFailed => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::TypeError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::ExecAbortError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::BusyLoadingError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::NoScriptError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::InvalidClientConfig => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::Moved => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::Ask => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::TryAgain => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::ClusterDown => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::CrossSlot => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::MasterDown => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::IoError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::ClientError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::ExtensionError => AppError::InternalServerError,
            r2d2_redis::redis::ErrorKind::ReadOnly => AppError::InternalServerError,
            _ => AppError::InternalServerError,
        }
    }
}
impl From<ParseIntError> for AppError {
    fn from(_: ParseIntError) -> Self {
        Self::BadRequest
    }
}
impl From<serde_json::Error> for AppError{
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Io => AppError::InternalServerError,
            _ => AppError::BadRequest,
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AppError::BadRequest,
            _ => AppError::InternalServerError
        }
    }
}

impl std::error::Error for AppError{}