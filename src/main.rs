use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use sysinfo::System;
use std::time::Duration;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;


#[tokio::main]
async fn main() {
    println!("Server Monitoring Initializing!");

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
    let mut data: VecDeque<serde_json::Value> = VecDeque::with_capacity(3600);

    loop {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let collected_data = serde_json::json!({
            "cpu": sys.global_cpu_usage(),
            "ram": (sys.used_memory() as f64 / sys.total_memory() as f64),
            "timestamp": timestamp
            });

        // remove oldest data
        data.push_back(collected_data);

        if data.len() == 3600 {
            data.pop_front(); 
        }

        let json_string = serde_json::to_string(&data).unwrap();

        if socket
            .send(Message::Text(json_string.into()))
            .await
            .is_err()
        {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
