use std::process::Stdio;

use crate::process_is_running;


struct TerminalThread {

}

impl TerminalThread {
    pub fn new() -> TerminalThread {
        TerminalThread {}
    }

    pub fn run(&mut self) {
        let mut ttyd_process = std::process::Command::new("ttyd")
            .arg("-W") // Writeable
            // .arg("-c david:4369") // Set credentials
            .arg("-p 8001") // Set port
            .arg("ipython")
            .arg("ipython_startup.py")
            .arg("-i") // Interactive
            .arg("--no-banner")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .expect("Failed to start ttyd process");

        while process_is_running() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        ttyd_process
            .kill()
            .expect("Failed to kill ttyd process");
    }
}


pub fn terminal_thread() {
    let mut terminal_thread = TerminalThread::new();
    terminal_thread.run();
}
