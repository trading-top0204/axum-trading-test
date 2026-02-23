use axum::{extract::State, Json};
use rust_decimal::Decimal;
use sqlx::Row;
use uuid::Uuid;
use sqlx::postgres::Postgres;

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    models::PlaceOrderRequest,
    state::AppState,
};

pub async fn get_portfolio(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT p.symbol, p.shares, s.name, s.current_price
        FROM portfolio p
        JOIN stocks s ON p.symbol = s.symbol
        WHERE p.user_id = $1 AND p.shares > 0
        "#,
    )
    .bind(auth.0)
    .fetch_all(&state.pool)
    .await?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let symbol: String = r.get("symbol");
            let shares: Decimal = r.get("shares");
            let name: String = r.get("name");
            let price: Decimal = r.get("current_price");
            let value = shares * price;
            serde_json::json!({
                "symbol": symbol,
                "name": name,
                "shares": shares,
                "current_price": price,
                "value": value
            })
        })
        .collect();

        let balance: Decimal = sqlx::query_scalar::<Postgres, Decimal>("SELECT balance FROM wallets WHERE user_id = $1")
        .bind(auth.0)
        .fetch_optional(&state.pool)
        .await?
        .unwrap_or(Decimal::ZERO);

    Ok(Json(serde_json::json!({
        "balance": balance,
        "holdings": items
    })))
}

pub async fn get_orders(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query(
        "SELECT id, symbol, order_type, shares, price_per_share, total_amount, created_at FROM orders WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(auth.0)
    .fetch_all(&state.pool)
    .await?;

    let orders: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id"),
                "symbol": r.get::<String, _>("symbol"),
                "order_type": r.get::<String, _>("order_type"),
                "shares": r.get::<Decimal, _>("shares"),
                "price_per_share": r.get::<Decimal, _>("price_per_share"),
                "total_amount": r.get::<Decimal, _>("total_amount"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            })
        })
        .collect();

    Ok(Json(orders))
}

pub async fn get_stocks(State(state): State<AppState>) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query("SELECT symbol, name, current_price FROM stocks")
        .fetch_all(&state.pool)
        .await?;

    let stocks: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "symbol": r.get::<String, _>("symbol"),
                "name": r.get::<String, _>("name"),
                "current_price": r.get::<Decimal, _>("current_price")
            })
        })
        .collect();

    Ok(Json(stocks))
}

pub async fn place_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<PlaceOrderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let order_type = req.order_type.to_uppercase();
    if order_type != "BUY" && order_type != "SELL" {
        return Err(AppError::Internal("order_type must be BUY or SELL".into()));
    }

    let shares = Decimal::try_from(req.shares)
        .map_err(|_| AppError::Internal("Invalid shares".into()))?;
    if shares <= Decimal::ZERO {
        return Err(AppError::Internal("Shares must be positive".into()));
    }

    let symbol = req.symbol.to_uppercase();
    let stock: Option<(Decimal,)> = sqlx::query_as("SELECT current_price FROM stocks WHERE symbol = $1")
        .bind(&symbol)
        .fetch_optional(&state.pool)
        .await?;

    let price = stock.ok_or(AppError::InvalidSymbol)?.0;
    let total = shares * price;

    let mut tx = state.pool.begin().await?;

    if order_type == "BUY" {
        let balance: Decimal = sqlx::query_scalar::<Postgres, Decimal>  ("SELECT balance FROM wallets WHERE user_id = $1 FOR UPDATE")
            .bind(auth.0)
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or(Decimal::ZERO);

        if balance < total {
            return Err(AppError::InsufficientBalance);
        }

        sqlx::query("UPDATE wallets SET balance = balance - $1, updated_at = NOW() WHERE user_id = $2")
            .bind(total)
            .bind(auth.0)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO portfolio (user_id, symbol, shares)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, symbol) DO UPDATE SET shares = portfolio.shares + $3
            "#,
        )
        .bind(auth.0)
        .bind(&symbol)
        .bind(shares)
        .execute(&mut *tx)
        .await?;
    } else {
        let current_shares: Option<Decimal> = sqlx::query_scalar(
            "SELECT shares FROM portfolio WHERE user_id = $1 AND symbol = $2 FOR UPDATE",
        )
        .bind(auth.0)
        .bind(&symbol)
        .fetch_optional(&mut *tx)
        .await?;

        let current = current_shares.unwrap_or(Decimal::ZERO);
        if current < shares {
            return Err(AppError::InsufficientShares);
        }

        sqlx::query("UPDATE portfolio SET shares = shares - $1 WHERE user_id = $2 AND symbol = $3")
            .bind(shares)
            .bind(auth.0)
            .bind(&symbol)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE wallets SET balance = balance + $1, updated_at = NOW() WHERE user_id = $2")
            .bind(total)
            .bind(auth.0)
            .execute(&mut *tx)
            .await?;
    }

    let order_id: Uuid = sqlx::query_scalar(
        "INSERT INTO orders (user_id, symbol, order_type, shares, price_per_share, total_amount) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(auth.0)
    .bind(&symbol)
    .bind(&order_type)
    .bind(shares)
    .bind(price)
    .bind(total)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "order_id": order_id,
        "symbol": symbol,
        "order_type": order_type,
        "shares": shares,
        "price_per_share": price,
        "total_amount": total
    })))
}
