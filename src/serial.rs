//! Own only the NOTPSXSerial processes started by this Play session.
use crate::{
    pipeline::{Control, Event},
    play::Serial,
};
use std::{
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

pub(crate) struct Owned(pub Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            crate::pipeline::stop(&mut self.0);
        }
    }
}

pub fn command(
    build: &Path,
    settings: &Serial,
    exe: Option<&Path>,
    monitor: bool,
) -> Result<Command, String> {
    let mut command = crate::serial_support::launcher(settings)?;
    command.current_dir(build);
    if let Some(exe) = exe {
        command
            .arg("/exe")
            .arg(exe.strip_prefix(build).unwrap_or(exe));
    } else {
        command.arg("/debug");
    }
    command.arg("/dest").arg(&settings.port);
    if settings.fast {
        command.arg("/fast");
    }
    if monitor {
        command.arg("/m");
    }
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    crate::pipeline::quiet(&mut command);
    Ok(command)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Setup,
    Monitor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Request {
    Probe,
    Pause,
    Resume,
    Reset,
}
impl Request {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Probe => b"PING",
            Self::Pause => b"HALT",
            Self::Resume => b"CONT",
            Self::Reset => b"REST",
        }
    }
    fn reply(self) -> &'static [u8] {
        match self {
            Self::Probe => b"PONG",
            Self::Pause => b"HLTD",
            Self::Resume | Self::Reset => b"OKAY",
        }
    }
}

struct Pending {
    request: Request,
    since: Instant,
    reply: Vec<u8>,
}
impl Pending {
    fn new(request: Request) -> Self {
        Self {
            request,
            since: Instant::now(),
            reply: Vec::new(),
        }
    }
    fn observe(&mut self, chunk: &str) -> Option<bool> {
        // Keep only this request's output; old upload/debug acknowledgements must
        // never acknowledge a new command. Replies can span PTY reads.
        self.reply.extend_from_slice(chunk.as_bytes());
        let accepted = self.reply.windows(4).any(|w| w == self.request.reply());
        let rejected = self.reply.windows(4).any(|w| w == b"UNSP" || w == b"ONLY");
        if self.reply.len() > 256 {
            self.reply.drain(..self.reply.len() - 256);
        }
        if rejected {
            Some(false)
        } else if accepted {
            Some(true)
        } else {
            None
        }
    }
}

