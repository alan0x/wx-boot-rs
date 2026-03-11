use salvo::prelude::*;

use crate::{context, utils, AppResult};

#[handler]
pub async fn upload(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let folder: String = req.query::<String>("folder").unwrap_or_default();
    if folder.is_empty() {
        return context::render_parse_query_error_json(res);
    }
    if !utils::fs::is_safe_dir_path(&folder) {
        return Err(crate::Error::Internal("store folder is illegal".into()));
    }
    //TODO: folder name valid.

    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }

    let unique = utils::str_to_bool(
        &req.query::<String>("unique")
            .unwrap_or_else(|| "true".into()),
    );
    let unzip = utils::str_to_bool(
        &req.query::<String>("unzip")
            .unwrap_or_else(|| "false".into()),
    );
    let store_dir = format!("users/{}/buckets/{}", cuser.id, folder);
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
pub async fn serve_file(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let rest_path = crate::safe_url_path(&req.param::<String>("*path").unwrap_or_default());
    println!("rest_path: {}", rest_path);
    if rest_path.is_empty() {
        return context::render_not_found_json(res);
    }
    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }

    let file_path = join_path!("users", &cuser.id.to_string(), "buckets", &rest_path);

    println!("file_path: {}", file_path);
    let attched_name = req.queries().get("file_name").map(String::as_str);
    utils::fs::send_local_file(file_path, req.headers(), res, attched_name).await;
    Ok(())
}
