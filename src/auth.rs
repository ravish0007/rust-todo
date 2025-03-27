use axum::{
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use password_auth::generate_hash;
use rinja_axum::Template;
use sqlx::SqlitePool;

use crate::errors::{AppError, MyCustomError};
use crate::users::{AuthSession, Credentials};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignUpTemplate {}

pub fn router(conn: SqlitePool) -> Router<()> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/login", get(self::get::login))
        .route("/logout", get(self::get::logout))
        .route("/signup", get(self::get::signup))
        .route("/signup", post(self::post::signup))
        .with_state(conn.clone())
}

mod post {
    use super::*;

    pub async fn signup(
        State(db): State<SqlitePool>,
        Form(creds): Form<Credentials>,
    ) -> Result<impl IntoResponse, AppError> {
        let headers = HeaderMap::new();

        // Check if the user already exists
        let existing_user = sqlx::query("SELECT * FROM users WHERE username = ?")
            .bind(&creds.username)
            .fetch_optional(&db)
            .await;

        match existing_user {
            Ok(Some(_)) => {
                return Err(AppError::UserAlreadyExist);
            }
            Ok(None) => {}
            Err(_) => {
                return Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
                    MyCustomError("Database error".to_string()),
                ))))
            }
        }

        let password_hash = generate_hash(creds.password);

        let _ = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
            .bind(creds.username)
            .bind(password_hash)
            .execute(&db)
            .await;

        Ok((headers, Redirect::to("/login")))
    }

    pub async fn login(
        mut auth_session: AuthSession,
        Form(creds): Form<Credentials>,
    ) -> Result<impl IntoResponse, AppError> {
        let headers = HeaderMap::new();
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Ok((headers, Redirect::to("/login")));
            }
            Err(_) => {
                return Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
                    MyCustomError("Something went wrong".to_string()),
                ))))
            }
        };

        if auth_session.login(&user).await.is_err() {
            return Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
                MyCustomError("Internal server wrong".to_string()),
            ))));
        }

        return Ok((headers, Redirect::to("/todos")));
    }
}

mod get {
    use super::*;

    pub async fn login() -> Html<String> {
        Html(LoginTemplate {}.render().unwrap())
    }

    pub async fn signup() -> Html<String> {
        Html(SignUpTemplate {}.render().unwrap())
    }

    pub async fn logout(mut auth_session: AuthSession) -> Result<impl IntoResponse, AppError> {
        let headers = HeaderMap::new();
        match auth_session.logout().await {
            Ok(_) => Ok((headers, Redirect::to("/login"))),
            Err(_) => Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
                MyCustomError("Internal server wrong".to_string()),
            )))),
        }
    }
}
