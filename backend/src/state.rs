use sqlx::PgPool;

use crate::email::EmailClient;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub email_client: EmailClient,
}
