
## MinQTT - WIP

A light open-source Rust-based MQTT client library implemented utilizing Tokio.

### Features

- Lightweight and efficient
- Asynchronous operations using Tokio
- Supports MQTT 3.1.1

### Project Progress
- Successfully implemented CONNECT and CONNACK packets.
- PUBLISH packets are now functional.
- PINGREQ and PINGRESP packets are now implemented.
- Implemented SUBSCRIBE packets.
- Decoding PUBLISH packets.
  
### TODO
- Simplify API.
- Clean up debugging outputs.
- Add support for different QoS levels.
- Integrate encryption into packets.

### Usage
The `main.rs` file contains an example program of how one can utilize the library to implement their own MQTT client.
