use rinja_axum::Template;
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct DeleteTodo {
    pub id: ID,
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

#[derive(Template)]
#[template(path = "todos.html")]
pub struct Todos {
    pub todos: Vec<Todo>,
}
