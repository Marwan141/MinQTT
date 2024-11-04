use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::packet::{MqttConnect, MqttPublish, MqttSubscribe,MqttPingReq};
use tokio::time::{Duration, timeout, sleep};


// Connect to the MQTT Broker via TCP/IP
pub async fn connect_to_broker(broker: &str, port: u16, client_id: &str) -> tokio::io::Result<TcpStream> {
    println!("Connecting to broker at {}:{}", broker, port);
    let mut stream = TcpStream::connect((broker, port)).await?;
    //println!("Connected to broker, sending CONNECT packet...");

    let connect_packet = MqttConnect::new(client_id).encode();

    // Send CONNECT packet
    stream.write_all(&connect_packet).await?;
    //println!("CONNECT packet sent!");

    Ok(stream)
}


pub async fn send_publish_message(stream: &mut TcpStream, payload: String, topic: String){
    let encoded_packet = MqttPublish::new(topic, payload, true, 0, false).encode();
    if let Err(e) = stream.write_all(&encoded_packet).await {
        println!("Error occurred when sending PUBLISH packet: {}", e);
    } else {
        // QoS levels logic!
        println!("PUBLISH packet has been sent.");
    }
}

pub async fn subscribe_to_topic(stream: &mut TcpStream, topic: &str, id:u16){
    let encoded_packet = MqttSubscribe::new(topic.to_string(), id).encode();
    if let Err(e) = stream.write_all(&encoded_packet).await {
        println!("Error occurred when sending SUBSCRIBE packet: {}", e);
    } else {
        // QoS levels logic (TBI)
        //println!("SUBSCRIBE packet has been sent.");
        let mut buffer = [0u8; 4]; 
        match timeout(Duration::from_secs(1), stream.read_exact(&mut buffer)).await {
            Ok(Ok(_)) => {
                match buffer[0] >> 4 {
                    9 => { // SUBACK packet type
                        println!("-------------------------------");
                        // Check the return code in the SUBACK packet
                        let return_code = buffer[3]; // Adjust index based on actual packet structure
                        if return_code == 0x00 || return_code == 0x01 {
                            println!("Status: Subscription to {} successful!", topic);
                        } else {
                            println!("Status: Subscription to {} failed with return code: {}", topic, return_code);
                        }
                        println!("-------------------------------");
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
                eprintln!("Timeout waiting for SUBACK packet for {}", topic);
            }
        }
    }

}

pub async fn subscribe_to_topic_vector(stream: &mut TcpStream, topics: Vec<String>){}


pub async fn read_connack(stream: &mut TcpStream) -> Result<bool, Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 4];
    stream.read_exact(&mut buffer).await?;
    // Check if the buffer contains the expected CONNACK packet
    if buffer[0] == 0x20 && buffer[1] == 0x02 && buffer[3] == 0x00 { //first byte is 0x20 (Connack), second byte is remaining length and third is success.
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn read_pingresp(stream: &mut TcpStream) -> Result<bool, Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer).await?; 
    if buffer[0] == 0x00 { // Successful PINGRESP
        Ok(true)
    } else {
        Ok(false)

    }

}

pub async fn run_main_loop(mut stream: &mut TcpStream, subscriptions: Vec<&str>){
    if let Ok(true) = read_connack(&mut stream).await {
        println!("Connected to the broker!");
        let mut counter = 1;
        for sub in subscriptions{
            subscribe_to_topic(&mut stream, sub, counter).await;
            counter += 1;
        } 

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

                    0 => {

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
                }
            }    
            sleep(Duration::from_millis(100)).await;
        }
 
    } else {
        eprintln!("Failed to receive CONNACK; Can't connect to broker.");
    }
}