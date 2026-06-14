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
    let response = auth_service::forgot_password(&state.db, &state.frontend_url, payload).await?;

    Ok(Json(response))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<StatusCode, ApiError> {
    auth_service::reset_password(&state.db, payload).await?;

    Ok(StatusCode::NO_CONTENT)
}
