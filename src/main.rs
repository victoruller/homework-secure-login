use axum::{
    routing::{ get, post },
    http::StatusCode,
    extract::State,
    Json,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};

const PORT: &str = "3000";

#[tokio::main]
async fn main() {

    let options = SqliteConnectOptions::new()
        .filename("users.db")
        .create_if_missing(true);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )",
    )
    .execute(&db)
    .await
    .unwrap();


    let app = Router::new()
        .route("/", get(ola))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}")).await.unwrap(); // 0.0.0.0 indica que aceita qualquer praca de rede
    println!("Servidor ouvindo na porta {PORT}");
    axum::serve(listener, app).await.unwrap();
}

async fn ola() -> &'static str {
    "Olá, da VM"
}


#[derive(Deserialize)]
struct Credentials {
    user: String,
    password: String,
}


async fn login(Json(c): Json<Credentials>) -> (StatusCode, Json<Value>) {
    println!("Tentativa de login: {}", c.user);

    if c.user == "admin" && c.password == "12345678" {
        (StatusCode::OK, Json(json!({ "user": c.user })))
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({ "erro": "Usuário ou senha incorretos"})))
    }
}

async fn register(
    State(db): State<SqlitePool>,
    Json(c): Json<Credentials>,
) -> (StatusCode, Json<Value>) {
    let result = sqlx::query("INSERT INTO users (user, password) VALUES (?, ?)")
        .bind(&c.user)
        .bind(&c.password)
        .execute(&db)
        .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({"message": "Conta criada"}))),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(json!({ "erro": "Esse usuário já existe" }))
        ),
        Err(e) => {
            eprintln!("Erro ao registrar: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Erro interno" })))
        }
    }
}