use chrono::{DateTime, Utc};
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::db;
use crate::models::*;
use crate::schema::*;
use crate::{context, AppResult};

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path).get(list).post(create).push(
        Router::with_path(r"{id:\d+}")
            .get(show)
            .patch(update)
            .delete(delete),
    )
}

#[handler]
pub async fn show(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    // let cuser = current_user!(depot, res);
    // if !cuser.in_kernel {
    //     return context::render_access_denied_json(res);
    // }

    let mut conn = db::connect()?;
    show_record!(req, depot, res, Notification, notifications, &mut conn);
    Ok(())
}
#[handler]
pub async fn list(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    // let cuser = current_user!(depot, res);
    // if !cuser.in_kernel {
    //     return context::render_access_denied_json(res);
    // }

    let mut conn = db::connect()?;
    list_records!(
        req,
        res,
        Notification,
        notifications::table,
        "updated_at desc",
        NOTIFICATION_FILTER_FIELDS.clone(),
        NOTIFICATION_JOINED_OPTIONS.clone(),
        ID_SUBJECT_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}

#[handler]
pub async fn delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    let mut conn = db::connect()?;
    let record = get_record_by_param!(req, res, Notification, notifications, &mut conn);

    diesel::delete(notifications::table.filter(notifications::id.eq(record.id)))
        .execute(&mut conn)?;

    context::render_done_json(res)
}

#[handler]
pub async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        owner_id: i64,
        kind: String,
        subject: String,
        body: String,
        extra: Value,
    }

    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;

    let notification = NewNotification {
        owner_id: pdata.owner_id,
        sender_id: Some(cuser.id),

        kind: &pdata.kind,
        subject: &pdata.subject,
        body: &pdata.body,
        extra: pdata.extra.clone(),
        updated_by: Some(cuser.id),
        created_by: Some(cuser.id),
    };
    let notification = diesel::insert_into(notifications::table)
        .values(&notification)
        .get_result::<Notification>(&mut conn)?;
    drop(conn);
    res.render(Json(notification));

    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        subject: Option<String>,
        body: Option<String>,
        kind: Option<String>,
    }
    #[derive(AsChangeset, Debug)]
    #[diesel(table_name = notifications)]
    struct NotificationData<'a> {
        subject: Option<&'a str>,
        body: Option<&'a str>,
        kind: Option<&'a str>,
        updated_by: i64,
        updated_at: DateTime<Utc>,
    }

    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }

    let mut conn = db::connect()?;
    let notification = get_record_by_param!(req, res, Notification, notifications, &mut conn);

    let notification = diesel::update(&notification)
        .set(&NotificationData {
            subject: pdata.subject.as_deref(),
            body: pdata.body.as_deref(),
            kind: pdata.kind.as_deref(),
            updated_by: cuser.id,
            updated_at: Utc::now(),
        })
        .get_result::<Notification>(&mut conn)?;
    res.render(Json(notification));
    Ok(())
}
