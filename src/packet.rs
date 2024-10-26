pub enum PacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
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
pub struct MqttPublish {
    pub topic: String,
    pub payload: Vec<u8>,
}

impl MqttPublish {
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push((PacketType::Publish as u8) << 4); // PUBLISH packet type

        // We need to add remaining length (simplified, assumes small packets)
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
