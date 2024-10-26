mod packet;
mod client;
use client::read_pingresp;
use packet::{MqttConnect, MqttPublish, MqttPingReq};
use tokio::time::{sleep, Duration};
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

                let keep_alive_int = Duration::from_secs(60);
                tokio::spawn(async move{
                    loop {
                        sleep(keep_alive_int).await;

                        let pingreq_packet = MqttPingReq {}.encode();
                        if let Err(e) = stream.write_all(&pingreq_packet).await {
                            eprintln!("Failed to send PINGREQ packet");
                            break;
                        }
                        println!("Sent PINGREQ");

                        if let Err(e) = read_pingresp(&mut stream).await {
                            eprintln!("Failed to read PINGRESP: {}", e);
                            break;
                        }
    
                        println!("Received PINGRESP");
                    }
                }).await.unwrap();
            }

        }
        Err(e) => {
            eprintln!("Failed to connect to the broker: {}", e);
        }
    }   
}
