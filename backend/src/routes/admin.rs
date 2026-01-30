use axum::{extract::Path, routing::{get, post}, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::Db, error::{AppError, AppResult}};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateEventRequest {
    pub name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct EventDto {
    pub id: Uuid,
    pub name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/admin/events",
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
    let rec = sqlx::query_as!(
        EventDto,
        r#"insert into events (id, name, starts_at, ends_at)
           values ($1, $2, $3, $4)
           returning id, name, starts_at, ends_at"#,
        id,
        req.name,
        req.starts_at,
        req.ends_at
    )
    .fetch_one(&db.pool)
    .await?;

    Ok(Json(rec))
}

#[utoipa::path(
    get,
    path = "/events",
    responses((status=200, body=[EventDto]))
)]
pub async fn list_events(
    axum::extract::State(db): axum::extract::State<Db>,
) -> AppResult<Json<Vec<EventDto>>> {
    let rows = sqlx::query_as!(EventDto, r#"select id, name, starts_at, ends_at from events order by starts_at desc"#)
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

#[derive(Serialize, ToSchema)]
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
    path = "/admin/events/{event_id}/ticket_types",
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
    let exists = sqlx::query_scalar!("select exists(select 1 from events where id = $1) as \"exists!\"", event_id)
        .fetch_one(&db.pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound);
    }

    let id = Uuid::new_v4();
    let rec = sqlx::query_as!(
        TicketTypeDto,
        r#"insert into ticket_types (id, event_id, name, price_cents, inventory_total, inventory_remaining, sale_starts_at, sale_ends_at)
           values ($1,$2,$3,$4,$5,$5,$6,$7)
           returning id, event_id, name, price_cents, inventory_total, inventory_remaining, sale_starts_at, sale_ends_at"#,
        id,
        event_id,
        req.name,
        req.price_cents,
        req.inventory_total,
        req.sale_starts_at,
        req.sale_ends_at
    )
    .fetch_one(&db.pool)
    .await?;

    Ok(Json(rec))
}

#[utoipa::path(
    get,
    path = "/events/{event_id}/ticket_types",
    params(("event_id" = Uuid, Path, description = "Event id")),
    responses((status=200, body=[TicketTypeDto]))
)]
pub async fn list_ticket_types(
    axum::extract::State(db): axum::extract::State<Db>,
    Path(event_id): Path<Uuid>,
) -> AppResult<Json<Vec<TicketTypeDto>>> {
    let rows = sqlx::query_as!(
        TicketTypeDto,
        r#"select id, event_id, name, price_cents, inventory_total, inventory_remaining, sale_starts_at, sale_ends_at
           from ticket_types where event_id = $1 order by created_at asc"#,
        event_id
    )
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(rows))
}

pub fn router() -> Router<Db> {
    Router::new()
        .route("/admin/events", post(create_event))
        .route("/events", get(list_events))
        .route(
            "/admin/events/:event_id/ticket_types",
            post(create_ticket_type),
        )
        .route(
            "/events/:event_id/ticket_types",
            get(list_ticket_types),
        )
}
