use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{auth, db::Db, error::{AppError, AppResult}};

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses((status=200, body=LoginResponse))
)]
pub async fn login(
    axum::extract::State(db): axum::extract::State<Db>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let username = req.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::BadRequest("username required".into()));
    }

    let row = sqlx::query!(
        r#"insert into users (id, username)
           values ($1, $2)
           on conflict (username) do update set username = excluded.username
           returning id, username"#,
        Uuid::new_v4(),
        username
    )
    .fetch_one(&db.pool)
    .await?;

    let token = auth::issue_token(row.id, &row.username).map_err(AppError::Internal)?;

    Ok(Json(LoginResponse {
        token,
        user_id: row.id,
        username: row.username,
    }))
}

pub fn router() -> Router<Db> {
    Router::new()
        .route("/api/auth/login", post(login))
        // compatibility
        .route("/auth/login", post(login))
}
