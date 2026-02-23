use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/stock_trading".into()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".into()),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        })
    }
}
