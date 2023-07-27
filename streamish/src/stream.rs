use std::net::Ipv4Addr;
use std::process::{Command, Child};

pub struct Stream {
    streaming_process: Child,
}

impl Stream {
    pub fn new(port: u16, addr: Ipv4Addr) -> Self {
        setup_v4l2_ctl_params();

        println!("Streamish: Setting up stream to {}:{}", addr, port);

        let streaming_process = Command::new("ffmpeg")
            .arg("-f v4l2")
            .arg("-input_format h264")
            .arg("-video_size 1280x720")
            .arg("-r 30")
            .arg("-i /dev/video0")
            .arg("-c:v copy")
            .arg("-f h264")
            .arg(format!("udp://{}:{}", addr, port))
            .spawn()
            .expect("Failed to start streaming process");

        Self {
            streaming_process,
        }
    }

    pub fn stop(&mut self) {
        self.streaming_process.kill().expect("Failed to kill streaming process");

        println!("Streamish: Stopped stream");
    }
}

fn setup_v4l2_ctl_params() {
    Command::new("v4l2-ctl")
            .arg("--set-ctrl repeat_sequence_header=1,video_bitrate=5000000")
            .spawn()
            .expect("Failed to set v4l2-ctl params");
}