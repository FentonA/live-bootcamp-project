use auth_service::{
    app_state::app_state::{AppState, CodeStore},
    data_stores::{
        postgres_user_store::PostgresUserStore, redis_two_fa_code_store::RedisTwoFACodeStore,
    },
    domain::data_store::TwoFaCodeStoreError,
    get_postgres_pool, get_redis_client,
    hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    hashmap_user_store::HashmapUserStore,
    hashset_banned_token_store::HashsetBannedTokenStore,
    mock_email_client::MockEmailClient,
    utils::constants::{test, DATABASE_URL, DEFAULT_REDIS_HOSTNAME},
    Application,
};
use reqwest::{cookie::Jar, Client};
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgConnection, PgPoolOptions},
    Connection, Executor, PgPool,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: Client,
    pub banned_token_store: Arc<RwLock<HashsetBannedTokenStore>>,
    pub two_fa_code_store: CodeStore,
    pub db_name: String,
    pub clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (pg_pool, db_name) = configure_postgresql().await;
        let redis_conn = configure_redis();
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn)));
        let email_client = Arc::new(MockEmailClient);
        let app_state = AppState::new(
            user_store,
            token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Could not build application");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        TestApp {
            address,
            cookie_jar,
            http_client,
            banned_token_store: token_store,
            two_fa_code_store,
            db_name,
            clean_up_called: false,
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

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("could not get logout route")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("could not get verify token route")
    }

    pub async fn clean_up(&mut self) {
        delete_database(&self.db_name).await;
        self.clean_up_called = true
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            eprintln!(
                "⚠️  TestApp dropped without clean_up! Cleaning database '{}'...",
                self.db_name
            );

            let db_name = self.db_name.clone();

            // Spawn a thread to do async cleanup
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    delete_database(&db_name).await;
                    eprintln!("✅ Successfully cleaned up database '{}'", db_name);
                });
            })
            .join()
            .ok();
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@exanple.com", Uuid::new_v4())
}

async fn configure_postgresql() -> (PgPool, String) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url.expose_secret(), &db_name).await;

    let postgresql_conn_url_with_db =
        format!("{}/{}", postgresql_conn_url.expose_secret(), db_name);

    let pool = get_postgres_pool(&Secret::new(postgresql_conn_url_with_db))
        .await
        .expect("Failed to create Postgres connection pool!");

    (pool, db_name)
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

fn configure_redis() -> Arc<RwLock<redis::Connection>> {
    let conn = get_redis_client(DEFAULT_REDIS_HOSTNAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection");

    Arc::new(RwLock::new(conn))
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url = &DATABASE_URL;

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url.expose_secret())
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}
