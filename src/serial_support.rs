//! Installation-owned serial tools, platform checks and adapter discovery.
use crate::{
    pipeline::{Control, Event},
    play::Serial,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub path: String,
    pub label: String,
    pub identity: Option<String>,
    pub usb: bool,
}

pub fn ports() -> Result<Vec<Port>, String> {
    let mut ports: Vec<_> = serialport::available_ports()
        .map_err(|e| format!("Cannot list serial adapters: {e}"))?
        .into_iter()
        .filter(|p| !cfg!(target_os = "macos") || !p.port_name.starts_with("/dev/tty."))
        .map(|p| {
            let (label, identity, usb) = match p.port_type {
                serialport::SerialPortType::UsbPort(u) => {
                    let name = u
                        .product
                        .or(u.manufacturer)
                        .unwrap_or_else(|| format!("USB adapter {:04X}:{:04X}", u.vid, u.pid));
                    let identity = u
                        .serial_number
                        .filter(|s| !s.is_empty())
                        .map(|s| format!("{:04x}:{:04x}:{s}", u.vid, u.pid));
                    (format!("{name} ({})", p.port_name), identity, true)
                }
                serialport::SerialPortType::BluetoothPort => {
                    (format!("Bluetooth serial ({})", p.port_name), None, false)
                }
                _ => (p.port_name.clone(), None, false),
            };
            Port {
                path: p.port_name,
                label,
                identity,
                usb,
            }
        })
        .collect();
    ports.sort_by(|a, b| a.path.cmp(&b.path));
    ports.dedup_by(|a, b| a.path == b.path);
    Ok(ports)
}

/// Never silently replace a missing remembered device with a different adapter.
pub fn select_port(settings: &mut Serial, ports: &[Port]) -> Result<(), String> {
    let selected = if let Some(id) = &settings.device_id {
        let matches: Vec<_> = ports
            .iter()
            .filter(|p| p.identity.as_ref() == Some(id))
            .collect();
        match matches.as_slice() {
            [port] => Some(*port),
            [] => return Err("The saved adapter is disconnected. Connect it or choose another adapter in PSX connection.".into()),
            _ => matches.into_iter().find(|p| p.path == settings.port),
        }
    } else if !settings.port.is_empty() {
        ports.iter().find(|p| p.path == settings.port)
    } else {
        // Enumerating a port is read-only; probe only the selected USB adapter.
        let candidates: Vec<_> = ports.iter().filter(|p| p.usb).collect();
        (candidates.len() == 1).then(|| candidates[0])
    };
    let port = selected.ok_or_else(|| if ports.is_empty() {
        "No serial adapters found. Connect the adapter. If it still does not appear, check its driver in your operating system.".to_owned()
    } else {
        "Choose your PSX adapter in PSX connection. Epok will not send data to other serial devices.".to_owned()
    })?;
    settings.port = port.path.clone();
    settings.device_id = port.identity.clone();
    settings.validate()
}

