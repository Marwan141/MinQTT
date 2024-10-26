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
        packet.push(((PacketType::PingReq as u8) << 4));
        packet.push(0x00); // Remaining length is 0 for PINGREQ
        packet
    }
}


pub struct MqttSubscribe{
    pub topic: String,
    pub id: u16,
}
impl MqttSubscribe{
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
    pub payload: Vec<u8>,
    pub dup: bool,
    pub retain: bool
}

impl MqttPublish {
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
        packet.extend_from_slice(&self.payload);

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
} 
