use crate::model::usage::UsageRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert(
    db: &PgPool,
    api_key_id: Uuid,
    account_id: Uuid,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cost_usd: f64,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO usage_logs (id, api_key_id, account_id, model, input_tokens, output_tokens, cost_usd)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(api_key_id)
    .bind(account_id)
    .bind(model)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(cost_usd)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn daily_cost_for_key(
    db: &PgPool,
    api_key_id: Uuid,
    date: chrono::NaiveDate,
) -> sqlx::Result<f64> {
    let row: (f64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(cost_usd), 0.0)
        FROM usage_logs
        WHERE api_key_id = $1 AND created_at::date = $2
        "#,
    )
    .bind(api_key_id)
    .bind(date)
    .fetch_one(db)
    .await?;
    Ok(row.0)
}

pub async fn recent_by_key(
    db: &PgPool,
    api_key_id: Uuid,
    limit: i64,
) -> sqlx::Result<Vec<UsageRecord>> {
    sqlx::query_as(
        "SELECT * FROM usage_logs WHERE api_key_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(api_key_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
