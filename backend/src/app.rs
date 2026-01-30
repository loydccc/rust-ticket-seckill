use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::Config, db::Db, openapi::ApiDoc, routes};

pub async fn build_router(cfg: Config, db: Db) -> anyhow::Result<Router<Db>> {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(cfg.rate_limit_rps)
        .burst_size(cfg.rate_limit_burst)
        .finish()
        .expect("valid governor config");

    let app = Router::new()
        .with_state(db)
        .merge(routes::health::router())
        .merge(routes::auth::router())
        .merge(routes::admin::router())
        .merge(routes::seckill::router())
        .merge(routes::orders::router())
        .route("/", get(|| async { (StatusCode::OK, "ticket-seckill-backend") }))
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .layer(GovernorLayer {
            config: std::sync::Arc::new(governor_conf),
        })
        .fallback(|| async { (StatusCode::NOT_FOUND, "not found") }.into_response());

    Ok(app)
}
