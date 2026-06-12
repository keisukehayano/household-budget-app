use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiError,
    models::{
        auth::{AuthResponse, LoginRequest, RegisterRequest},
        user::AuthUserResponse,
    },
    repositories::user as user_repository,
    security::jwt,
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

    if password.len() < 8 {
        errors.push("パスワード8文字以上で入力してください。".to_string())
    }

    if password.len() > 128 {
        errors.push("パスワードは128文字以内で入力してください。".to_string())
    }

    if !errors.is_empty() {
        return Err(ApiError::bad_request(errors));
    }

    Ok(())
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
