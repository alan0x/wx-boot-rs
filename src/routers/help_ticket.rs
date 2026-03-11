use chrono::{DateTime, Utc};
use diesel::prelude::*;
use reqwest::Client;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::db;
use crate::models::help_ticket::*;
use crate::models::*;
use crate::schema::*;
use crate::{context, utils, AppResult};

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .get(list)
        .post(create)
        .push(Router::with_path("buckets").post(upload))
        .push(Router::with_path(r"buckets/{**path}").get(serve_file))
        .push(
            Router::with_path(r"{id:\d+}")
                .get(show)
                .patch(update)
                .put(update)
                .push(Router::with_path("set_recalled").post(set_recalled)),
        )
}
#[handler]
pub async fn upload(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    // let cuser = current_user!(depot, res);
    // if !cuser.in_kernel {
    //     return context::render_access_denied_json(res);
    // }
    // drop(conn);

    let unique = utils::str_to_bool(
        &req.query::<String>("unique")
            .unwrap_or_else(|| "true".into()),
    );
    let unzip = utils::str_to_bool(
        &req.query::<String>("unzip")
            .unwrap_or_else(|| "false".into()),
    );
    let store_dir = format!("tickets/buckets");
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
    let rest_path = crate::safe_url_path(&req.param::<String>("path").unwrap_or_default());
    if rest_path.is_empty() {
        return context::render_not_found_json(res);
    }
    let file_path = join_path!("tickets", "buckets", &rest_path);

    println!("file_path: {}", file_path);
    let attched_name = req.queries().get("file_name").map(String::as_str);
    utils::fs::send_local_file(file_path, req.headers(), res, attched_name).await;
    Ok(())
}

#[handler]
pub async fn show(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    show_record!(req, depot, res, HelpTicket, help_tickets, &mut conn);

    Ok(())
}
#[handler]
pub async fn list(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    let query = help_tickets::table.filter(help_tickets::is_recalled.eq(false));
    list_records!(
        req,
        res,
        HelpTicket,
        query,
        "updated_at desc",
        HELP_TICKET_FILTER_FIELDS.clone(),
        HELP_TICKET_JOINED_OPTIONS.clone(),
        ID_SUBJECT_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}

#[handler]
pub async fn set_recalled(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        value: bool,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let help_ticket = get_record_by_param!(req, res, HelpTicket, help_tickets, &mut conn);
    if !cuser.in_kernel && cuser.id != help_ticket.owner_id {
        return context::render_access_denied_json(res);
    }
    diesel::update(&help_ticket)
        .set(help_tickets::is_recalled.eq(pdata.value))
        .execute(&mut conn)?;
    context::render_done_json(res)
}

#[derive(Serialize, Debug)]
struct DingTalkBootLinkContent {
    text: String,
    title: String,
    #[serde(rename = "messageUrl")]
    message_url: String,
}
#[derive(Serialize, Debug)]
struct DingTalkBootPayload {
    #[serde(rename = "msgtype")]
    msg_type: String,
    link: DingTalkBootLinkContent,
}

#[handler]
pub async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        kind: String,
        subject: String,
        #[serde(default)]
        label_ids: Vec<Option<i64>>,
        content: String,
        extra: Option<Value>,
    }

    let cuser = current_user!(depot, res);
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;

    let help_ticket = NewHelpTicket {
        owner_id: cuser.id,
        kind: &pdata.kind,
        subject: &pdata.subject,
        label_ids: pdata.label_ids.clone(),
        is_recalled: false,
        is_resolved: false,
        is_processed: false,
        content: &pdata.content,
        extra: pdata.extra.clone(),
        updated_by: Some(cuser.id),
        created_by: Some(cuser.id),
    };
    let help_ticket = diesel::insert_into(help_tickets::table)
        .values(&help_ticket)
        .get_result::<HelpTicket>(&mut conn)?;
    drop(conn);
    res.render(Json(help_ticket));

    // 向钉钉机器人发送post请求
    println!("=============向钉钉机器人发送post请求");
    let client = Client::new();
    let key_word = "运维事件";
    let payload = DingTalkBootPayload {
        msg_type: String::from("link"),
        link: DingTalkBootLinkContent {
            title: format!("[{}]: {} 提交工单", key_word, cuser.display_name),
            text: format!("{}, {}", &pdata.subject, &pdata.content),
            message_url: format!("https://pitun.cc/admin/help_tickets"),
        },
    };

    let res = client.post("https://oapi.dingtalk.com/robot/send?access_token=5ad8a5746b6812082f359ff9c06c339567bd11a3e6a137eeb15651a67c2fa62c").body(serde_json::to_string(&payload).unwrap()).header("Content-Type", "application/json");
    println!("res: {:?}", res);
    let res = res.send().await;
    println!("res: {:?}", res);
    Ok(())
}

