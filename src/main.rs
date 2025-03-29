use std::{sync::Arc, time::Duration};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::IntoResponse,
    routing::{get, post},
};
use futures::TryStreamExt;
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use mongodb::{
    Client, Collection, Database,
    bson::{doc, oid::ObjectId, to_document},
};
use serde::{Deserialize, Serialize};

const SECRET_KEY: &str = "secret_key";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Auth {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db: Database = init_db().await?;

    let identity_collection: Arc<Collection<Identity>> = init_identity_collection(&db);
    let auth_collection: Arc<Collection<Auth>> = init_auth_collection(&db);

    let app: Router = app(identity_collection, auth_collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Server up and running on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

fn app(
    identity_collection: Arc<Collection<Identity>>,
    auth_collection: Arc<Collection<Auth>>,
) -> Router {
    let crud_router = crud_router(Arc::clone(&identity_collection)).route_layer(
        from_fn_with_state(Arc::clone(&auth_collection), login_required),
    );
    let auth_router = auth_router(Arc::clone(&auth_collection));

    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route("/protected", get(protected))
        .route_layer(from_fn_with_state(
            Arc::clone(&auth_collection),
            login_required,
        ))
        .merge(crud_router)
        .merge(auth_router)
}

async fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let client: Client = Client::with_uri_str("mongodb://localhost:27017/").await?;
    let database = client.database("restful_axum");
    database.run_command(doc! { "ping" : 1 }).await?;

    Ok(database)
}

fn init_identity_collection(database: &Database) -> Arc<Collection<Identity>> {
    Arc::new(database.collection::<Identity>("identity"))
}

fn init_auth_collection(database: &Database) -> Arc<Collection<Auth>> {
    Arc::new(database.collection::<Auth>("auth"))
}

fn crud_router(collection: Arc<Collection<Identity>>) -> Router {
    Router::new()
        .route("/identity", post(create_identity).get(get_all_identities))
        .route(
            "/identity/{id}",
            get(get_identity)
                .patch(update_identity)
                .delete(delete_identity),
        )
        .with_state(Arc::clone(&collection))
}

fn auth_router(collection: Arc<Collection<Auth>>) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .with_state(Arc::clone(&collection))
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

async fn signup(
    State(collection): State<Arc<Collection<Auth>>>,
    Json(credentials): Json<Auth>,
) -> impl IntoResponse {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = match argon2.hash_password(&credentials.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response();
        }
    };

    let result = collection
        .insert_one(Auth {
            email: credentials.email,
            password: password_hash,
        })
        .await;

    match result {
        Ok(result) => {
            let response_data = ApiResponse {
                message: "Auth created".to_string(),
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

async fn login(
    State(collection): State<Arc<Collection<Auth>>>,
    Json(credentials): Json<Auth>,
) -> impl IntoResponse {
    let result = collection
        .find_one(doc! { "email" : credentials.email })
        .await;

    let credentials_doc = match result {
        Ok(Some(result)) => result,
        Ok(None) => {
            let response_data = ApiResponse {
                message: "Credential does not exist".to_string(),
                data: (),
            };
            return (StatusCode::NOT_FOUND, Json(response_data)).into_response();
        }
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response();
        }
    };

    let parsed_hash = match PasswordHash::new(&credentials_doc.password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Internal Server Error : {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response();
        }
    };

    if let Err(e) =
        Argon2::default().verify_password(&credentials.password.as_bytes(), &parsed_hash)
    {
        eprintln!("Invalid Password : {}", e);
        let response = ApiResponse {
            message: "Invalid Password".to_string(),
            data: (),
        };
        return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
    };

    let auth_token = match generate_token(&credentials_doc.email) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Internal Server Error while generating auth token: {}", e);
            let response_data = ApiResponse {
                message: "Internal Server Error".to_string(),
                data: (),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response();
        }
    };

    let response = ApiResponse {
        message: "You are logged in".to_string(),
        data: auth_token,
    };

    (StatusCode::OK, Json(response)).into_response()
}

fn generate_token(email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let my_claims = Claims {
        sub: email.to_string(),
        exp: get_current_timestamp() + Duration::new(3600, 0).as_secs(),
    };
    encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(SECRET_KEY.as_bytes()),
    )
}

async fn login_required(
    State(collection): State<Arc<Collection<Auth>>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let headers = match req.headers().get("Authorization") {
        Some(headers) => match headers.to_str() {
            Ok(headers) => headers,
            Err(e) => {
                eprintln!("Internal Server Error : {}", e);
                let response_data = ApiResponse {
                    message: "Internal Server Error".to_string(),
                    data: (),
                };
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(response_data)).into_response();
            }
        },
        None => {
            eprintln!("Missing headers");
            let response_data = ApiResponse {
                message: "Missing headers".to_string(),
                data: (),
            };
            return (StatusCode::BAD_REQUEST, Json(response_data)).into_response();
        }
    };
    let split_headers = headers.split_whitespace().collect::<Vec<&str>>();

    if split_headers.len() != 2 {
        eprintln!("Invalid Token Format");
        let response_data = ApiResponse {
            message: "Invalid Token Format".to_string(),
            data: (),
        };
        return (StatusCode::BAD_REQUEST, Json(response_data)).into_response();
    }

    let token = split_headers[1];

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => token_data,
        Err(e) => {
            eprintln!("Token Error : {}", e);
            let response_data = ApiResponse {
                message: e.to_string(),
                data: (),
            };
            return (StatusCode::BAD_REQUEST, Json(response_data)).into_response();
        }
    };

    let email = token_data.claims.sub;
    let result = collection
        .find_one(doc! {
            "email": &email
        })
        .await;

    match result {
        Ok(_) => {
            req.extensions_mut().insert(email);
            next.run(req).await
        }
        Err(err) => (StatusCode::UNAUTHORIZED, err.to_string()).into_response(),
    }
}

async fn protected(Extension(email): Extension<String>) -> impl IntoResponse {
    let response = ApiResponse {
        message: format!("Hello. You are logged in using {}", email),
        data: {},
    };

    (StatusCode::OK, Json(response))
}
