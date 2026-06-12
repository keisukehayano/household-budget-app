use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::{errors::ApiError, security::jwt, state::AppState};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = Arc::<AppState>::from_ref(state);

        let authorization = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ApiError::unauthorized("認証が必要です。"))?;

        let token = authorization
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::unauthorized("Authorization header が不正です。"))?;

        let claims = jwt::verify_token(token, &state.jwt_secret)
            .map_err(|_| ApiError::unauthorized("トークンが不正または期限切れです。"))?;

        let user_id = jwt::parse_user_id(&claims)
            .map_err(|_| ApiError::unauthorized("トークンが不正です。"))?;

        Ok(Self { id: user_id })
    }
}
