use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
}
