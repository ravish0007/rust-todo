use axum::{
    extract::{Json, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect},
    Form,
};

use rinja_axum::Template;
use sqlx::SqlitePool;

use crate::errors::{AppError, MyCustomError};
use crate::todos::model::{DeleteTodo, NewTodo, Todo, Todos, UpdateTodo};
use crate::users::AuthSession;

pub async fn index(
    auth_session: AuthSession,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    match auth_session.user {
        Some(user) => {
            let res = sqlx::query_as::<_, Todo>(
                "SELECT id, content, done FROM todos WHERE user_id = (?)",
            )
            .bind(user.id)
            .fetch_all(&db)
            .await;

            match res {
                Ok(todos) => {
                    let x = Todos { todos  };
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

        None => Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
            MyCustomError("Internal server error".to_string()),
        )))),
    }
}

pub async fn new(
    auth_session: AuthSession,
    State(db): State<SqlitePool>,
    Form(todo): Form<NewTodo>,
) -> Result<impl IntoResponse, AppError> {
    match auth_session.user {
        Some(user) => {
            let _ = sqlx::query("INSERT INTO todos (content, user_id) VALUES (?, ?)")
                .bind(todo.content)
                .bind(user.id)
                .execute(&db)
                .await;

            let headers = HeaderMap::new();
            Ok((headers, Redirect::to("/todos")))
        }

        None => Err(AppError::Render(rinja_axum::Error::Custom(Box::new(
            MyCustomError("Internal server error".to_string()),
        )))),
    }
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
