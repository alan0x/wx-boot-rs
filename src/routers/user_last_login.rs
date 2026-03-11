use diesel::prelude::*;
use salvo::prelude::*;

use crate::db;
use crate::models::*;
use crate::schema::*;
use crate::{context, AppResult};

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path).get(list).delete(delete)
}
#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    // 只允许kernel用户查询
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }

    let mut conn = db::connect()?;
    let query = user_last_login::table;
    list_records!(
        req,
        res,
        UserLastLogin,
        query,
        "updated_at desc",
        USER_LAST_LOGIN_FILTER_FIELDS.clone(),
        USER_LAST_LOGIN_JOINED_OPTIONS.clone(),
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
    let record = get_record_by_param!(req, res, UserLastLogin, user_last_login, &mut conn);
    diesel::delete(user_last_login::table.filter(user_last_login::id.eq(record.id)))
        .execute(&mut conn)?;

    context::render_done_json(res)
}
