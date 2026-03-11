use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use chrono::Utc;
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;

use crate::db;
use crate::models::order::*;
use crate::models::*;
use crate::schema::*;
use crate::things::order::{calc_actual_amount, update_user_by_order};
use crate::utils::uuid_string;
use crate::{context, AppResult};

use wechat_pay_rust_sdk::model::{MicroParams, PayerInfo};
use wechat_pay_rust_sdk::pay::{PayNotifyTrait, WechatPay};
use wechat_pay_rust_sdk::response::SignData;

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .get(list)
        .post(create)
        .push(Router::with_path(r"<id:/\d+/>").get(show))
        .push(Router::with_path("calc_amount").post(calc_amount))
}

pub fn public_root(path: impl Into<String>) -> Router {
    Router::with_path(path).push(Router::with_path("notify").post(notify))
}

#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    let cuser = current_user!(depot, res);
    if !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    list_records!(
        req, res, Order, orders::table, "updated_at desc",
        ORDER_FILTER_FIELDS.clone(), ORDER_JOINED_OPTIONS.clone(),
        ID_SUBJECT_SEARCH_TMPL, &mut conn
    );
    Ok(())
}

#[handler]
pub async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        reason: String,
        description: String,
    }
    #[derive(Serialize, Debug)]
    struct ResultData {
        order: Order,
        code: Option<String>,
        message: Option<String>,
        /// 预支付交易会话标识，有效期 2 小时，用于小程序端调起支付
        prepay_id: Option<String>,
        /// 签名数据，直接透传给小程序 wx.requestPayment()
        sign_data: Option<SignData>,
    }

    let cuser = current_user!(depot, res);
    if cuser.weixin_openid.is_none() {
        return context::render_bad_request_json_with_detail(res, "user has no openid");
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let order_id = uuid_string();
    let amount_info = calc_actual_amount(cuser, pdata.reason.clone())?;

    // 实际金额为 0 时（折扣抵扣）直接标记为 SUCCESS，无需拉起支付
    let new_order = if amount_info.actual_amount == BigDecimal::from(0) {
        NewOrder {
            order_id: &order_id,
            paid_reason: &pdata.reason,
            amount: amount_info.origin_amount.clone(),
            trade_state: "SUCCESS",
            payment_id: "",
            paid_at: Some(Utc::now()),
            paid_by: Some(cuser.id),
            updated_by: Some(cuser.id),
            created_by: Some(cuser.id),
            actual_amount: amount_info.actual_amount.clone(),
            discount: amount_info,
        }
    } else {
        NewOrder {
            order_id: &order_id,
            paid_reason: &pdata.reason,
            amount: amount_info.origin_amount.clone(),
            trade_state: "NEW",
            payment_id: "",
            paid_at: None,
            paid_by: None,
            updated_by: Some(cuser.id),
            created_by: Some(cuser.id),
            actual_amount: amount_info.actual_amount.clone(),
            discount: amount_info,
        }
    };

    let mut conn = db::connect()?;
    let order = conn.transaction::<_, crate::Error, _>(|conn| {
        let order: Order = diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result::<Order>(conn)?;
        if order.trade_state == "SUCCESS" {
            let _ = update_user_by_order(&order, cuser, conn);
        }
        Ok(order)
    })?;
    drop(conn);

    if order.trade_state != "SUCCESS" {
        // 微信价格单位为分
        let weixin_amount = (order.actual_amount.to_f32().unwrap() * 100.).floor() as i32;
        let wechat_pay = WechatPay::from_env();
        let micro_res = wechat_pay
            .micro_pay(MicroParams::new(
                &pdata.description,
                &order_id,
                weixin_amount.into(),
                PayerInfo {
                    openid: cuser.weixin_openid.clone().unwrap_or_default(),
                },
            ))
            .await;

        match micro_res {
            Ok(body) => res.render(Json(ResultData {
                order,
                code: body.code,
                message: body.message,
                prepay_id: body.prepay_id,
                sign_data: body.sign_data,
            })),
            Err(_) => return context::render_bad_request_json(res),
        }
    } else {
        res.render(Json(ResultData {
            order,
            code: None,
            message: None,
            prepay_id: None,
            sign_data: None,
        }));
    }

    Ok(())
}

#[handler]
pub async fn show(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    show_record!(req, depot, res, Order, orders, &mut conn);
    Ok(())
}

#[handler]
pub async fn calc_amount(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        reason: String,
    }
    let cuser = current_user!(depot, res);
    if cuser.weixin_openid.is_none() {
        return context::render_bad_request_json_with_detail(res, "user has no openid");
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let result = calc_actual_amount(cuser, pdata.reason)?;
    res.render(Json(result));
    Ok(())
}

// 微信支付回调数据结构
#[derive(Deserialize, Debug)]
struct NotifyResource {
    ciphertext: String,
    associated_data: String,
    nonce: String,
}
#[derive(Deserialize, Debug)]
struct NotifyData {
    resource: NotifyResource,
}

#[handler]
pub async fn notify(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let pdata: NotifyData = parse_posted_data!(req, res, NotifyData);
    let wechat_pay = WechatPay::from_env();

    let result = wechat_pay
        .decrypt_paydata(
            pdata.resource.ciphertext,
            pdata.resource.nonce,
            pdata.resource.associated_data,
        )
        .unwrap();

    let mut conn = db::connect()?;

    let paid_time = chrono::DateTime::parse_from_rfc3339(&result.success_time).ok();

    let user: User = users::table
        .filter(users::weixin_openid.eq(Some(result.payer.openid)))
        .first::<User>(&mut conn)?;

    let order: Order = orders::table
        .filter(orders::order_id.eq(&result.out_trade_no))
        .first::<Order>(&mut conn)?;

    // 幂等处理：只处理 NEW 状态，防止微信重复回调
    if order.trade_state == "NEW" {
        let order = diesel::update(&order)
            .set((
                orders::trade_state.eq(&result.trade_state),
                orders::payment_id.eq(result.transaction_id),
                orders::paid_at.eq(paid_time),
                orders::paid_by.eq(user.id),
            ))
            .get_result::<Order>(&mut conn)?;

        let _ = update_user_by_order(&order, &user, &mut conn);
    }

    Ok(())
}
