use anyhow::Context;
use ticket_seckill_backend::{config, config::Config, db::Db};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::init_tracing();

    let cfg = Config::from_env().context("load config")?;
    let db = Db::connect(&cfg.database_url).await?;

    // Ensure migrations are applied on startup in dev/test.
    db.migrate().await?;

    let listener = tokio::net::TcpListener::bind(&cfg.server_addr)
        .await
        .with_context(|| format!("bind {}", cfg.server_addr))?;

    let app = ticket_seckill_backend::app::build_router(cfg.clone(), db.clone()).await?;

    info!(addr = %cfg.server_addr, "server listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}
