use sqlx::PgPool;
use uuid::Uuid;

pub async fn validate_session(token: &str, db: &PgPool) -> Option<Uuid> {
    sqlx::query_scalar!(
        r#"SELECT user_id as "user_id: Uuid"
            FROM sessions WHERE token = $1 AND expires_at > NOW()"#,
        token
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
}

pub async fn delete_session(token: &str, db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM sessions WHERE token = $1",
        token
    )
    .execute(db)
    .await?;

    Ok(())
}