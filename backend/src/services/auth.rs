use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use chrono::{Duration, Utc};
use rand_core::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::email::EmailClient;

use crate::{
    errors::ApiError,
    models::{
        auth::{
            AuthResponse, ChangePasswordRequest, ForgotPasswordRequest, ForgotPasswordResponse,
            LoginRequest, RegisterRequest, ResetPasswordRequest,
        },
        user::AuthUserResponse,
    },
    repositories::{password_reset as password_reset_repository, user as user_repository},
    security::{jwt, reset_token},
};

pub async fn register(
    db: &PgPool,
    jwt_secret: &str,
    payload: RegisterRequest,
) -> Result<AuthResponse, ApiError> {
    let email = normalize_email(&payload.email);
    validate_email_and_password(&email, &payload.password)?;

    let existing_user = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to find user by email");
            ApiError::internal_server_error()
        })?;

    if existing_user.is_some() {
        return Err(ApiError::conflict(
            "このメールアドレスは既に登録されています。",
        ));
    }

    let password_hash = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    let user = user_repository::create_user(db, user_id, &email, &password_hash)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to create user");
            ApiError::internal_server_error()
        })?;

    let token = jwt::generate_token(&user, jwt_secret).map_err(|error| {
        tracing::error!(?error, "failed to generate jwt");
        ApiError::internal_server_error()
    })?;

    Ok(AuthResponse {
        token,
        user: AuthUserResponse::from(user),
    })
}

pub async fn login(
    db: &PgPool,
    jwt_secret: &str,
    payload: LoginRequest,
) -> Result<AuthResponse, ApiError> {
    let email = normalize_email(&payload.email);

    let Some(user) = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to find user by email");
            ApiError::internal_server_error()
        })?
    else {
        return Err(ApiError::unauthorized(
            "メールアドレスまたはパスワードが正しくありません。",
        ));
    };

    let is_valid_password = verify_password(&payload.password, &user.password_hash)?;

    if !is_valid_password {
        return Err(ApiError::unauthorized(
            "メールアドレスまたはパスワードが正しくありません。",
        ));
    }

    let token = jwt::generate_token(&user, jwt_secret).map_err(|error| {
        tracing::error!(?error, "failed to generate jwt");
        ApiError::internal_server_error()
    })?;

    Ok(AuthResponse {
        token,
        user: AuthUserResponse::from(user),
    })
}

pub async fn find_me(db: &PgPool, user_id: Uuid) -> Result<AuthUserResponse, ApiError> {
    let user = user_repository::find_user_by_id(db, user_id)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to find user by id");
            ApiError::internal_server_error()
        })?
        .ok_or_else(|| ApiError::unauthorized("ユーザーが見つかりません。"))?;

    Ok(AuthUserResponse::from(user))
}

pub async fn change_password(
    db: &PgPool,
    jwt_secret: &str,
    user_id: Uuid,
    payload: ChangePasswordRequest,
) -> Result<AuthResponse, ApiError> {
    let user = user_repository::find_user_by_id(db, user_id)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to find user by id");
            ApiError::internal_server_error()
        })?
        .ok_or_else(|| ApiError::unauthorized("ユーザーが見つかりません。"))?;

    let is_valid_current_password =
        verify_password(&payload.current_password, &user.password_hash)?;

    if !is_valid_current_password {
        return Err(ApiError::bad_request(vec![
            "現在のパスワードが正しくありません。".to_string(),
        ]));
    }

    if payload.current_password == payload.new_password {
        return Err(ApiError::bad_request(vec![
            "新しいパスワードは現在のパスワードと異なるものを入力してください。".to_string(),
        ]));
    }

    validate_password(&payload.new_password)?;

    let new_password_hash = hash_password(&payload.new_password)?;

    let updated_user = user_repository::update_user_password_hash_and_increment_token_version(
        db,
        user_id,
        &new_password_hash,
    )
    .await
    .map_err(|error| {
        tracing::error!(?error, "failed to update password hash");
        ApiError::internal_server_error()
    })?
    .ok_or_else(|| ApiError::unauthorized("ユーザーが見つかりません。"))?;

    let token = jwt::generate_token(&updated_user, jwt_secret).map_err(|error| {
        tracing::error!(?error, "failed to generate jwt after password change");
        ApiError::internal_server_error()
    })?;

    Ok(AuthResponse {
        token,
        user: AuthUserResponse::from(updated_user),
    })
}

