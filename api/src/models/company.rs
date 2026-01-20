use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub user_id: Uuid,
    pub requires_tax_withholdings: bool,
    pub tax_withholding_rate: Option<Decimal>,
}