pub struct Task<'a> {
    pub tx: &'a Sender<Event>,
    pub rx: &'a Receiver<Control>,
}
impl Task<'_> {
    pub fn terminal(&self, command: Command, timeout: Duration) -> Result<String, String> {
        self.check()?;
        let mut terminal = crate::serial_terminal::Terminal::spawn(command)?;
        let started = Instant::now();
        let mut output = String::new();
        loop {
            self.check()?;
            let status = terminal.poll()?;
            let drained = terminal.drained();
            for chunk in terminal.output.try_iter() {
                output.push_str(&chunk);
                if output.len() > 65536 {
                    let mut n = output.len() - 32768;
                    while !output.is_char_boundary(n) {
                        n += 1;
                    }
                    output.drain(..n);
                }
            }
            if let Some(status) = status
                && drained
            {
                return if status.success() {
                    Ok(output)
                } else {
                    Err(format!("Serial tool failed: {}", output.trim()))
                };
            }
            if started.elapsed() >= timeout {
                return Err(format!(
                    "The serial tool did not respond in time. {}",
                    output.trim()
                ));
            }
            thread::sleep(Duration::from_millis(30));
        }
    }
    pub fn check(&self) -> Result<(), String> {
        if matches!(
            self.rx.try_recv(),
            Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected)
        ) {
            Err("Serial preparation cancelled.".into())
        } else {
            Ok(())
        }
    }
    pub fn log(&self, text: impl Into<String>) {
        let _ = self.tx.send(Event::Log(text.into()));
    }
    pub fn stage(&self, text: &str) {
        let _ = self.tx.send(Event::Stage(text.into()));
        self.log(text);
    }

    /// Drain both pipes while waiting, with a deadline and bounded output. Never blocks the UI.
    pub fn capture(&self, mut command: Command, timeout: Duration) -> Result<String, String> {
        self.check()?;
        crate::pipeline::quiet(&mut command);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = crate::serial::Owned(
            command
                .spawn()
                .map_err(|e| format!("Cannot start serial component: {e}"))?,
        );
        let (send, receive) = mpsc::channel();
        let pipes: Vec<Box<dyn Read + Send>> = vec![
            Box::new(child.0.stdout.take().unwrap()),
            Box::new(child.0.stderr.take().unwrap()),
        ];
        let readers: Vec<_> = pipes
            .into_iter()
            .map(|mut pipe| {
                let send = send.clone();
                thread::spawn(move || {
                    let mut buffer = [0; 2048];
                    while let Ok(n) = pipe.read(&mut buffer) {
                        if n == 0 {
                            break;
                        }
                        if send
                            .send(String::from_utf8_lossy(&buffer[..n]).into_owned())
                            .is_err()
                        {
                            break;
                        }
                    }
                })
            })
            .collect();
        drop(send);
        let started = Instant::now();
        let mut output = String::new();
        let result = loop {
            if let Err(e) = self.check() {
                break Err(e);
            }
            let status = match child.0.try_wait() {
                Ok(s) => s,
                Err(e) => break Err(e.to_string()),
            };
            let complete = status.is_some() && readers.iter().all(|r| r.is_finished());
            for chunk in receive.try_iter() {
                output.push_str(&chunk);
                if output.len() > 65536 {
                    let mut n = output.len() - 32768;
                    while !output.is_char_boundary(n) {
                        n += 1;
                    }
                    output.drain(..n);
                }
            }
            if complete {
                break if status.unwrap().success() {
                    Ok(output)
                } else {
                    Err(format!("Serial component failed: {}", output.trim()))
                };
            }
            if started.elapsed() >= timeout {
                break Err("The serial component did not respond in time. Check the console, runtime and port access, then retry.".into());
            }
            thread::sleep(Duration::from_millis(30));
        };
        drop(child);
        for reader in readers {
            let _ = reader.join();
        }
        result
    }
}

#[derive(Deserialize)]
struct Package {
    revision: String,
    source: String,
    base_url: String,
    files: Vec<File>,
}
#[derive(Deserialize)]
struct File {
    name: String,
    sha256: String,
    size: u64,
}
fn package() -> Package {
    serde_json::from_str(include_str!("../tools/serial-dependency.json"))
        .expect("Bundled serial manifest")
}
pub fn managed_directory() -> PathBuf {
    installation_directory()
        .join("tools/notpsxserial")
        .join(package().revision)
}
fn installation_directory() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(crate::workspace::editor_home)
}
pub fn tools_installed() -> bool {
    installed(&managed_directory(), &package())
}
fn valid(bytes: &[u8], file: &File) -> bool {
    bytes.len() as u64 == file.size && format!("{:x}", Sha256::digest(bytes)) == file.sha256
}
fn installed(folder: &Path, package: &Package) -> bool {
    package
        .files
        .iter()
        .all(|file| fs::read(folder.join(&file.name)).is_ok_and(|b| valid(&b, file)))
}

