use chrono::Utc;
use diesel::prelude::*;
use salvo::prelude::*;

use crate::models::order::*;
use crate::models::*;
use crate::schema::*;
use crate::{context, db, AppResult};

#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let query = orders::table.filter(orders::created_by.eq(cuser.id));
    let mut conn = db::connect()?;
    list_records!(
        req,
        res,
        Order,
        query,
        "updated_at desc",
        ORDER_FILTER_FIELDS.clone(),
        ORDER_JOINED_OPTIONS.clone(),
        ID_NAME_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}
