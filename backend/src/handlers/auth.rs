use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    errors::ApiError,
    models::{
        auth::{
            AuthResponse, ChangePasswordRequest, ForgotPasswordRequest, ForgotPasswordResponse,
            LoginRequest, RegisterRequest, ResetPasswordRequest,
        },
        user::AuthUserResponse,
    },
    security::current_user::CurrentUser,
    services::auth as auth_service,
    state::AppState,
};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    let response = auth_service::register(&state.db, &state.jwt_secret, payload).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let response = auth_service::login(&state.db, &state.jwt_secret, payload).await?;

    Ok(Json(response))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
) -> Result<Json<AuthUserResponse>, ApiError> {
    let response = auth_service::find_me(&state.db, current_user.id).await?;

    Ok(Json(response))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let response =
        auth_service::change_password(&state.db, &state.jwt_secret, current_user.id, payload)
            .await?;

    Ok(Json(response))
}

pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, ApiError> {
    let response =
        auth_service::forgot_password(&state.db, &state.frontend_url, &state.email_client, payload)
            .await?;

    Ok(Json(response))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<StatusCode, ApiError> {
    auth_service::reset_password(&state.db, payload).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
        response::Response,
        routing::post,
    };
    use chrono::{Duration, Utc};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        email::EmailClient,
        repositories::{password_reset as password_reset_repository, user as user_repository},
        security::reset_token,
    };

    const TEST_JWT_SECRET: &str = "test-jwt-secret";

    async fn build_test_app(pool: PgPool) -> Router {
        let state = Arc::new(AppState {
            db: pool,
            jwt_secret: TEST_JWT_SECRET.to_string(),
            frontend_url: "http://127.0.0.1:5173".to_string(),
            email_client: EmailClient::new_for_tests(),
        });

        Router::new()
            .route("/api/auth/forgot-password", post(forgot_password))
            .route("/api/auth/reset-password", post(reset_password))
            .with_state(state)
    }

    async fn create_test_user(pool: &PgPool, email: &str) -> crate::models::user::UserRow {
        user_repository::create_user(pool, Uuid::new_v4(), email, "dummy-password-hash")
            .await
            .expect("test user should be created")
    }

    fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn response_json(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read");

        serde_json::from_slice(&bytes).expect("response body should be valid JSON")
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn forgot_password_returns_generic_message_and_creates_token(pool: PgPool) {
        let app = build_test_app(pool.clone()).await;
        let user = create_test_user(&pool, "forgot@example.com").await;

        let response = app
            .oneshot(json_request(
                Method::POST,
                "/api/auth/forgot-password",
                json!({ "email": user.email }),
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;

        assert_eq!(
            body["message"],
            "入力されたメールアドレスが登録されている場合、パスワード再設定用の案内を送信しました。"
        );

        let token_count = sqlx::query_scalar::<_, i64>(
            r#"
            select count(*)
            from password_reset_tokens
            where user_id = $1
            "#,
        )
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .expect("password reset token count should be fetched");

        assert_eq!(token_count, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn forgot_password_returns_bad_request_for_invalid_email(pool: PgPool) {
        let app = build_test_app(pool).await;

        let response = app
            .oneshot(json_request(
                Method::POST,
                "/api/auth/forgot-password",
                json!({ "email": "invalid-email" }),
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;

        assert_eq!(body["message"], "入力内容が不正です。");
        assert_eq!(body["details"], json!(["メールアドレスの形式が不正です。"]));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn reset_password_returns_no_content_and_consumes_token(pool: PgPool) {
        let app = build_test_app(pool.clone()).await;
        let user = create_test_user(&pool, "reset@example.com").await;
        let raw_token = "handler-reset-token";
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

        let response = app
            .oneshot(json_request(
                Method::POST,
                "/api/auth/reset-password",
                json!({
                    "token": raw_token,
                    "newPassword": "NewPassword123"
                }),
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let user_row = user_repository::find_user_by_id(&pool, user.id)
            .await
            .expect("updated user should be fetched")
            .expect("updated user should exist");
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
        .expect("used_at should be fetched");

        assert_eq!(user_row.token_version, user.token_version + 1);
        assert!(used_at.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn reset_password_returns_bad_request_for_invalid_token(pool: PgPool) {
        let app = build_test_app(pool).await;

        let response = app
            .oneshot(json_request(
                Method::POST,
                "/api/auth/reset-password",
                json!({
                    "token": "missing-token",
                    "newPassword": "NewPassword123"
                }),
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;

        assert_eq!(body["message"], "入力内容が不正です。");
        assert_eq!(
            body["details"],
            json!(["再設定リンクが無効、または有効期限切れです。"])
        );
    }
}
