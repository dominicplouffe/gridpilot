//! `gridpilot` — run the window manager, or talk to a running one.

mod ipc;
mod settings;
mod simulate;

use gridpilot_core::config::{Config, REFERENCE_CONFIG};
use gridpilot_core::features::builtin;
use gridpilot_core::testing::MockPlatform;
use gridpilot_core::Engine;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "\
GridPilot — Divvy-style grid placement and Omarchy-style tiling for macOS

USAGE:
    gridpilot run [--config PATH]     start the window manager (menu bar app)
    gridpilot init                    write the reference config to ~/.config/gridpilot/
    gridpilot check [PATH]            validate a config and list binding conflicts
    gridpilot simulate                try the engine on a fake desktop (any OS)
    gridpilot keymap [--markdown]     print the key bindings a config produces
    gridpilot settings [--no-open]    edit settings in the browser
    gridpilot capslock status|off     is Caps Lock acting as mod? / give it back (macOS)
    gridpilot <command> [args...]     send a command to the running instance

COMMANDS (see `gridpilot commands` for all):
    grid show | grid shortcut \"Left half\" | grid apply 0,0 3x4 / 6x4
    focus left | swap right | workspace 3 | move-to-workspace 3 [--silent]
    float toggle | fullscreen | pop | layout next | scratchpad toggle
    keys | reload | theme next | windows | workspaces
";

pub(crate) fn config_path(args: &[String]) -> PathBuf {
    args.iter()
        .position(|a| a == "--config")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(Config::default_path)
}

fn load(path: &Path) -> Result<Config, String> {
    if path.exists() {
        Config::load(path)
    } else {
        Ok(Config::default())
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("-h" | "--help" | "help") => {
            print!("{HELP}");
            Ok(())
        }
        Some("-V" | "--version") => {
            println!("gridpilot {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("run") => run(&args),
        Some("init") => init(),
        Some("check") => check(args.get(1).map(PathBuf::from).unwrap_or_else(Config::default_path)),
        Some("simulate") => load(&config_path(&args)).and_then(simulate::run),
        Some("keymap") => keymap(&args),
        Some("settings") => settings::run(&args),
        Some("capslock") => match args.get(1).map(String::as_str) {
            Some("off") => capslock_off(),
            Some("status") | None => capslock_status(),
            Some(other) => Err(format!("unknown `capslock {other}` (expected status or off)")),
        },
        Some(_) => ipc::send(&quote(&args)).map(|out| {
            if !out.is_empty() {
                println!("{out}");
            }
        }),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gridpilot: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Re-quote CLI words so `gridpilot grid shortcut "Left half"` survives.
fn quote(args: &[String]) -> String {
    args.iter()
        .map(|a| if a.contains(char::is_whitespace) || a.is_empty() { format!("\"{a}\"") } else { a.clone() })
        .collect::<Vec<_>>()
        .join(" ")
}

fn init() -> Result<(), String> {
    let path = Config::default_path();
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, REFERENCE_CONFIG).map_err(|e| e.to_string())?;
    println!("wrote {}", path.display());
    Ok(())
}

fn check(path: PathBuf) -> Result<(), String> {
    let config = load(&path)?;
    let e = Engine::new(Box::new(MockPlatform::with_displays(1)), config, builtin());
    println!("{}: ok", path.display());
    println!(
        "{} features, {} commands, {} key bindings",
        e.installed_features().len(),
        e.commands().len(),
        e.keymap().len()
    );
    for w in &e.warnings {
        println!("warning: {w}");
    }
    if e.warnings.is_empty() {
        Ok(())
    } else {
        Err(format!("{} warning(s)", e.warnings.len()))
    }
}

fn keymap(args: &[String]) -> Result<(), String> {
    let config = load(&config_path(args))?;
    let e = Engine::new(Box::new(MockPlatform::with_displays(1)), config, builtin());
    let mut keys = e.key_help();
    let order = |f: &str| e.installed_features().iter().position(|(id, _)| *id == f).unwrap_or(usize::MAX);
    keys.sort_by(|a, b| order(&a.feature).cmp(&order(&b.feature)).then(a.command.cmp(&b.command)));
    let markdown = args.iter().any(|a| a == "--markdown");
    let mut last = String::new();
    for k in keys {
        if markdown && k.feature != last {
            let summary = e.installed_features().iter().find(|(id, _)| *id == k.feature).map_or("", |(_, s)| *s);
            println!("\n### `{}` — {summary}\n\n| Keys | Action | Command |\n|---|---|---|", k.feature);
            last = k.feature.clone();
        }
        if markdown {
            println!("| `{}` | {} | `{}` |", k.chord, k.description, k.command);
        } else {
            println!("{:<28} {:<36} {}", k.chord, k.description, k.command);
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn capslock_off() -> Result<(), String> {
    gridpilot_macos::capslock::force_off()?;
    println!("Caps Lock is back to normal.");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn capslock_off() -> Result<(), String> {
    Err("Caps Lock remapping only exists on macOS".into())
}

#[cfg(target_os = "macos")]
fn capslock_status() -> Result<(), String> {
    let config = load(&Config::default_path())?;
    println!("capslock_mod in config: {}", config.general.capslock_mod);
    let remapped = gridpilot_macos::capslock::is_remapped()?;
    println!(
        "Caps Lock remapped right now: {}",
        if remapped { "yes (holding it sends GridPilot's mod)" } else { "no (it types capitals)" }
    );
    match ipc::send("warnings") {
        Ok(w) if w.trim().is_empty() => println!("GridPilot is running, no warnings."),
        Ok(w) => println!("GridPilot is running. Warnings:\n{w}"),
        Err(_) => println!("GridPilot is not running (Caps Lock is only remapped while it runs)."),
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn capslock_status() -> Result<(), String> {
    Err("Caps Lock remapping only exists on macOS".into())
}

#[cfg(target_os = "macos")]
fn run(args: &[String]) -> Result<(), String> {
    let path = config_path(args);
    let config = load(&path)?;
    gridpilot_macos::run(config, Some(path), |poster| match ipc::serve(move |line, reply| poster.post(line, reply)) {
        Ok(p) => eprintln!("gridpilot: listening on {}", p.display()),
        Err(e) => eprintln!("gridpilot: IPC disabled: {e}"),
    })
}

#[cfg(not(target_os = "macos"))]
fn run(_args: &[String]) -> Result<(), String> {
    Err("no window-system backend for this OS yet; try `gridpilot simulate`".into())
}
