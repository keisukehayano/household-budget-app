use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    errors::ApiError,
    models::{
        auth::{AuthResponse, ChangePasswordRequest, LoginRequest, RegisterRequest},
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
) -> Result<StatusCode, ApiError> {
    auth_service::change_password(&state.db, current_user.id, payload).await?;

    Ok(StatusCode::NO_CONTENT)
}
