use auth_service::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
use auth_service::get_postgres_pool;
use auth_service::utils::constants::{DATABASE_URL, DEFAULT_REDIS_HOSTNAME};
use auth_service::{
    app_state::app_state::AppState,
    domain::{email_client, EmailClient},
    get_redis_client,
    hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    hashmap_user_store::HashmapUserStore,
    hashset_banned_token_store::HashsetBannedTokenStore,
    mock_email_client::MockEmailClient,
    utils::constants::prod,
    Application,
};
use sqlx::sqlx_macros;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;
    let redis_conn = configure_redis();
    let userstore = PostgresUserStore::new(pg_pool);
    let tokenstore = HashsetBannedTokenStore::new();
    let two_fa_code_store = RedisTwoFACodeStore::new(redis_conn);
    let email_client = Arc::new(MockEmailClient);
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
