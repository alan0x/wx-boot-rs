use salvo::prelude::*;

use crate::{context, things, utils, AppResult};

// static SCALED_SIZES: [usize; 3] = [1280, 640, 320];

#[handler]
pub async fn show(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    // let width = req.param::<usize>("width").unwrap_or(320);
    // let height = req.param::<usize>("height").unwrap_or(320);
    let ext = req.param::<String>("ext").unwrap_or_else(|| "webp".into());
    let fallbacks = utils::fallbacks_in_query(req);
    let file_path = if let Some(avatar) = &cuser.avatar {
        join_path!(
            cuser.avatar_base_dir(false),
            avatar,
            format!("origin.{ext}")
        )
    } else {
        join_path!("avatars/defaults", format!("origin.webp"))
    };
    // utils::fs::send_local_file(file_path, req.headers(), res, None).await;
    utils::fs::send_local_file_with_fallbacks(file_path, req.headers(), res, None, &fallbacks)
        .await;
    Ok(())
}

#[handler]
pub async fn upload(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let file = req.file("image").await;
    if file.is_none() {
        return context::render_bad_request_json_with_detail(res, "not found file in file field");
    }
    let file = file.unwrap();
    let user = things::user::avatar::upload(cuser.id, file).await?;
    res.render(Json(user));

    Ok(())
}

#[handler]
pub async fn delete(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    ::std::fs::remove_dir_all(&cuser.avatar_base_dir(true)).ok();

    Ok(())
}
