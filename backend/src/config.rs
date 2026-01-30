use std::net::SocketAddr;

use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone, Debug)]
pub struct Config {
    pub app_env: String,
    pub server_addr: SocketAddr,
    pub database_url: String,
    pub rate_limit_rps: u32,
    pub rate_limit_burst: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
        let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        std::env::set_var("RUST_LOG", &rust_log);

        let server_addr: SocketAddr = std::env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()?;

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ticketing".to_string());

        let rate_limit_rps: u32 = std::env::var("RATE_LIMIT_RPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let rate_limit_burst: u32 = std::env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        Ok(Self {
            app_env,
            server_addr,
            database_url,
            rate_limit_rps,
            rate_limit_burst,
        })
    }
}

pub fn init_tracing() {
    // If RUST_LOG is not set, Config::from_env sets it.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let use_json = std::env::var("LOG_FORMAT")
        .map(|v| v == "json")
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }
}