mod test_dingtalk_boot {
    use super::*;
    #[tokio::test]
    async fn test_dingtalk_boot() {
        // 向钉钉机器人发送post请求
        let client = Client::new();
        let key_word = "运维事件";
        let payload = DingTalkBootPayload {
            msg_type: String::from("link"),
            link: DingTalkBootLinkContent {
                title: format!("[{}]: {} 提交工单", key_word, "yy"),
                text: format!("{}, {}", "书籍缺失", "9787544796057"),
                message_url: format!("https://pitun.cc/admin/help_tickets"),
            },
        };
        let res = client.post("https://oapi.dingtalk.com/robot/send?access_token=5ad8a5746b6812082f359ff9c06c339567bd11a3e6a137eeb15651a67c2fa62c").body(serde_json::to_string(&payload).unwrap()).header("Content-Type", "application/json");
        println!("res: {:?}", res);
        let res = res.send().await;
        println!("res: {:?}", res);
        assert_eq!(1, 1);
    }
}

mod test_dingtalk_boot2 {
    use super::*;
    #[tokio::test]
    async fn test_dingtalk_boot() {
        // 向钉钉机器人发送post请求
        let client = Client::new();
        let key_word = "运维事件";
        let payload = DingTalkBootPayload {
            msg_type: String::from("link"),
            link: DingTalkBootLinkContent {
                title: format!("[{}]: 豆瓣爬虫故障", key_word),
                text: format!("请更新cookie",),
                message_url: format!("https://pitun.cc/admin/"),
            },
        };
        let res = client.post("https://oapi.dingtalk.com/robot/send?access_token=5ad8a5746b6812082f359ff9c06c339567bd11a3e6a137eeb15651a67c2fa62c").body(serde_json::to_string(&payload).unwrap()).header("Content-Type", "application/json");
        println!("res: {:?}", res);
        let res = res.send().await;
        println!("res: {:?}", res);
        assert_eq!(1, 1);
    }
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        subject: Option<String>,
        content: Option<String>,
        extra: Option<Value>,
        label_ids: Option<Vec<Option<i64>>>,

        is_recalled: Option<bool>,
        is_resolved: Option<bool>,
        is_processed: Option<bool>,
    }
    #[derive(AsChangeset, Debug)]
    #[diesel(table_name = help_tickets)]
    struct HelpTicketData<'a> {
        subject: Option<&'a str>,
        content: Option<&'a str>,
        extra: Option<Value>,
        label_ids: Option<&'a Vec<Option<i64>>>,
        is_recalled: Option<bool>,
        is_resolved: Option<bool>,
        is_processed: Option<bool>,
        updated_by: i64,
        updated_at: DateTime<Utc>,
    }

    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);

    let mut conn = db::connect()?;
    let help_ticket = get_record_by_param!(req, res, HelpTicket, help_tickets, &mut conn);

    if help_ticket.is_recalled {
        return context::render_bad_request_json_with_detail(res, "this help ticket recalled");
    }

    let help_ticket = diesel::update(&help_ticket)
        .set(&HelpTicketData {
            subject: pdata.subject.as_deref(),
            content: pdata.content.as_deref(),
            extra: pdata.extra.clone(),
            label_ids: pdata.label_ids.as_ref(),
            is_recalled: pdata.is_recalled,
            is_resolved: pdata.is_resolved,
            is_processed: pdata.is_processed,
            updated_by: cuser.id,
            updated_at: Utc::now(),
        })
        .get_result::<HelpTicket>(&mut conn)?;
    res.render(Json(help_ticket));
    Ok(())
}
