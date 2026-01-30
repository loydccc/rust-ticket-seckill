use crate::{db::Db, error::AppError};
use chrono::Utc;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
struct IntentRow {
    id: Uuid,
    user_id: Uuid,
    ticket_type_id: Uuid,
    idempotency_key: String,
}

pub fn spawn_intent_worker(db: Db) {
    tokio::spawn(async move {
        info!("purchase_intents worker started");
        loop {
            if let Err(e) = tick(&db).await {
                error!(err = ?e, "purchase_intents worker tick failed");
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });
}

async fn tick(db: &Db) -> anyhow::Result<()> {
    // Claim a small batch of ACTIVE intents.
    let intents: Vec<IntentRow> = sqlx::query_as(
        r#"select id, user_id, ticket_type_id, idempotency_key
           from purchase_intents
           where status='ACTIVE'
           order by created_at asc
           limit 50"#,
    )
    .fetch_all(&db.pool)
    .await?;

    if intents.is_empty() {
        return Ok(());
    }

    for intent in intents {
        match try_fulfill_intent(db, &intent).await {
            Ok(()) => {}
            Err(AppError::Conflict(msg)) => {
                // not started / out of stock - keep ACTIVE but record last_error
                let _ = sqlx::query(
                    r#"update purchase_intents set last_error=$2, updated_at=now() where id=$1"#,
                )
                .bind(intent.id)
                .bind(msg)
                .execute(&db.pool)
                .await;
            }
            Err(e) => {
                error!(intent_id=%intent.id, err=?e, "intent fulfill error");
                let _ = sqlx::query(
                    r#"update purchase_intents set last_error=$2, updated_at=now() where id=$1"#,
                )
                .bind(intent.id)
                .bind(format!("{e}"))
                .execute(&db.pool)
                .await;
            }
        }
    }

    Ok(())
}

async fn try_fulfill_intent(db: &Db, intent: &IntentRow) -> Result<(), AppError> {
    let mut tx = db.pool.begin().await.map_err(AppError::Db)?;

    // If already fulfilled by another tick, stop.
    let status: Option<(String, Option<Uuid>)> = sqlx::query_as(
        r#"select status, order_id from purchase_intents where id=$1 for update"#,
    )
    .bind(intent.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Db)?;

    let Some((status, order_id)) = status else {
        tx.rollback().await.map_err(AppError::Db)?;
        return Ok(());
    };

    if status != "ACTIVE" {
        tx.rollback().await.map_err(AppError::Db)?;
        return Ok(());
    }

    if let Some(oid) = order_id {
        // double-check order exists
        let ok: Option<(Uuid,)> = sqlx::query_as("select id from orders where id=$1")
            .bind(oid)
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::Db)?;
        if ok.is_some() {
            sqlx::query(
                r#"update purchase_intents set status='FULFILLED', updated_at=now() where id=$1"#,
            )
            .bind(intent.id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Db)?;
            tx.commit().await.map_err(AppError::Db)?;
            return Ok(());
        }
    }

    // Try create order using same atomic decrement strategy.
    let now = Utc::now();

    // 1) If user already has active order, attach it and mark intent fulfilled.
    if let Some((oid,)) = sqlx::query_as::<_, (Uuid,)>(
        r#"select id from orders where user_id=$1 and ticket_type_id=$2 and status in ('CREATED','PAID') order by created_at desc limit 1"#,
    )
    .bind(intent.user_id)
    .bind(intent.ticket_type_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Db)?
    {
        sqlx::query(
            r#"update purchase_intents set status='FULFILLED', order_id=$2, updated_at=now() where id=$1"#,
        )
        .bind(intent.id)
        .bind(oid)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Db)?;
        tx.commit().await.map_err(AppError::Db)?;
        return Ok(());
    }

    // 2) Atomic decrement
    let price: Option<(i64,)> = sqlx::query_as(
        r#"update ticket_types
           set inventory_remaining = inventory_remaining - 1
           where id = $1
             and inventory_remaining >= 1
             and sale_starts_at <= $2
             and sale_ends_at > $2
           returning price_cents"#,
    )
    .bind(intent.ticket_type_id)
    .bind(now)
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Db)?;

    let Some((price_cents,)) = price else {
        tx.rollback().await.map_err(AppError::Db)?;
        return Err(AppError::Conflict("out of stock or not in sale window".into()));
    };

    // 3) Insert order, idempotency_key fixed per intent.
    let order_id = Uuid::new_v4();
    let inserted = sqlx::query_as::<_, (Uuid,)>(
        r#"insert into orders (id, user_id, ticket_type_id, qty, amount_cents, status, idempotency_key)
           values ($1,$2,$3,1,$4,'CREATED',$5)
           returning id"#,
    )
    .bind(order_id)
    .bind(intent.user_id)
    .bind(intent.ticket_type_id)
    .bind(price_cents)
    .bind(&intent.idempotency_key)
    .fetch_one(&mut *tx)
    .await;

    let oid = match inserted {
        Ok((oid,)) => oid,
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.constraint() == Some("uq_orders_user_ticket_type_active")
                    || db_err.constraint() == Some("uq_orders_user_idempotency")
                {
                    let (oid,) = sqlx::query_as::<_, (Uuid,)>(
                        r#"select id from orders where user_id=$1 and ticket_type_id=$2 and status in ('CREATED','PAID') order by created_at desc limit 1"#,
                    )
                    .bind(intent.user_id)
                    .bind(intent.ticket_type_id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(AppError::Db)?;
                    oid
                } else {
                    return Err(AppError::Db(e));
                }
            } else {
                return Err(AppError::Db(e));
            }
        }
    };

    debug!(intent_id=%intent.id, order_id=%oid, "intent fulfilled");

    sqlx::query(
        r#"update purchase_intents set status='FULFILLED', order_id=$2, last_error=null, updated_at=now() where id=$1"#,
    )
    .bind(intent.id)
    .bind(oid)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)?;

    tx.commit().await.map_err(AppError::Db)?;
    Ok(())
}
