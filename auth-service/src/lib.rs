use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::serve::Serve;
use axum::Router;
use std::error::Error;
use tokio::join;
use tower_http::services::ServeDir;

pub mod routes;
use routes::signup::signup;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-token", get(verify_token))
            .route("/verify-2fa", get(verify_2fa));

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("Listening on {}", &self.address);
        self.server.await
    }
}

async fn login() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn logout() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn verify_token() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
