use std::sync::Arc;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub dummy_hash: Arc<String>,
}