use axum::{http::HeaderMap, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::Db, error::{AppError, AppResult}};
use utoipa::ToSchema;

pub const IDEMPOTENCY_HEADER: &str = "idempotency-key";

#[derive(Deserialize, ToSchema)]
pub struct SeckillRequest {
    pub user_id: String,
    pub ticket_type_id: Uuid,
    #[schema(default = 1)]
    pub qty: i32,
}

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
    post,
    path = "/seckill",
    request_body = SeckillRequest,
    params(("idempotency-key" = String, Header, description = "Idempotency key (per user). Strongly recommended.")),
    responses((status=200, body=OrderDto), (status=409, description="Out of stock"))
)]
pub async fn seckill(
    axum::extract::State(db): axum::extract::State<Db>,
    headers: HeaderMap,
    Json(req): Json<SeckillRequest>,
) -> AppResult<Json<OrderDto>> {
    if req.user_id.is_empty() {
        return Err(AppError::BadRequest("user_id required".into()));
    }
    if req.qty != 1 {
        return Err(AppError::BadRequest("only qty=1 supported in MVP".into()));
    }

    let idempotency_key = headers
        .get(IDEMPOTENCY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut tx = db.pool.begin().await?;

    if let Some(key) = &idempotency_key {
        let existing = sqlx::query_as::<_, OrderDto>(
            r#"select id, user_id, ticket_type_id, qty, amount_cents, status, created_at
               from orders
               where user_id = $1 and idempotency_key = $2"#,
        )
        .bind(&req.user_id)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(order) = existing {
            tx.commit().await?;
            return Ok(Json(order));
        }
    }

    // Atomic inventory decrement.
    // We enforce time window (sale_starts_at <= now < sale_ends_at).
    let now = Utc::now();
    let updated: Option<(i64,)> = sqlx::query_as(
        r#"update ticket_types
           set inventory_remaining = inventory_remaining - 1
           where id = $1
             and inventory_remaining >= 1
             and sale_starts_at <= $2
             and sale_ends_at > $2
           returning price_cents"#,
    )
    .bind(req.ticket_type_id)
    .bind(now)
    .fetch_optional(&mut *tx)
    .await?;

    let price_cents = match updated {
        Some((price_cents,)) => price_cents,
        None => {
            tx.rollback().await?;
            return Err(AppError::Conflict("out of stock or not in sale window".into()));
        }
    };

    let order_id = Uuid::new_v4();
    let rec = sqlx::query_as::<_, OrderDto>(
        r#"insert into orders (id, user_id, ticket_type_id, qty, amount_cents, status, idempotency_key)
           values ($1,$2,$3,$4,$5,'pending', $6)
           returning id, user_id, ticket_type_id, qty, amount_cents, status, created_at"#,
    )
    .bind(order_id)
    .bind(&req.user_id)
    .bind(req.ticket_type_id)
    .bind(req.qty)
    .bind(price_cents)
    .bind(idempotency_key)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(rec))
}

pub fn router() -> Router<Db> {
    Router::new().route("/seckill", post(seckill))
}
