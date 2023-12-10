use big_brother::interface::std_interface::StdInterface;
use streamish::Streamish;
use sysinfo::{SystemExt, ProcessExt};

pub(crate) mod config;
mod stream;
mod streamish;

fn main() {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    for (pid, process) in system.processes() {
        if process.name() == "streamish" && usize::from(*pid) != std::process::id() as usize {
            println!("Killing existing streamish process {:?}", pid);
            process.kill();
        }
    }

    let mut interface = StdInterface::new().expect("Failed to create interface");

    let mut streamish = Streamish::new(&mut interface);
    streamish.run();
}
