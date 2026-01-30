use chrono::{Duration, Utc};
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use ticket_seckill_backend::{app, config::Config, db::Db};

async fn setup() -> (String, PgPool) {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let db = Db::connect(&database_url).await.unwrap();
    db.migrate().await.unwrap();

    // Clean between tests.
    sqlx::query("truncate table purchase_intents, orders, users, ticket_types, events restart identity cascade")
        .execute(&db.pool)
        .await
        .unwrap();

    let cfg = Config {
        app_env: "test".into(),
        server_addr: "127.0.0.1:0".parse().unwrap(),
        database_url,
        rate_limit_rps: 10_000,
        rate_limit_burst: 10_000,
    };

    let listener = tokio::net::TcpListener::bind(cfg.server_addr).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app::build_router(cfg, db.clone()).await.unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    (format!("http://{}", addr), db.pool)
}

async fn login(client: &Client, base: &str, username: &str) -> String {
    let res = client
        .post(format!("{}/api/auth/login", base))
        .json(&json!({"username": username}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    res["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn grab_is_idempotent_and_atomic() {
    let (base, pool) = setup().await;
    let client = Client::new();

    let token_u1 = login(&client, &base, "u1").await;
    let token_u2 = login(&client, &base, "u2").await;

    // Create event
    let starts_at = Utc::now();
    let ends_at = starts_at + Duration::hours(2);
    let ev = client
        .post(format!("{}/api/admin/events", base))
        .json(&json!({"name":"concert","starts_at":starts_at,"ends_at":ends_at}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let event_id = ev["id"].as_str().unwrap();

    // Create ticket type inventory 1
    let sale_starts_at = Utc::now() - Duration::minutes(1);
    let sale_ends_at = Utc::now() + Duration::minutes(30);
    let tt = client
        .post(format!("{}/api/admin/events/{}/ticket_types", base, event_id))
        .json(&json!({
            "name":"A",
            "price_cents":100,
            "inventory_total":1,
            "sale_starts_at": sale_starts_at,
            "sale_ends_at": sale_ends_at
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let ticket_type_id = tt["id"].as_str().unwrap();

    // First grab should succeed
    let idem = "k1";
    let order1 = client
        .post(format!("{}/api/tickets/grab", base))
        .bearer_auth(&token_u1)
        .header("idempotency-key", idem)
        .json(&json!({"ticket_type_id": ticket_type_id, "qty": 1}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    // Repeat with same idempotency key should return same order id
    let order2 = client
        .post(format!("{}/api/tickets/grab", base))
        .bearer_auth(&token_u1)
        .header("idempotency-key", idem)
        .json(&json!({"ticket_type_id": ticket_type_id, "qty": 1}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(order1["id"], order2["id"]);

    // Another user should be out of stock
    let resp = client
        .post(format!("{}/api/tickets/grab", base))
        .bearer_auth(&token_u2)
        .header("idempotency-key", "k2")
        .json(&json!({"ticket_type_id": ticket_type_id, "qty": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 409);

    // inventory_remaining should be 0
    let remaining: i32 = sqlx::query_scalar("select inventory_remaining from ticket_types where id = $1")
        .bind(uuid::Uuid::parse_str(ticket_type_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
}