fn execute(
    command: Command,
    mode: Mode,
    tx: &Sender<Event>,
    rx: &Receiver<Control>,
) -> Result<(), String> {
    if matches!(
        rx.try_recv(),
        Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected)
    ) {
        return Err("Serial session cancelled before connecting.".into());
    }
    let mut terminal = crate::serial_terminal::Terminal::spawn(command)?;
    let start = Instant::now();
    let mut ready = false;
    let mut transcript = String::new();
    let mut exited = None;
    let mut last_progress = None;
    let mut pending: Option<Pending> = None;
    let mut probing = false;
    let mut uploaded = false;
    let mut runtime_ready = false;
    let mut monitor_ready = false;
    let result = loop {
        match terminal.poll() {
            Ok(status) => exited = status.or(exited),
            Err(e) => break Err(e.to_string()),
        }
        // Drain the terminal before accepting an exit status. nops
        // can print a transport failure and still exit with status zero.
        let output_complete = exited.is_some() && terminal.drained();
        let mut response = None;
        let mut chunks = 0;
        // Bound draining so TTY traffic cannot starve Stop or command timeouts.
        for chunk in terminal.output.try_iter().take(128) {
            chunks += 1;
            let _ = tx.send(Event::Log(chunk.trim().to_owned()));
            transcript.push_str(&chunk);
            uploaded |= transcript
                .lines()
                .any(|line| line.trim() == "Send finished!");
            runtime_ready |= transcript.contains("EPOK: runtime ready");
            monitor_ready |= transcript.contains("2-way Serial monitor active");
            if let Some(request) = pending.as_mut() {
                response = response.or(request.observe(&chunk));
            }
            if transcript.len() > 16384 {
                let mut n = transcript.len() - 8192;
                while !transcript.is_char_boundary(n) {
                    n += 1;
                }
                transcript.drain(..n);
            }
        }
        if mode != Mode::Setup
            && !ready
            && let Some(value) = upload_progress(&transcript)
            && last_progress != Some(value)
        {
            last_progress = Some(value);
            let _ = tx.send(Event::Progress(f32::from(value) / 100.));
        }
        if let Some(accepted) = response {
            let request = pending.take().unwrap().request;
            let _ = tx.send(Event::SerialCommandPending(false));
            if request == Request::Probe {
                if !accepted {
                    break Err("Unirom rejected the runtime connection test.".into());
                }
                if !ready {
                    ready = true;
                    let _ = tx.send(Event::SerialConnected);
                }
            } else if !accepted {
                let _ = tx.send(Event::Log(format!(
                    "Unirom rejected {request:?} in the current console state."
                )));
            } else {
                match request {
                    Request::Pause => {
                        let _ = tx.send(Event::Paused(true));
                    }
                    Request::Resume => {
                        let _ = tx.send(Event::Paused(false));
                    }
                    Request::Reset => {
                        let _ = tx.send(Event::Log("Unirom acknowledged reset. Returning to the loader depends on the console's boot setup; wait for it before Play.".into()));
                        break Ok(());
                    }
                    Request::Probe => unreachable!(),
                }
            }
        }
        let lower = transcript.to_lowercase();
        if !ready
            && [
                "no response",
                "timed out",
                "couldn't",
                "cannot open",
                "access to the port",
                "error:",
                "error!",
                "response: false",
                "failed",
            ]
            .iter()
            .any(|s| lower.contains(s))
        {
            break Err(
                "NOTPSXSerial reported a failure. Check the serial log, port and Unirom state."
                    .into(),
            );
        }
        if output_complete
            && chunks < 128
            && let Some(status) = exited
        {
            break if status.success()
                && mode == Mode::Setup
                && crate::serial_support::confirmed_record(&transcript, "Response: True")
            {
                Ok(())
            } else {
                Err(format!(
                    "NOTPSXSerial exited ({status}) without confirming the requested operation. Check the log and return the PSX to the Unirom loader before retrying."
                ))
            };
        }
        if mode == Mode::Monitor && !probing && uploaded && runtime_ready && monitor_ready {
            terminal.write(Request::Probe.bytes())?;
            pending = Some(Pending::new(Request::Probe));
            probing = true;
            let _ = tx.send(Event::Stage("Checking the PSX resident handler".into()));
        }
        if pending
            .as_ref()
            .is_some_and(|p| p.since.elapsed() > Duration::from_secs(5))
        {
            let request = pending.take().unwrap().request;
            if request == Request::Probe {
                if ready {
                    break Err("The PSX resident handler stopped responding. The last command's outcome is unknown; the console may have rebooted or lost its handler. Return to the Unirom loader before Play.".into());
                }
                break Err("The executable was sent, but the resident Unirom handler did not answer after runtime startup. Check the console, SIO interrupt state and kernel compatibility; return to the loader before retrying.".into());
            }
            let _ = tx.send(Event::Log(format!("{request:?} was not confirmed by Unirom. Console state is unknown; checking the handler before accepting another command.")));
            // The protocol has no request IDs. A delayed OKAY must not acknowledge
            // a later Continue/Reset: first fence it with a distinct PING/PONG.
            terminal.write(Request::Probe.bytes())?;
            pending = Some(Pending::new(Request::Probe));
        }
        match rx.try_recv() {
            Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                break Err(
                    "Serial session cancelled; disconnect does not reset or resume the console."
                        .into(),
                );
            }
            Ok(control) if ready => {
                if pending.is_some() {
                    let _ = tx.send(Event::Log(
                        "Waiting for the previous PSX command; no additional command was sent."
                            .into(),
                    ));
                } else {
                    let request = match control {
                        Control::Pause => Request::Pause,
                        Control::Resume => Request::Resume,
                        Control::Reset => Request::Reset,
                        Control::Stop => unreachable!(),
                    };
                    terminal.write(request.bytes())?;
                    pending = Some(Pending::new(request));
                    let _ = tx.send(Event::SerialCommandPending(true));
                    let _ = tx.send(Event::Log(format!(
                        "PSX {request:?} requested; waiting for Unirom."
                    )));
                }
            }
            Ok(_) | Err(mpsc::TryRecvError::Empty) => {}
        }
        if !ready
            && start.elapsed() > Duration::from_secs(if mode == Mode::Setup { 20 } else { 300 })
        {
            break Err(
                "NOTPSXSerial timed out waiting for Unirom / the Epok startup message.".into(),
            );
        }
        thread::sleep(Duration::from_millis(40));
    };
    drop(terminal);
    result
}

