mod email;
mod errors;
mod handlers;
mod models;
mod repositories;
mod security;
mod services;
mod state;
mod validators;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{email::EmailClient, state::AppState};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct DbHealthResponse {
    status: String,
    database: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn db_health(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<DbHealthResponse> {
    let database_name: String = sqlx::query_scalar("select current_database()")
        .fetch_one(&state.db)
        .await
        .expect("failed to fetch database name");

    Json(DbHealthResponse {
        status: "ok".to_string(),
        database: database_name,
    })
}

fn load_env() {
    dotenvy::dotenv().ok();
    dotenvy::from_path("../.env").ok();
}

#[tokio::main]
async fn main() {
    load_env();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    let email_client = EmailClient::from_env().expect("failed to initialize email client");

    let backend_host = env::var("BACKEND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let backend_port = env::var("BACKEND_PORT").unwrap_or_else(|_| "8080".to_string());
    let server_addr = format!("{}:{}", backend_host, backend_port);

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("filed to connect database");

    let state = Arc::new(AppState {
        db,
        jwt_secret,
        frontend_url,
        email_client,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/db-health", get(db_health))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/me", get(handlers::auth::me))
        .route(
            "/api/auth/change-password",
            post(handlers::auth::change_password),
        )
        .route(
            "/api/auth/forgot-password",
            post(handlers::auth::forgot_password),
        )
        .route(
            "/api/auth/reset-password",
            post(handlers::auth::reset_password),
        )
        .route(
            "/api/transactions/summary",
            get(handlers::transactions::summarize_transactions),
        )
        .route(
            "/api/transactions",
            get(handlers::transactions::list_transactions)
                .post(handlers::transactions::create_transaction),
        )
        .route(
            "/api/transactions/{id}",
            axum::routing::put(handlers::transactions::update_transaction)
                .delete(handlers::transactions::delete_transaction),
        )
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&server_addr)
        .await
        .expect("failed to bind");

    println!("server running on http://{}", server_addr);

    axum::serve(listener, app).await.expect("server error");
}
