use auth_service::Application;
use reqwest::Client;

pub struct TestApp {
    pub address: String,
    pub http_client: Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.1:0")
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
    pub async fn signup(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .send()
            .await
            .expect("could not get http client")
    }

    pub async fn login(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
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
