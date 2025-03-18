use axum::{
    extract::{Json, State}, 
    Form,
    http::HeaderMap,
    response::{ Html, IntoResponse, Redirect }
};

use sqlx::SqlitePool;
use rinja_axum::Template;

use crate::todos::model::{DeleteTodo, UpdateTodo, NewTodo, Todo, Todos};
use crate::errors::{AppError, MyCustomError};

pub async fn index(State(db): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
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

pub async fn new(
    State(db): State<SqlitePool>,
    Form(todo): Form<NewTodo>,
) -> Result<impl IntoResponse, AppError> {
    let _ = sqlx::query("INSERT INTO todos (content) VALUES (?)")
        .bind(todo.content)
        .execute(&db)
        .await;

    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}

pub async fn update(
    State(db): State<SqlitePool>,
    Json(todo): Json<UpdateTodo>,
) -> Result<impl IntoResponse, AppError> {
    let _ = sqlx::query("UPDATE todos set done = (?), content = (?) where id = (?)")
        .bind(todo.done)
        .bind(todo.content)
        .bind(todo.id)
        .execute(&db)
        .await;

    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}

pub async fn delete(
    State(db): State<SqlitePool>,
    Json(todo): Json<DeleteTodo>,
) -> Result<impl IntoResponse, AppError> {
    let _ = sqlx::query("DELETE from todos where id = (?)")
        .bind(todo.id)
        .execute(&db)
        .await;

    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}
