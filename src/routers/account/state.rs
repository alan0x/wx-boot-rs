use chrono::Utc;
use salvo::prelude::*;

use crate::redis;
use crate::AppResult;

use ::redis::Commands;

#[handler]
pub async fn online(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let rc = redis::connect();
    if let Ok(mut rc) = rc {
        let cuser = current_user!(depot, res);
        let user_id = cuser.id.to_string();

        // 检查是否第一次
        let exist = rc.hexists::<&str, String, String>("user_state", user_id.clone());

        if let Err(_) = exist {
            // 之前不存在
            log::info!(
                "<{}> {}({}) login, IP: {:?}",
                chrono::Local::now(),
                cuser.display_name,
                cuser.id,
                req.remote_addr()
            );
        }

        let _ =
            rc.hset::<&str, String, String, String>("user_state", user_id, Utc::now().to_rfc3339());
    }

    res.render("ok");

    Ok(())
}
