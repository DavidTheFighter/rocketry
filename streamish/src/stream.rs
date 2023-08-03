use std::net::Ipv4Addr;
use std::process::{Command, Child, Stdio};

pub struct Stream {
    streaming_process: Child,
    pub port: u16,
    pub stream_addr: Ipv4Addr,
}

impl Stream {
    pub fn new(port: u16, addr: Ipv4Addr) -> Self {
        setup_v4l2_ctl_params();

        println!("Streamish: Setting up stream to {}:{}", addr, port);

        let streaming_process = Command::new("ffmpeg")
            .args(["-f", "v4l2"])
            .args(["-input_format", "h264"])
            .args(["-video_size", "1280x720"])
            .args(["-r", "30"])
            .args(["-i", "/dev/video0"])
            .args(["-c:v", "copy"])
            .args(["-f", "h264"])
            .arg(format!("udp://{}:{}", addr, port))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .expect("Failed to start streaming process");

        Self {
            streaming_process,
            port,
            stream_addr: addr,
        }
    }

    pub fn stop(&mut self) {
        self.streaming_process.kill().expect("Failed to kill streaming process");

        println!("Streamish: Stopped stream");
    }
}

fn setup_v4l2_ctl_params() {
    Command::new("v4l2-ctl")
            .arg("--set-ctrl")
            .arg("repeat_sequence_header=1,video_bitrate=5000000")
            .spawn()
            .expect("Failed to set v4l2-ctl params");
}