use axum::{routing::get, Router, response::IntoResponse};
use sqlx::{sqlite::SqlitePoolOptions,  Pool, Sqlite};
use tower_livereload::LiveReloadLayer;

use errors::AppError;

mod todos;
mod errors;


use todos::handlers::{index, update, delete, new};

async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS user (id integer primary key AUTOINCREMENT, email text not null unique, password text not null)")
        .execute(pool).await?;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS todos (id integer primary key AUTOINCREMENT, content text, done boolean default false, user_id text, FOREIGN KEY (user_id) REFERENCES user(id))")
        .execute(pool).await?;
    Ok(())
}


async fn not_found() ->  impl IntoResponse  {
    AppError::NotFound
}

#[tokio::main]
async fn main() {
    let conn = SqlitePoolOptions::new()
        .connect("todos.sqlite?mode=rwc")
        .await
        .unwrap();

    migrate(&conn).await.unwrap();

    let app = Router::new()
        // .route("/", get(route_to_todos))
        // .route("/login", get(render_login).post(process_login))
        // .route("/logout", get(process_logout))
        .route("/todos", get(index).post(new).put(update).delete(delete))
        .with_state(conn)
        .layer(LiveReloadLayer::new())
        .fallback(not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


// async fn process_logout(cookie: CookieManager) -> impl IntoResponse {
//     cookie.add(Cookie::new("session", ""));
//     Redirect::to("/login")
// }

// async fn route_to_todos(cookie: CookieManager) -> impl IntoResponse {
//     cookie.add(Cookie::new("session", ""));
//     Redirect::to("/todos")
// }



