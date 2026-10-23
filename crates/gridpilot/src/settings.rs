//! `gridpilot settings`: a settings page in the browser.
//!
//! A tiny HTTP server on 127.0.0.1 serves one page (`settings.html`) and a
//! JSON API. Saving writes `gridpilot.toml` (keeping the previous file as
//! `gridpilot.toml.bak`) and asks a running GridPilot to reload.
//!
//! Only this machine can connect, and every API call must carry the random
//! token from the URL the command opened, so other web pages can't read or
//! change the config. The `Host` header is checked too, against DNS rebinding.

use crate::ipc;
use gridpilot_core::config::Config;
use gridpilot_core::features::builtin;
use gridpilot_core::features::system::{load_theme, theme_names};
use gridpilot_core::testing::MockPlatform;
use gridpilot_core::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const PAGE: &str = include_str!("settings.html");
const MAX_BODY: usize = 2 * 1024 * 1024;

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    fn query_param(&self, name: &str) -> Option<&str> {
        self.query.split('&').filter_map(|kv| kv.split_once('=')).find(|(k, _)| *k == name).map(|(_, v)| v)
    }
}

pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl Response {
    fn json(status: u16, v: Value) -> Response {
        Response { status, content_type: "application/json", body: v.to_string().into_bytes() }
    }

    fn error(status: u16, message: impl Into<String>) -> Response {
        Response::json(status, json!({ "error": message.into() }))
    }
}

/// Talks to the running GridPilot; swapped out in tests.
pub type Daemon = Box<dyn Fn(&str) -> Result<String, String>>;

pub struct Ctx {
    pub config_path: PathBuf,
    pub token: String,
    /// `127.0.0.1:<port>`; requests with any other Host are refused.
    pub host: String,
    pub daemon: Daemon,
    pub quit: bool,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let config_path = crate::config_path(args);
    let port: u16 = match args.iter().position(|a| a == "--port") {
        Some(i) => args.get(i + 1).and_then(|p| p.parse().ok()).ok_or("--port needs a number")?,
        None => 0,
    };
    let listener =
        TcpListener::bind(("127.0.0.1", port)).map_err(|e| format!("can't listen on 127.0.0.1:{port}: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let mut ctx = Ctx {
        config_path,
        token: random_token()?,
        host: format!("127.0.0.1:{port}"),
        daemon: Box::new(ipc::send),
        quit: false,
    };
    let url = format!("http://{}/?t={}", ctx.host, ctx.token);
    println!("GridPilot settings: {url}");
    println!("Editing {}. Press Ctrl-C or click Done in the page to stop.", ctx.config_path.display());
    if !args.iter().any(|a| a == "--no-open") {
        open_browser(&url);
    }
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        if let Err(e) = serve_one(stream, &mut ctx) {
            eprintln!("gridpilot settings: {e}");
        }
        if ctx.quit {
            break;
        }
    }
    let _ = (ctx.daemon)("hotkeys on");
    Ok(())
}

fn open_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    if std::process::Command::new(opener).arg(url).spawn().is_err() {
        println!("Open the address above in your browser.");
    }
}

fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| format!("no randomness for the session token: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn serve_one(stream: TcpStream, ctx: &mut Ctx) -> Result<(), String> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let req = read_request(&mut reader)?;
    let resp = handle(&req, ctx);
    let mut stream = stream;
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\
         X-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\n\
         Content-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'\r\n\
         Connection: close\r\n\r\n",
        resp.status,
        reason(resp.status),
        resp.content_type,
        resp.body.len()
    );
    stream.write_all(head.as_bytes()).and_then(|_| stream.write_all(&resp.body)).map_err(|e| e.to_string())
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "Error",
    }
}

fn read_request(r: &mut impl BufRead) -> Result<Request, String> {
    let mut line = String::new();
    r.read_line(&mut line).map_err(|e| e.to_string())?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or("empty request")?.to_string();
    let target = parts.next().ok_or("no request target")?;
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut headers = Vec::new();
    loop {
        let mut h = String::new();
        r.read_line(&mut h).map_err(|e| e.to_string())?;
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
        if headers.len() > 100 {
            return Err("too many headers".into());
        }
    }
    let len: usize = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    if len > MAX_BODY {
        return Err("request body too large".into());
    }
    let mut body = vec![0; len];
    r.read_exact(&mut body).map_err(|e| e.to_string())?;
    Ok(Request { method, path: path.to_string(), query: query.to_string(), headers, body })
}