/// Verify the entire package before publishing it; an interrupted download never becomes an installation.
fn install(
    folder: &Path,
    sources: &[PathBuf],
    package: &Package,
    task: &Task<'_>,
) -> Result<(), String> {
    if installed(folder, package) {
        return Ok(());
    }
    let parent = folder.parent().ok_or("Invalid serial tools directory")?;
    fs::create_dir_all(parent).map_err(|e| format!(
        "Cannot install serial tools in {}: {e}. This Epok installation must be writable. Move it to a writable folder or ask its administrator to install the components.", parent.display()))?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join("install.lock"))
        .map_err(|e| e.to_string())?;
    lock.try_lock().map_err(
        |_| "Another Epok window is preparing serial tools. Wait for it to finish, then retry.",
    )?;
    if installed(folder, package) {
        return Ok(());
    }
    task.check()?;
    let temporary = parent.join(format!("download-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&temporary).map_err(|e| e.to_string())?;
    let result = (|| {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;
        for file in &package.files {
            task.check()?;
            if file.name.contains(['/', '\\']) || file.name.starts_with('.') {
                return Err("Invalid serial package entry".into());
            }
            task.log(format!("Preparing {}", file.name));
            let bytes = std::iter::once(folder)
                .chain(sources.iter().map(PathBuf::as_path))
                .find_map(|p| fs::read(p.join(&file.name)).ok().filter(|b| valid(b, file)));
            let bytes = match bytes {
                Some(bytes) => bytes,
                None => {
                    let mut bytes = Vec::new();
                    client
                        .get(format!("{}{}", package.base_url, file.name))
                        .send()
                        .and_then(|r| r.error_for_status())
                        .map_err(|e| {
                            format!("Download failed. Check your connection and retry: {e}")
                        })?
                        .take(file.size + 1)
                        .read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())?;
                    bytes
                }
            };
            task.check()?;
            if !valid(&bytes, file) {
                return Err(format!(
                    "The downloaded {} failed verification. Retry preparing the tools.",
                    file.name
                ));
            }
            fs::write(temporary.join(&file.name), bytes).map_err(|e| e.to_string())?;
        }
        fs::write(temporary.join("SOURCE.txt"), format!("NOTPSXSerial, unmodified\nSource: {}\nLicense: MPL-2.0; see LICENSE and THIRD_PARTY_NOTICES.txt\n", package.source)).map_err(|e| e.to_string())?;
        task.check()?;
        let backup = parent.join(format!("previous-{}", uuid::Uuid::new_v4()));
        let previous = folder.exists();
        if previous {
            fs::rename(folder, &backup).map_err(|e| e.to_string())?;
        }
        if let Err(e) = fs::rename(&temporary, folder) {
            if previous {
                let _ = fs::rename(&backup, folder);
            }
            return Err(e.to_string());
        }
        // Keep a repaired package's previous contents recoverable; never remove user files.
        Ok(())
    })();
    if temporary.exists() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

pub fn host_label() -> String {
    format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH)
}
fn managed_supported(os: &str, arch: &str) -> Result<(), String> {
    match (os, arch) {
        ("windows", "x86" | "x86_64")
        | ("macos", "aarch64" | "x86_64")
        | ("linux", "aarch64" | "x86_64") => Ok(()),
        _ => Err(format!(
            "Serial components are not available for {os} / {arch}. Use a supported host or an emulator target."
        )),
    }
}
fn mono() -> Option<PathBuf> {
    [
        "mono",
        "/opt/homebrew/bin/mono",
        "/usr/local/bin/mono",
        "/Library/Frameworks/Mono.framework/Versions/Current/bin/mono",
    ]
    .into_iter()
    .find_map(crate::dependencies::find_executable)
}
fn framework_release(output: &str) -> Option<u32> {
    output
        .lines()
        .find(|l| l.split_whitespace().next() == Some("Release"))?
        .split_whitespace()
        .last()
        .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok())
}
fn runtime(task: &Task<'_>) -> Result<(), String> {
    if cfg!(windows) {
        let mut command = Command::new("reg.exe");
        command.args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full",
            "/v",
            "Release",
            "/reg:32",
        ]);
        let output = task.capture(command, Duration::from_secs(10)).map_err(|_| "Cannot find .NET Framework 4.7.2 or later. Install the Microsoft .NET Framework 4.8 runtime for this Windows version, then retry.")?;
        if framework_release(&output).is_none_or(|r| r < 461808) {
            return Err("Serial tools require .NET Framework 4.7.2 or later. Install the Microsoft .NET Framework 4.8 runtime for your Windows version, then retry.".into());
        }
    } else {
        if mono().is_none() && cfg!(target_os = "macos") {
            let brew = ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"].into_iter().find_map(crate::dependencies::find_executable)
                .ok_or("Serial tools need Mono. Install Homebrew from https://brew.sh, then use Prepare tools; Epok will install Mono. You can also provide an existing Mono installation.")?;
            task.stage("Installing the serial runtime with Homebrew...");
            let mut command = Command::new(brew);
            command
                .args(["install", "mono"])
                .env("HOMEBREW_NO_AUTO_UPDATE", "1")
                .env("HOMEBREW_NO_ENV_HINTS", "1")
                .env("NONINTERACTIVE", "1");
            task.capture(command, Duration::from_secs(1200))?;
        }
        let path = mono().ok_or("Serial tools need Mono. Install your distribution's Mono runtime and serial-port support, then retry. Linux support is experimental.")?;
        let mut command = Command::new(path);
        command.arg("--version");
        task.capture(command, Duration::from_secs(15))
            .map_err(|e| {
                format!(
                    "Mono cannot run on {}. Install a compatible runtime. {e}",
                    host_label()
                )
            })?;
    }
    Ok(())
}

