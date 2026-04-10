use crate::model::apikey::ApiKeyRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_hash(db: &PgPool, key_hash: &str) -> sqlx::Result<Option<ApiKeyRecord>> {
    sqlx::query_as("SELECT * FROM api_keys WHERE key_hash = $1")
        .bind(key_hash)
        .fetch_optional(db)
        .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<ApiKeyRecord>> {
    sqlx::query_as("SELECT * FROM api_keys WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn list_all(db: &PgPool) -> sqlx::Result<Vec<ApiKeyRecord>> {
    sqlx::query_as("SELECT * FROM api_keys ORDER BY created_at DESC")
        .fetch_all(db)
        .await
}

pub async fn delete(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
