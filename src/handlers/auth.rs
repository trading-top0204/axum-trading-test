use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    config::Config,
    error::AppError,
    middleware::auth::Claims,
    models::{AuthResponse, LoginRequest, RegisterRequest},
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let password_hash = hash(&req.password, DEFAULT_COST).map_err(|e| AppError::Internal(e.to_string()))?;

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.constraint().is_some() {
                return AppError::UserExists;
            }
        }
        AppError::Database(e)
    })?;

    sqlx::query("INSERT INTO wallets (user_id, balance) VALUES ($1, 10000.00)")
        .bind(user_id)
        .execute(&state.pool)
        .await?;

    let token = create_jwt(user_id, &req.email, &state.config)?;

    Ok(Json(AuthResponse {
        token,
        user_id,
        email: req.email,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.pool)
        .await?;

    let (user_id, password_hash): (Uuid, String) = match row {
        Some(r) => (r.get("id"), r.get("password_hash")),
        None => return Err(AppError::InvalidCredentials),
    };

    if !verify(&req.password, &password_hash).map_err(|e| AppError::Internal(e.to_string()))? {
        return Err(AppError::InvalidCredentials);
    }

    let token = create_jwt(user_id, &req.email, &state.config)?;

    Ok(Json(AuthResponse {
        token,
        user_id,
        email: req.email,
    }))
}

fn create_jwt(user_id: Uuid, email: &str, config: &Config) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::hours(config.jwt_expiry_hours)).timestamp();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}
