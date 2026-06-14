use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::UserRow;

pub async fn find_user_by_email(db: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        select
            id,
            email,
            password_hash,
            token_version,
            created_at,
            updated_at
        from users
        where email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_id(db: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        select
            id,
            email,
            password_hash,
            token_version,
            created_at,
            updated_at
        from users
        where id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn create_user(
    db: &PgPool,
    id: Uuid,
    email: &str,
    password_hash: &str,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        insert into users (
            id,
            email,
            password_hash
        )
        values ($1, $2, $3)
        returning
            id,
            email,
            password_hash,
            token_version,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .fetch_one(db)
    .await
}

pub async fn update_user_password_hash_and_increment_token_version(
    db: &PgPool,
    id: Uuid,
    password_hash: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        update users
        set
            password_hash = $1,
            token_version = token_version + 1,
            updated_at = now()
        where id = $2
        returning
            id,
            email,
            password_hash,
            token_version,
            created_at,
            updated_at
        "#,
    )
    .bind(password_hash)
    .bind(id)
    .fetch_optional(db)
    .await
}
