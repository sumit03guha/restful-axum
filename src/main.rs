use std::sync::Arc;

use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use mongodb::{Client, Collection, Database, bson::Document};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Identity {
    name: String,
    age: u8,
}

#[tokio::main]
async fn main() {
    let db: Database = init_db().await;
    let identity_collection: Collection<Document> = init_identity_collection(db);
    let app: Router = app(identity_collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "Server up and running on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

fn app(collection: Collection<Document>) -> Router {
    let crud_router = crud_router(collection);
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .merge(crud_router)
}

async fn init_db() -> Database {
    let client: Client = Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap();

    client.database("restful_axum")
}

fn init_identity_collection(database: Database) -> Collection<Document> {
    database.collection::<Document>("identity")
}

fn crud_router(collection: Collection<Document>) -> Router {
    Router::new()
        .route("/identity", post(create_identity).get(get_all_identities))
        .route(
            "/identity/{id}",
            get(get_identity)
                .patch(update_identity)
                .delete(delete_identity),
        )
        .with_state(Arc::new(collection))
}

async fn create_identity(Json(identity): Json<Identity>) -> impl IntoResponse {
    println!(
        "Identity :: name : {}, age : {}",
        identity.name, identity.age
    );
    (StatusCode::CREATED, "Created").into_response()
}

async fn get_all_identities() -> impl IntoResponse {
    (StatusCode::FOUND, "Fetched all").into_response()
}

async fn get_identity(Path(id): Path<u8>) -> impl IntoResponse {
    (StatusCode::FOUND, "Fetched").into_response()
}

async fn update_identity(Path(id): Path<u8>) -> impl IntoResponse {
    (StatusCode::OK, "Updated").into_response()
}

async fn delete_identity(Path(id): Path<u8>) -> impl IntoResponse {
    (StatusCode::OK, "Delted").into_response()
}
