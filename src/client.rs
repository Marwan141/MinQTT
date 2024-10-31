use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::packet::{MqttConnect, MqttPublish, MqttSubscribe};
use tokio::time::{Duration, timeout};
// Connect to the MQTT Broker via TCP/IP
pub async fn connect_to_broker(broker: &str, port: u16, client_id: &str) -> tokio::io::Result<TcpStream> {
    println!("Connecting to broker at {}:{}", broker, port);
    let mut stream = TcpStream::connect((broker, port)).await?;
    println!("Connected to broker, sending CONNECT packet...");

    let connect_packet = MqttConnect::new(client_id).encode();

    // Send CONNECT packet
    stream.write_all(&connect_packet).await?;
    println!("CONNECT packet sent!");

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
    println!("{:?}", encoded_packet);
    if let Err(e) = stream.write_all(&encoded_packet).await {
        println!("Error occurred when sending SUBSCRIBE packet: {}", e);
    } else {
        // QoS levels logic!
        println!("SUBSCRIBE packet has been sent.");
        let mut buffer = [0u8; 4]; 
        match timeout(Duration::from_secs(1), stream.read_exact(&mut buffer)).await {
            Ok(Ok(_)) => {
                println!("Received raw packet data: {:?}", &buffer);
                match buffer[0] >> 4 {
                    9 => { // SUBACK packet type
                        println!("Received SUBACK for {}", topic);
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