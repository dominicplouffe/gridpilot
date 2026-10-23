//! Hyprland's special workspace: a stash of floating windows summoned over
//! whatever you are doing (`mod+s`) and sent away again.

use crate::command::{Args, CmdResult};
use crate::engine::Engine;
use crate::feature::{Feature, Registrar};
use crate::world::SCRATCHPAD;

pub struct ScratchpadFeature;

impl Feature for ScratchpadFeature {
    fn id(&self) -> &'static str {
        "scratchpad"
    }

    fn summary(&self) -> &'static str {
        "A summonable stash of floating windows"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command(
            "scratchpad",
            "scratchpad toggle|move",
            "Show/hide the scratchpad, or move the focused window in or out of it",
            scratchpad,
        );
        r.bind("mod+s", "scratchpad toggle", "Toggle scratchpad");
        r.bind("mod+grave", "scratchpad toggle", "Toggle scratchpad");
        r.bind("mod+shift+s", "scratchpad move", "Move window to scratchpad");
        r.bind("mod+shift+grave", "scratchpad move", "Move window to scratchpad");
    }
}

fn scratchpad(e: &mut Engine, a: &Args) -> CmdResult {
    match a.get(0).unwrap_or("toggle") {
        "toggle" | "show" | "hide" => {
            if e.world.scratchpad_on.is_some() {
                e.world.scratchpad_on = None;
                if let Some(ws) = e.world.current_workspace() {
                    e.focus_fallback(&ws);
                }
                return Ok(None);
            }
            let first =
                e.world.workspaces.get(SCRATCHPAD).and_then(|ws| ws.last_focused.or_else(|| ws.windows().next()));
            let Some(first) = first else { return Ok(Some("the scratchpad is empty".into())) };
            let d = e.current_display()?;
            e.world.scratchpad_on = Some(d);
            e.focus_window(first);
            Ok(None)
        }
        "move" => {
            let id = e.require_focused()?;
            if e.world.windows[&id].workspace == SCRATCHPAD {
                // Take it back out onto the workspace underneath.
                let d = e.current_display()?;
                let ws = e.world.active_workspace(d).ok_or("no workspace")?.to_string();
                e.move_window_to_workspace(id, &ws, true);
                e.world.scratchpad_on = None;
                e.focus_window(id);
            } else {
                e.move_window_to_workspace(id, SCRATCHPAD, false);
            }
            Ok(None)
        }
        other => Err(format!("unknown `scratchpad {other}`")),
    }
}
