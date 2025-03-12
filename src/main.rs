use axum::{Router, routing::get};

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
    Router::new().route("/", get(|| async { "Hello World" }))
}
