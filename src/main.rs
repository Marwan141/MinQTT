mod packet;
mod client;
use packet::{MqttPublish, MqttPingReq};

use tokio::time::timeout;

use tokio::time::{sleep, Duration};
use client::{connect_to_broker, read_pingresp, subscribe_to_topic};
use client::read_connack;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
            if let Ok(true) = read_connack(&mut stream).await {
                println!("Connected to the broker!");
                let mut counter = 1;
                for sub in subscriptions{
                    subscribe_to_topic(&mut stream, sub, counter).await;
                    counter += 1;
                } 
           
                sleep(Duration::from_millis(1000)).await;
            
                let keep_alive_int = Duration::from_secs(60);
                let mut last_ping = tokio::time::Instant::now();
                loop {
                    // Check if it's time to send a PINGREQ
                    if last_ping.elapsed() >= keep_alive_int {
                        let pingreq_packet = MqttPingReq {}.encode();
                        if let Err(e) = stream.write_all(&pingreq_packet).await {
                            eprintln!("Failed to send PINGREQ packet: {}", e);
                            break;
                        }
                        println!("Sent PINGREQ");
                        last_ping = tokio::time::Instant::now();
                    }
                    
                    let mut buffer = [0u8; 1];
                    match timeout(Duration::from_secs(1), stream.read_exact(&mut buffer)).await {
                        Ok(Ok(_)) => match buffer[0] >> 4 {
                            13 => {
                                if let Ok(true) = read_pingresp(&mut stream).await {
                                    println!("Received PINGRESP packet.");
                                }
                                else{
                                    println!("Error regarding PINGRESP packet.");
                                }
                                
                            }
                            3 => {
                                println!("Received PUBLISH packet.");
                                let received_publish = MqttPublish::decode(buffer.to_vec(), &mut stream);
                                let pub_struct = received_publish.await;
                                println!("-------------------------------");
                                println!("Topic: {}", pub_struct.topic);
                                println!("Payload: {}", pub_struct.payload);
                                println!("-------------------------------");
                            }
                            _ => {
                                println!("Received unknown packet type {:?}", buffer[0] >> 4);
                            }
                        },
                        Ok(Err(e)) => {
                            eprintln!("Failed to read packet: {:?}", e);
                            break;
                        }
                        Err(_) => {
                            // Timeout expired
                            println!("Nothing is in the stream.");
                        }
                    }    
                    sleep(Duration::from_millis(100)).await;
                }
         
            } else {
                eprintln!("Failed to receive CONNACK; Can't connect to broker.");
            }

        }
        Err(e) => {
            eprintln!("Failed to connect to the broker: {}", e);
        }
        
    }   
}
