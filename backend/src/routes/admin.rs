use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::Db,
    error::{AppError, AppResult},
};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateEventRequest {
    pub name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct EventDto {
    pub id: Uuid,
    pub name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/admin/events",
    request_body = CreateEventRequest,
    responses((status=200, body=EventDto))
)]
pub async fn create_event(
    axum::extract::State(db): axum::extract::State<Db>,
    Json(req): Json<CreateEventRequest>,
) -> AppResult<Json<EventDto>> {
    if req.ends_at <= req.starts_at {
        return Err(AppError::BadRequest("ends_at must be after starts_at".into()));
    }

    let id = Uuid::new_v4();
    let rec = sqlx::query_as::<_, EventDto>(
        r#"insert into events (id, name, starts_at, ends_at)
           values ($1, $2, $3, $4)
           returning id, name, starts_at, ends_at"#,
    )
    .bind(id)
    .bind(req.name)
    .bind(req.starts_at)
    .bind(req.ends_at)
    .fetch_one(&db.pool)
    .await?;

    Ok(Json(rec))
}

#[utoipa::path(
    get,
    path = "/api/events",
    responses((status=200, body=[EventDto]))
)]
pub async fn list_events(
    axum::extract::State(db): axum::extract::State<Db>,
) -> AppResult<Json<Vec<EventDto>>> {
    let rows = sqlx::query_as::<_, EventDto>(
        r#"select id, name, starts_at, ends_at from events order by starts_at desc"#,
    )
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTicketTypeRequest {
    pub name: String,
    pub price_cents: i64,
    pub inventory_total: i32,
    pub sale_starts_at: DateTime<Utc>,
    pub sale_ends_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct TicketTypeDto {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub inventory_total: i32,
    pub inventory_remaining: i32,
    pub sale_starts_at: DateTime<Utc>,
    pub sale_ends_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/admin/events/{event_id}/ticket_types",
    params(("event_id" = Uuid, Path, description = "Event id")),
    request_body = CreateTicketTypeRequest,
    responses((status=200, body=TicketTypeDto))
)]
pub async fn create_ticket_type(
    axum::extract::State(db): axum::extract::State<Db>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<CreateTicketTypeRequest>,
) -> AppResult<Json<TicketTypeDto>> {
    if req.inventory_total <= 0 {
        return Err(AppError::BadRequest("inventory_total must be > 0".into()));
    }
    if req.price_cents < 0 {
        return Err(AppError::BadRequest("price_cents must be >= 0".into()));
    }
    if req.sale_ends_at <= req.sale_starts_at {
        return Err(AppError::BadRequest("sale_ends_at must be after sale_starts_at".into()));
    }

    // ensure event exists
    let exists: bool = sqlx::query_scalar("select exists(select 1 from events where id = $1)")
        .bind(event_id)
        .fetch_one(&db.pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound);
    }

    let id = Uuid::new_v4();
    let rec = sqlx::query_as::<_, TicketTypeDto>(
        r#"insert into ticket_types (id, event_id, name, price_cents, inventory_total, inventory_remaining, sale_starts_at, sale_ends_at)
           values ($1,$2,$3,$4,$5,$5,$6,$7)
           returning id, event_id, name, price_cents, inventory_total, inventory_remaining, sale_starts_at, sale_ends_at"#,
    )
    .bind(id)
    .bind(event_id)
    .bind(req.name)
    .bind(req.price_cents)
    .bind(req.inventory_total)
    .bind(req.sale_starts_at)
    .bind(req.sale_ends_at)
    .fetch_one(&db.pool)
    .await?;

    Ok(Json(rec))
}

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/ticket_types",
    params(("event_id" = Uuid, Path, description = "Event id")),
    responses((status=200, body=[TicketTypeDto]))
)]
pub async fn list_ticket_types(
    axum::extract::State(db): axum::extract::State<Db>,
    Path(event_id): Path<Uuid>,
) -> AppResult<Json<Vec<TicketTypeDto>>> {
    let rows = sqlx::query_as::<_, TicketTypeDto>(
        r#"select id, event_id, name, price_cents, inventory_total, inventory_remaining, sale_starts_at, sale_ends_at
           from ticket_types where event_id = $1 order by created_at asc"#,
    )
    .bind(event_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTicketTypeFlatRequest {
    pub event_id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub inventory_total: i32,
    pub sale_starts_at: DateTime<Utc>,
    pub sale_ends_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/admin/ticket-types",
    request_body = CreateTicketTypeFlatRequest,
    responses((status=200, body=TicketTypeDto))
)]
pub async fn create_ticket_type_flat(
    axum::extract::State(db): axum::extract::State<Db>,
    Json(req): Json<CreateTicketTypeFlatRequest>,
) -> AppResult<Json<TicketTypeDto>> {
    create_ticket_type(
        axum::extract::State(db),
        Path(req.event_id),
        Json(CreateTicketTypeRequest {
            name: req.name,
            price_cents: req.price_cents,
            inventory_total: req.inventory_total,
            sale_starts_at: req.sale_starts_at,
            sale_ends_at: req.sale_ends_at,
        }),
    )
    .await
}

pub fn router() -> Router<Db> {
    Router::new()
        // required paths
        .route("/api/admin/events", post(create_event))
        .route("/api/events", get(list_events))
        .route("/api/admin/events/:event_id/ticket_types", post(create_ticket_type))
        .route("/api/events/:event_id/ticket_types", get(list_ticket_types))
        .route("/api/admin/ticket-types", post(create_ticket_type_flat))
        // compatibility (old, no /api prefix)
        .route("/admin/events", post(create_event))
        .route("/events", get(list_events))
        .route("/admin/events/:event_id/ticket_types", post(create_ticket_type))
        .route("/events/:event_id/ticket_types", get(list_ticket_types))
}
