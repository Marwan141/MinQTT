mod packet;
mod client;
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
    let mut subscriptions = Vec::new();
    subscriptions.push("sub1");
    subscriptions.push("sub2");

    match connect_to_broker(broker, port, client_id).await {
        Ok(mut stream) => {
            if let Ok(true) = read_connack(&mut stream).await {
                println!("Connected to the broker!");
                let counter = 1;
                for sub in subscriptions{
                    let subscription_packet = MqttSubscribe 
                    {
                        topic: sub.to_string(),
                        id:counter,
                    }.encode();
                    if let Err(e) = stream.write_all(&subscription_packet).await{
                        eprintln!("Failed to subscribe: {}", e);
                    } else{
                        println!("Subscribing to {}!", sub);
                    }
                    // Read and verify SUBACK
                    let mut buffer = [0u8; 4]; // Adjust buffer size as needed
                    match timeout(Duration::from_secs(1), stream.read_exact(&mut buffer)).await {
                        Ok(Ok(_)) => {
                            println!("Received raw packet data: {:?}", &buffer);
                            match buffer[0] >> 4 {
                                9 => { // SUBACK packet type
                                    println!("Received SUBACK for {}", sub);
                                    // Check the return code in the SUBACK packet
                                    let return_code = buffer[3]; // Adjust index based on actual packet structure
                                    if return_code == 0x00 || return_code == 0x01 {
                                        println!("Subscription to {} successful!", sub);
                                    } else {
                                        println!("Subscription to {} failed with return code: {}", sub, return_code);
                                    }
                                }
                                _ => {
                                    println!("Received unexpected packet type {:?}", buffer[0] >> 4);
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            eprintln!("Failed to read SUBACK packet: {:?}", e);
                        }
                        Err(_) => {
                            eprintln!("Timeout waiting for SUBACK packet for {}", sub);
                        }
                    }
                } 
           
                sleep(Duration::from_millis(1000)).await;
                // One thing to note is that SUBACK packets could be checked here before entering the main loop
                

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
                                println!("Received PINGRESP packet.");
                            }
                            3 => {
                                
                                let received_publish = MqttPublish::decode(buffer.to_vec(), &mut stream);
                                let pub_struct = received_publish.await;
                                println!("Received PUBLISH packet.");
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
