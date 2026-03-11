use chrono::Utc;
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;
use std::path::Path;
use tokio::io::AsyncWriteExt;

use crate::db;
use crate::models::*;
use crate::schema::*;

use crate::{utils, AppResult};

pub fn public_root(path: impl Into<String>) -> Router {
    Router::with_path(path).post(exchange)
}

#[handler]
pub async fn exchange(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        code: String,
        user_id: i64,
    }
    let pdata = parse_posted_data!(req, res, PostedData);

    println!("====== code:{:?}", &pdata.code);
    // 根据code获得access token
    // get https://api.weixin.qq.com/sns/oauth2/access_token?appid=APPID&secret=SECRET&code=CODE&grant_type=authorization_code
    // {
    //     "access_token":"ACCESS_TOKEN",
    //     "expires_in":7200,
    //     "refresh_token":"REFRESH_TOKEN",
    //     "openid":"OPENID",
    //     "scope":"SCOPE",
    //     "is_snapshotuser": 1,
    //     "unionid": "UNIONID"
    // }
    #[derive(Deserialize, Debug)]
    struct WeChatMpExchangeAccessTokenResponse {
        access_token: String,
        expires_in: i64,
        refresh_token: String,
        openid: String,
        scope: String,
        // is_snapshotuser: i32,
        unionid: Option<String>,
    }
    let client = reqwest::Client::new();
    let mut url = reqwest::Url::parse("https://api.weixin.qq.com/sns/oauth2/access_token")?;
    println!("====== 000000");

    url.query_pairs_mut()
        .append_pair("appid", &crate::wechat_mp_appid())
        .append_pair("secret", &crate::wechat_mp_secret())
        .append_pair("code", &pdata.code)
        .append_pair("grant_type", "authorization_code");

    println!("====== 111111");
    let resp = client.get(url.as_str()).send().await?;

    if !resp.status().is_success() {
        return Err(StatusError::failed_dependency()
            .with_summary("wechat mp exchange accesstoken not response")
            .with_detail("wechat mp exchange accesstoken not response")
            .into());
    }
    println!("====== 222222");

    let resp_str = resp.text().await?;
    println!("====== 33333, {:?}", resp_str);

    let resp_data = serde_json::from_str::<WeChatMpExchangeAccessTokenResponse>(&resp_str);
    println!("====== exchange resp:{:?}", &resp_data);

    if resp_data.is_err() {
        return Err(StatusError::bad_request()
            .with_detail("exchange access failed")
            .with_summary(&resp_str)
            .into());
    }
    let resp_data = resp_data.unwrap();
    // if resp_data.unionid.is_none() {
    //     return Err(StatusError::bad_request()
    //         .with_detail("use has no unionid")
    //         .into());
    // }
    // 使用 access token 拉取用户信息
    // get https://api.weixin.qq.com/sns/userinfo?access_token=ACCESS_TOKEN&openid=OPENID&lang=zh_CN
    // {
    //     "openid": "OPENID",
    //     "nickname": NICKNAME,
    //     "sex": 1,
    //     "province":"PROVINCE",
    //     "city":"CITY",
    //     "country":"COUNTRY",
    //     "headimgurl":"https://thirdwx.qlogo.cn/mmopen/g3MonUZtNHkdmzicIlibx6iaFqAc56vxLSUfpb6n5WKSYVY0ChQKkiaJSgQ1dZuTOgvLLrhJbERQQ4eMsv84eavHiaiceqxibJxCfHe/46",
    //     "privilege":[ "PRIVILEGE1" "PRIVILEGE2"     ],
    //     "unionid": "o6_bmasdasdsad6_2sgVt7hMZOPfL"
    // }
    #[derive(Deserialize, Debug)]
    struct WeChatMpUserInfo {
        openid: String,
        nickname: String,
        sex: i32,
        province: String,
        city: String,
        country: String,
        headimgurl: String,
        unionid: Option<String>,
    }

    let mut url = reqwest::Url::parse("https://api.weixin.qq.com/sns/userinfo")?;
    url.query_pairs_mut()
        .append_pair("access_token", &resp_data.access_token)
        .append_pair("openid", &resp_data.openid)
        .append_pair("lang", "zh_CN");

    let resp = client.get(url.as_str()).send().await?;

    if !resp.status().is_success() {
        return Err(StatusError::failed_dependency()
            .with_summary("wechat mp get userinfo not response")
            .with_detail("wechat mp get userinfo not response")
            .into());
    }

    let resp_str = resp.text().await?;
    let resp_data = serde_json::from_str::<WeChatMpUserInfo>(&resp_str)?;
    println!("====== userinfo resp:{:?}", &resp_data);

    // if resp_data.unionid.is_none() {
    //     return Err(StatusError::bad_request()
    //         .with_detail("use has no unionid")
    //         .into());
    // }

    let mut conn = db::connect()?;
    let user: User = get_record!(res, pdata.user_id, User, users, &mut conn);

    // 下载图片
    let client = reqwest::Client::new();
    let response = client.get(resp_data.headimgurl).send().await?;
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE);
    let ext = if let Some(content_type) = content_type {
        let mime_arr: Vec<&str> = content_type
            .to_str()
            .unwrap_or("image/jpeg")
            .split("/")
            .collect();
        let result = if mime_arr.len() > 1 {
            mime_arr[1]
        } else {
            "jpeg"
        };
        result
    } else {
        "jpeg"
    };
    let uuid_name = utils::uuid_string();
    let store_dir = join_path!(user.avatar_base_dir(true), &uuid_name);

    let avatar_path = join_path!(store_dir, format!("origin.{}", ext));
    if let Some(parent) = Path::new(&avatar_path).parent() {
        // try to create parent folder
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = response.bytes().await;

    if let Ok(content) = content {
        let mut file = tokio::fs::File::create(avatar_path).await?;

        file.write_all(&content).await?;
    }

    // 更新用户
    let _ = diesel::update(&user)
        .set((
            users::display_name.eq(&resp_data.nickname),
            users::avatar.eq(&*uuid_name),
            users::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(())
}

// 021H1DFa1bHjWH0kErFa15sdyG1H1DFH
#[cfg(test)]
mod test {
    use super::*;
    // https://book.douban.com/subject/24358626/
    // 多作者
    #[tokio::test]
    async fn extract_ext() {
        let client = reqwest::Client::new();
        let response = client.get("https://thirdwx.qlogo.cn/mmopen/g3MonUZtNHkdmzicIlibx6iaFqAc56vxLSUfpb6n5WKSYVY0ChQKkiaJSgQ1dZuTOgvLLrhJbERQQ4eMsv84eavHiaiceqxibJxCfHe/46").send().await;
        if let Ok(response) = response {
            let content_type = response.headers().get(reqwest::header::CONTENT_TYPE);
            let ext = if let Some(content_type) = content_type {
                let mime_arr: Vec<&str> = content_type.to_str().unwrap_or("").split("/").collect();
                println!("mime_arr: {:?}", mime_arr);
                let result = if mime_arr.len() > 1 {
                    mime_arr[1]
                } else {
                    "jpeg"
                };
                result
            } else {
                "jpeg"
            };

            println!("ext is: {}", ext);

            let uuid_name = utils::uuid_string();
            let store_dir = join_path!(
                join_path!(
                    "C:/Users/YY/Documents/projects/Elysia/space",
                    "users",
                    "test",
                    "avatars"
                ),
                &uuid_name
            );

            let avatar_path = join_path!(store_dir, format!("origin.{}", ext));
            println!("path: {}", avatar_path);
            if let Some(parent) = Path::new(&avatar_path).parent() {
                // try to create parent folder
                println!("try to create parent folder");
                tokio::fs::create_dir_all(parent).await;
            }
            let content = response.bytes().await;

            if let Ok(content) = content {
                println!("get bytes of content");
                let file = tokio::fs::File::create(avatar_path).await;

                if let Ok(mut file) = file {
                    file.write_all(&content).await;
                }
            }
        }

        assert_eq!(1, 1);
    }
}
