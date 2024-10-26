pub enum PacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
}
pub struct MqttConnect {
    protocol_name: String,
    protocol_level: u8,
    clean_session: bool,
    client_id: String,
}

pub struct ConnectPacket {
    protocol_name: String,
    protocol_level: u8,
    client_id: String,
    keep_alive: u16,
}

impl MqttConnect {
    pub fn new(client_id: &str) -> Self {
        Self {
            protocol_name: "MQTT".to_string(),
            protocol_level: 4,
            clean_session: true,
            client_id: client_id.to_string(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push((PacketType::Connect as u8) << 4);
        
        // Variable header and payload encoding (simplified)
        packet.extend_from_slice(&[0x00, 0x04]); // Length of "MQTT"
        packet.extend_from_slice(self.protocol_name.as_bytes());
        packet.push(self.protocol_level);
        packet.push(0); // No clean-sessions yet.

        // Client ID
        packet.push(0);
        packet.push(self.client_id.len() as u8);
        packet.extend_from_slice(self.client_id.as_bytes());
        
        packet
    }
}

pub struct MqttPublish {
    pub topic: String,
    pub payload: Vec<u8>,
}

impl MqttPublish {
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push((PacketType::Connect as u8) << 4); // PUBLISH packet type

        // Add remaining length (simplified, assumes small packets)
        packet.push((self.topic.len() + self.payload.len() + 2) as u8);

        // Topic
        packet.push(0);
        packet.push(self.topic.len() as u8);
        packet.extend_from_slice(self.topic.as_bytes());

        // Payload
        packet.extend_from_slice(&self.payload);
        
        packet
    }
}
