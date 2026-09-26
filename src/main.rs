mod cripto;
mod database;
mod state;
mod routes;

use std::sync::Arc;
use state::AppState;

const PORT: u16 = 3000;

#[tokio::main]
async fn main() {
    let db = database::connect().await;
    let dummy_hash = cripto::generate_hash("senha-que-ninguem-usa");

    let state = AppState {
        db,
        dummy_hash: Arc::new(dummy_hash),
    };

    let app = routes::create_router(state);

    // 0.0.0.0 = aceita conexões de qualquer placa de rede
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", PORT)).await.unwrap();
    println!("Servidor ouvindo na porta {PORT}");
    axum::serve(listener, app).await.unwrap();
}