//! Bounded stdio transport for LuaLS, clangd and the official Copilot server.
//! Process creation and all pipe I/O happen off the editor/render thread.
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
};

pub struct Spec {
    pub executable: PathBuf,
    pub arguments: Vec<OsString>,
    pub directory: PathBuf,
    pub framing: Framing,
}

#[derive(Clone, Copy)]
pub enum Framing {
    Lsp,
    JsonLines,
}
enum Control {
    Start { id: Value, name: String, spec: Spec },
    Send { name: String, message: Value },
    Stop,
}
struct Server {
    child: Child,
    input: SyncSender<Value>,
}

pub struct Service {
    control: SyncSender<Control>,
    pub events: Receiver<Value>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Default for Service {
    fn default() -> Self {
        let (control, requests) = mpsc::sync_channel(256);
        let (events, receiver) = mpsc::sync_channel(256);
        let worker = thread::spawn(move || {
            let mut servers: BTreeMap<String, Server> = BTreeMap::new();
            let mut generation = 0u64;
            while let Ok(request) = requests.recv() {
                match request {
                    Control::Start { id, name, spec } => {
                        if let Some(mut previous) = servers.remove(&name) {
                            let _ = previous.child.kill();
                            let _ = previous.child.wait();
                        }
                        generation += 1;
                        match spawn(&name, spec, events.clone(), generation) {
                            Ok(server) => {
                                servers.insert(name.clone(), server);
                                let _ = events
                                    .send(json!({"id":id,"result":{"started":true,"server":name,"generation":generation}}));
                            }
                            Err(error) => {
                                let _ = events.send(json!({"id":id,"error":error}));
                            }
                        }
                    }
                    Control::Send { name, message } => {
                        let result = servers
                            .get(&name)
                            .ok_or("Language service is not running".to_string())
                            .and_then(|server| {
                                server.input.try_send(message).map_err(|e| e.to_string())
                            });
                        if let Err(error) = result {
                            let _ = events.send(
                                json!({"event":"server_error","server":name,"message":error}),
                            );
                        }
                    }
                    Control::Stop => break,
                }
            }
            for (_, mut server) in servers {
                let _ = server.child.kill();
                let _ = server.child.wait();
            }
        });
        Self {
            control,
            events: receiver,
            worker: Some(worker),
        }
    }
}
impl Service {
    pub fn start(&self, id: Value, name: &str, spec: Spec) -> Result<(), String> {
        self.control
            .try_send(Control::Start {
                id,
                name: name.into(),
                spec,
            })
            .map_err(|e| e.to_string())
    }
    pub fn send(&self, name: &str, message: Value) -> Result<(), String> {
        self.control
            .try_send(Control::Send {
                name: name.into(),
                message,
            })
            .map_err(|e| e.to_string())
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        let (_, empty_events) = mpsc::channel();
        drop(std::mem::replace(&mut self.events, empty_events));
        let (empty_control, _) = mpsc::sync_channel(1);
        let control = std::mem::replace(&mut self.control, empty_control);
        let _ = control.try_send(Control::Stop);
        drop(control);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn spawn(
    name: &str,
    spec: Spec,
    events: SyncSender<Value>,
    generation: u64,
) -> Result<Server, String> {
    let framing = spec.framing;
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.arguments)
        .current_dir(spec.directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::pipeline::quiet(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("{}: {e}", spec.executable.display()))?;
    let mut stdin = child.stdin.take().ok_or("Server stdin unavailable")?;
    let stdout = child.stdout.take().ok_or("Server stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("Server stderr unavailable")?;
    let (input, messages) = mpsc::sync_channel::<Value>(128);
    thread::spawn(move || {
        while let Ok(message) = messages.recv() {
            let bytes = match serde_json::to_vec(&message) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            let header = match framing {
                Framing::Lsp => write!(stdin, "Content-Length: {}\r\n\r\n", bytes.len()),
                Framing::JsonLines => Ok(()),
            };
            if header
                .and_then(|_| stdin.write_all(&bytes))
                .and_then(|_| match framing {
                    Framing::JsonLines => stdin.write_all(b"\n"),
                    Framing::Lsp => Ok(()),
                })
                .and_then(|_| stdin.flush())
                .is_err()
            {
                break;
            }
        }
    });
    let output_name = name.to_string();
    let output_events = events.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            match match framing {
                Framing::Lsp => read_message(&mut reader),
                Framing::JsonLines => read_json_line(&mut reader),
            } {
                Ok(Some(message)) => {
                    if output_events
                        .send(json!({"event":"lsp","server":output_name,"generation":generation,"message":message}))
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {
                    let _=output_events.send(json!({"event":"server_error","server":output_name,"generation":generation,"message":"Language service exited. Open Setup to restart it."}));
                    break;
                }
                Err(error) => {
                    let _ = output_events
                        .send(json!({"event":"server_error","server":output_name,"generation":generation,"message":error}));
                    break;
                }
            }
        }
    });
    let log_name = name.to_string();
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = Vec::new();
        // Limit individual log lines as well as the channel's retained messages.
        while let Ok(size) = reader.by_ref().take(8192).read_until(b'\n', &mut line) {
            if size == 0 {
                break;
            }
            let message = String::from_utf8_lossy(&line).trim().to_string();
            let _ =
                events.try_send(json!({"event":"server_log","server":log_name,"message":message}));
            line.clear();
        }
    });
    Ok(Server { child, input })
}

