//! Private child terminal: nops needs console APIs; Redux must not open a console.
//! Windows uses ConPTY (Windows 10 1809+); Unix uses a controlling PTY.
use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize};
use std::{
    io::{Read, Write},
    process::Command,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};

pub struct Terminal {
    child: Box<dyn Child + Send + Sync>,
    master: Option<Box<dyn MasterPty + Send>>,
    reader: Option<thread::JoinHandle<()>>,
    input: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    pub output: Receiver<String>,
}
impl Terminal {
    pub fn process_id(&self) -> Option<u32> {
        self.child.process_id()
    }
    pub fn spawn(command: Command) -> Result<Self, String> {
        let pair=std::panic::catch_unwind(|| portable_pty::native_pty_system().openpty(PtySize {rows:40,cols:160,pixel_width:0,pixel_height:0}))
            .map_err(|_| "This OS does not provide the private child terminal. Windows needs Windows 10 version 1809 or later.")?
            .map_err(|e|format!("Cannot create the private child terminal: {e}. Windows requires Windows 10 version 1809 or later; Unix requires PTY access."))?;
        let mut builder = CommandBuilder::new(command.get_program());
        builder.args(command.get_args());
        if let Some(cwd) = command.get_current_dir() {
            builder.cwd(cwd);
        }
        for (key, value) in command.get_envs() {
            if let Some(value) = value {
                builder.env(key, value);
            } else {
                builder.env_remove(key);
            }
        }
        builder.env("TERM", "xterm");
        let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
        let input = Arc::new(Mutex::new(
            pair.master.take_writer().map_err(|e| e.to_string())?,
        ));
        let writer = input.clone();
        let (tx, output) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut filter = Text::default();
            let mut buffer = [0; 4096];
            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                let text = filter.push(&buffer[..n]);
                // ConPTY requests the cursor position before starting with INHERIT_CURSOR.
                for _ in 0..std::mem::take(&mut filter.cursor_queries) {
                    let mut writer = writer.lock().unwrap();
                    let _ = writer.write_all(b"\x1b[1;1R");
                    let _ = writer.flush();
                }
                if !text.is_empty() && tx.send(text).is_err() {
                    break;
                }
            }
        });
        // Drain/respond before spawning: even a failed launch must be able to close ConPTY.
        let child = match pair.slave.spawn_command(builder) {
            Ok(child) => child,
            Err(error) => {
                drop(pair);
                let _ = worker.join();
                return Err(format!("Cannot launch child process: {error}"));
            }
        };
        drop(pair.slave);
        Ok(Self {
            child,
            master: Some(pair.master),
            reader: Some(worker),
            input: Some(input),
            output,
        })
    }
    pub fn poll(&mut self) -> Result<Option<ExitStatus>, String> {
        let status = self.child.try_wait().map_err(|e| e.to_string())?;
        // Close ConPTY only while the reader is draining it, then wait for its final bytes.
        if status.is_some() {
            self.input.take();
            self.master.take();
        }
        Ok(status)
    }
    pub fn drained(&self) -> bool {
        self.reader.as_ref().is_none_or(|r| r.is_finished())
    }
    /// Input to the owned child, serialized with terminal cursor responses.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        let mut input = self
            .input
            .as_ref()
            .ok_or("The child terminal has closed")?
            .lock()
            .map_err(|_| "The child terminal input failed")?;
        input
            .write_all(bytes)
            .and_then(|_| input.flush())
            .map_err(|e| e.to_string())
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            #[cfg(windows)]
            if let Some(pid) = self.child.process_id() {
                let mut command = Command::new("taskkill.exe");
                command
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());
                crate::pipeline::quiet(&mut command);
                let _ = command.status();
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        self.input.take();
        self.master.take();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

/// Remove terminal control sequences across read boundaries; preserve ordinary log text.
#[derive(Default)]
struct Text {
    escape: u8,
    csi: Vec<u8>,
    cursor_queries: usize,
}
impl Text {
    fn push(&mut self, bytes: &[u8]) -> String {
        let mut text = Vec::new();
        for &byte in bytes {
            self.escape = match (self.escape, byte) {
                (0, 0x1b) => 1,
                (0, b'\n' | b'\r' | b'\t' | 0x20..=0xff) => {
                    text.push(byte);
                    0
                }
                (0, _) => 0,
                (1, b'[') => {
                    self.csi.clear();
                    2
                }
                (1, b']') => 3,
                (1, b'(' | b')') => 5,
                (1, _) => 0,
                (2, 0x40..=0x7e) => {
                    if byte == b'n' && self.csi == b"6" {
                        self.cursor_queries += 1;
                    }
                    0
                }
                (2, _) => {
                    if self.csi.len() < 32 {
                        self.csi.push(byte);
                    }
                    2
                }
                (3, 7) => 0,
                (3, 0x1b) => 4,
                (3, _) => 3,
                (4, b'\\') => 0,
                (4, _) => 3,
                _ => 0,
            };
        }
        String::from_utf8_lossy(&text).into_owned()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_program_closes_its_terminal_without_blocking() {
        let (tx, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let result =
                Terminal::spawn(Command::new("epok-missing-serial-program-89570135")).map(|_| ());
            let _ = tx.send(result);
        });
        assert!(
            rx.recv_timeout(std::time::Duration::from_secs(10))
                .expect("Terminal cleanup timed out")
                .is_err()
        );
        worker.join().unwrap();
    }
    #[test]
    fn terminal_codes_can_span_reads_without_hiding_the_handshake() {
        let mut text = Text::default();
        assert_eq!(text.push(b"\x1b[3"), "");
        assert_eq!(
            text.push(b"2mResponse: True\x1b[0m\r\n"),
            "Response: True\r\n"
        );
        assert_eq!(text.push(b"\x1b]0;window title\x1b"), "");
        assert_eq!(text.push(b"\\EPOK: runtime ready"), "EPOK: runtime ready");
        assert_eq!(text.push(b"\x1b[6"), "");
        assert_eq!(text.push(b"n"), "");
        assert_eq!(text.cursor_queries, 1);
    }
}