pub fn launcher(settings: &Serial) -> Result<Command, String> {
    let _ = settings; // Legacy executable preferences are import hints, never launch paths.
    let exe = managed_directory().join("nops.exe");
    if !cfg!(windows)
        && exe
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
    {
        // Runtime capability is checked by prepare before any real launch.
        let mut command = Command::new(mono().unwrap_or_else(|| PathBuf::from("mono")));
        command.arg(exe);
        Ok(command)
    } else {
        Ok(Command::new(exe))
    }
}

pub fn prepare(settings: &Serial, task: &Task<'_>) -> Result<(), String> {
    task.stage(&format!("Checking serial support on {}...", host_label()));
    managed_supported(std::env::consts::OS, std::env::consts::ARCH)?;
    runtime(task)?;
    task.stage("Preparing verified PSX serial tools...");
    let mut sources = vec![
        installation_directory().join(".tools/serial-bundle"),
        crate::workspace::editor_home().join(".tools/serial-bundle"),
        crate::workspace::user_data()
            .join("tools/notpsxserial")
            .join(package().revision),
    ];
    // Migrate a previously configured copy only after verifying each pinned file.
    // It is copied into this installation; Play never depends on the old location.
    if !settings.executable.trim().is_empty()
        && let Some(exe) = crate::dependencies::find_executable(&settings.executable)
        && let Some(parent) = exe.parent()
    {
        sources.push(parent.to_path_buf());
    }
    install(&managed_directory(), &sources, &package(), task)?;
    task.stage("Checking that the serial tools can run...");
    let mut command = launcher(settings)?;
    command.current_dir(session_directory()?);
    let output = task.terminal(command, Duration::from_secs(15))?;
    let lower = output.to_lowercase();
    if !["nops", "notpsxserial", "unirom"]
        .iter()
        .any(|s| lower.contains(s))
    {
        return Err("The serial component did not identify itself as NOTPSXSerial. Repair the tools in PSX connection.".into());
    }
    task.log("Serial tools ready.");
    Ok(())
}

