use std::fs::create_dir_all;
use std::path::Path;

use diesel::prelude::*;
use salvo::http::form::FilePart;

use crate::models::*;
use crate::schema::*;
use crate::{db, utils, AppResult, Error};

pub async fn upload(user_id: i64, file: &FilePart) -> AppResult<User> {
    let hash = utils::hash_file_md5(file.path())?;
    let store_dir = join_path!(super::avatar_base_dir(user_id, true), &hash);
    let ext = utils::fs::get_file_ext(file.path());
    if !utils::fs::is_image_ext(&ext) {
        return Err(Error::Public("unsupported image format".into()));
    }
    let dest_path = join_path!(&store_dir, format!("origin.{}", ext));

    if !Path::new(&store_dir).exists() {
        create_dir_all(&store_dir)?;
    }
    if !Path::new(&dest_path).exists() {
        std::fs::copy(file.path(), &dest_path)?;
    }

    let metadata = utils::media::get_image_info(&dest_path).await?;

    // 转一张320的， 剩下的是原图

    for size in [640, 320] {
        if metadata.width >= size && metadata.height >= size {
            let resized_file = join_path!(&store_dir, format!("{}x{}.webp", size, size));

            if let Err(e) =
                utils::media::resize_image(Some(size), Some(size), &dest_path, &resized_file).await
            {
                tracing::error!(error = ?e, "resize image failed");
            }
        }
    }

    let conn = &mut db::connect()?;
    let user = diesel::update(users::table.find(user_id))
        .set(users::avatar.eq(&*hash))
        .get_result::<User>(conn)?;

    Ok(user)
}
