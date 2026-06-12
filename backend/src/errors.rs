use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
    details: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    message: String,
    details: Vec<String>,
}

impl ApiError {
    pub fn bad_request(details: Vec<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "入力内容が不正です。".to_string(),
            details,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "サーバー内部でエラーが発生しました。".to_string(),
            details: Vec::new(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
            details: Vec::new(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;

        let body = Json(ApiErrorResponse {
            message: self.message,
            details: self.details,
        });

        (status, body).into_response()
    }
}
