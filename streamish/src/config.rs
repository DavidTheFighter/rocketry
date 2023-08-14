use std::process::Command;

use hal::comms_hal::NetworkAddress;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamishCommandSet {
    pub host: NetworkAddress,
    pub pre_commands: Vec<StreamishCommand>,
    pub streaming_command: StreamishCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamishCommand {
    pub command: String,
    pub args: Vec<String>,
}

impl StreamishCommandSet {
    pub fn fill_template(&mut self, template: &str, value: &str) {
        for command in self.pre_commands.iter_mut() {
            for arg in command.args.iter_mut() {
                *arg = arg.replace(template, value);
            }
        }

        for arg in self.streaming_command.args.iter_mut() {
            *arg = arg.replace(template, value);
        }
    }
}

impl StreamishCommand {
    pub fn to_command(&self) -> Command {
        let mut command = Command::new(&self.command);
        for arg in &self.args {
            command.arg(arg);
        }
        command
    }

    pub fn as_string(&self) -> String {
        let mut command = self.command.clone();
        for arg in &self.args {
            command.push_str(&format!(" {}", arg));
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use hal::comms_hal::NetworkAddress;

    #[test]
    fn test_host_serialization() {
        let host = NetworkAddress::GroundCamera(42);
        let serialized = serde_json::to_string(&host).unwrap();
        assert_eq!(serialized, "{\"GroundCamera\":42}");
    }
}