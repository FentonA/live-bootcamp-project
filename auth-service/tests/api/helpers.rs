use auth_service::{
    app_state::app_state::AppState, hashmap_user_store::HashmapUserStore, Application,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let app_state = AppState::new(Arc::new(RwLock::new(user_store)));
        let app = Application::build(app_state, "127.0.0.1:0")
            .await
            .expect("Could not build application");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = Client::new();

        TestApp {
            address,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("could not get http client")
    }

    // TODO: Implement helper functions for all other routes (signup, login, logout, verify-2fa, and verify-token)
    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("could not get http client")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("could not get login route")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("could not get logout route")
    }

    pub async fn verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/verify-2fa", &self.address))
            .send()
            .await
            .expect("could not get verify 2fa route")
    }

    pub async fn verify_token(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/verify-token", &self.address))
            .send()
            .await
            .expect("could not get verify token route")
    }
}

pub fn get_random_email() -> String {
    format!("{}@exanple.com", Uuid::new_v4())
}