pub async fn forgot_password(
    db: &PgPool,
    frontend_url: &str,
    email_client: &EmailClient,
    payload: ForgotPasswordRequest,
) -> Result<ForgotPasswordResponse, ApiError> {
    let email = normalize_email(&payload.email);

    if email.is_empty() || !email.contains('@') {
        return Err(ApiError::bad_request(vec![
            "メールアドレスの形式が不正です。".to_string(),
        ]));
    }

    let response = ForgotPasswordResponse {
        message:
            "入力されたメールアドレスが登録されている場合、パスワード再設定用の案内を送信しました。"
                .to_string(),
    };

    let Some(user) = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to find user for password reset");
            ApiError::internal_server_error()
        })?
    else {
        return Ok(response);
    };

    let token = reset_token::generate_password_reset_token();
    let token_hash = reset_token::hash_password_reset_token(&token);

    let expires_at = Utc::now()
        .checked_add_signed(Duration::minutes(30))
        .expect("valid password reset token expiration");

    password_reset_repository::mark_user_password_reset_tokens_used(db, user.id)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to mark old password reset tokens used");
            ApiError::internal_server_error()
        })?;

    password_reset_repository::create_password_reset_token(
        db,
        Uuid::new_v4(),
        user.id,
        &token_hash,
        expires_at,
    )
    .await
    .map_err(|error| {
        tracing::error!(?error, "failed to create password reset token");
        ApiError::internal_server_error()
    })?;

    let reset_url = format!(
        "{}/reset-password?token={}",
        frontend_url.trim_end_matches('/'),
        token
    );

    if let Err(error) = email_client
        .send_password_reset_email(&user.email, &reset_url)
        .await
    {
        tracing::error!(
            ?error,
            email = %user.email,
            "failed to send password reset email"
        );
    }

    Ok(response)
}

