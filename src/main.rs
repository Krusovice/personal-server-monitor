
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};

use serde_json::Value;
use sysinfo::System;
use tokio::time::{sleep, Duration};
use tokio::sync::broadcast;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

type Metrics = Value;
type SharedData = Arc<Mutex<VecDeque<Metrics>>>;

#[derive(Clone)]
struct AppState {
    data: SharedData,
    metrics_tx: broadcast::Sender<Metrics>,
}

#[tokio::main]
async fn main() {
    println!("Server Monitoring Initializing!");
    
    let data: SharedData = Arc::new(Mutex::new(VecDeque::with_capacity(3600)));
    let (metrics_tx, _metrics_rx) = broadcast::channel::<Metrics>(32);

    let state = AppState {
        data: data.clone(),
        metrics_tx: metrics_tx.clone(),
    };

    tokio::spawn(activate_metrics_monitor(
        data.clone(),
        metrics_tx.clone(),
    ));

    let app = Router::new()
        .route("/ws/metrics", get(ws_handler))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8010".parse().unwrap();

    println!("listening on {}", addr);

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn read_metrics( sys: &mut System ) -> Value {
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

    collected_data
}

async fn activate_metrics_monitor(
    data: SharedData,
    tx: broadcast::Sender<Metrics>,
) {
    let mut sys = System::new();

    loop {
        let metrics = read_metrics(&mut sys);
        
        // Adding collected metrics to data array, and keeping data array at max length of 3600
        {
            let mut guard = data.lock().await;
            guard.push_back(metrics.clone());
            if guard.len() > 3600 {
                guard.pop_front();
            }
        }

        //println!("{:#?}", data);
        let _ = tx.send(metrics); // ignore lagged receivers
        sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_socket(
    mut socket: WebSocket,
    data: SharedData,
    tx: broadcast::Sender<Metrics>
) {
    // Sending metrics history, that is stored in data variable
    let metrics_history = {
        let guard = data.lock().await;
        guard.clone()
    };

    let metrics_history_json = serde_json::to_string(&metrics_history).unwrap();
    if socket.send(Message::Text(metrics_history_json.into())).await.is_err() {
        return;
    }

    // Sending live metrics
    let mut rx = tx.subscribe();

    while let Ok(metrics) = rx.recv().await {
        let json = serde_json::to_string(&metrics).unwrap();
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(
        socket,
        state.data.clone(),
        state.metrics_tx.clone(),
    ))
}

