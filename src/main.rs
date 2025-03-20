use std::sync::Arc;

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use futures::TryStreamExt;
use mongodb::{
    Client, Collection, Database,
    bson::{doc, oid::ObjectId},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string_pretty};

#[derive(Debug, Serialize, Deserialize)]
struct Identity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    age: u8,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    message: String,
    data: T,
}

#[tokio::main]
async fn main() {
    let db: Database = init_db().await;
    let identity_collection: Collection<Identity> = init_identity_collection(db);
    let app: Router = app(identity_collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "Server up and running on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

fn app(collection: Collection<Identity>) -> Router {
    let crud_router = crud_router(collection);
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .merge(crud_router)
}

async fn init_db() -> Database {
    let client: Client = Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap();
    let database = client.database("restful_axum");
    database
        .run_command(doc! { "ping" : 1 })
        .await
        .expect("Failed to ping DB");

    database
}

fn init_identity_collection(database: Database) -> Collection<Identity> {
    database.collection::<Identity>("identity")
}

fn crud_router(collection: Collection<Identity>) -> Router {
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

async fn create_identity(
    State(id_collection): State<Arc<Collection<Identity>>>,
    Json(identity): Json<Identity>,
) -> impl IntoResponse {
    let result = id_collection
        .insert_one(Identity {
            id: None,
            name: identity.name,
            age: identity.age,
        })
        .await;

    match result {
        Ok(result) => (
            StatusCode::CREATED,
            format!("Created with id : {}", result.inserted_id),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_all_identities(
    State(collection): State<Arc<Collection<Identity>>>,
) -> impl IntoResponse {
    let cursor = collection.find(doc! {}).await.unwrap();

    let result: Vec<Identity> = cursor.try_collect().await.unwrap();

    (StatusCode::FOUND, format!("Fetched : {:?}", result)).into_response()
}

async fn get_identity(
    State(collection): State<Arc<Collection<Identity>>>,
    Path(id): Path<ObjectId>,
) -> impl IntoResponse {
    let result = collection
        .find_one(doc! {
            "_id": id
        })
        .await
        .unwrap();

    match result {
        Some(identity) => {
            let id_res = ApiResponse {
                message: "Fetched".to_string(),
                data: identity,
            };
            let res_data = to_string_pretty(&id_res).unwrap();
            (StatusCode::FOUND, res_data).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!(
                {
                    "message": "Does not exist"
                }
            )),
        )
            .into_response(),
    }
}

async fn update_identity(Path(id): Path<u8>) -> impl IntoResponse {
    (StatusCode::OK, "Updated").into_response()
}

async fn delete_identity(Path(id): Path<u8>) -> impl IntoResponse {
    (StatusCode::OK, "Delted").into_response()
}
