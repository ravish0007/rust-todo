use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use rinja_axum::Template;

#[derive(Debug)]
pub struct MyCustomError(pub String);

impl std::error::Error for MyCustomError {}
impl std::fmt::Display for MyCustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum AppError {
    /// not found
    NotFound,
    /// user already found in db
    UserAlreadyExist,
    /// could not render template
    Render(#[from] rinja_axum::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct ErrorTmpl {
            error: AppError,
        }

        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::UserAlreadyExist => StatusCode::CONFLICT,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let tmpl = ErrorTmpl { error: self };

        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}
