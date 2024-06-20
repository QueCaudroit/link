use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Redirect,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Serialize, sqlx::FromRow)]
struct Link {
    id: i32,
    link: String,
    count: i32,
}

#[derive(Deserialize)]
struct LinkCreationRequest {
    link: String,
}

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .unwrap(),
    );
    let app = Router::new()
        .route("/links", get(get_links).post(post_link))
        .route("/links/:id", delete(delete_link))
        .route("/:id", get(get_url))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_links(State(pool): State<Arc<Pool<Postgres>>>) -> Json<Vec<Link>> {
    let links: Vec<Link> = sqlx::query_as("select * from links")
        .fetch_all(pool.as_ref())
        .await
        .unwrap();
    Json(links)
}

async fn post_link(
    State(pool): State<Arc<Pool<Postgres>>>,
    Json(body): Json<LinkCreationRequest>,
) -> Json<Link> {
    let link: Link = sqlx::query_as("insert into links(link, count) values ($1, 0) returning *")
        .bind(body.link)
        .fetch_one(pool.as_ref())
        .await
        .unwrap();
    Json(link)
}

async fn delete_link(State(pool): State<Arc<Pool<Postgres>>>, Path(id): Path<i32>) -> Json<Link> {
    let link: Link = sqlx::query_as("delete from links where id = $1 returning *")
        .bind(id)
        .fetch_one(pool.as_ref())
        .await
        .unwrap();
    Json(link)
}

async fn get_url(State(pool): State<Arc<Pool<Postgres>>>, Path(id): Path<i32>) -> Redirect {
    let link: Link = sqlx::query_as("select * from links where id = $1 limit 1")
        .bind(id)
        .fetch_one(pool.as_ref())
        .await
        .unwrap();
    sqlx::query("update links set count = count + 1 where id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .unwrap();
    Redirect::permanent(&link.link)
}
