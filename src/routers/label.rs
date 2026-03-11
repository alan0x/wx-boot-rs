use chrono::{DateTime, Utc};
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;

use crate::db;
use crate::models::*;
use crate::schema::*;
use crate::{context, AppResult};

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path).get(list).post(create).push(
        Router::with_path(r"<id:/\d+/>")
            .get(show)
            .patch(update)
            .put(update)
            .delete(delete),
    )
}

#[handler]
pub async fn show(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let id = get_id_param!(req, res);
    let mut conn = db::connect()?;
    let mut query = labels::table.filter(labels::id.eq(id)).into_boxed();
    if !cuser.in_kernel {
        query = query.filter(
            labels::owner_id
                .eq(crate::SYSTEM_LABEL_OWNER_ID)
                .or(labels::owner_id.eq(cuser.id)),
        );
    }
    let record = query.first::<Label>(&mut conn).ok();
    if let Some(record) = record {
        res.render(Json(record));
    } else {
        return context::render_not_found_json(res);
    }
    Ok(())
}

#[handler]
pub async fn delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let record = get_record_by_param!(req, res, Label, labels, &mut conn);
    if !cuser.in_kernel && record.owner_id != cuser.id {
        return context::render_access_denied_json(res);
    }
    diesel::delete(labels::table.filter(labels::id.eq(record.id))).execute(&mut conn)?;
    context::render_done_json(res)
}

#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let mut query = labels::table.into_boxed();
    if !cuser.in_kernel {
        query = query.filter(
            labels::owner_id
                .eq(crate::SYSTEM_LABEL_OWNER_ID)
                .or(labels::owner_id.eq(cuser.id)),
        );
    }
    list_records!(
        req, res, Label, query, "updated_at desc",
        LABEL_FILTER_FIELDS.clone(), LABEL_JOINED_OPTIONS.clone(),
        ID_NAME_SEARCH_TMPL, &mut conn
    );
    Ok(())
}

#[handler]
pub async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        name: String,
    }
    let cuser = current_user!(depot, res);
    let pdata = parse_posted_data!(req, res, PostedData);
    let name = pdata.name.trim().to_string();

    if !cuser.in_kernel {
        if crate::utils::validator::validate_custom_label_name(&name).is_err() {
            return Err(StatusError::bad_request()
                .with_summary("分类名格式错误")
                .with_detail("分类名必须为1-4个汉字")
                .into());
        }
    }
    if name == crate::UNCLASSIFIED_LABEL_NAME {
        return Err(StatusError::conflict()
            .with_summary("分类名冲突")
            .with_detail("该名称为系统保留分类")
            .into());
    }
    let mut conn = db::connect()?;
    let query = labels::table
        .filter(labels::owner_id.eq(cuser.id))
        .filter(labels::name.eq(&name));
    if diesel_exists!(query, &mut conn) {
        return Err(StatusError::conflict()
            .with_summary("分类名冲突")
            .with_detail("你已经创建过同名分类")
            .into());
    }
    let label = NewLabel {
        owner_id: cuser.id,
        name: &name,
        updated_by: Some(cuser.id),
        created_by: Some(cuser.id),
    };
    let label = diesel::insert_into(labels::table)
        .values(&label)
        .get_result::<Label>(&mut conn)?;
    drop(conn);
    res.render(Json(label));
    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        name: Option<String>,
    }
    #[derive(AsChangeset, Debug)]
    #[diesel(table_name = labels)]
    struct LabelData<'a> {
        name: Option<&'a str>,
        updated_by: i64,
        updated_at: DateTime<Utc>,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let label = get_record_by_param!(req, res, Label, labels, &mut conn);
    if !cuser.in_kernel && label.owner_id != cuser.id {
        return context::render_access_denied_json(res);
    }
    let name = pdata.name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return context::render_parse_data_error_json(res);
    }
    if !cuser.in_kernel {
        if crate::utils::validator::validate_custom_label_name(&name).is_err() {
            return Err(StatusError::bad_request()
                .with_summary("分类名格式错误")
                .with_detail("分类名必须为1-4个汉字")
                .into());
        }
    }
    if label.owner_id == crate::SYSTEM_LABEL_OWNER_ID && !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    if name == crate::UNCLASSIFIED_LABEL_NAME {
        return Err(StatusError::conflict()
            .with_summary("分类名冲突")
            .with_detail("该名称为系统保留分类")
            .into());
    }
    let query = labels::table
        .filter(labels::owner_id.eq(label.owner_id))
        .filter(labels::name.eq(&name))
        .filter(labels::id.ne(label.id));
    if diesel_exists!(query, &mut conn) {
        return Err(StatusError::conflict()
            .with_summary("分类名冲突")
            .with_detail("同一用户下不能有重复分类名")
            .into());
    }
    let label = diesel::update(&label)
        .set(&LabelData {
            name: Some(&name),
            updated_by: cuser.id,
            updated_at: Utc::now(),
        })
        .get_result::<Label>(&mut conn)?;
    res.render(Json(label));
    Ok(())
}
