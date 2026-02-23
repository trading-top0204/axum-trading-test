use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;
#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct PortfolioItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub shares: rust_decimal::Decimal,
}