/// Route a request. Pure apart from the config file and `ctx.daemon`.
pub fn handle(req: &Request, ctx: &mut Ctx) -> Response {
    if req.header("host") != Some(ctx.host.as_str()) {
        return Response::error(403, "wrong Host header");
    }
    if req.method == "GET" && req.path == "/" {
        if req.query_param("t") != Some(ctx.token.as_str()) {
            return Response {
                status: 403,
                content_type: "text/plain; charset=utf-8",
                body: b"Open this page with `gridpilot settings`.\n".to_vec(),
            };
        }
        let page = PAGE.replace("__GRIDPILOT_TOKEN__", &ctx.token);
        return Response { status: 200, content_type: "text/html; charset=utf-8", body: page.into_bytes() };
    }
    if !req.path.starts_with("/api/") {
        return Response::error(404, "not found");
    }
    if req.header("x-gridpilot-token") != Some(ctx.token.as_str()) {
        return Response::error(403, "missing or wrong session token");
    }
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/state") => Response::json(200, state(ctx)),
        ("POST", "/api/validate") => match body_config(req) {
            Ok(config) => Response::json(200, validate(config)),
            Err(e) => Response::error(400, e),
        },
        ("POST", "/api/save") => match body_config(req).and_then(|c| save(ctx, c)) {
            Ok(v) => Response::json(200, v),
            Err(e) => Response::error(400, e),
        },
        ("POST", "/api/hotkeys") => {
            #[derive(Deserialize)]
            struct Body {
                on: bool,
            }
            match serde_json::from_slice::<Body>(&req.body) {
                Ok(b) => {
                    let r = (ctx.daemon)(if b.on { "hotkeys on" } else { "hotkeys off" });
                    Response::json(200, json!({ "ok": r.is_ok() }))
                }
                Err(e) => Response::error(400, e.to_string()),
            }
        }
        ("POST", "/api/quit") => {
            ctx.quit = true;
            Response::json(200, json!({ "ok": true }))
        }
        _ => Response::error(404, "not found"),
    }
}

fn body_config(req: &Request) -> Result<Config, String> {
    #[derive(Deserialize)]
    struct Body {
        config: Config,
    }
    serde_json::from_slice::<Body>(&req.body).map(|b| b.config).map_err(|e| format!("invalid settings: {e}"))
}

fn engine_for(config: Config) -> Engine {
    Engine::new(Box::new(MockPlatform::with_displays(1)), config, builtin())
}

fn validate(config: Config) -> Value {
    let e = engine_for(config);
    json!({ "warnings": e.warnings, "keymap": e.key_help() })
}

fn state(ctx: &Ctx) -> Value {
    let exists = ctx.config_path.exists();
    let (config, error) = if exists {
        match Config::load(&ctx.config_path) {
            Ok(c) => (c, None),
            Err(e) => (Config::default(), Some(e)),
        }
    } else {
        (Config::default(), None)
    };
    // The catalog shows every feature's bindings, even ones switched off,
    // so they can be switched back on from the page.
    let mut everything = config.clone();
    everything.features.disabled.clear();
    everything.general.default_bindings = true;
    let catalog = engine_for(everything);
    let commands: Vec<Value> = catalog
        .commands()
        .iter()
        .map(|c| json!({ "name": c.name, "usage": c.usage, "summary": c.summary, "feature": c.feature }))
        .collect();
    let features: Vec<Value> =
        catalog.installed_features().iter().map(|(id, summary)| json!({ "id": id, "summary": summary })).collect();
    let themes: Vec<Value> = theme_names()
        .iter()
        .filter_map(|n| load_theme(n).ok())
        .map(|t| serde_json::to_value(t).unwrap_or_default())
        .collect();
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "path": ctx.config_path.display().to_string(),
        "exists": exists,
        "error": error,
        "config": config,
        "defaults": catalog.default_bindings(),
        "features": features,
        "commands": commands,
        "themes": themes,
        "layouts": catalog.layouts().names().collect::<Vec<_>>(),
        "validation": validate(config.clone()),
        "running": (ctx.daemon)("version").is_ok(),
    })
}

const HEADER: &str = "\
# GridPilot configuration, written by `gridpilot settings`.
# Every option, explained: https://github.com/dominicplouffe/gridpilot/blob/main/config/gridpilot.toml
# The previous version of this file is kept next to it as gridpilot.toml.bak.

";

fn save(ctx: &Ctx, config: Config) -> Result<Value, String> {
    let text = toml::to_string_pretty(&config).map_err(|e| format!("can't write settings: {e}"))?;
    // Never write something GridPilot itself can't read back.
    let reread = Config::parse(&text).map_err(|e| format!("internal error, settings don't round-trip: {e}"))?;
    let warnings = engine_for(reread).warnings;
    write_with_backup(&ctx.config_path, &format!("{HEADER}{text}"))?;
    let (reloaded, reload_error) = match (ctx.daemon)("reload") {
        Ok(_) => (true, None),
        Err(e) if e.contains("not running") => (false, None),
        Err(e) => (false, Some(e)),
    };
    Ok(json!({
        "saved": ctx.config_path.display().to_string(),
        "reloaded": reloaded,
        "reload_error": reload_error,
        "warnings": warnings,
    }))
}

