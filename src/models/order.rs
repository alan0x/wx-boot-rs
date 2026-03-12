use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::db::url_filter::JoinedOption;
use crate::schema::*;

use crate::things::order::AmountAndDiscount;

pub static ORDER_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec!["id", "trade_state", "updated_by", "created_by"]
        .into_iter()
        .map(String::from)
        .collect()
});
pub static ORDER_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);
#[derive(Identifiable, Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct Order {
    pub id: i64,

    pub order_id: String,
    pub paid_reason: String,
    pub amount: BigDecimal,
    pub trade_state: String,
    pub payment_id: String,

    pub paid_at: Option<DateTime<Utc>>,
    pub paid_by: Option<i64>,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub discount: AmountAndDiscount,
    pub actual_amount: BigDecimal,
}
#[derive(Insertable, Deserialize, Clone, Debug)]
#[diesel(table_name = orders)]
pub struct NewOrder<'a> {
    pub order_id: &'a str,
    pub paid_reason: &'a str,

    pub amount: BigDecimal,
    pub trade_state: &'a str,
    pub payment_id: &'a str,

    pub paid_at: Option<DateTime<Utc>>,
    pub paid_by: Option<i64>,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
    pub discount: AmountAndDiscount,
    pub actual_amount: BigDecimal,
}
