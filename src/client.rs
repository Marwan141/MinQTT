use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use crate::packet::PacketType;
use crate::packet::MqttConnect;

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
    println!("Reading PINGRESP");
    
    let mut buffer = [0u8; 2];
    stream.read_exact(&mut buffer).await?; 
    println!("{:?}", buffer);
    if buffer[0] == 0xD0 && buffer[1] == 0x00 {
        Ok(true)
    } else {
        Ok(false)

    }

}