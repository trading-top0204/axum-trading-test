use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct Stock {
    pub symbol: String,
    pub name: String,
    pub current_price: rust_decimal::Decimal,
}
