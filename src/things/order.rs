use crate::db;
use crate::models::order::*;
use crate::models::*;
use crate::schema::*;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::sql_types::Jsonb;

use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;

use std::str::FromStr;

use crate::AppResult;

#[derive(Serialize, Deserialize, Clone, Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = Jsonb)]
pub struct Discount {
    pub reason: String,
    pub amount: BigDecimal,
}
#[derive(Serialize, Deserialize, Clone, Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = Jsonb)]
pub struct AmountAndDiscount {
    pub discounts: Vec<Discount>,
    pub origin_amount: BigDecimal,
    pub actual_amount: BigDecimal,
}
diesel_json_type!(AmountAndDiscount);

pub fn calc_actual_amount(user: &User, reason: String) -> AppResult<AmountAndDiscount> {
    let mut actual_amount = if reason == String::from("mounthly") {
        BigDecimal::from_str("8").unwrap()
    } else if reason == String::from("yearly") {
        BigDecimal::from_str("80").unwrap()
    } else {
        BigDecimal::from_str("180").unwrap()
    };

    let origin_amount = actual_amount.clone();

    let mut discounts: Vec<Discount> = vec![];

    // 如果之前是月度会员或者年度会员， 并且开通7天之内， 目前购买永久会员， 则可以抵扣
    // 查询用户所有7天内的order
    if reason == String::from("lifetime") {
        let mut conn = db::connect()?;

        let now = Utc::now();
        let start_date = Utc::now() - Duration::days(7);
        let orders: Vec<Order> = orders::table
            .filter(
                orders::paid_by
                    .eq(user.id)
                    .and(orders::paid_at.between(start_date, now)),
            )
            .load::<Order>(&mut conn)?;

        let yearly_order_paid_within_7_days = orders
            .clone()
            .into_iter()
            .filter(|order| {
                order.paid_reason == String::from("yearly")
                    && order.trade_state == String::from("SUCCESS")
            })
            .collect::<Vec<Order>>();

        if yearly_order_paid_within_7_days.len() > 0 {
            let discount_amount = yearly_order_paid_within_7_days[0].amount.clone();
            discounts.push(Discount {
                reason: String::from("7_days_yealy_paid_discount"),
                amount: discount_amount.clone(),
            });

            actual_amount = actual_amount - discount_amount;
        } else {
            let mounthly_order_paid_within_7_days = orders
                .clone()
                .into_iter()
                .filter(|order| {
                    order.paid_reason == String::from("mounthly")
                        && order.trade_state == String::from("SUCCESS")
                })
                .collect::<Vec<Order>>();

            if mounthly_order_paid_within_7_days.len() > 0 {
                let discount_amount = mounthly_order_paid_within_7_days[0].amount.clone();
                discounts.push(Discount {
                    reason: String::from("7_days_mounthly_paid_discount"),
                    amount: discount_amount.clone(),
                });

                actual_amount = actual_amount - discount_amount;
            }
        }
    }

    if let Some(contribute) = user.contribute {
        if contribute >= 3 {
            let mut discount_amount = BigDecimal::from(contribute / 3);

            // 避免出现负数
            if discount_amount > actual_amount {
                discount_amount = actual_amount.clone();
            }
            discounts.push(Discount {
                reason: String::from("bookmark_discount"),
                amount: discount_amount.clone(),
            });

            actual_amount = actual_amount - discount_amount;
        }
    }

    // 返回试算结果
    Ok(AmountAndDiscount {
        discounts,
        actual_amount,
        origin_amount,
    })
}

pub fn update_user_by_order(order: &Order, user: &User, conn: &mut PgConnection) -> AppResult<()> {
    // 设置会员日期, 更新user
    if order.trade_state == String::from("SUCCESS") {
        // 如果之前没有expired_at， 或者expired_at在过去， 那么使用now作为base time
        // 如果expired_at在未来， 则使用expired_at作为base time
        let base_time = if user.is_member.is_none() {
            Utc::now()
        } else if user.is_member == Some(false) {
            Utc::now()
        } else if user.expired_at.is_none() {
            Utc::now()
        } else {
            let current_expired_at: DateTime<Utc> = user.expired_at.unwrap();

            let base_time = if current_expired_at < Utc::now() {
                Utc::now()
            } else {
                current_expired_at
            };
            base_time
        };

        let next_expired_at = if order.paid_reason == String::from("mounthly") {
            base_time + Duration::days(30)
        } else if order.paid_reason == String::from("yearly") {
            base_time + Duration::days(365)
        } else if order.paid_reason == String::from("lifetime") {
            // 暂时100年
            base_time + Duration::days(365 * 100)
        } else {
            base_time
        };

        let contribute = if let Some(cur_contribute) = user.contribute {
            let discount = order
                .discount
                .discounts
                .iter()
                .find(|discount| discount.reason == String::from("bookmark_discount"));

            if let Some(discount) = discount {
                let contribute_used = discount.amount.clone() * BigDecimal::from(3);
                let contribute_used = contribute_used.to_i64().unwrap_or(0);

                cur_contribute - contribute_used
            } else {
                cur_contribute
            }
        } else {
            0
        };

        diesel::update(&user)
            .set((
                users::is_member.eq(Some(true)),
                users::expired_at.eq(Some(next_expired_at)),
                users::contribute.eq(Some(contribute)),
            ))
            .get_result::<User>(conn)?;
    } else {
        // @todo 错误处理
    }

    Ok(())
}
