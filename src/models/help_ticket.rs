use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::url_filter::JoinedOption;
use crate::schema::*;

pub static HELP_TICKET_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec!["id", "owner_id", "updated_by", "created_by"]
        .into_iter()
        .map(String::from)
        .collect()
});
pub static HELP_TICKET_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);
#[derive(Identifiable, Queryable, Serialize, Clone, Debug)]
#[diesel(table_name = help_tickets)]
pub struct HelpTicket {
    pub id: i64,
    pub owner_id: i64,
    pub kind: String,
    pub subject: String,
    pub label_ids: Vec<Option<i64>>,
    pub is_recalled: bool,
    pub content: String,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub is_resolved: bool,
    pub extra: Option<Value>,
    pub is_processed: bool,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = help_tickets)]
pub struct NewHelpTicket<'a> {
    pub owner_id: i64,
    pub kind: &'a str,
    pub subject: &'a str,
    pub label_ids: Vec<Option<i64>>,
    pub is_recalled: bool,
    pub is_resolved: bool,
    pub is_processed: bool,
    pub content: &'a str,
    pub extra: Option<Value>,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}
