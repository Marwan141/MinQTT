mod packet;
mod client;
use client::{connect_to_broker, run_main_loop};


#[tokio::main]
async fn main() {
    let broker = "test.mosquitto.org";
    let port = 1883;
    let client_id = "minqtt_client";
    let mut subscriptions = Vec::new();
    subscriptions.push("sub1");
    subscriptions.push("RAG");
    subscriptions.push("sub2");

    match connect_to_broker(broker, port, client_id).await {
        Ok(mut stream) => {
            run_main_loop(&mut stream, subscriptions).await;
        }
        Err(e) => {
            eprintln!("Failed to connect to the broker: {}", e);
        }
        
    }   
}
