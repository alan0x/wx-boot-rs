use once_cell::sync::Lazy;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::url_filter::JoinedOption;
use crate::schema::*;

pub static STREAM_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "id",
        "owner_id",
        "kind",
        "parent_id",
        "relied_entity",
        "relied_id",
        "subject",
        "is_recalled",
        "is_rejected",
        "is_resolved",
        "updated_by",
        "created_by",
    ]
    .into_iter()
    .map(String::from)
    .collect()
});
pub static STREAM_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);

#[derive(Identifiable, Queryable, Serialize, Clone, Debug)]
#[diesel(table_name = interflow_streams)]
pub struct Stream {
    pub id: i64,
    pub owner_id: i64,
    pub kind: String,
    pub parent_id: Option<i64>,
    pub relied_entity: Option<String>,
    pub relied_id: Option<i64>,
    pub subject: String,
    pub is_recalled: bool,
    pub is_rejected: bool,
    pub is_handled: bool,
    pub is_resolved: bool,

    pub extra: Option<Value>,
    pub description: Option<String>,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Clone, Debug)]
#[diesel(table_name = interflow_streams)]
pub struct NewStream<'a> {
    pub owner_id: i64,
    pub kind: &'a str,
    pub parent_id: Option<i64>,
    pub relied_entity: Option<&'a str>,
    pub relied_id: Option<i64>,
    pub subject: &'a str,
    pub is_recalled: bool,
    pub is_rejected: bool,
    pub is_handled: bool,
    pub is_resolved: bool,

    pub extra: Option<Value>,
    pub description: Option<&'a str>,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}

pub static THREAD_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "id",
        "owner_id",
        "origin_id",
        "stream_id",
        "replied_id",
        "is_primary",
        "is_recalled",
        "is_rejected",
        "is_resolved",
        "is_handled",
        "content",
        "kind",
        "updated_by",
        "created_by",
    ]
    .into_iter()
    .map(String::from)
    .collect()
});
pub static THREAD_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);

#[derive(Identifiable, Insertable, Queryable, Serialize, Clone, Debug)]
#[diesel(table_name = interflow_threads)]
pub struct Thread {
    pub id: i64,
    pub owner_id: i64,
    pub kind: String,
    pub origin_id: Option<i64>,
    pub stream_id: i64,
    pub replied_id: Option<i64>,
    pub at_ids: Vec<Option<i64>>,
    pub is_primary: bool,
    pub is_recalled: bool,
    pub is_rejected: bool,
    pub is_handled: bool,
    pub is_resolved: bool,
    pub is_internal: bool,
    pub content: Value,
    pub attachments: Value,

    pub extra: Option<Value>,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = interflow_threads)]
pub struct NewThread<'a> {
    pub owner_id: i64,
    pub kind: &'a str,
    pub origin_id: Option<i64>,
    pub stream_id: i64,
    pub replied_id: Option<i64>,
    pub at_ids: Vec<Option<i64>>,
    pub is_primary: bool,
    pub is_recalled: bool,
    pub is_rejected: bool,
    pub is_handled: bool,
    pub is_resolved: bool,
    pub is_internal: bool,
    pub content: Value,
    pub attachments: Value,

    pub extra: Option<Value>,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}
