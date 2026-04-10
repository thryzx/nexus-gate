use crate::model::account::AccountRecord;
use crate::state::AppState;
use anyhow::Result;
use uuid::Uuid;

/// Create a new account record.
pub async fn create(
    state: &AppState,
    name: &str,
    platform: &str,
    account_type: &str,
    credentials_enc: &str,
    priority: i32,
    max_concurrency: i32,
    proxy_url: Option<&str>,
    fingerprint_profile_id: Option<Uuid>,
) -> Result<AccountRecord> {
    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        INSERT INTO accounts (id, name, platform, account_type, credentials_enc, status, priority, max_concurrency, proxy_url, fingerprint_profile_id)
        VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(platform)
    .bind(account_type)
    .bind(credentials_enc)
    .bind(priority)
    .bind(max_concurrency)
    .bind(proxy_url)
    .bind(fingerprint_profile_id)
    .fetch_one(&state.db)
    .await?;

    Ok(row)
}

/// List all accounts, optionally filtered by platform.
pub async fn list(state: &AppState, platform: Option<&str>) -> Result<Vec<AccountRecord>> {
    let rows = match platform {
        Some(p) => {
            sqlx::query_as::<_, AccountRecord>(
                "SELECT * FROM accounts WHERE platform = $1 ORDER BY priority DESC",
            )
            .bind(p)
            .fetch_all(&state.db)
            .await?
        }
        None => {
            sqlx::query_as::<_, AccountRecord>(
                "SELECT * FROM accounts ORDER BY priority DESC",
            )
            .fetch_all(&state.db)
            .await?
        }
    };
    Ok(rows)
}

/// Mark an account as temporarily unavailable (529/overload).
/// Sets a Redis cooldown key for automatic reactivation.
pub async fn mark_unavailable(state: &AppState, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE accounts SET status = 'unavailable', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    let mut conn = state.redis_conn().await?;
    let key = format!("cooldown:{id}");
    redis::cmd("SET")
        .arg(&key)
        .arg("1")
        .arg("EX")
        .arg(state.config.scheduler.overload_cooldown_secs)
        .exec_async(&mut conn)
        .await?;

    Ok(())
}

/// Mark account as error (401 — invalid credentials).
pub async fn mark_error(state: &AppState, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE accounts SET status = 'error', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Mark account as blocked (403 — org disabled, access denied).
pub async fn mark_blocked(state: &AppState, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE accounts SET status = 'blocked', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Reactivate an account.
pub async fn reactivate(state: &AppState, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE accounts SET status = 'active', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}
