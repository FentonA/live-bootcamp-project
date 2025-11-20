use auth_service::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
use auth_service::get_postgres_pool;
use auth_service::utils::constants::{
    prod, DATABASE_URL, DEFAULT_REDIS_HOSTNAME, POSTMARK_AUTH_TOKEN,
};
use auth_service::utils::tracing::init_tracing;
use auth_service::{
    app_state::app_state::AppState, domain::Email, get_redis_client,
    hashset_banned_token_store::HashsetBannedTokenStore,
    services::postmark_email_client::PostmarkEmailClient, Application,
};
use reqwest::Client;
use secrecy::Secret;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to install color_eyre");
    let pg_pool = configure_postgresql().await;
    let redis_conn = configure_redis();
    let userstore = PostgresUserStore::new(pg_pool);
    let tokenstore = HashsetBannedTokenStore::new();
    let two_fa_code_store = RedisTwoFACodeStore::new(redis_conn);
    let email_client = Arc::new(configure_postmark_email_client());
    let app_state = AppState::new(
        Arc::new(RwLock::new(userstore)),
        Arc::new(RwLock::new(tokenstore)),
        Arc::new(RwLock::new(two_fa_code_store)),
        email_client,
    );
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Could not build the app");

    app.run().await.expect("Could not run app");
}

async fn configure_postgresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> Arc<RwLock<redis::Connection>> {
    let conn = get_redis_client(DEFAULT_REDIS_HOSTNAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection");

    Arc::new(RwLock::new(conn))
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(Secret::new(prod::email_client::SENDER.to_owned())).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
}
