use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("template error")]
    Template(#[from] askama::Error),
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(error) => {
                tracing::error!(?error, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong.".to_string(),
                )
            }
            AppError::Template(error) => {
                tracing::error!(?error, "template error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong.".to_string(),
                )
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found.".to_string()),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "That request could not be verified.".to_string(),
            ),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };

        (status, Html(format!("<h1>{status}</h1><p>{message}</p>"))).into_response()
    }
}

pub fn render<T: Template>(template: T) -> Result<Html<String>, AppError> {
    Ok(Html(template.render()?))
}
