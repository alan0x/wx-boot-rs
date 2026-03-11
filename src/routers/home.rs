use crate::redis;
use salvo::prelude::*;

use ::redis::Commands;

use crate::context;
use crate::AppResult;

#[handler]
pub async fn index(res: &mut Response) -> AppResult<()> {
    res.render("wx-boot-rs v0.1.0");
    Ok(())
}

#[handler]
pub async fn user_state(res: &mut Response) -> AppResult<()> {
    let rc = redis::connect();
    if let Ok(mut rc) = rc {
        let all_online_user_result = rc.hkeys::<&'static str, Vec<String>>("user_state");
        if let Ok(all_online_user) = all_online_user_result {
            #[derive(Serialize, Debug)]
            struct ResultData {
                online_users: Vec<String>,
            }
            res.render(Json(ResultData {
                online_users: all_online_user,
            }));
        } else {
            return context::render_internal_server_error_json_with_detail(res, "redis query error");
        }
    } else {
        return context::render_internal_server_error_json_with_detail(res, "redis connect error");
    }
    Ok(())
}

#[handler]
pub async fn show_logs(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    let entries =
        std::fs::read_dir(std::env::var("LOG_LOCATION").unwrap_or(String::from("/data/log_files")))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect::<Vec<_>>();
    res.render(Json(entries));
    Ok(())
}

#[handler]
pub async fn show_log(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    let rest_path = crate::safe_url_path(&req.param::<String>("path").unwrap_or_default());
    let log_dir = std::env::var("LOG_LOCATION").unwrap_or(String::from("/data/log_files"));
    let file_path = format!("{}/{}", log_dir, rest_path);
    match std::fs::read_to_string(&file_path) {
        Ok(content) => res.render(Text::Plain(content)),
        Err(_) => res.render(Text::Plain("Log file not found.")),
    }
    Ok(())
}