pub async fn reset_password(db: &PgPool, payload: ResetPasswordRequest) -> Result<(), ApiError> {
    validate_password(&payload.new_password)?;

    let token = payload.token.trim();

    if token.is_empty() {
        return Err(ApiError::bad_request(vec![
            "再設定トークンが不正です。".to_string(),
        ]));
    }

    let token_hash = reset_token::hash_password_reset_token(token);

    let reset_token_row =
        password_reset_repository::find_valid_password_reset_token(db, &token_hash)
            .await
            .map_err(|error| {
                tracing::error!(?error, "failed to find password reset token");
                ApiError::internal_server_error()
            })?
            .ok_or_else(|| {
                ApiError::bad_request(vec![
                    "再設定リンクが無効、または有効期限切れです。".to_string(),
                ])
            })?;

    let new_password_hash = hash_password(&payload.new_password)?;

    let is_updated = password_reset_repository::consume_token_and_update_password(
        db,
        reset_token_row.id,
        reset_token_row.user_id,
        &new_password_hash,
    )
    .await
    .map_err(|error| {
        tracing::error!(?error, "failed to reset password");
        ApiError::internal_server_error()
    })?;

    if !is_updated {
        return Err(ApiError::bad_request(vec![
            "再設定リンクが無効、または有効期限切れです。".to_string(),
        ]));
    }

    Ok(())
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_email_and_password(email: &str, password: &str) -> Result<(), ApiError> {
    let mut errors = Vec::new();

    if email.is_empty() {
        errors.push("メールアドレスを入力してください。".to_string());
    } else if !email.contains('@') {
        errors.push("メールアドレスの形式が不正です。".to_string());
    }

    errors.extend(validate_password_errors(password));

    if !errors.is_empty() {
        return Err(ApiError::bad_request(errors));
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    let errors = validate_password_errors(password);

    if !errors.is_empty() {
        return Err(ApiError::bad_request(errors));
    }

    Ok(())
}

fn validate_password_errors(password: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if password.len() < 8 {
        errors.push("パスワードは8文字以上で入力してください。".to_string());
    }

    if password.len() > 128 {
        errors.push("パスワードは128文字以内で入力してください。".to_string());
    }

    errors
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|password_hash| password_hash.to_string())
        .map_err(|error| {
            tracing::error!(?error, "failed tp parse password hash");
            ApiError::internal_server_error()
        })
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|error| {
        tracing::error!(?error, "failed to parse password hash");
        ApiError::internal_server_error()
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{body::to_bytes, response::IntoResponse};
    use chrono::Utc;
    use serde_json::Value;
    use sqlx::{PgPool, Row};

    use crate::{
        email::EmailClient,
        models::auth::{ForgotPasswordRequest, ResetPasswordRequest},
        repositories::{password_reset as password_reset_repository, user as user_repository},
    };

    async fn create_test_user(
        pool: &PgPool,
        email: &str,
        password: &str,
    ) -> crate::models::user::UserRow {
        let password_hash = hash_password(password).expect("test password hash should be created");

        user_repository::create_user(pool, Uuid::new_v4(), email, &password_hash)
            .await
            .expect("test user should be created")
    }

    async fn count_password_reset_tokens(pool: &PgPool, user_id: Uuid) -> i64 {
        sqlx::query_scalar(
            r#"
            select count(*)
            from password_reset_tokens
            where user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("password reset token count should be fetched")
    }

    async fn error_response_json(error: ApiError) -> (axum::http::StatusCode, Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error response body should be read");
        let json = serde_json::from_slice(&bytes).expect("error response body should be json");

        (status, json)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn forgot_password_returns_generic_message_for_unknown_email(pool: PgPool) {
        let response = forgot_password(
            &pool,
            "http://127.0.0.1:5173",
            &EmailClient::new_for_tests(),
            ForgotPasswordRequest {
                email: "missing@example.com".to_string(),
            },
        )
        .await
        .expect("forgot password should succeed");

        assert_eq!(
            response.message,
            "入力されたメールアドレスが登録されている場合、パスワード再設定用の案内を送信しました。"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn forgot_password_marks_old_tokens_used_and_creates_new_token(pool: PgPool) {
        let user = create_test_user(&pool, "reset@example.com", "OldPassword123").await;
        let old_token_hash = reset_token::hash_password_reset_token("old-token");

        password_reset_repository::create_password_reset_token(
            &pool,
            Uuid::new_v4(),
            user.id,
            &old_token_hash,
            Utc::now() + Duration::minutes(30),
        )
        .await
        .expect("old password reset token should be created");

        let response = forgot_password(
            &pool,
            "http://127.0.0.1:5173/",
            &EmailClient::new_for_tests(),
            ForgotPasswordRequest {
                email: user.email.clone(),
            },
        )
        .await
        .expect("forgot password should succeed");

        assert_eq!(
            response.message,
            "入力されたメールアドレスが登録されている場合、パスワード再設定用の案内を送信しました。"
        );
        assert_eq!(count_password_reset_tokens(&pool, user.id).await, 2);

        let rows = sqlx::query(
            r#"
            select token_hash, used_at
            from password_reset_tokens
            where user_id = $1
            order by created_at asc
            "#,
        )
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .expect("password reset tokens should be loaded");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get::<String, _>("token_hash"), old_token_hash);
        assert!(
            rows[0]
                .get::<Option<chrono::DateTime<Utc>>, _>("used_at")
                .is_some()
        );
        assert!(
            rows[1]
                .get::<Option<chrono::DateTime<Utc>>, _>("used_at")
                .is_none()
        );
        assert_ne!(rows[1].get::<String, _>("token_hash"), old_token_hash);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn reset_password_updates_password_and_consumes_token(pool: PgPool) {
        let user = create_test_user(&pool, "reset@example.com", "OldPassword123").await;
        let raw_token = "valid-reset-token";
        let token_hash = reset_token::hash_password_reset_token(raw_token);

        let token_row = password_reset_repository::create_password_reset_token(
            &pool,
            Uuid::new_v4(),
            user.id,
            &token_hash,
            Utc::now() + Duration::minutes(30),
        )
        .await
        .expect("password reset token should be created");

        reset_password(
            &pool,
            ResetPasswordRequest {
                token: raw_token.to_string(),
                new_password: "NewPassword123".to_string(),
            },
        )
        .await
        .expect("reset password should succeed");

        let updated_user = user_repository::find_user_by_id(&pool, user.id)
            .await
            .expect("updated user should be fetched")
            .expect("updated user should exist");

        assert!(verify_password("NewPassword123", &updated_user.password_hash).unwrap());
        assert_eq!(updated_user.token_version, user.token_version + 1);

        let used_at = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
            r#"
            select used_at
            from password_reset_tokens
            where id = $1
            "#,
        )
        .bind(token_row.id)
        .fetch_one(&pool)
        .await
        .expect("used_at should be loaded");

        assert!(used_at.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn reset_password_rejects_used_token(pool: PgPool) {
        let user = create_test_user(&pool, "reset@example.com", "OldPassword123").await;
        let raw_token = "single-use-token";
        let token_hash = reset_token::hash_password_reset_token(raw_token);

        password_reset_repository::create_password_reset_token(
            &pool,
            Uuid::new_v4(),
            user.id,
            &token_hash,
            Utc::now() + Duration::minutes(30),
        )
        .await
        .expect("password reset token should be created");

        reset_password(
            &pool,
            ResetPasswordRequest {
                token: raw_token.to_string(),
                new_password: "NewPassword123".to_string(),
            },
        )
        .await
        .expect("first reset password should succeed");

        let error = reset_password(
            &pool,
            ResetPasswordRequest {
                token: raw_token.to_string(),
                new_password: "AnotherPassword123".to_string(),
            },
        )
        .await
        .expect_err("used token should be rejected");

        let (status, body) = error_response_json(error).await;

        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(body["message"], "入力内容が不正です。");
        assert_eq!(
            body["details"],
            serde_json::json!(["再設定リンクが無効、または有効期限切れです。"])
        );
    }
}