/// ConPTY can flatten the newline before nops' final "Bye!". Accept that
/// known trailer without treating help text or arbitrary response prefixes as
/// acknowledgements. Call only once the process output has been drained.
pub(crate) fn confirmed_record(output: &str, expected: &str) -> bool {
    output.lines().any(|line| {
        let line = line.trim();
        line.eq_ignore_ascii_case(expected)
            || (line
                .get(..expected.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(expected))
                && line[expected.len()..].trim().eq_ignore_ascii_case("Bye!"))
    })
}

fn ping_ok(output: &str) -> bool {
    // nops can exit zero on failure, and its help mentions PONG. Require the actual reply.
    (confirmed_record(output, "Response: True") || confirmed_record(output, "Got response: PONG"))
        && !output.to_lowercase().contains("response: false")
}
pub fn probe(settings: &Serial, task: &Task<'_>) -> Result<(), String> {
    settings.validate()?;
    task.stage(&format!("Testing Unirom on {}...", settings.port));
    check_port(settings)?;
    let mut command = launcher(settings)?;
    command
        .current_dir(session_directory()?)
        .args(["/dest", &settings.port, "/ping"]);
    if settings.fast {
        command.arg("/fast");
    }
    let output = task.terminal(command, Duration::from_secs(8)).map_err(|e| format!(
        "Unirom did not answer the optional ping on {}. Direct Play does not require this test; return the console to its Unirom loader and try Play. {e}", settings.port))?;
    if !ping_ok(&output) {
        return Err("Unirom did not confirm the optional ping. Direct Play can still work; return to the Unirom loader and try Play.".into());
    }
    task.log("Unirom responded. The PSX is ready.");
    Ok(())
}
pub fn check_port(settings: &Serial) -> Result<(), String> {
    settings.validate()?;
    // Test OS access first. No bytes are written and DTR/RTS are not explicitly toggled.
    let baud = if settings.fast { 510000 } else { 115200 };
    let port = serialport::new(&settings.port, baud).timeout(Duration::from_millis(250)).open()
        .map_err(|e| format!("Cannot access {}: {e}. Close other serial programs and check device permissions{}.", settings.port, if cfg!(target_os="linux") {" (your distribution's serial-device group/udev rules)"} else {" or the adapter driver"}))?;
    drop(port);
    Ok(())
}
pub fn session_directory() -> Result<PathBuf, String> {
    // Mutable COMPORT.TXT is session state, not an installed component.
    let work = crate::workspace::user_data().join("serial-session");
    fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    Ok(work)
}

