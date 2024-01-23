use std::sync::Arc;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rocket::serde::json::serde_json;
use shared::comms_hal::NetworkAddress;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder,
    },
    ice_transport::{ice_connection_state::RTCIceConnectionState, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{
        track_local_static_rtp::TrackLocalStaticRTP, TrackLocal, TrackLocalWriter,
    },
};

pub struct WebRtcStream {
    pub camera_address: NetworkAddress,
    rtp_tx: Sender<Vec<u8>>,
    stream_done_rx: tokio::sync::oneshot::Receiver<()>,
    stream_closed: bool,
    stream_session: String,
    runtime: Runtime,
}

impl WebRtcStream {
    pub fn new(camera_address: NetworkAddress, browser_session: String) -> Result<Self, String> {
        let (rtp_tx, rtp_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(10);
        let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<Result<String, String>>(1);
        let (stream_done_tx, stream_done_rx) = tokio::sync::oneshot::channel::<()>();

        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        runtime.spawn(Self::setup_webrtc_stream(
            browser_session,
            result_tx,
            stream_done_tx,
            rtp_rx,
        ));

        let result = runtime
            .block_on(result_rx.recv())
            .expect("Didn't receive result from WebRTC setup");

        match result {
            Ok(stream_session) => Ok(Self {
                camera_address,
                rtp_tx,
                stream_done_rx,
                stream_closed: false,
                stream_session,
                runtime,
            }),
            Err(err) => Err(err),
        }
    }

    async fn setup_webrtc_stream(
        browser_session: String,
        result_tx: Sender<Result<String, String>>,
        stream_done_tx: tokio::sync::oneshot::Sender<()>,
        mut rtp_rx: Receiver<Vec<u8>>,
    ) {
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut m = MediaEngine::default();
        m.register_default_codecs()
            .expect("Failed to register default codecs");

        // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
        // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
        // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
        // for each PeerConnection.
        let mut registry = Registry::new();

        // Use the default set of Interceptors
        registry = register_default_interceptors(registry, &mut m)
            .expect("Failed to register default interceptors");

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let peer_connection = api.new_peer_connection(config).await;
        if let Err(err) = peer_connection {
            result_tx
                .send(Err(format!("Failed to create peer connection: {}", err)))
                .await
                .expect("Failed to send result");
            return;
        }
        let peer_connection = Arc::new(peer_connection.unwrap());

        let video_track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        // Add this newly created track to the PeerConnection
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .expect("Failed to add video track to peer connection");

        // Read incoming RTCP packets
        // Before these packets are returned they are processed by interceptors. For things
        // like NACK this needs to be called.
        tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
            Result::<(), ()>::Ok(())
        });

        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

        let done_tx1 = done_tx.clone();
        // Set the handler for ICE connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection.on_ice_connection_state_change(Box::new(
            move |connection_state: RTCIceConnectionState| {
                println!("Connection State has changed {connection_state}");
                if connection_state == RTCIceConnectionState::Failed {
                    let _ = done_tx1.try_send(());
                }
                Box::pin(async {})
            },
        ));

        let done_tx2 = done_tx.clone();
        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    let _ = done_tx2.try_send(());
                }

                Box::pin(async {})
            },
        ));

        let desc_data = String::from_utf8(
            BASE64_STANDARD
                .decode(&browser_session)
                .expect("Failed to decode base64"),
        )
        .expect("Failed to convert to utf8");
        let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)
            .expect("Failed to unmarshal base64");

        // Set the remote SessionDescription
        peer_connection
            .set_remote_description(offer)
            .await
            .expect("Failed to set remote description");

        // Create an answer
        let answer = peer_connection
            .create_answer(None)
            .await
            .expect("Failed to create answer");

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection
            .set_local_description(answer)
            .await
            .expect("Failed to set local description");

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        // Output the answer in base64 so we can paste it in browser
        if let Some(local_desc) = peer_connection.local_description().await {
            let json_str =
                serde_json::to_string(&local_desc).expect("Failed to serialize local description");
            let b64 = BASE64_STANDARD.encode(json_str);

            if let Err(err) = result_tx.send(Ok(b64)).await {
                result_tx
                    .send(Err(format!(
                        "Failed to send session description: {:?}",
                        err
                    )))
                    .await
                    .expect("Failed to send result");
                return;
            }
        } else {
            result_tx
                .send(Err("Failed to get local description".to_owned()))
                .await
                .expect("Failed to send result");
            return;
        }

        println!(
            "Set up WebRTC stream {}...{}",
            desc_data[0..4].to_owned(),
            desc_data[desc_data.len() - 4..].to_owned(),
        );

        let done_tx3 = done_tx.clone();
        // Read RTP packets forever and send them to the WebRTC Client
        tokio::spawn(async move {
            println!("Waiting on data");
            while let Some(buffer) = rtp_rx.recv().await {
                if let Err(_err) = video_track.write(&buffer[0..]).await {
                    let _ = done_tx3.try_send(());
                    return;
                }
            }
        });

        done_rx.recv().await;

        peer_connection
            .close()
            .await
            .expect("Failed to close peer connection");

        println!(
            "Closing WebRTC stream {}...{}",
            desc_data[0..4].to_owned(),
            desc_data[desc_data.len() - 4..].to_owned(),
        );

        stream_done_tx.send(()).expect("Failed to send stream done");
    }

    pub fn send_data(&self, buffer: Vec<u8>) {
        self.runtime
            .block_on(self.rtp_tx.send(buffer))
            .expect("Failed to send data");
    }

    pub fn stream_closed(&mut self) -> bool {
        if self.stream_closed {
            return true;
        }

        if let Ok(_) = self.stream_done_rx.try_recv() {
            self.stream_closed = true;
            return true;
        }

        return false;
    }

    pub fn name(&self) -> String {
        format!(
            "{}...{}",
            self.stream_session[0..4].to_owned(),
            self.stream_session[self.stream_session.len() - 4..].to_owned(),
        )
    }

    pub fn get_session_desc(&self) -> String {
        self.stream_session.clone()
    }
}