fn read_json_line(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    const LIMIT: usize = 8 * 1024 * 1024;
    let mut bytes = Vec::new();
    let count = reader
        .take((LIMIT + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| error.to_string())?;
    if count == 0 {
        return Ok(None);
    }
    if count > LIMIT {
        return Err("Chat server message exceeds 8 MiB".into());
    }
    if bytes.last() != Some(&b'\n') {
        return Err("Truncated chat server message".into());
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut length = None;
    let mut headers = 0;
    loop {
        let mut line = String::new();
        let count = (&mut *reader)
            .take(8193)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if count == 0 {
            return if headers == 0 {
                Ok(None)
            } else {
                Err("Truncated language-server header".into())
            };
        }
        headers += count;
        if headers > 8192 {
            return Err("Language-server headers exceed 8 KiB".into());
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("Content-Length")
        {
            length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Invalid LSP Content-Length")?,
            );
        }
    }
    let length = length.ok_or("LSP Content-Length is missing")?;
    if length > 8 * 1024 * 1024 {
        return Err("Language-server message exceeds 8 MiB".into());
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(|e| e.to_string())?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framing_handles_unicode_consecutive_messages_and_eof() {
        let first = json!({"jsonrpc":"2.0","method":"test","params":"日本語"});
        let body = serde_json::to_vec(&first).unwrap();
        let mut wire = format!(
            "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\ncontent-length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        wire.extend(body);
        wire.extend(b"Content-Length: 2\r\n\r\n{}");
        let mut reader = BufReader::new(wire.as_slice());
        assert_eq!(read_message(&mut reader).unwrap(), Some(first));
        assert_eq!(read_message(&mut reader).unwrap(), Some(json!({})));
        assert_eq!(read_message(&mut reader).unwrap(), None);
    }
    #[test]
    fn oversized_and_incomplete_messages_are_rejected() {
        for bytes in [
            b"Content-Length: 99999999\r\n\r\n".as_slice(),
            b"Content-Length: 5\r\n\r\n{}",
            b"Content-Type: text/plain\r\n\r\n{}",
        ] {
            assert!(read_message(&mut BufReader::new(bytes)).is_err());
        }
    }
    #[test]
    fn chat_framing_preserves_unicode_and_rejects_truncated_messages() {
        let message = json!({"method":"session/update","params":{"text":"日本語\ncode"}});
        let mut wire = serde_json::to_vec(&message).unwrap();
        wire.extend(b"\n{}\n");
        let mut reader = BufReader::new(wire.as_slice());
        assert_eq!(read_json_line(&mut reader).unwrap(), Some(message));
        assert_eq!(read_json_line(&mut reader).unwrap(), Some(json!({})));
        assert_eq!(read_json_line(&mut reader).unwrap(), None);
        assert!(read_json_line(&mut BufReader::new(b"{}".as_slice())).is_err());
        assert!(read_json_line(&mut BufReader::new(b"broken\n".as_slice())).is_err());
    }
}
