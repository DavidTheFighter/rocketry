use streamish::Streamish;
use sysinfo::{SystemExt, ProcessExt, Pid};

mod broadcast;
mod stream;
mod streamish;

fn main() {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    for (pid, process) in system.processes() {
        if process.name() == "streamish" && usize::from(*pid) != std::process::id() as usize {
            println!("Streamish: Killing existing streamish process {:?}", pid);
            process.kill();
        }
    }

    let mut streamish = Streamish::new();
    streamish.run();
}
