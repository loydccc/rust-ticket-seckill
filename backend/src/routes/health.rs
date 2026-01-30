use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthzResponse {
    pub ok: bool,
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses((status = 200, description = "OK", body = HealthzResponse))
)]
pub async fn healthz() -> Json<HealthzResponse> {
    Json(HealthzResponse { ok: true })
}

pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz))
}
