
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

pub enum PacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
    Subscribe = 8,
    PingReq = 12,
}
pub struct MqttConnect {
    protocol_name: String,
    protocol_level: u8,
    clean_session: bool,
    client_id: String,
}

impl MqttConnect {
    pub fn new(client_id: &str) -> Self {
        Self {
            protocol_name: "MQTT".to_string(),
            protocol_level: 4, // 4 is MQTT 3.1.1
            clean_session: true,
            client_id: client_id.to_string(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push((PacketType::Connect as u8) << 4);
        
        let mut variable_header_and_payload = Vec::new();
        
        // Protocol Name
        variable_header_and_payload.extend_from_slice(&[0x00, 0x04]); // Length of "MQTT"
        variable_header_and_payload.extend_from_slice(self.protocol_name.as_bytes());
        
        // Protocol Level
        variable_header_and_payload.push(self.protocol_level);
        
        // Connect Flags
        let connect_flags = if self.clean_session { 0x02 } else { 0x00 };
        variable_header_and_payload.push(connect_flags);
        
        // Keep Alive
        variable_header_and_payload.extend_from_slice(&[0x00, 0x3C]); 
        
        // Client ID
        variable_header_and_payload.push(0x00);
        variable_header_and_payload.push(self.client_id.len() as u8);
        variable_header_and_payload.extend_from_slice(self.client_id.as_bytes());
        

        let remaining_length = variable_header_and_payload.len() as u8;

        packet.push(remaining_length);
        packet.extend_from_slice(&variable_header_and_payload);
        packet
    }
}



pub struct MqttPingReq{

}

impl MqttPingReq{
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push((PacketType::PingReq as u8) << 4);
        packet.push(0x00); // Remaining length is 0 for PINGREQ
        packet
    }
}


pub struct MqttSubscribe{
    pub topic: String,
    pub id: u16, // To be fixed later - ID is used to identify packets whilst being in acknowledgement process (Some kind of DHCP but for IDs?)
}
impl MqttSubscribe{
    pub fn new(topic:String, id:u16) -> Self{
        MqttSubscribe{
            topic,
            id
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push(((PacketType::Subscribe as u8) << 4) + 2);
        packet.push((self.topic.len() + 5) as u8);

        // Packet ID
        packet.push((self.id >> 8) as u8); // MSB Packet ID
        packet.push((self.id & 0xFF) as u8); // LSB Packet ID
        // Payload
        let topic_len = self.topic.len();
        packet.push((topic_len >> 8) as u8); // MSB Length
        packet.push((topic_len & 0xFF) as u8); // LSB Length
        packet.extend_from_slice(self.topic.as_bytes());

        // QoS Level
        packet.push(0); 

        packet
    }
}
pub struct MqttPublish {
    pub topic: String,
    pub payload: String,
    pub dup: bool,
    pub qos: u8,
    pub retain: bool
}

impl MqttPublish {
    pub fn new(topic: String, payload: String, dup: bool, qos: u8, retain: bool) -> Self {
        MqttPublish {
            topic,
            payload,
            dup,
            qos,
            retain,
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        let mut fixed_header = 0;

        fixed_header |= (PacketType::Publish as u8) << 4;

        if self.dup {
            fixed_header |= 0x08;
        }

        fixed_header |= (0 & 0x03) << 1; // To be changed in the future for QoS implementations

        if self.retain {
            fixed_header |= 0x01;
        }

        packet.push(fixed_header);
        packet.push((self.topic.len() + self.payload.len() + 2) as u8);

        // Topic
        let topic_len = self.topic.len();
        packet.push((topic_len >> 8) as u8);
        packet.push((topic_len & 0xFF) as u8);
        packet.extend_from_slice(self.topic.as_bytes());

        // Payload
        packet.extend_from_slice(self.payload.as_bytes());

        let remaining_length = packet.len() - 1; // Exclude the fixed header byte
        let mut remaining_length_bytes = Vec::new();
        let mut x = remaining_length;
        loop {
            let mut encoded_byte = (x % 128) as u8;
            x /= 128;
            if x > 0 {
                encoded_byte |= 128;
            }
            remaining_length_bytes.push(encoded_byte);
            if x == 0 {
                break;
            }
        }

        packet.splice(1..1, remaining_length_bytes.iter().cloned());
        packet
    }
    pub async fn decode(header:Vec<u8>, stream:&mut TcpStream) -> MqttPublish {
        let fixed_header = header[0];
        let qos = (fixed_header >> 1) & 0x03;
        let dup = (fixed_header & 0x08) != 0;
        let retain = (fixed_header & 0x01) != 0;

        let mut remaining_length = 0;
        let mut multiplier = 1;

        // Decode remaining length
        loop {
            let mut encoded_byte = [0u8; 1];
            match stream.read_exact(&mut encoded_byte).await {
                Ok(_) => {
                    let byte = encoded_byte[0];
                    remaining_length += (byte & 127) as usize * multiplier;
                    multiplier *= 128;
                    if byte & 128 == 0 {
                        break; // All remanining length bytes have been read when MSB is 0
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read remaining length: {:?}", e);
                    return MqttPublish {
                        topic: String::new(),
                        payload: String::new(),
                        dup,
                        qos,
                        retain,
                    };
                }
            }
        }

        let mut packet = vec![0u8; remaining_length];
        if let Err(e) = stream.read_exact(&mut packet).await { // Read the remaining length
            eprintln!("Failed to read packet: {:?}", e);
            return MqttPublish {
                topic: String::new(),
                payload: String::new(),
                dup,
                qos,
                retain,
            };
        }

       // Decode topic
       let topic_len = u16::from_be_bytes([packet[0], packet[1]]) as usize;
       let topic = String::from_utf8(packet[2..(2 + topic_len)].to_vec()).unwrap();

       // Decode payload
       let payload = String::from_utf8(packet[2 + topic_len..].to_vec()).unwrap();

       MqttPublish::new(topic, payload, dup, qos, retain)
    }
} 
