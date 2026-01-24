/* 
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
*/
use serde_json::Value;
use sysinfo::System;
use tokio::time::{sleep, Duration};
// use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

type SharedData = Arc<Mutex<VecDeque<Value>>>;

#[tokio::main]
async fn main() {
    println!("Server Monitoring Initializing!");
    let sys = System::new();
    let data: SharedData = Arc::new(Mutex::new(VecDeque::with_capacity(3600)));


    continuous_metrics_monitor(sys, data).await;

}

    



async fn continuous_metrics_monitor(mut sys: System, data: SharedData ) {

    loop {
        let collected_metrics = read_metrics(&mut sys);
        
        // Adding collected metrics to data array, and keeping data array at max length of 3600
        {
            let mut guard = data.lock().await;
            guard.push_back(collected_metrics);
            if guard.len() > 3600 {
                guard.pop_front();
            }
        }

        println!("{:#?}", data);

        sleep(Duration::from_secs(1)).await;
    }
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
/*
fn store_metrics_data(
    mut data_array: VecDeque<serde_json::Value>,
    collected_data: serde_json::json! ) {

    // remove oldest data
    data_array.push_back(collected_data);

    if data_array.len() == 3600 {
        data_array.pop_front(); 
    }

    let json_string = serde_json::to_string(&data).unwrap();

    
}



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


handle_socket(mut socket: WebSocket) {
    if socket
        .send(Message::Text(json_string.into()))
        .await
        .is_err()
    {
        break;
    }
}





*/

