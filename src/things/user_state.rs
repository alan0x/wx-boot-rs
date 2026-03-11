use crate::redis;

use ::redis::Commands;

// 每60s检查一次redis， 找到所有已经登陆的用户， 检查上次同步时间
// 如果超过一定时间判断为连接中断
pub async fn user_state_check() -> () {
    loop {
        println!("checking users' login state");

        // 查询redis
        let rc = redis::connect();
        if let Ok(mut rc) = rc {
            let all_online_user_result = rc.hkeys::<&'static str, Vec<String>>("user_state");

            if let Ok(all_online_user) = all_online_user_result {
                for online_user in all_online_user {
                    let time_stamp_result =
                        rc.hget::<&'static str, &str, String>("user_state", &online_user);
                    if let Ok(time_stamp) = time_stamp_result {
                        // println!("user: {} last online time: {}", online_user, time_stamp);
                        // check if time_stamp is too old
                        let time_stamp = chrono::DateTime::parse_from_rfc3339(&time_stamp);
                        if let Ok(time_stamp) = time_stamp {
                            let now = chrono::Utc::now();
                            let duration = now.signed_duration_since(time_stamp);
                            // 两个完整的心跳周期， 也就是10分钟， 如果超过10分钟没有心跳， 则认为用户已经下线
                            if duration.num_seconds() > 600 {
                                println!("user: {} is offline", &online_user);
                                // update user state
                                let _ = rc
                                    .hdel::<&'static str, &str, String>("user_state", &online_user);
                            }
                        }
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
