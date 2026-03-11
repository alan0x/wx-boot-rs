use chrono::{DateTime, Utc};
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::db;
use crate::models::interflow::*;
use crate::models::*;
use crate::schema::*;
use crate::{context, AppResult};

#[handler]
pub fn show(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    show_record!(req, depot, res, Thread, interflow_threads, &mut conn);

    Ok(())
}

#[handler]
pub fn list(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    let query = interflow_threads::table.filter(interflow_threads::is_recalled.eq(false));
    list_records!(
        req,
        res,
        Thread,
        query,
        "updated_at desc",
        THREAD_FILTER_FIELDS.clone(),
        THREAD_JOINED_OPTIONS.clone(),
        ID_SUBJECT_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}
#[handler]
pub async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        kind: String,
        origin_id: Option<i64>,
        stream_id: i64,
        replied_id: Option<i64>,
        at_ids: Vec<Option<i64>>,
        is_primary: bool,
        is_internal: bool,
        content: Value,
        attachments: Value,

        extra: Option<Value>,
    }

    let cuser = current_user!(depot, res);
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;

    let stream = interflow_streams::table
        .find(pdata.stream_id)
        .first::<Stream>(&mut conn)?;

    let thread = diesel::insert_into(interflow_threads::table)
        .values(&NewThread {
            owner_id: cuser.id,
            kind: &pdata.kind,
            origin_id: pdata.origin_id,
            stream_id: stream.id,
            replied_id: pdata.replied_id,
            at_ids: pdata.at_ids,
            is_primary: pdata.is_primary,
            is_recalled: false,
            is_rejected: false,
            is_handled: false,
            is_resolved: false,
            is_internal: pdata.is_internal,
            content: pdata.content.clone(),
            attachments: pdata.attachments.clone(),
            extra: pdata.extra.clone(),
            updated_by: Some(cuser.id),
            created_by: Some(cuser.id),
        })
        .get_result::<Thread>(&mut conn)?;
    drop(conn);
    res.render(Json(thread));

    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        content: Option<Value>,
        attachments: Option<Value>,
        extra: Option<Value>,

        is_recalled: Option<bool>,
        is_rejected: Option<bool>,
        is_handled: Option<bool>,
        is_resolved: Option<bool>,
    }
    #[derive(AsChangeset, Debug)]
    #[diesel(table_name = interflow_threads)]
    struct ThreadData {
        content: Option<Value>,
        attachments: Option<Value>,
        extra: Option<Value>,

        is_recalled: Option<bool>,
        is_rejected: Option<bool>,
        is_handled: Option<bool>,
        is_resolved: Option<bool>,

        updated_by: i64,
        updated_at: DateTime<Utc>,
    }

    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);

    let mut conn = db::connect()?;
    let thread = get_record_by_param!(req, res, Thread, interflow_threads, &mut conn);

    if thread.is_recalled {
        return context::render_bad_request_json_with_detail(res, "this thread recalled");
    }

    let thread = diesel::update(&thread)
        .set(&ThreadData {
            content: pdata.content.clone(),
            attachments: pdata.attachments.clone(),
            extra: pdata.extra.clone(),

            is_recalled: pdata.is_recalled,
            is_rejected: pdata.is_rejected,
            is_handled: pdata.is_handled,
            is_resolved: pdata.is_resolved,

            updated_by: cuser.id,
            updated_at: Utc::now(),
        })
        .get_result::<Thread>(&mut conn)?;
    res.render(Json(thread));
    Ok(())
}
