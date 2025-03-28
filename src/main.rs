use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use futures::TryStreamExt;
use mongodb::{
    Client, Collection, Database,
    bson::{doc, oid::ObjectId, to_document},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Identity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    age: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentityUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    age: Option<u8>,
}

impl IdentityUpdate {
    fn validate(&self) -> Result<(), String> {
        if self.age.is_none() && self.name.is_none() {
            Err("Either age or name must be provided.".to_string())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    message: String,
    data: T,
}

struct Auth {
    email: String,
    password: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db: Database = init_db().await?;
    let identity_collection: Collection<Identity> = init_identity_collection(db);
    let auth_collection: Collection<Auth> = init_auth_collection(db);
    let app: Router = app(identity_collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Server up and running on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

fn app(collection: Collection<Identity>) -> Router {
    let crud_router = crud_router(collection);
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .merge(crud_router)
}

async fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let client: Client = Client::with_uri_str("mongodb://localhost:27017/").await?;
    let database = client.database("restful_axum");
    database.run_command(doc! { "ping" : 1 }).await?;

    Ok(database)
}

fn init_identity_collection(database: Database) -> Collection<Identity> {
    database.collection::<Identity>("identity")
}

fn init_auth_collection(database: Database) -> Collection<Auth> {
    database.collection::<Auth>("auth")
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

fn auth_router(collection: Collection<Auth>)

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
        Ok(result) => {
            let response_data = ApiResponse {
                message: "Identity created".to_string(),
                data: result.inserted_id,
            };
            (StatusCode::CREATED, Json(response_data)).into_response()
        }
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response()
        }
    }
}

async fn get_all_identities(
    State(collection): State<Arc<Collection<Identity>>>,
) -> impl IntoResponse {
    match collection.find(doc! {}).await {
        Ok(cursor) => match cursor.try_collect::<Vec<Identity>>().await {
            Ok(result) => {
                let response_data = ApiResponse {
                    message: "Fetched all identities".to_string(),
                    data: result,
                };

                (StatusCode::OK, Json(response_data)).into_response()
            }
            Err(e) => {
                eprintln!("Internal Server Error : {}", e);
                let response_data = ApiResponse {
                    message: "Internal Server Error".to_string(),
                    data: Vec::<Identity>::new(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response()
            }
        },
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: Vec::<Identity>::new(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response()
        }
    }
}

async fn get_identity(
    State(collection): State<Arc<Collection<Identity>>>,
    Path(id): Path<ObjectId>,
) -> impl IntoResponse {
    let result = collection
        .find_one(doc! {
            "_id": id
        })
        .await;

    match result {
        Ok(Some(identity)) => {
            let response_data = ApiResponse {
                message: "Fetched".to_string(),
                data: identity,
            };

            (StatusCode::OK, Json(response_data)).into_response()
        }
        Ok(None) => {
            let response_data = ApiResponse {
                message: "Identity does not exist".to_string(),
                data: (),
            };
            (StatusCode::NOT_FOUND, Json(response_data)).into_response()
        }
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response()
        }
    }
}

async fn update_identity(
    State(collection): State<Arc<Collection<Identity>>>,
    Path(id): Path<ObjectId>,
    Json(id_data): Json<IdentityUpdate>,
) -> impl IntoResponse {
    if let Err(e) = id_data.validate() {
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    let filter = doc! {
        "_id":id
    };

    let update_data = match to_document(&id_data) {
        Ok(document) => document,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let update = doc! { "$set": update_data };
    let result = collection.update_one(filter, update).await;

    match result {
        Ok(data) => {
            if data.matched_count == 0 {
                let response_data = ApiResponse {
                    message: "Document not found".to_string(),
                    data: (),
                };
                (StatusCode::NOT_FOUND, Json(response_data)).into_response()
            } else if data.modified_count == 0 {
                let response_data = ApiResponse {
                    message: "No changes made".to_string(),
                    data: (),
                };
                (StatusCode::OK, Json(response_data)).into_response()
            } else {
                let response_data = ApiResponse {
                    message: "Updated".to_string(),
                    data: (),
                };
                (StatusCode::OK, Json(response_data)).into_response()
            }
        }
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response()
        }
    }
}

async fn delete_identity(
    State(collection): State<Arc<Collection<Identity>>>,
    Path(id): Path<ObjectId>,
) -> impl IntoResponse {
    let filter = doc! {"_id":id};

    let result = collection.delete_one(filter).await;

    match result {
        Ok(result_data) => {
            if result_data.deleted_count == 1 {
                let response_data = ApiResponse {
                    message: "Deleted".to_string(),
                    data: (),
                };
                (StatusCode::OK, Json(response_data)).into_response()
            } else {
                let response_data = ApiResponse {
                    message: "Document not found".to_string(),
                    data: (),
                };
                (StatusCode::NOT_FOUND, Json(response_data)).into_response()
            }
        }
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response()
        }
    }
}
