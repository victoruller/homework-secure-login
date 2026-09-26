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
use argon2::{
    Argon2, 
    password_hash::{
        PasswordHasher, 
        PasswordVerifier
    },
};

mod banco;
mod cripto;
mod estado;
mod rotas;




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
            password_hash TEXT NOT NULL
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


async fn login(
    State(db): State<SqlitePool>,
    Json(c): Json<Credentials>,
) -> (StatusCode, Json<Value>) {

    // coleta o hash que está na primeira linha onde o user é o c.user
    let row: Option<(String,)> = 
        sqlx::query_as("SELECT password_hash FROM users WHERE user = ?")
            .bind(&c.user)
            .fetch_optional(&db)
            .await
            .unwrap();

    let Some((hash,)) = row else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"erro": "Usuário ou senha incorretos"})));
    };

    let password = c.password;
    let ok = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .unwrap();

    if ok {
        (StatusCode::OK, Json(json!({"usuario": c.user})))
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({"erro": "Usuário ou senha incorretos"})))
    }
}


async fn register(
    State(db): State<SqlitePool>,
    Json(c): Json<Credentials>,
) -> (StatusCode, Json<Value>) {

    let password = c.password;

    // gera hash da senha em tread separada
    let hash = tokio::task::spawn_blocking(move || generate_hash(&password))
        .await
        .unwrap();

    let result = sqlx::query("INSERT INTO users (user, password_hash) VALUES (?, ?)")
        .bind(&c.user)
        .bind(&hash)
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

fn generate_hash(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes())
        .unwrap()
        .to_string()
}


fn verify_password(password: &str, hash: &str) -> bool {
    Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .is_ok()
}