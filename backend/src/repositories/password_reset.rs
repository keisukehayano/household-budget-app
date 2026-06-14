use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::password_reset::PasswordResetTokenRow;

pub async fn mark_user_password_reset_tokens_used(
    db: &PgPool,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        update password_reset_tokens
        set used_at = now()
        where user_id = $1
          and used_at is null
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn create_password_reset_token(
    db: &PgPool,
    id: Uuid,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<PasswordResetTokenRow, sqlx::Error> {
    sqlx::query_as::<_, PasswordResetTokenRow>(
        r#"
        insert into password_reset_tokens (
            id,
            user_id,
            token_hash,
            expires_at
        )
        values ($1, $2, $3, $4)
        returning
            id,
            user_id,
            token_hash,
            expires_at,
            used_at,
            created_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(db)
    .await
}

pub async fn find_valid_password_reset_token(
    db: &PgPool,
    token_hash: &str,
) -> Result<Option<PasswordResetTokenRow>, sqlx::Error> {
    sqlx::query_as::<_, PasswordResetTokenRow>(
        r#"
        select
            id,
            user_id,
            token_hash,
            expires_at,
            used_at,
            created_at
        from password_reset_tokens
        where token_hash = $1
          and used_at is null
          and expires_at > now()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
}

pub async fn consume_token_and_update_password(
    db: &PgPool,
    token_id: Uuid,
    user_id: Uuid,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    let mut transaction = db.begin().await?;

    let token_result = sqlx::query(
        r#"
        update password_reset_tokens
        set used_at = now()
        where id = $1
          and used_at is null
          and expires_at > now()
        "#,
    )
    .bind(token_id)
    .execute(&mut *transaction)
    .await?;

    if token_result.rows_affected() == 0 {
        transaction.rollback().await?;
        return Ok(false);
    }

    let user_result = sqlx::query(
        r#"
        update users
        set
            password_hash = $1,
            token_version = token_version + 1,
            updated_at = now()
        where id = $2
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(user_result.rows_affected() == 1)
}
