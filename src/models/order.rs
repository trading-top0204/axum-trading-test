use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;
#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub order_type: String,
    pub shares: rust_decimal::Decimal,
    pub price_per_share: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub order_type: String,  // "BUY" or "SELL"
    pub shares: f64,
}
