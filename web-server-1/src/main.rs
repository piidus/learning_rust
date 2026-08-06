//! Web server application built with Axum and Askama templates.

use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, Router},
};

/// The main entry point for the web server application.
///
/// Binds to `0.0.0.0:8080` (or `SERVER_ADDR`/`PORT` if set) to allow external web traffic and serves the routes.
///
/// # Parameters
/// None.
///
/// # Returns
/// None directly; runs the Tokio asynchronous runtime until process termination.
///
/// # Errors
/// Panics if binding to the network address or serving the application fails.
#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR")
        .or_else(|_| std::env::var("PORT").map(|p| format!("0.0.0.0:{}", p)))
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    // main listener
    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on http://{}", addr);

    // app start point
    let app = Router::new().route("/", get(home));

    // serve
    axum::serve(listener, app).await.unwrap();
}

/// Handler for the home page (`/`) route.
///
/// # Parameters
/// None.
///
/// # Returns
/// An HTML response rendered from the `Index` template.
///
/// # Errors
/// Panics if template rendering fails.
async fn home() -> impl IntoResponse {
    let template = Index {};
    Html(template.render().unwrap())
}

/// Template structure representing the `index.html` template.
#[derive(Template)]
#[template(path = "index.html")]
struct Index {}