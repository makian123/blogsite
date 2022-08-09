use actix_web::{HttpResponse, ResponseError};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use r2d2_redis::{redis::RedisError, RedisConnectionManager};
use std::{fmt::Display, num::ParseIntError, sync::Arc};

use crate::database::db_utils::{psql_connect_to_db, redis_connect_to_db};

/** Used for storing the database connections when handling requests */
pub struct AppState {
    pub psql_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    pub redis_pool: Arc<Pool<RedisConnectionManager>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            psql_pool: self.psql_pool.clone(),
            redis_pool: self.redis_pool.clone(),
        }
    }
}
impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("psql_pool", &self.psql_pool.state())
            .field("redis_pool", &self.redis_pool.state())
            .finish()
    }
}

impl AppState {
    pub fn new(cons: Option<u32>) -> Self {
        AppState {
            psql_pool: psql_connect_to_db(cons),
            redis_pool: redis_connect_to_db(cons),
        }
    }
}

/** Holds the errors we will used during request processing */
#[derive(Debug)]

pub enum AppError {
    UnauthorizedError,
    InternalServerError,
    BadRequest,
    Forbidden,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            AppError::Forbidden => actix_web::http::StatusCode::FORBIDDEN,
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
            _ => AppError::InternalServerError,
        }
    }
}
impl From<RedisError> for AppError {
    fn from(_: RedisError) -> Self {
        AppError::InternalServerError
    }
}
impl From<ParseIntError> for AppError {
    fn from(_: ParseIntError) -> Self {
        Self::BadRequest
    }
}
impl From<serde_json::Error> for AppError {
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
            _ => AppError::InternalServerError,
        }
    }
}

impl std::error::Error for AppError {}
