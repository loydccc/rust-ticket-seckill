use axum::{extract::Path, routing::{get, post}, Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{db::Db, error::{AppError, AppResult}};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct OrderDto {
    pub id: Uuid,
    pub user_id: String,
    pub ticket_type_id: Uuid,
    pub qty: i32,
    pub amount_cents: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/orders/{order_id}",
    params(("order_id" = Uuid, Path, description = "Order id")),
    responses((status=200, body=OrderDto), (status=404))
)]
pub async fn get_order(
    axum::extract::State(db): axum::extract::State<Db>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let order = sqlx::query_as::<_, OrderDto>(
        r#"select id, user_id, ticket_type_id, qty, amount_cents, status, created_at
           from orders where id = $1"#,
    )
    .bind(order_id)
    .fetch_optional(&db.pool)
    .await?;

    match order {
        Some(o) => Ok(Json(o)),
        None => Err(AppError::NotFound),
    }
}

#[utoipa::path(
    post,
    path = "/orders/{order_id}/pay",
    params(("order_id" = Uuid, Path, description = "Order id")),
    responses((status=200, body=OrderDto), (status=404), (status=409))
)]
pub async fn pay_order(
    axum::extract::State(db): axum::extract::State<Db>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let mut tx = db.pool.begin().await?;

    let order = sqlx::query_as::<_, OrderDto>(
        r#"select id, user_id, ticket_type_id, qty, amount_cents, status, created_at
           from orders where id = $1 for update"#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(order) = order else {
        tx.rollback().await?;
        return Err(AppError::NotFound);
    };

    if order.status != "pending" {
        tx.rollback().await?;
        return Err(AppError::Conflict("order not payable".into()));
    }

    let updated = sqlx::query_as::<_, OrderDto>(
        r#"update orders set status = 'paid', paid_at = now()
           where id = $1
           returning id, user_id, ticket_type_id, qty, amount_cents, status, created_at"#,
    )
    .bind(order_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

pub fn router() -> Router<Db> {
    Router::new()
        .route("/orders/:order_id", get(get_order))
        .route("/orders/:order_id/pay", post(pay_order))
}
