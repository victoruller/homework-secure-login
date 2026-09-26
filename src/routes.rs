use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    cripto::{generate_hash, verify_password},
    database,
    state::AppState,
};

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .with_state(state)
}

async fn hello() -> &'static str {
    "Olá, da VM"
}

async fn login(
    State(st): State<AppState>,
    Json(c): Json<Credentials>,
) -> (StatusCode, Json<Value>) {
    let stored = match database::get_password_hash(&st.db, &c.username).await {
        Ok(stored) => stored,
        Err(e) => {
            eprintln!("Erro no login: {e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Erro interno" })));
        }
    };

    let exists = stored.is_some();
    let hash = stored.unwrap_or_else(|| st.dummy_hash.to_string());

    let password = c.password;
    let ok = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .unwrap();

    if ok && exists {
        (StatusCode::OK, Json(json!({ "username": c.username })))
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Usuário ou senha incorretos" })))
    }
}

async fn register(
    State(st): State<AppState>,
    Json(c): Json<Credentials>,
) -> (StatusCode, Json<Value>) {
    let password = c.password;
    let hash = tokio::task::spawn_blocking(move || generate_hash(&password))
        .await
        .unwrap();

    match database::insert_user(&st.db, &c.username, &hash).await {
        Ok(()) => (StatusCode::CREATED, Json(json!({ "message": "Conta criada" }))),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Esse usuário já existe" })),
        ),
        Err(e) => {
            eprintln!("Erro ao registrar: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Erro interno" })))
        }
    }
}