fn write_with_backup(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    if path.exists() {
        let mut bak = path.as_os_str().to_owned();
        bak.push(".bak");
        std::fs::copy(path, PathBuf::from(bak)).map_err(|e| format!("can't back up {}: {e}", path.display()))?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn ctx(dir: &str) -> (Ctx, Rc<RefCell<Vec<String>>>) {
        let path = std::env::temp_dir()
            .join(format!("gridpilot-settings-test-{dir}-{}", std::process::id()))
            .join("gridpilot.toml");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
        let sent = Rc::new(RefCell::new(Vec::new()));
        let log = sent.clone();
        let ctx = Ctx {
            config_path: path,
            token: "secret".into(),
            host: "127.0.0.1:9".into(),
            daemon: Box::new(move |line| {
                log.borrow_mut().push(line.to_string());
                Ok(String::new())
            }),
            quit: false,
        };
        (ctx, sent)
    }

    fn req(method: &str, target: &str, token: Option<&str>, body: &str) -> Request {
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let mut headers = vec![("Host".to_string(), "127.0.0.1:9".to_string())];
        if let Some(t) = token {
            headers.push(("X-GridPilot-Token".into(), t.into()));
        }
        Request {
            method: method.into(),
            path: path.into(),
            query: query.into(),
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    fn body(r: &Response) -> Value {
        serde_json::from_slice(&r.body).unwrap()
    }

    #[test]
    fn page_and_api_need_the_token_and_host() {
        let (mut c, _) = ctx("auth");
        assert_eq!(handle(&req("GET", "/", None, ""), &mut c).status, 403);
        let page = handle(&req("GET", "/?t=secret", None, ""), &mut c);
        assert_eq!(page.status, 200);
        assert!(String::from_utf8(page.body).unwrap().contains("\"secret\""));
        assert_eq!(handle(&req("GET", "/api/state", None, ""), &mut c).status, 403);
        assert_eq!(handle(&req("GET", "/api/state", Some("nope"), ""), &mut c).status, 403);
        let mut rebinding = req("GET", "/api/state", Some("secret"), "");
        rebinding.headers[0].1 = "evil.example:9".into();
        assert_eq!(handle(&rebinding, &mut c).status, 403);
    }

    #[test]
    fn state_lists_defaults_features_and_themes() {
        let (mut c, _) = ctx("state");
        let r = handle(&req("GET", "/api/state", Some("secret"), ""), &mut c);
        assert_eq!(r.status, 200);
        let v = body(&r);
        assert_eq!(v["exists"], false);
        assert!(v["defaults"].as_array().unwrap().len() > 60);
        assert!(v["features"].as_array().unwrap().iter().any(|f| f["id"] == "grid"));
        assert!(v["themes"].as_array().unwrap().iter().any(|t| t["name"] == "cockpit"));
        assert_eq!(v["config"]["modifiers"]["mod"], "hyper");
        assert!(v["validation"]["warnings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn save_writes_toml_backs_up_and_reloads() {
        let (mut c, sent) = ctx("save");
        let mut config = Config::default();
        config.modifiers.insert("mod".into(), "ctrl+alt".into());
        config.bindings.insert("mod+m".into(), "none".into());
        config.grid.shortcuts[0].keys = Some("grid+h".into());
        let payload = json!({ "config": config }).to_string();
        let r = handle(&req("POST", "/api/save", Some("secret"), &payload), &mut c);
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        assert_eq!(body(&r)["reloaded"], true);
        assert_eq!(sent.borrow().last().map(String::as_str), Some("reload"));
        let written = Config::load(&c.config_path).unwrap();
        assert_eq!(written, config);

        // A second save keeps the first file as .bak.
        handle(&req("POST", "/api/save", Some("secret"), &json!({ "config": Config::default() }).to_string()), &mut c);
        let bak = std::fs::read_to_string(c.config_path.with_extension("toml.bak")).unwrap();
        assert!(bak.contains("ctrl+alt"));
        let _ = std::fs::remove_dir_all(c.config_path.parent().unwrap());
    }

    #[test]
    fn validate_reports_conflicts_and_bad_input() {
        let (mut c, _) = ctx("validate");
        let mut config = Config::default();
        config.bindings.insert("ctrl+alt+cmd+k".into(), "keys".into());
        config.bindings.insert("mod+bogus".into(), "keys".into());
        let r = handle(&req("POST", "/api/validate", Some("secret"), &json!({ "config": config }).to_string()), &mut c);
        let warnings = body(&r)["warnings"].to_string();
        assert!(warnings.contains("bogus"), "{warnings}");
        let bad =
            handle(&req("POST", "/api/validate", Some("secret"), r#"{"config":{"grid":{"size":"0x9"}}}"#), &mut c);
        assert_eq!(bad.status, 400);
    }

    #[test]
    fn hotkeys_and_quit() {
        let (mut c, sent) = ctx("hotkeys");
        handle(&req("POST", "/api/hotkeys", Some("secret"), r#"{"on":false}"#), &mut c);
        assert_eq!(sent.borrow().last().map(String::as_str), Some("hotkeys off"));
        handle(&req("POST", "/api/quit", Some("secret"), ""), &mut c);
        assert!(c.quit);
    }

    #[test]
    fn default_config_round_trips_through_toml() {
        let text = toml::to_string_pretty(&Config::default()).unwrap();
        assert_eq!(Config::parse(&text).unwrap(), Config::default());
    }
}
