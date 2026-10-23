//! Omarchy/Hyprland workspaces, emulated: each display shows one workspace
//! and windows of the others are parked out of sight. Works on macOS without
//! touching Spaces or disabling SIP.

use super::focus::{display_step, display_towards, focus_on_display};
use crate::command::{Args, CmdResult};
use crate::engine::Engine;
use crate::feature::{EngineEvent, Feature, Registrar};
use crate::platform::{DisplayId, UiRequest};
use crate::world::{workspace_order, SCRATCHPAD};

pub struct WorkspacesFeature;

impl Feature for WorkspacesFeature {
    fn id(&self) -> &'static str {
        "workspaces"
    }

    fn summary(&self) -> &'static str {
        "Numbered workspaces per display, moving windows and workspaces between displays"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("workspace", "workspace <name|next|prev|previous>", "Show a workspace", workspace);
        r.command(
            "move-to-workspace",
            "move-to-workspace <name|next|prev> [--silent]",
            "Move the focused window to a workspace, following it unless --silent",
            move_to_workspace,
        );
        r.command(
            "move-to-display",
            "move-to-display <next|prev|left|right|up|down>",
            "Move the focused window to another display",
            move_to_display,
        );
        r.command(
            "move-workspace-to-display",
            "move-workspace-to-display <next|prev|left|right|up|down>",
            "Move the current workspace, with its windows, to another display",
            move_workspace_to_display,
        );
        r.command("workspaces", "workspaces", "List workspaces", |e, _| {
            let names = e.world.workspace_names();
            let lines: Vec<String> = names
                .iter()
                .map(|n| {
                    let ws = &e.world.workspaces[n];
                    let shown = if e.world.active.values().any(|a| a == n) { "*" } else { " " };
                    format!(
                        "{shown} {n:<4} display {}  {:<9} {} windows",
                        ws.display,
                        ws.layout.name(),
                        ws.windows().count()
                    )
                })
                .collect();
            Ok(Some(lines.join("\n")))
        });

        let count = r.config().workspaces.count.clamp(1, 10);
        for n in 1..=count {
            let key = n % 10; // workspace 10 is on the 0 key
            r.bind(&format!("mod+{key}"), &format!("workspace {n}"), &format!("Switch to workspace {n}"));
            r.bind(
                &format!("mod+shift+{key}"),
                &format!("move-to-workspace {n}"),
                &format!("Move window to workspace {n}"),
            );
        }
        r.bind("mod+tab", "workspace next", "Next workspace");
        r.bind("mod+shift+tab", "workspace prev", "Previous workspace");
        r.bind("mod+backslash", "workspace previous", "Former workspace");
        // `mod` already holds three modifiers, so Omarchy's third-layer keys
        // (silent moves, moving whole workspaces) are commands without a
        // default binding: `move-to-workspace N --silent`,
        // `move-workspace-to-display next`.
        r.bind("mod+shift+period", "move-to-display next", "Move window to next display");
        r.bind("mod+shift+comma", "move-to-display prev", "Move window to previous display");

        r.on(|e, ev| {
            if let EngineEvent::WorkspaceShown { name, .. } = ev {
                e.present(UiRequest::Notify { title: "Workspace".into(), body: name.clone() });
            }
        });
    }
}

/// Non-empty or visible workspaces on the current display, in order.
fn on_display(e: &Engine, d: DisplayId) -> Vec<String> {
    let mut v: Vec<String> = e
        .world
        .workspaces
        .values()
        .filter(|ws| ws.display == d && ws.name != SCRATCHPAD)
        .filter(|ws| !ws.is_empty() || e.world.active.get(&d) == Some(&ws.name))
        .map(|ws| ws.name.clone())
        .collect();
    v.sort_by(|a, b| workspace_order(a, b));
    v
}

fn resolve(e: &Engine, which: &str) -> Result<String, String> {
    let d = e.current_display()?;
    let cur = e.world.active_workspace(d).unwrap_or("1").to_string();
    Ok(match which {
        "next" | "prev" => {
            let list = on_display(e, d);
            let i = list.iter().position(|n| *n == cur).unwrap_or(0);
            let n = list.len().max(1);
            let j = if which == "next" { (i + 1) % n } else { (i + n - 1) % n };
            list.get(j).cloned().unwrap_or(cur)
        }
        "previous" => e.world.previous.get(&d).cloned().unwrap_or(cur),
        name if name == SCRATCHPAD => return Err("use `scratchpad toggle` for the scratchpad".into()),
        name => name.to_string(),
    })
}

fn workspace(e: &mut Engine, a: &Args) -> CmdResult {
    let name = resolve(e, a.require(0, "a workspace")?)?;
    e.show_workspace(&name);
    Ok(None)
}

fn move_to_workspace(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let name = resolve(e, a.require(0, "a workspace")?)?;
    e.move_window_to_workspace(id, &name, !a.has_flag("--silent"));
    Ok(None)
}

fn target_display(e: &Engine, a: &Args) -> Result<Option<DisplayId>, String> {
    Ok(match a.require(0, "next/prev or a direction")? {
        "next" => display_step(e, true),
        "prev" => display_step(e, false),
        dir => display_towards(e, dir.parse()?),
    })
}

fn move_to_display(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let Some(d) = target_display(e, a)? else { return Ok(None) };
    let ws = e.world.active_workspace(d).ok_or("display shows no workspace")?.to_string();
    e.move_window_to_workspace(id, &ws, false);
    focus_on_display(e, d);
    e.focus_window(id);
    Ok(None)
}

fn move_workspace_to_display(e: &mut Engine, a: &Args) -> CmdResult {
    let from = e.current_display()?;
    let Some(to) = target_display(e, a)? else { return Ok(None) };
    if to == from {
        return Ok(None);
    }
    let name = e.world.active_workspace(from).ok_or("no workspace")?.to_string();
    let (Some(from_r), Some(to_r)) = (e.world.display(from).map(|d| d.visible), e.world.display(to).map(|d| d.visible))
    else {
        return Ok(None);
    };
    let ids: Vec<_> = e.world.workspaces[&name].windows().collect();
    for id in ids {
        if let Some(w) = e.world.windows.get_mut(&id) {
            w.float_frame = w.float_frame.reproject(&from_r, &to_r);
        }
    }
    e.world.workspaces.get_mut(&name).expect("active workspace exists").display = to;
    e.world.active.remove(&from);
    // The display we left needs something to show.
    let replacement = (1..)
        .map(|n: u32| n.to_string())
        .find(|n| {
            n != &name
                && !e.world.active.values().any(|a| a == n)
                && e.world.workspaces.get(n).is_none_or(|ws| ws.display == from)
        })
        .expect("unbounded");
    e.ensure_workspace(&replacement, from);
    e.world.workspaces.get_mut(&replacement).expect("just ensured").display = from;
    e.world.active.insert(from, replacement);
    if let Some(old) = e.world.active.insert(to, name.clone()) {
        e.world.previous.insert(to, old);
    }
    e.world.focused_display = Some(to);
    e.focus_fallback(&name);
    Ok(None)
}
