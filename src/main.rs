use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use sysinfo::System;
use std::time::Duration;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws/metrics", get(ws_handler));

    let addr: SocketAddr = "0.0.0.0:8010".parse().unwrap();

    println!("listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut sys = System::new();

    loop {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let payload = serde_json::json!({
            "cpu": sys.global_cpu_usage(),
            "ram": (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0
        });

        if socket
            .send(Message::Text(payload.to_string().into()))
            .await
            .is_err()
        {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
