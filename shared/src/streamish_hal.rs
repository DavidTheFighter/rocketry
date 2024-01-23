use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamishCommand {
    StartCameraStream { port: u16 },
    StopCameraStream,
    StopApplication,
}
