use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    errors::ApiError,
    models::transaction::{
        TransactionCreateRequest, TransactionListQuery, TransactionListResponse,
        TransactionResponse, TransactionSummaryResponse, TransactionUpdateRequest,
    },
    security::current_user::CurrentUser,
    services::transaction as transaction_service,
    state::AppState,
};

pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<TransactionListQuery>,
) -> Result<Json<TransactionListResponse>, ApiError> {
    let response =
        transaction_service::list_transactions(&state.db, current_user.id, query).await?;

    Ok(Json(response))
}

pub async fn summarize_transactions(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<TransactionListQuery>,
) -> Result<Json<TransactionSummaryResponse>, ApiError> {
    let response =
        transaction_service::summarize_transactions(&state.db, current_user.id, query).await?;

    Ok(Json(response))
}

pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(payload): Json<TransactionCreateRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), ApiError> {
    let transaction =
        transaction_service::create_transaction(&state.db, current_user.id, payload).await?;

    Ok((StatusCode::CREATED, Json(transaction)))
}

pub async fn update_transaction(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransactionUpdateRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let transaction =
        transaction_service::update_transaction(&state.db, current_user.id, id, payload).await?;

    Ok(Json(transaction))
}

pub async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    transaction_service::delete_transaction(&state.db, current_user.id, id).await?;

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
        routing::{get, put},
    };
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        models::user::UserRow, repositories::user as user_repository, security::jwt,
        state::AppState,
    };

    const TEST_JWT_SECRET: &str = "test-jwt-secret";

    async fn create_test_user(pool: &PgPool, email: &str) -> UserRow {
        user_repository::create_user(pool, Uuid::new_v4(), email, "dummy-password-hash")
            .await
            .expect("test user should be created")
    }

    async fn build_test_app(pool: PgPool) -> (Router, String) {
        let test_user = create_test_user(&pool, "handler@example.com").await;
        let token = jwt::generate_token(&test_user, TEST_JWT_SECRET)
            .expect("test token should be generated");
        let state = Arc::new(AppState {
            db: pool,
            jwt_secret: TEST_JWT_SECRET.to_string(),
        });

        (
            Router::new()
                .route("/api/transactions/summary", get(summarize_transactions))
                .route(
                    "/api/transactions",
                    get(list_transactions).post(create_transaction),
                )
                .route(
                    "/api/transactions/{id}",
                    put(update_transaction).delete(delete_transaction),
                )
                .with_state(state),
            token,
        )
    }

    fn json_request(method: Method, uri: &str, body: Value, token: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn empty_request(method: Method, uri: &str, token: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    async fn response_json(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read");

        serde_json::from_slice(&bytes).expect("response body should be valid JSON")
    }

    async fn post_transaction(app: &Router, token: &str, body: Value) -> Value {
        let response = app
            .clone()
            .oneshot(json_request(Method::POST, "/api/transactions", body, token))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::CREATED);

        response_json(response).await
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn post_transactions_creates_transaction(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let response = app
            .oneshot(json_request(
                Method::POST,
                "/api/transactions",
                json!({
                    "type": "expense",
                    "date": "2024-06-11",
                    "category": "food",
                    "amount": 1200,
                    "memo": "昼食"
                }),
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response_json(response).await;

        assert_eq!(body["type"], "expense");
        assert_eq!(body["date"], "2024-06-11");
        assert_eq!(body["category"], "food");
        assert_eq!(body["amount"], 1200);
        assert_eq!(body["memo"], "昼食");
        assert!(body["id"].is_string());
        assert!(body["createdAt"].is_string());
        assert!(body["updatedAt"].is_string());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn post_transactions_returns_bad_request_for_invalid_payload(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let response = app
            .oneshot(json_request(
                Method::POST,
                "/api/transactions",
                json!({
                    "type": "invalid",
                    "date": "2024-06-11",
                    "category": "unknown",
                    "amount": 0,
                    "memo": ""
                }),
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;

        assert_eq!(body["message"], "入力内容が不正です。");

        let details = body["details"].as_array().expect("details should be array");

        assert!(details.contains(&json!("種類は income または expense を指定してください。")));
        assert!(details.contains(&json!("カテゴリが不正です。")));
        assert!(details.contains(&json!("金額は1円以上で入力してください。")));
        assert!(details.contains(&json!("メモを入力してください。")));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_transactions_returns_paginated_list(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-11",
                "category": "food",
                "amount": 1200,
                "memo": "昼食"
            }),
        )
        .await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-12",
                "category": "transport",
                "amount": 580,
                "memo": "電車代"
            }),
        )
        .await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "income",
                "date": "2024-06-25",
                "category": "salary",
                "amount": 250000,
                "memo": "給与"
            }),
        )
        .await;

        let response = app
            .oneshot(empty_request(
                Method::GET,
                "/api/transactions?month=2024-06&q=支出&sort=date-desc&page=1&limit=1",
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;

        assert_eq!(body["items"].as_array().unwrap().len(), 1);
        assert_eq!(body["items"][0]["memo"], "電車代");

        assert_eq!(body["pagination"]["page"], 1);
        assert_eq!(body["pagination"]["limit"], 1);
        assert_eq!(body["pagination"]["total"], 2);
        assert_eq!(body["pagination"]["totalPages"], 2);
        assert_eq!(body["pagination"]["hasNext"], true);
        assert_eq!(body["pagination"]["hasPrevious"], false);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_transactions_returns_bad_request_for_invalid_query(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let response = app
            .oneshot(empty_request(
                Method::GET,
                "/api/transactions?month=2024/06&sort=invalid&page=0&limit=101",
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;

        assert_eq!(body["message"], "入力内容が不正です。");

        let details = body["details"].as_array().expect("details should be array");

        assert!(details.contains(&json!("month は YYYY-MM 形式で指定してください。")));
        assert!(details.contains(&json!(
            "sort は date-desc, date-asc, amount-desc, amount-asc のいずれかを指定してください。"
        )));
        assert!(details.contains(&json!("page は1以上の整数で指定してください。")));
        assert!(details.contains(&json!("limit は100以下で指定してください。")));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_transaction_summary_returns_all_matching_totals(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "income",
                "date": "2024-06-25",
                "category": "salary",
                "amount": 250000,
                "memo": "給与"
            }),
        )
        .await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-11",
                "category": "food",
                "amount": 1200,
                "memo": "昼食"
            }),
        )
        .await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-12",
                "category": "food",
                "amount": 800,
                "memo": "夕食"
            }),
        )
        .await;

        post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-13",
                "category": "daily",
                "amount": 980,
                "memo": "日用品"
            }),
        )
        .await;

        let response = app
            .oneshot(empty_request(
                Method::GET,
                "/api/transactions/summary?month=2024-06&page=1&limit=1",
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;

        assert_eq!(body["totalIncome"], 250000);
        assert_eq!(body["totalExpense"], 2980);
        assert_eq!(body["balance"], 247020);

        let category_summaries = body["categorySummaries"]
            .as_array()
            .expect("categorySummaries should be array");

        let food_summary = category_summaries
            .iter()
            .find(|summary| summary["category"] == "food")
            .expect("food summary should exist");

        assert_eq!(food_summary["total"], 2000);

        let daily_summary = category_summaries
            .iter()
            .find(|summary| summary["category"] == "daily")
            .expect("daily summary should exist");

        assert_eq!(daily_summary["total"], 980);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn put_transactions_updates_existing_transaction(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let created = post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-11",
                "category": "food",
                "amount": 1200,
                "memo": "昼食"
            }),
        )
        .await;

        let id = created["id"].as_str().expect("id should be string");

        let response = app
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/transactions/{id}"),
                json!({
                    "type": "expense",
                    "date": "2024-06-12",
                    "category": "daily",
                    "amount": 2000,
                    "memo": "更新後メモ"
                }),
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;

        assert_eq!(body["id"], id);
        assert_eq!(body["date"], "2024-06-12");
        assert_eq!(body["category"], "daily");
        assert_eq!(body["amount"], 2000);
        assert_eq!(body["memo"], "更新後メモ");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn put_transactions_returns_not_found_for_missing_id(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let response = app
            .oneshot(json_request(
                Method::PUT,
                "/api/transactions/00000000-0000-0000-0000-000000000000",
                json!({
                    "type": "expense",
                    "date": "2024-06-12",
                    "category": "daily",
                    "amount": 2000,
                    "memo": "更新後メモ"
                }),
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response_json(response).await;

        assert_eq!(body["message"], "指定された取引が見つかりません。");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_transactions_deletes_existing_transaction(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let created = post_transaction(
            &app,
            &token,
            json!({
                "type": "expense",
                "date": "2024-06-11",
                "category": "food",
                "amount": 1200,
                "memo": "昼食"
            }),
        )
        .await;

        let id = created["id"].as_str().expect("id should be string");

        let delete_response = app
            .clone()
            .oneshot(empty_request(
                Method::DELETE,
                &format!("/api/transactions/{id}"),
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let get_response = app
            .oneshot(empty_request(Method::GET, "/api/transactions", &token))
            .await
            .expect("request should succeed");

        assert_eq!(get_response.status(), StatusCode::OK);

        let body = response_json(get_response).await;

        assert_eq!(body["pagination"]["total"], 0);
        assert_eq!(body["items"].as_array().unwrap().len(), 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_transactions_returns_not_found_for_missing_id(pool: PgPool) {
        let (app, token) = build_test_app(pool).await;

        let response = app
            .oneshot(empty_request(
                Method::DELETE,
                "/api/transactions/00000000-0000-0000-0000-000000000000",
                &token,
            ))
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response_json(response).await;

        assert_eq!(body["message"], "指定された取引が見つかりません。");
    }
}
