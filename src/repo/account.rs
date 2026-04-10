use crate::model::account::AccountRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<AccountRecord>> {
    sqlx::query_as("SELECT * FROM accounts WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_active_by_platform(
    db: &PgPool,
    platform: &str,
) -> sqlx::Result<Vec<AccountRecord>> {
    sqlx::query_as(
        "SELECT * FROM accounts WHERE platform = $1 AND status = 'active' ORDER BY priority DESC",
    )
    .bind(platform)
    .fetch_all(db)
    .await
}

pub async fn update_status(db: &PgPool, id: Uuid, status: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE accounts SET status = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
