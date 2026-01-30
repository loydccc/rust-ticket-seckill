use axum::{http::HeaderMap, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthUser, db::Db, error::{AppError, AppResult}};
use utoipa::ToSchema;

pub const IDEMPOTENCY_HEADER: &str = "idempotency-key";

#[derive(Deserialize, ToSchema)]
pub struct GrabRequest {
    pub ticket_type_id: Uuid,
    #[schema(default = 1)]
    pub qty: i32,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct OrderDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticket_type_id: Uuid,
    pub qty: i32,
    pub amount_cents: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/tickets/grab",
    request_body = GrabRequest,
    params(
        ("idempotency-key" = String, Header, description = "Idempotency key (per user). Recommended.")
    ),
    responses((status=200, body=OrderDto), (status=409, description="Out of stock / not started / already grabbed"), (status=401))
)]
pub async fn grab(
    axum::extract::State(db): axum::extract::State<Db>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<GrabRequest>,
) -> AppResult<Json<OrderDto>> {
    if req.qty != 1 {
        return Err(AppError::BadRequest("only qty=1 supported in MVP".into()));
    }

    let idempotency_key = headers
        .get(IDEMPOTENCY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut tx = db.pool.begin().await?;

    // If idempotency key matches an existing order, return it.
    if let Some(key) = &idempotency_key {
        let existing = sqlx::query_as::<_, OrderDto>(
            r#"select id, user_id, ticket_type_id, qty, amount_cents, status, created_at
               from orders
               where user_id = $1 and idempotency_key = $2"#,
        )
        .bind(auth.user_id)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(order) = existing {
            tx.commit().await?;
            return Ok(Json(order));
        }
    }

    // Quick check: if user already has an active order (CREATED/PAID) for this ticket type, return it.
    if let Some(order) = sqlx::query_as::<_, OrderDto>(
        r#"select id, user_id, ticket_type_id, qty, amount_cents, status, created_at
           from orders
           where user_id = $1 and ticket_type_id = $2 and status in ('CREATED','PAID')"#,
    )
    .bind(auth.user_id)
    .bind(req.ticket_type_id)
    .fetch_optional(&mut *tx)
    .await?
    {
        tx.commit().await?;
        return Ok(Json(order));
    }

    // Atomic inventory decrement in Postgres (no oversell): single UPDATE guarded by remaining>=1 + time window.
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
    let inserted = sqlx::query_as::<_, OrderDto>(
        r#"insert into orders (id, user_id, ticket_type_id, qty, amount_cents, status, idempotency_key)
           values ($1,$2,$3,$4,$5,'CREATED', $6)
           returning id, user_id, ticket_type_id, qty, amount_cents, status, created_at"#,
    )
    .bind(order_id)
    .bind(auth.user_id)
    .bind(req.ticket_type_id)
    .bind(req.qty)
    .bind(price_cents)
    .bind(idempotency_key)
    .fetch_one(&mut *tx)
    .await;

    let rec = match inserted {
        Ok(v) => v,
        Err(e) => {
            // If we lost a race on unique constraints, return the existing order.
            if let Some(db_err) = e.as_database_error() {
                if db_err.constraint() == Some("uq_orders_user_ticket_type_active")
                    || db_err.constraint() == Some("uq_orders_user_idempotency")
                {
                    let existing = sqlx::query_as::<_, OrderDto>(
                        r#"select id, user_id, ticket_type_id, qty, amount_cents, status, created_at
                           from orders
                           where user_id = $1 and ticket_type_id = $2 and status in ('CREATED','PAID')
                           order by created_at desc
                           limit 1"#,
                    )
                    .bind(auth.user_id)
                    .bind(req.ticket_type_id)
                    .fetch_one(&mut *tx)
                    .await?;

                    tx.commit().await?;
                    return Ok(Json(existing));
                }
            }
            return Err(AppError::Db(e));
        }
    };

    tx.commit().await?;
    Ok(Json(rec))
}

pub fn router() -> Router<Db> {
    Router::new()
        .route("/api/tickets/grab", post(grab))
        // compatibility
        .route("/seckill", post(grab))
}
