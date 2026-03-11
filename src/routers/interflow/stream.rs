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
    show_record!(req, depot, res, Stream, interflow_streams, &mut conn);

    Ok(())
}

#[handler]
pub async fn upload(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let folder: String = req.query::<String>("folder").unwrap_or_default();
    if folder.is_empty() {
        return context::render_parse_query_error_json(res);
    }
    if !crate::utils::fs::is_safe_dir_path(&folder) {
        return Err(crate::Error::Internal("store folder is illegal".into()));
    }
    //TODO: folder name valid.
    // let cuser = current_user!(depot, res);
    // if !cuser.in_kernel {
    //     return context::render_access_denied_json(res);
    // }

    let unique = crate::utils::str_to_bool(
        &req.query::<String>("unique")
            .unwrap_or_else(|| "true".into()),
    );
    let unzip = crate::utils::str_to_bool(
        &req.query::<String>("unzip")
            .unwrap_or_else(|| "false".into()),
    );
    let store_dir = format!("stream/buckets/{}", folder);
    let data = if !unzip {
        crate::utils::fs::upload_files(req, &store_dir, unique).await?
    } else {
        crate::utils::fs::smart_upload_files(req, Some("file"), &store_dir, unique).await?
    };

    println!("upload data: {:?}", data.files.len());
    res.render(Json(data));
    Ok(())
}
#[handler]
pub async fn serve_file(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) -> AppResult<()> {
    let rest_path = crate::safe_url_path(&req.param::<String>("*path").unwrap_or_default());
    println!("rest_path: {}", rest_path);
    if rest_path.is_empty() {
        return context::render_not_found_json(res);
    }
    let file_path = join_path!("stream", "buckets", &rest_path);

    println!("file_path: {}", file_path);
    let attched_name = req.queries().get("file_name").map(String::as_str);
    crate::utils::fs::send_local_file(file_path, req.headers(), res, attched_name).await;
    Ok(())
}

#[handler]
pub fn list(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    let query = interflow_streams::table.filter(interflow_streams::is_recalled.eq(false));
    list_records!(
        req,
        res,
        Stream,
        query,
        "updated_at desc",
        STREAM_FILTER_FIELDS.clone(),
        STREAM_JOINED_OPTIONS.clone(),
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
        subject: String,
        relied_entity: Option<String>,
        relied_id: Option<i64>,

        extra: Option<Value>,
        description: Option<String>,
    }

    let cuser = current_user!(depot, res);
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;

    let stream = NewStream {
        parent_id: None,
        owner_id: cuser.id,
        kind: &pdata.kind,
        subject: &pdata.subject,
        is_recalled: false,
        is_rejected: false,
        is_resolved: false,
        is_handled: false,

        relied_entity: pdata.relied_entity.as_deref(),
        relied_id: pdata.relied_id,

        extra: pdata.extra.clone(),

        description: pdata.description.as_deref(),
        updated_by: Some(cuser.id),
        created_by: Some(cuser.id),
    };
    let stream = diesel::insert_into(interflow_streams::table)
        .values(&stream)
        .get_result::<Stream>(&mut conn)?;
    drop(conn);
    res.render(Json(stream));

    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        subject: Option<String>,
        is_recalled: Option<bool>,
        is_resolved: Option<bool>,
        is_rejected: Option<bool>,
        is_handled: Option<bool>,

        extra: Option<Value>,
    }
    #[derive(AsChangeset, Debug)]
    #[diesel(table_name = interflow_streams)]
    struct StreamData<'a> {
        subject: Option<&'a str>,
        extra: Option<Value>,
        is_recalled: Option<bool>,
        is_resolved: Option<bool>,
        is_rejected: Option<bool>,
        is_handled: Option<bool>,
        updated_by: i64,
        updated_at: DateTime<Utc>,
    }

    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);

    let mut conn = db::connect()?;
    let stream = get_record_by_param!(req, res, Stream, interflow_streams, &mut conn);

    if stream.is_recalled {
        return context::render_bad_request_json_with_detail(res, "this stream recalled");
    }

    let stream = diesel::update(&stream)
        .set(&StreamData {
            subject: pdata.subject.as_deref(),
            extra: pdata.extra.clone(),
            is_recalled: pdata.is_recalled,
            is_resolved: pdata.is_resolved,
            is_rejected: pdata.is_rejected,
            is_handled: pdata.is_handled,
            updated_by: cuser.id,
            updated_at: Utc::now(),
        })
        .get_result::<Stream>(&mut conn)?;
    res.render(Json(stream));
    Ok(())
}
