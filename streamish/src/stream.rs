use std::net::Ipv4Addr;
use std::process::{Command, Child, Stdio};

use serde::{Serialize, Deserialize};

pub struct Stream {
    streaming_process: Child,
    pub port: u16,
    pub stream_addr: Ipv4Addr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamishCommandSet {
    pre_commands: Vec<StreamishCommand>,
    streaming_command: StreamishCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamishCommand {
    command: String,
    args: Vec<String>,
}

impl Stream {
    pub fn new(port: u16, addr: Ipv4Addr) -> Self {
        println!("Setting up stream for {}:{}...", addr, port);

        let config_file = std::fs::read_to_string("streamish-cmds.json")
            .expect("Failed to read streamish-cmds.json");
        let mut command_set = serde_json::from_str::<StreamishCommandSet>(&config_file)
            .expect("Failed to parse streamish-cmds.json");

        command_set.fill_template("{stream_address}", &format!("{}:{}", addr, port));

        for command in command_set.pre_commands.iter() {
            println!("\t$ {}", command.as_string());
            command.to_command()
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect(&format!("Failed to run pre-command: \"{}\"", command.command));
        }

        println!("\t$ {}", command_set.streaming_command.as_string());
        let streaming_process = command_set.streaming_command.to_command()
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .expect(&format!("Failed to start streaming process: \"{}\"", command_set.streaming_command.command));

        println!("Set up stream to {}:{}", addr, port);

        Self {
            streaming_process,
            port,
            stream_addr: addr,
        }
    }

    pub fn stop(&mut self) {
        self.streaming_process.kill().expect("Failed to kill streaming process");

        println!("Stopped stream");
    }
}

impl StreamishCommandSet {
    fn fill_template(&mut self, template: &str, value: &str) {
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
    fn to_command(&self) -> Command {
        let mut command = Command::new(&self.command);
        for arg in &self.args {
            command.arg(arg);
        }
        command
    }

    fn as_string(&self) -> String {
        let mut command = self.command.clone();
        for arg in &self.args {
            command.push_str(&format!(" {}", arg));
        }
        command
    }
}