use axum::{
    Router,
    response::IntoResponse,
    routing::{get, post},
};

#[tokio::main]
async fn main() {
    let app = app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "Server up and running on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

fn app() -> Router {
    let crud_router = crud_router();
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .merge(crud_router)
}

fn crud_router() -> Router {
    Router::new()
        .route("/identity", post(create_identity))
        .route(
            "/identity/{id}",
            get(get_identity)
                .patch(update_identity)
                .delete(delete_identity),
        )
}

async fn create_identity() -> impl IntoResponse {}

async fn get_identity() -> impl IntoResponse {}

async fn update_identity() -> impl IntoResponse {}

async fn delete_identity() -> impl IntoResponse {}
