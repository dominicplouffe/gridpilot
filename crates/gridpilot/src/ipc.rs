//! A line-based Unix socket, like `hyprctl` or `aerospace`:
//! the client writes one command line, the daemon answers
//! `ok\n<output>` or `err\n<message>` and closes.

// Only the macOS daemon serves today; the client half is used everywhere.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use gridpilot_core::command::CmdResult;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub type Reply = Box<dyn FnOnce(CmdResult) + Send>;

pub fn socket_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRIDPILOT_SOCKET") {
        return PathBuf::from(p);
    }
    let dir = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    dir.join(format!("gridpilot-{user}.sock"))
}

/// Accept connections on a background thread. `post` must hand the command
/// to the thread that owns the engine and eventually call the reply.
pub fn serve(post: impl Fn(String, Reply) + Send + 'static) -> std::io::Result<PathBuf> {
    let path = socket_path();
    if UnixStream::connect(&path).is_ok() {
        return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "GridPilot is already running"));
    }
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    std::thread::Builder::new().name("gridpilot-ipc".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(match stream.try_clone() {
                Ok(s) => s,
                Err(_) => continue,
            });
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            let (tx, rx) = mpsc::channel();
            post(
                line.trim().to_string(),
                Box::new(move |r| {
                    let _ = tx.send(r);
                }),
            );
            let answer = match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(Ok(out)) => format!("ok\n{}", out.unwrap_or_default()),
                Ok(Err(e)) => format!("err\n{e}"),
                Err(_) => "err\nGridPilot did not answer in time".to_string(),
            };
            let mut stream = stream;
            let _ = stream.write_all(answer.as_bytes());
        }
    })?;
    Ok(path)
}

/// Send one command line to the running daemon.
pub fn send(line: &str) -> Result<String, String> {
    let path = socket_path();
    let mut s = UnixStream::connect(&path)
        .map_err(|e| format!("GridPilot is not running ({}: {e}). Start it with `gridpilot run`.", path.display()))?;
    s.write_all(format!("{line}\n").as_bytes()).map_err(|e| e.to_string())?;
    let mut answer = String::new();
    s.read_to_string(&mut answer).map_err(|e| e.to_string())?;
    match answer.split_once('\n') {
        Some(("ok", out)) => Ok(out.to_string()),
        Some(("err", msg)) => Err(msg.to_string()),
        _ => Err(format!("unexpected reply: {answer}")),
    }
}
