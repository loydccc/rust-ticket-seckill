use axum::{routing::{get, post}, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{auth::AuthUser, db::Db, error::{AppError, AppResult}};

#[derive(Deserialize, ToSchema)]
pub struct CreateIntentRequest {
    pub ticket_type_id: Uuid,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct IntentDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticket_type_id: Uuid,
    pub status: String,
    pub order_id: Option<Uuid>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/purchase-intents",
    request_body = CreateIntentRequest,
    responses((status=200, body=IntentDto), (status=401), (status=409))
)]
pub async fn create_intent(
    axum::extract::State(db): axum::extract::State<Db>,
    auth: AuthUser,
    Json(req): Json<CreateIntentRequest>,
) -> AppResult<Json<IntentDto>> {
    let id = Uuid::new_v4();
    let idem = format!("intent:{}", id);

    let rec = sqlx::query_as::<_, IntentDto>(
        r#"insert into purchase_intents (id, user_id, ticket_type_id, status, idempotency_key)
           values ($1,$2,$3,'ACTIVE',$4)
           returning id, user_id, ticket_type_id, status, order_id, last_error, created_at, updated_at"#,
    )
    .bind(id)
    .bind(auth.user_id)
    .bind(req.ticket_type_id)
    .bind(idem)
    .fetch_one(&db.pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.constraint() == Some("uq_purchase_intents_user_ticket_active") {
                return AppError::Conflict("active intent already exists".into());
            }
        }
        AppError::Db(e)
    })?;

    Ok(Json(rec))
}

#[utoipa::path(
    get,
    path = "/api/purchase-intents/me",
    responses((status=200, body=[IntentDto]), (status=401))
)]
pub async fn my_intents(
    axum::extract::State(db): axum::extract::State<Db>,
    auth: AuthUser,
) -> AppResult<Json<Vec<IntentDto>>> {
    let rows = sqlx::query_as::<_, IntentDto>(
        r#"select id, user_id, ticket_type_id, status, order_id, last_error, created_at, updated_at
           from purchase_intents where user_id=$1 order by created_at desc"#,
    )
    .bind(auth.user_id)
    .fetch_all(&db.pool)
    .await?;

    Ok(Json(rows))
}

pub fn router() -> Router<Db> {
    Router::new()
        .route("/api/purchase-intents", post(create_intent))
        .route("/api/purchase-intents/me", get(my_intents))
}
