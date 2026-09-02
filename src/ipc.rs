use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::path::Path;

#[cfg(windows)]
use std::net::{TcpListener, TcpStream};

#[cfg(unix)]
pub const IPC_SOCKET_PATH: &str = "/tmp/nv_overlay.sock";

#[cfg(windows)]
pub const IPC_TCP_ADDR: &str = "127.0.0.1:48899";

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

/// Sends an IPC command to the running NV-Overlay instance.
pub fn send_ipc_command(cmd: IpcCommand) -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(IPC_SOCKET_PATH)?;
        stream.write_all(cmd.as_str().as_bytes())?;
        stream.write_all(b"\n")?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(response.trim().to_string())
    }

    #[cfg(windows)]
    {
        let mut stream = TcpStream::connect(IPC_TCP_ADDR)?;
        stream.write_all(cmd.as_str().as_bytes())?;
        stream.write_all(b"\n")?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(response.trim().to_string())
    }
}

/// Starts the background IPC server to listen for shortcut triggers.
pub fn start_ipc_server<F>(callback: F) -> Option<Arc<AtomicBool>>
where
    F: Fn(IpcCommand) -> String + Send + Sync + 'static,
{
    #[cfg(unix)]
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
                log::warn!("Could not bind Unix IPC socket at {}: {}", IPC_SOCKET_PATH, e);
                return None;
            }
        };

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let cb = Arc::new(callback);

        thread::spawn(move || {
            log::info!("Unix IPC server listening on {}", IPC_SOCKET_PATH);
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

    #[cfg(windows)]
    {
        let listener = match TcpListener::bind(IPC_TCP_ADDR) {
            Ok(l) => l,
            Err(e) => {
                log::warn!("Could not bind Windows TCP IPC server at {}: {}", IPC_TCP_ADDR, e);
                return None;
            }
        };

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let cb = Arc::new(callback);

        thread::spawn(move || {
            log::info!("Windows TCP IPC server listening on {}", IPC_TCP_ADDR);
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
        });

        Some(running)
    }
}