pub fn discover(settings: &mut Serial, task: &Task<'_>) -> Result<(), String> {
    task.stage("Looking for serial adapters...");
    let mut ports = ports()?;
    // Windows exposes some Bluetooth ports only through SERIALCOMM, without USB metadata.
    // Label those explicitly so a new user does not mistake them for the PSX cable.
    if cfg!(windows) && ports.iter().any(|p| !p.usb) {
        let mut command = Command::new("reg.exe");
        command.args(["query", r"HKLM\HARDWARE\DEVICEMAP\SERIALCOMM"]);
        match task.capture(command, Duration::from_secs(5)) {
            Ok(output) => {
                for line in output
                    .lines()
                    .filter(|l| l.to_lowercase().contains(r"\device\bthmodem"))
                {
                    if let Some(name) = line.split_whitespace().last()
                        && let Some(port) = ports.iter_mut().find(|p| p.path == name && !p.usb)
                    {
                        port.label = format!("Bluetooth serial ({name})");
                    }
                }
            }
            Err(error) if error.starts_with("Serial preparation cancelled") => return Err(error),
            Err(_) => {} // The native port list is still usable if optional labels are unavailable.
        }
    }
    task.check()?;
    let _ = task.tx.send(Event::SerialPorts(ports.clone()));
    select_port(settings, &ports)?;
    let _ = task.tx.send(Event::SerialConfigured(settings.clone()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn port(path: &str, id: Option<&str>) -> Port {
        Port {
            path: path.into(),
            label: path.into(),
            identity: id.map(str::to_owned),
            usb: true,
        }
    }
    #[test]
    fn reconnect_tracks_identity_and_never_replaces_a_missing_device() {
        let mut s = Serial {
            port: "COM3".into(),
            device_id: Some("my-adapter".into()),
            ..Default::default()
        };
        select_port(&mut s, &[port("COM9", Some("my-adapter"))]).unwrap();
        assert_eq!(s.port, "COM9");
        assert!(select_port(&mut s, &[port("COM9", Some("another"))]).is_err());
        assert!(
            select_port(
                &mut Serial::default(),
                &[port("COM1", None), port("COM2", None)]
            )
            .is_err()
        );
        let mut fresh = Serial::default();
        select_port(&mut fresh, &[port("COM2", None)]).unwrap();
        assert_eq!(fresh.port, "COM2");
    }
    #[test]
    fn framework_and_architecture_are_capabilities_not_os_guesses() {
        assert_eq!(
            framework_release("    Release    REG_DWORD    0x80eb1"),
            Some(528049)
        );
        assert_eq!(framework_release("Release REG_DWORD unknown"), None);
        assert!(managed_supported("windows", "aarch64").is_err());
        assert!(managed_supported("macos", "aarch64").is_ok());
        assert!(managed_supported("linux", "x86_64").is_ok());
    }
    #[test]
    fn serial_components_live_under_the_running_editor_directory() {
        let exe = std::env::current_exe().unwrap();
        assert_eq!(
            managed_directory(),
            exe.parent()
                .unwrap()
                .join("tools/notpsxserial")
                .join(package().revision)
        );
    }
    #[test]
    fn help_and_successful_exit_are_not_a_handshake() {
        assert!(!ping_ok("/ping returns PONG!\nBye!"));
        assert!(!ping_ok("Waiting for PING/PONG\nResponse: False"));
        assert!(ping_ok("Starting command: PING\nResponse: True\nBye!"));
        assert!(ping_ok("Starting command: PING\nResponse: True Bye!"));
        assert!(ping_ok("Got response: PONG Bye!"));
        assert!(ping_ok("Response: TrueBye!"));
        assert!(!ping_ok("Response: Trueish Bye!"));
        assert!(!ping_ok("Expected Response: True Bye!"));
        assert!(!ping_ok("Response: True error"));
        assert!(!ping_ok("Response: True Bye!\nResponse: False"));
    }
    #[test]
    fn verified_offline_package_is_installed_and_bad_repair_cannot_replace_it() {
        let root =
            std::env::temp_dir().join(format!("epok-serial-package-{}", uuid::Uuid::new_v4()));
        let bundled = root.join("bundle");
        fs::create_dir_all(&bundled).unwrap();
        let bytes = b"verified test component";
        let mut package = Package {
            revision: "test".into(),
            source: "test source".into(),
            base_url: "https://127.0.0.1:1/".into(),
            files: vec![File {
                name: "nops.exe".into(),
                sha256: format!("{:x}", Sha256::digest(bytes)),
                size: bytes.len() as u64,
            }],
        };
        fs::write(bundled.join("nops.exe"), bytes).unwrap();
        let (tx, _) = mpsc::channel();
        let (_stop, rx) = mpsc::channel();
        let task = Task { tx: &tx, rx: &rx };
        let folder = root.join("tools/test");
        install(&folder, std::slice::from_ref(&bundled), &package, &task).unwrap();
        assert!(installed(&folder, &package));
        assert!(folder.join("SOURCE.txt").is_file());
        package.files[0].sha256 = "incorrect".into();
        assert!(install(&folder, &[bundled], &package, &task).is_err());
        assert_eq!(fs::read(folder.join("nops.exe")).unwrap(), bytes);
        fs::remove_dir_all(root).unwrap();
    }
}
