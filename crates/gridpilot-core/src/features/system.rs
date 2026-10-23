//! Housekeeping: reload, the key cheat sheet, themes, gaps, exec and
//! introspection commands used by the CLI.

use crate::command::{Args, CmdResult};
use crate::config::config_dir;
use crate::engine::Engine;
use crate::feature::{Feature, Registrar};
use crate::platform::UiRequest;
use crate::theme::Theme;

pub struct SystemFeature;

#[derive(Default)]
struct SavedGaps(Option<(f64, f64)>);

impl Feature for SystemFeature {
    fn id(&self) -> &'static str {
        "system"
    }

    fn summary(&self) -> &'static str {
        "Reload, key cheat sheet, themes, gaps, exec and introspection"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("reload", "reload", "Re-read the config file", |e, _| {
            e.reload()?;
            Ok(Some("config reloaded".into()))
        });
        r.command("keys", "keys", "Show every key binding", keys);
        r.command("exec", "exec <shell command>", "Run a shell command", |e, a| {
            let cmd = a.rest(0);
            if cmd.is_empty() {
                return Err("exec needs a command".into());
            }
            e.platform().spawn(&cmd)?;
            Ok(None)
        });
        r.command("theme", "theme <name|next|list>", "Switch the colour theme", theme);
        r.command("gaps", "gaps toggle", "Toggle all tiling gaps off and back on", |e, _| {
            let saved = e.ext::<SavedGaps>().0.take();
            match saved {
                Some((inner, outer)) => {
                    e.config.tiling.gaps_inner = inner;
                    e.config.tiling.gaps_outer = outer;
                }
                None => {
                    let g = (e.config.tiling.gaps_inner, e.config.tiling.gaps_outer);
                    e.ext::<SavedGaps>().0 = Some(g);
                    e.config.tiling.gaps_inner = 0.0;
                    e.config.tiling.gaps_outer = 0.0;
                }
            }
            Ok(None)
        });
        r.command("windows", "windows", "List managed windows", |e, _| {
            let lines: Vec<String> = e
                .world
                .windows
                .values()
                .map(|w| {
                    let mut flags = Vec::new();
                    if w.floating {
                        flags.push("floating");
                    }
                    if w.pinned {
                        flags.push("pinned");
                    }
                    if w.fullscreen.is_some() {
                        flags.push("fullscreen");
                    }
                    let focus = if e.world.focused == Some(w.info.id) { "*" } else { " " };
                    format!(
                        "{focus} {:<8} ws {:<10} {:<28} {} [{}]",
                        w.info.id,
                        w.workspace,
                        w.info.app,
                        w.info.title,
                        flags.join(",")
                    )
                })
                .collect();
            Ok(Some(lines.join("\n")))
        });
        r.command("commands", "commands", "List every command", |e, _| {
            let lines: Vec<String> = e
                .commands()
                .iter()
                .map(|c| format!("{:<26} [{}] {}\n    {}", c.name, c.feature, c.summary, c.usage))
                .collect();
            Ok(Some(lines.join("\n")))
        });
        r.command("features", "features", "List installed features", |e, _| {
            let lines: Vec<String> = e.installed_features().iter().map(|(id, s)| format!("{id:<12} {s}")).collect();
            Ok(Some(lines.join("\n")))
        });
        r.command("warnings", "warnings", "Show recent warnings (bad bindings, platform errors)", |e, _| {
            Ok(Some(e.warnings.join("\n")))
        });
        r.command("hotkeys", "hotkeys on|off", "Enable or pause GridPilot's global hotkeys", |e, a| {
            let on = match a.require(0, "on or off")? {
                "on" => true,
                "off" => false,
                other => return Err(format!("hotkeys: expected on/off, got `{other}`")),
            };
            e.set_hotkeys_enabled(on)?;
            Ok(None)
        });
        r.command("settings", "settings", "Open the settings page in the browser", |e, _| {
            let exe = std::env::current_exe().map_err(|err| format!("can't find the gridpilot binary: {err}"))?;
            e.platform().spawn(&format!("'{}' settings", exe.display().to_string().replace('\'', "'\\''")))?;
            Ok(None)
        });
        r.command("quit", "quit", "Stop GridPilot", |e, _| {
            e.quit_requested = true;
            Ok(None)
        });
        r.command("version", "version", "Print the version", |_, _| {
            Ok(Some(format!("gridpilot {}", env!("CARGO_PKG_VERSION"))))
        });

        r.bind("mod+k", "keys", "Show key bindings");
        r.bind("mod+shift+r", "reload", "Reload config");
        r.bind("mod+shift+backspace", "gaps toggle", "Toggle gaps");
        r.bind("mod+shift+space", "theme next", "Next theme");
        r.bind("mod+return", "exec open -na Terminal", "Terminal");
        r.bind("mod+shift+b", "exec open -a Safari", "Browser");
        r.bind("mod+e", "exec open ~", "File manager");
    }
}

fn keys(e: &mut Engine, _: &Args) -> CmdResult {
    let help = e.key_help();
    let text: Vec<String> =
        help.iter().map(|k| format!("{:<22} {:<36} {}", k.display, k.description, k.command)).collect();
    e.present(UiRequest::ShowKeys(help));
    Ok(Some(text.join("\n")))
}

/// A built-in theme, or `~/.config/gridpilot/themes/<name>.toml`.
pub fn load_theme(name: &str) -> Result<Theme, String> {
    if let Some(t) = Theme::builtin(name) {
        return Ok(t);
    }
    let path = config_dir().join("themes").join(format!("{name}.toml"));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("theme `{name}`: {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("theme `{name}`: {e}"))
}

/// Built-in themes followed by the user's own.
pub fn theme_names() -> Vec<String> {
    let mut names: Vec<String> = Theme::builtins().into_iter().map(|t| t.name).collect();
    if let Ok(dir) = std::fs::read_dir(config_dir().join("themes")) {
        for entry in dir.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|x| x == "toml") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if !names.iter().any(|n| n == stem) {
                        names.push(stem.to_string());
                    }
                }
            }
        }
    }
    names
}

fn theme(e: &mut Engine, a: &Args) -> CmdResult {
    let names = theme_names();
    let want = a.require(0, "a theme name, next or list")?;
    let name = match want {
        "list" => return Ok(Some(names.join("\n"))),
        "next" => {
            let i = names.iter().position(|n| *n == e.theme.name).map_or(0, |i| (i + 1) % names.len());
            names[i].clone()
        }
        n => n.to_string(),
    };
    let t = load_theme(&name)?;
    e.theme = t.clone();
    e.config.theme.name = name.clone();
    e.present(UiRequest::ThemeChanged(t));
    e.present(UiRequest::Notify { title: "Theme".into(), body: name.clone() });
    Ok(Some(name))
}
