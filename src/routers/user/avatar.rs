use salvo::prelude::*;

use crate::models::*;
use crate::schema::*;
use crate::{db, utils, AppResult};

#[handler]
pub async fn show(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let rest_path = crate::safe_url_path(&req.param::<String>("path").unwrap_or_default());

    if rest_path.is_empty() {
        return Err(StatusError::not_found().into());
    }

    let mut conn = db::connect()?;
    let user: User = get_record_by_param!(req, res, User, users, &mut conn);
    drop(conn);

    if user.avatar.is_none() {
        return Err(StatusError::not_found().into());
    }

    let file_path = join_path!(
        user.avatar_base_dir(false),
        user.avatar.unwrap(),
        &rest_path
    );

    let fallbacks = utils::fallbacks_in_query(req);

    // utils::fs::send_local_or_s3_file(file_path, req.headers(), res, None).await;
    // utils::fs::send_local_file(file_path, req.headers(), res, None).await;

    utils::fs::send_local_file_with_fallbacks(file_path, req.headers(), res, None, &fallbacks)
        .await;

    Ok(())
}
