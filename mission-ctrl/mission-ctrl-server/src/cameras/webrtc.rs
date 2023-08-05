use webrtc::{peer_connection::configuration::RTCConfiguration, ice_transport::ice_server::RTCIceServer};


pub struct WebRtcStream {

}

impl WebRtcStream {
    pub fn new() -> Self {
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        Self {

        }
    }
}