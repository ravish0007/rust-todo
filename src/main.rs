use axum::{
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_login::{
    login_required,
    tower_sessions::{Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use time::Duration;
use tower_livereload::LiveReloadLayer;
use tower_sessions::cookie::Key;
use tower_sessions_sqlx_store::SqliteStore;

use errors::AppError;

mod auth;
mod errors;
mod todos;
mod users;

use todos::handlers::{delete, index, new, update};
use users::Backend;

async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS users (id integer primary key AUTOINCREMENT, username text not null unique, password text not null)")
        .execute(pool).await?;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS todos (id integer primary key AUTOINCREMENT, content text, done boolean default false, user_id integer, FOREIGN KEY (user_id) REFERENCES users(id))")
        .execute(pool).await?;
    Ok(())
}

async fn not_found() -> impl IntoResponse {
    AppError::NotFound
}

async fn get_index() -> Result<impl IntoResponse, AppError> {
    let headers = HeaderMap::new();
    Ok((headers, Redirect::to("/todos")))
}

#[tokio::main]
async fn main() {
    let conn = SqlitePoolOptions::new()
        .connect("todos.sqlite?mode=rwc")
        .await
        .unwrap();

    migrate(&conn).await.unwrap();

    let session_store = SqliteStore::new(conn.clone());
    let _ = session_store.migrate().await;

    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    let backend = Backend::new(conn.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        .route("/", get(get_index))
        .route("/todos", get(index).post(new).put(update).delete(delete))
        .route_layer(login_required!(Backend, login_url = "/login"))
        .with_state(conn.clone());

    let auth_router = auth::router(conn);

    let merged_app = app
        .merge(auth_router)
        .fallback(not_found)
        .layer(auth_layer)
        .layer(LiveReloadLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, merged_app).await.unwrap();
}
