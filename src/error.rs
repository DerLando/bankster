use axum::{http::StatusCode, response::Response};

pub(crate) enum ApiError {
    SQLError(sqlx::Error),
    TemplateError(askama::Error),
    FilterError,
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self::SQLError(e)
    }
}

impl From<askama::Error> for ApiError {
    fn from(value: askama::Error) -> Self {
        Self::TemplateError(value)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::SQLError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("SQL Error: {e}")).into_response()
            }
            Self::TemplateError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
            Self::FilterError => (StatusCode::BAD_REQUEST, "Invalid filter").into_response(),
        }
    }
}
