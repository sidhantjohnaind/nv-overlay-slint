use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub const IPC_SOCKET_PATH: &str = "/tmp/nv_overlay.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommand {
    Toggle,
    Cycle,
    Settings,
    Quit,
    Ping,
}

impl IpcCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            IpcCommand::Toggle => "TOGGLE",
            IpcCommand::Cycle => "CYCLE",
            IpcCommand::Settings => "SETTINGS",
            IpcCommand::Quit => "QUIT",
            IpcCommand::Ping => "PING",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "TOGGLE" => Some(IpcCommand::Toggle),
            "CYCLE" => Some(IpcCommand::Cycle),
            "SETTINGS" => Some(IpcCommand::Settings),
            "QUIT" => Some(IpcCommand::Quit),
            "PING" => Some(IpcCommand::Ping),
            _ => None,
        }
    }
}

/// Sends an IPC command to the running NV-Overlay instance on Ubuntu/Linux.
pub fn send_ipc_command(cmd: IpcCommand) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(IPC_SOCKET_PATH)?;
    stream.write_all(cmd.as_str().as_bytes())?;
    stream.write_all(b"\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response.trim().to_string())
}

/// Starts the background Unix domain socket server to listen for Ubuntu GNOME shortcut triggers.
pub fn start_ipc_server<F>(callback: F) -> Option<Arc<AtomicBool>>
where
    F: Fn(IpcCommand) -> String + Send + Sync + 'static,
{
    let path = Path::new(IPC_SOCKET_PATH);
    if path.exists() {
        // Try connecting to see if another instance is already running
        if let Ok(mut stream) = UnixStream::connect(path) {
            let _ = stream.write_all(b"PING\n");
            let mut res = String::new();
            if stream.read_to_string(&mut res).is_ok() && res.contains("PONG") {
                log::info!("Another NV-Overlay instance is already running on socket.");
                return None;
            }
        }
        let _ = std::fs::remove_file(path);
    }

    let listener = match UnixListener::bind(path) {
        Ok(l) => l,
        Err(e) => {
            log::warn!("Could not bind Linux IPC socket at {}: {}", IPC_SOCKET_PATH, e);
            return None;
        }
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);
    let cb = Arc::new(callback);

    thread::spawn(move || {
        log::info!("Linux IPC server listening on {}", IPC_SOCKET_PATH);
        for stream in listener.incoming() {
            if !running_clone.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 128];
                if let Ok(n) = stream.read(&mut buf) {
                    let req = String::from_utf8_lossy(&buf[..n]);
                    if let Some(cmd) = IpcCommand::parse(&req) {
                        let resp = if cmd == IpcCommand::Ping {
                            "PONG\n".to_string()
                        } else {
                            format!("OK: {}\n", cb(cmd))
                        };
                        let _ = stream.write_all(resp.as_bytes());
                    }
                }
            }
        }
        let _ = std::fs::remove_file(IPC_SOCKET_PATH);
    });

    Some(running)
}
