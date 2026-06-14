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
