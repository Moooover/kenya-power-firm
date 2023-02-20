use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    App, HttpResponse,
};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal server error")]
    InternalServerError(#[from] anyhow::Error),
}

impl error::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let err_json = json!({ "error": self.to_string() });
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(err_json)
    }
}
