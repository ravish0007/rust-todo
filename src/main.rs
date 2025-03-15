use axum::{
    extract::{Json, State},
    http::header::SET_COOKIE,
    http::HeaderMap,
    http::StatusCode,
    response::{AppendHeaders, Html, IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use axum_cookie::prelude::*;
use rinja_axum::Template;
use serde::Deserialize;
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite, SqlitePool};
use tower_livereload::LiveReloadLayer;

#[derive(Debug)]
struct MyCustomError(String);

impl std::error::Error for MyCustomError {}
impl std::fmt::Display for MyCustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type ID = i32;
#[derive(FromRow)]
pub struct Todo {
    pub id: ID,
    pub content: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct NewTodo {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateTodo {
    pub done: bool,
    pub content: String,
    pub id: ID,
}

#[derive(Deserialize)]
pub struct DeleteTodo {
    pub id: ID,
}

async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS user (id integer primary key AUTOINCREMENT, email text, password text)")
        .execute(pool).await?;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS todos (id integer primary key AUTOINCREMENT, content text, done boolean default false, user_id text, FOREIGN KEY (user_id) REFERENCES user(id))")
        .execute(pool).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let conn = SqlitePoolOptions::new()
        .connect("todos.sqlite?mode=rwc")
        .await
        .unwrap();

    migrate(&conn).await.unwrap();

    let app = Router::new()
        .route("/", get(route_to_todos))
        // .route("/login", get(render_login).post(process_login))
        .route("/logout", get(process_logout))
        .route("/todos", get(index).post(new).put(update).delete(delete))
        .with_state(conn)
        .layer(LiveReloadLayer::new())
        .layer(CookieLayer::default());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "todos.html")]
struct Todos {
    todos: Vec<Todo>,
}

async fn process_logout(cookie: CookieManager) -> impl IntoResponse {
    cookie.add(Cookie::new("session", ""));
    Redirect::to("/login")
}

async fn route_to_todos(cookie: CookieManager) -> impl IntoResponse {
    cookie.add(Cookie::new("session", ""));
    Redirect::to("/todos")
}

async fn index(State(db): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
    let res = sqlx::query_as::<_, Todo>("SELECT id, content, done FROM todos")
        .fetch_all(&db)
        .await;

    match res {
        Ok(todos) => {
            let x = Todos { todos };
            let html = x.render().unwrap();

            Ok(Html(html))
        }
        Err(e) => {
            println!("error: {:?}", e);
            Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
                MyCustomError("failed to fetch from db".to_string()),
            ))))
        }
    }
}

async fn new(
    State(db): State<SqlitePool>,
    Form(todo): Form<NewTodo>,
) -> Result<impl IntoResponse, AppError> {
    let res = sqlx::query("INSERT INTO todos (content) VALUES (?)")
        .bind(todo.content)
        .execute(&db)
        .await;

    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}

async fn update(
    State(db): State<SqlitePool>,
    Json(todo): Json<UpdateTodo>,
) -> Result<impl IntoResponse, AppError> {
    let res = sqlx::query("UPDATE todos set done = (?), content = (?) where id = (?)")
        .bind(todo.done)
        .bind(todo.content)
        .bind(todo.id)
        .execute(&db)
        .await;

    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}

async fn delete(
    State(db): State<SqlitePool>,
    Json(todo): Json<DeleteTodo>,
) -> Result<impl IntoResponse, AppError> {
    let res = sqlx::query("DELETE from todos where id = (?)")
        .bind(todo.id)
        .execute(&db)
        .await;

    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
enum AppError {
    /// not found
    NotFound,
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