/// Pinned nops emits `Sending chunk N of M (P)%`, sometimes across PTY reads.
/// Only parse complete upload records; runtime percentages are not upload progress.
fn upload_progress(transcript: &str) -> Option<u8> {
    transcript
        .rmatch_indices("Sending chunk ")
        .find_map(|(start, _)| {
            let record = transcript[start..].split(['\r', '\n']).next()?;
            let (before, _) = record.split_once(")%")?;
            let (_, number) = before.rsplit_once('(')?;
            number.parse::<u8>().ok().filter(|v| *v <= 100)
        })
}
pub fn run(
    build: &Path,
    exe: &Path,
    settings: &Serial,
    host_data: bool,
    tx: &Sender<Event>,
    rx: &Receiver<Control>,
) -> Result<(), String> {
    settings.validate()?;
    let result: Result<(), String> = (|| {
        let _ = tx.send(Event::Stage("Sending program to PSX".into()));
        let _ = tx.send(Event::Log(
            "Enabling Unirom's resident serial/debug handler...".into(),
        ));
        execute(command(build, settings, None, false)?, Mode::Setup, tx, rx)?;
        if host_data {
            let _ = tx.send(Event::Log(
                "The monitor will also serve PCDrv data from the build directory.".into(),
            ));
        }
        let _ = tx.send(Event::Log(format!(
            "Uploading {} via {}. The PSX must be waiting in the Unirom loader.",
            exe.display(),
            settings.port
        )));
        execute(
            command(build, settings, Some(exe), true)?,
            Mode::Monitor,
            tx,
            rx,
        )?;
        Ok(())
    })();
    match result {
        Err(message) if message.starts_with("Serial session cancelled") => {
            let _ = tx.send(Event::Log(message));
            Ok(())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upload_percent_requires_a_complete_nops_record() {
        assert_eq!(upload_progress("Compiling (45)%"), None);
        assert_eq!(upload_progress("\r Sending chunk 3 of 8 (37"), None);
        assert_eq!(upload_progress("\r Sending chunk 3 of 8 (37)%"), Some(37));
        assert_eq!(
            upload_progress("Sending chunk 3 of 8 (37)%\r Sending chunk 4 of 8 (50"),
            Some(37)
        );
        assert_eq!(
            upload_progress("Sending chunk 8 of 8 (100)%\nSend finished!"),
            Some(100)
        );
        assert_eq!(upload_progress("Sending chunk 3 of 8 (255)%"), None);
    }
    #[test]
    fn cancelled_session_never_launches_a_transport() {
        let (tx, _) = mpsc::channel();
        let (stop, rx) = mpsc::channel();
        stop.send(Control::Stop).unwrap();
        assert!(
            execute(
                Command::new("missing-notpsxserial"),
                Mode::Monitor,
                &tx,
                &rx
            )
            .unwrap_err()
            .contains("before connecting")
        );
    }
    #[cfg(windows)]
    fn fake(script: &str) -> Command {
        let mut command = Command::new("powershell");
        command
            .args(["-NoProfile", "-Command", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        crate::pipeline::quiet(&mut command);
        command
    }
    #[test]
    #[cfg(windows)]
    fn zero_exit_after_transport_error_is_not_success() {
        let (tx, _) = mpsc::channel();
        let (_stop, rx) = mpsc::channel();
        let result = execute(
            fake("[Console]::Error.Write('No response from console'); exit 0"),
            Mode::Monitor,
            &tx,
            &rx,
        );
        assert!(result.unwrap_err().contains("reported a failure"));
    }
    #[test]
    #[cfg(windows)]
    fn monitor_requires_runtime_ready_and_stop_reaps_its_process() {
        let (tx, events) = mpsc::channel();
        let (stop, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            execute(
                fake(
                    "[Console]::Clear(); [Console]::WriteLine('Send finished!'); [Console]::WriteLine('2-way Serial monitor active'); [Console]::WriteLine('EPOK: runtime ready'); $r=''; while($r.Length -lt 4) { $r += [Console]::ReadKey($true).KeyChar }; if($r -ne 'PING'){exit 2}; [Console]::Write('PONG'); Start-Sleep -Seconds 60",
                ),
                Mode::Monitor,
                &tx,
                &rx,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut ready = false;
        while Instant::now() < deadline {
            if matches!(
                events.recv_timeout(Duration::from_millis(100)),
                Ok(Event::SerialConnected)
            ) {
                ready = true;
                break;
            }
        }
        stop.send(Control::Stop).unwrap();
        let result = worker.join().unwrap();
        assert!(ready, "The console startup message was not detected");
        assert!(result.unwrap_err().contains("cancelled"));
    }
    #[test]
    fn command_passes_paths_as_arguments_and_monitor_owns_build_directory() {
        let s = Serial {
            executable: "custom-nops".into(),
            port: "COM12".into(),
            fast: true,
            device_id: None,
        };
        let build = Path::new("project with spaces/build");
        let exe = build.join("game with spaces.ps-exe");
        let command = command(build, &s, Some(&exe), true).unwrap();
        let args: Vec<_> = command
            .get_args()
            .skip(usize::from(!cfg!(windows))) // Mono's first argument is the managed executable.
            .map(|v| v.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "/exe",
                "game with spaces.ps-exe",
                "/dest",
                "COM12",
                "/fast",
                "/m"
            ]
        );
        assert_eq!(command.get_current_dir(), Some(build));
    }
    #[test]
    fn setup_uses_owned_component_and_monitor_upload_keeps_the_port() {
        let settings = Serial {
            executable: "outside/Epok/nops.exe".into(),
            port: "COM12".into(),
            ..Default::default()
        };
        let build = Path::new("project with spaces/build");
        let exe = build.join("menu.ps-exe");
        let cmd = command(build, &settings, Some(&exe), true).unwrap();
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        #[cfg(windows)]
        {
            assert_eq!(
                cmd.get_program(),
                crate::serial_support::managed_directory().join("nops.exe")
            );
            assert_eq!(args, ["/exe", "menu.ps-exe", "/dest", "COM12", "/m"]);
        }
        assert!(args.iter().any(|a| a == "/m"));
        let setup = command(build, &settings, None, false).unwrap();
        assert!(setup.get_args().any(|a| a == "/debug"));
        assert!(!setup.get_args().any(|a| a == "/m"));
    }
    #[test]
    #[cfg(windows)]
    fn setup_requires_acknowledgement_and_successful_exit() {
        for (script, success) in [
            (
                "[Console]::Write('Response: Tr'); Start-Sleep -Milliseconds 50; [Console]::WriteLine('ue'); exit 0",
                true,
            ),
            ("[Console]::Write('Response: True Bye!'); exit 0", true),
            (
                "[Console]::Write('Response: Tr'); Start-Sleep -Milliseconds 50; [Console]::Write('ue B'); Start-Sleep -Milliseconds 50; [Console]::Write('ye!'); exit 0",
                true,
            ),
            ("[Console]::Write('Response: False Bye!'); exit 0", false),
            ("[Console]::Write('Response: True Bye!'); exit 1", false),
            (
                "[Console]::Write('Response: True'); Start-Sleep -Milliseconds 50; [Console]::Write('ish Bye!'); exit 0",
                false,
            ),
            (
                "[Console]::WriteLine('Response: True'); [Console]::Write('Response: False Bye!'); exit 0",
                false,
            ),
            ("[Console]::WriteLine('Usage help'); exit 0", false),
            ("[Console]::WriteLine('Response: True'); exit 1", false),
        ] {
            let (tx, events) = mpsc::channel();
            let (_stop, rx) = mpsc::channel();
            assert_eq!(
                execute(fake(script), Mode::Setup, &tx, &rx).is_ok(),
                success,
                "{script}"
            );
            assert!(
                !events
                    .try_iter()
                    .any(|e| matches!(e, Event::SerialConnected))
            );
        }
    }

    #[test]
    fn control_replies_span_reads_and_echo_is_not_acknowledgement() {
        let mut pause = Pending::new(Request::Pause);
        assert_eq!(pause.observe("HALT"), None);
        assert_eq!(pause.observe("HL"), None);
        assert_eq!(pause.observe("TD"), Some(true));
        let mut resume = Pending::new(Request::Resume);
        assert_eq!(resume.observe("CONT"), None);
        assert_eq!(resume.observe("OK"), None);
        assert_eq!(resume.observe("AY"), Some(true));
        let mut reset = Pending::new(Request::Reset);
        assert_eq!(reset.observe("REST"), None);
        assert_eq!(reset.observe("UN"), None);
        assert_eq!(reset.observe("SP"), Some(false));
        // A new request cannot inherit an earlier OKAY.
        assert_eq!(Pending::new(Request::Reset).observe("REST"), None);
        let mut probe = Pending::new(Request::Probe);
        assert_eq!(
            probe.observe("OKAY"),
            None,
            "A late acknowledgement cannot resynchronize the stream"
        );
        assert_eq!(probe.observe("PONG"), Some(true));
    }

    #[test]
    #[cfg(windows)]
    fn timed_out_reset_resynchronizes_before_accepting_another_control() {
        let (tx, events) = mpsc::channel();
        let (controls, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            execute(
                fake(
                    r#"
            function Read-Command { $r=''; while($r.Length -lt 4) { $r += [Console]::ReadKey($true).KeyChar }; return $r }
            [Console]::WriteLine('Send finished!')
            [Console]::WriteLine('2-way Serial monitor active')
            [Console]::WriteLine('EPOK: runtime ready')
            if((Read-Command) -ne 'PING'){exit 2}; [Console]::Write('PONG')
            if((Read-Command) -ne 'REST'){exit 2}
            Start-Sleep -Seconds 6
            [Console]::Write('OKAY')
            if((Read-Command) -ne 'PING'){exit 2}; [Console]::Write('PONG')
            if((Read-Command) -ne 'CONT'){exit 2}; [Console]::Write('OKAY')
            Start-Sleep -Seconds 60
        "#,
                ),
                Mode::Monitor,
                &tx,
                &rx,
            )
        });
        let mut connected = 0;
        let mut resynchronized = false;
        let mut resumed = false;
        let deadline = Instant::now() + Duration::from_secs(16);
        while Instant::now() < deadline {
            match events.recv_timeout(Duration::from_millis(100)) {
                Ok(Event::SerialConnected) => {
                    connected += 1;
                    controls.send(Control::Reset).unwrap();
                }
                Ok(Event::SerialCommandPending(false)) if connected == 1 && !resynchronized => {
                    resynchronized = true;
                    controls.send(Control::Resume).unwrap();
                }
                Ok(Event::Paused(false)) => {
                    resumed = true;
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                _ => {}
            }
        }
        let _ = controls.send(Control::Stop);
        assert!(worker.join().unwrap().unwrap_err().contains("cancelled"));
        assert_eq!(
            connected, 1,
            "Resynchronization must not restart the UI session"
        );
        assert!(resynchronized && resumed);
    }

    #[test]
    #[cfg(windows)]
    fn monitor_controls_share_one_process_and_reset_requires_reply() {
        let (tx, events) = mpsc::channel();
        let (controls, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            execute(
                fake(
                    r#"
            [Console]::WriteLine('Send finished!')
            [Console]::WriteLine('2-way Serial monitor active')
            [Console]::WriteLine('EPOK: runtime ready')
            foreach($pair in @(@('PING','PONG'),@('HALT','HLTD'),@('CONT','OKAY'),@('REST','OKAY'))) {
                $request=''
                while($request.Length -lt 4) { $request += [Console]::ReadKey($true).KeyChar }
                if($request -ne $pair[0]) { [Console]::WriteLine('Error: wrong command'); exit 2 }
                [Console]::Write($request)
                Start-Sleep -Milliseconds 100
                [Console]::Write($pair[1].Substring(0,2))
                Start-Sleep -Milliseconds 50
                [Console]::Write($pair[1].Substring(2))
            }
            Start-Sleep -Seconds 60
        "#,
                ),
                Mode::Monitor,
                &tx,
                &rx,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut states = Vec::new();
        let mut connected = false;
        let mut in_flight = false;
        while Instant::now() < deadline {
            match events.recv_timeout(Duration::from_millis(100)) {
                Ok(Event::SerialConnected) => {
                    connected = true;
                    controls.send(Control::Pause).unwrap();
                }
                Ok(Event::SerialCommandPending(p)) => {
                    in_flight |= p;
                }
                Ok(Event::Paused(p)) => {
                    states.push(p);
                    controls
                        .send(if p { Control::Resume } else { Control::Reset })
                        .unwrap();
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                _ => {}
            }
        }
        let _ = controls.send(Control::Stop);
        assert!(
            worker.join().unwrap().is_ok(),
            "Reset must receive OKAY before ending the session"
        );
        assert!(connected && in_flight);
        assert_eq!(states, [true, false]);
    }

    #[test]
    #[cfg(windows)]
    fn upload_and_runtime_markers_without_a_resident_reply_are_not_connected() {
        let (tx, events) = mpsc::channel();
        let (_controls, rx) = mpsc::channel();
        let result = execute(
            fake(
                "[Console]::WriteLine('Send finished!'); [Console]::WriteLine('2-way Serial monitor active'); [Console]::WriteLine('EPOK: runtime ready'); Start-Sleep -Seconds 60",
            ),
            Mode::Monitor,
            &tx,
            &rx,
        );
        assert!(result.unwrap_err().contains("did not answer"));
        assert!(
            !events
                .try_iter()
                .any(|e| matches!(e, Event::SerialConnected))
        );
    }
}
