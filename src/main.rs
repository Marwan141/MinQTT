mod packet;
mod client;
use packet::{MqttConnect, MqttPublish};
use client::connect_to_broker;
use client::read_connack;
use tokio::io::AsyncWriteExt;

#[tokio::main]

async fn main() {
    let broker = "test.mosquitto.org";
    let port = 1883;
    let client_id = "rust_mqtt_client";

    match connect_to_broker(broker, port, client_id).await {
        Ok(mut stream) => {
            if let Ok(true) = read_connack(&mut stream).await {
                println!("Connected to the broker!");
                
                // Send a PUBLISH packet
                let publish_packet = MqttPublish {
                    topic: "test/topic".to_string(),
                    payload: b"Hello MQTT!".to_vec(),
                }.encode();

                stream.write_all(&publish_packet).await.unwrap();
                println!("Published message!");
            }

        }
        Err(e) => {
            eprintln!("Failed to connect to the broker: {}", e);
        }
    }
}