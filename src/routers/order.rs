use bigdecimal::BigDecimal;
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;

use crate::db;
use crate::models::order::*;
use crate::models::*;
use crate::schema::*;
use crate::things::order::calc_actual_amount;
use crate::utils::uuid_string;
use crate::{context, AppResult};

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

    let cuser = current_user!(depot, res);
    let pdata = parse_posted_data!(req, res, PostedData);
    let order_id = uuid_string();
    let amount_info = calc_actual_amount(cuser, pdata.reason.clone())?;

    let order = NewOrder {
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
    };

    let mut conn = db::connect()?;
    let order: Order = diesel::insert_into(orders::table)
        .values(&order)
        .get_result::<Order>(&mut conn)?;
    drop(conn);

    // TODO: Integrate your payment provider here (e.g., WeChat Pay)
    // After payment confirmation, update order trade_state to "SUCCESS"

    res.render(Json(order));
    Ok(())
}

#[handler]
pub async fn show(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    show_record!(req, depot, res, Order, orders, &mut conn);
    Ok(())
}

#[handler]
pub async fn calc_amount(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        reason: String,
    }
    let cuser = current_user!(depot, res);
    let pdata = parse_posted_data!(req, res, PostedData);
    let result = calc_actual_amount(cuser, pdata.reason)?;
    res.render(Json(result));
    Ok(())
}

#[handler]
pub async fn notify(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    // TODO: Implement payment notification handler for your payment provider
    // This endpoint receives callbacks from the payment provider (e.g., WeChat Pay)
    // After verifying the payment, update the order status accordingly
    res.render("ok");
    Ok(())
}
