mod packet;
mod client;
use client::read_pingresp;
use packet::{MqttConnect, MqttPublish, MqttPingReq, MqttSubscribe};

use tokio::time::timeout;

use tokio::time::{sleep, Duration};
use client::connect_to_broker;
use client::read_connack;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]

async fn main() {
    let broker = "broker.hivemq.com";
    let port = 1883;
    let client_id = "minqtt_client";

    match connect_to_broker(broker, port, client_id).await {
        Ok(mut stream) => {
            if let Ok(true) = read_connack(&mut stream).await {
                println!("Connected to the broker!");
                
                let subscription_packet = MqttSubscribe 
                {
                    topic: "test".to_string(),
                    id:1,
                }.encode();
                println!("{:?}", subscription_packet);
                
                // One thing to note is that SUBACK packets could be checked here before entering the main loop
                if let Err(e) = stream.write_all(&subscription_packet).await{
                    eprintln!("Failed to subscribe: {}", e);
                } else{
                    println!("Success subscribing!")
                }

                let keep_alive_int = Duration::from_secs(5);
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
                    
                    let mut buffer = [0u8; 2];
                    match timeout(Duration::from_secs(1), stream.read_exact(&mut buffer)).await {
                        Ok(Ok(_)) => match buffer[0] >> 4 {
                            13 => {
                                println!("Received PINGRESP packet.");
                            }
                            9 => {
                                println!("Received SUBACK packet.")
                            }
                            3 => {
                                println!("Received PUBLISH packet.")
                                // Decoding
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
