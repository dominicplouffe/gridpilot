//! Hyprland window groups: several windows sharing one tile as tabs.

use crate::command::{Args, CmdResult};
use crate::engine::Engine;
use crate::feature::{Feature, Registrar};
use crate::platform::WindowId;

pub struct GroupsFeature;

impl Feature for GroupsFeature {
    fn id(&self) -> &'static str {
        "groups"
    }

    fn summary(&self) -> &'static str {
        "Tabbed window groups inside a tile"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("group", "group toggle|next|prev|index <n>|join <dir>|leave", "Manage tabbed window groups", group);
        r.bind("mod+g", "group toggle", "Toggle window grouping");
        r.bind("mod+shift+g", "group leave", "Move window out of group");
        // Tab-like keys, as ⌘⇧[ / ⌘⇧] switch tabs in most apps.
        r.bind("mod+rightbracket", "group next", "Next window in group");
        r.bind("mod+leftbracket", "group prev", "Previous window in group");
        r.bind("mod+shift+leftbracket", "group join left", "Move window into group on the left");
        r.bind("mod+shift+rightbracket", "group join right", "Move window into group on the right");
    }
}

fn group(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let (ws_name, tile_id) = e.world.tile_of(id).map(|(ws, t, _)| (ws.name.clone(), t)).ok_or("window is not tiled")?;
    match a.require(0, "a group action")? {
        "toggle" => {
            let tile = e
                .world
                .workspaces
                .get_mut(&ws_name)
                .and_then(|ws| ws.tiles.get_mut(&tile_id))
                .expect("tile_of found it");
            if !tile.grouped {
                tile.grouped = true;
                return Ok(None);
            }
            // Dissolve: every member but the active one gets its own tile.
            tile.grouped = false;
            let active = tile.active_window();
            let others: Vec<WindowId> = tile.windows.iter().copied().filter(|w| Some(*w) != active).collect();
            for w in others {
                e.world.detach(w);
                e.world.focused = active;
                e.world.attach(w, &ws_name, false);
            }
        }
        "next" | "prev" | "index" => {
            let forward = a.get(0) == Some("next");
            let index: Option<usize> =
                if a.get(0) == Some("index") { Some(a.parse::<usize>(1, "index")?.saturating_sub(1)) } else { None };
            let tile = e
                .world
                .workspaces
                .get_mut(&ws_name)
                .and_then(|ws| ws.tiles.get_mut(&tile_id))
                .expect("tile_of found it");
            let n = tile.windows.len();
            if n < 2 {
                return Ok(None);
            }
            tile.active = match index {
                Some(i) => i.min(n - 1),
                None if forward => (tile.active + 1) % n,
                None => (tile.active + n - 1) % n,
            };
            let w = tile.windows[tile.active];
            e.focus_window(w);
        }
        "join" => {
            let dir = a.direction(1)?;
            let Some(other) = e.neighbour(dir) else { return Ok(None) };
            let Some((ws_b, tb, _)) = e.world.tile_of(other) else { return Ok(None) };
            if ws_b.name != ws_name || tb == tile_id {
                return Ok(None);
            }
            e.world.detach(id);
            let ws = e.world.workspaces.get_mut(&ws_name).expect("exists");
            let tile = ws.tiles.get_mut(&tb).expect("neighbour tile exists");
            tile.grouped = true;
            tile.windows.push(id);
            tile.active = tile.windows.len() - 1;
            if let Some(w) = e.world.windows.get_mut(&id) {
                w.tile = Some(tb);
                w.floating = false;
            }
            e.focus_window(id);
        }
        "leave" => {
            let n = e.world.tile_of(id).map(|(_, _, t)| t.windows.len()).unwrap_or(0);
            if n < 2 {
                return Ok(None);
            }
            e.world.detach(id);
            e.world.attach(id, &ws_name, false);
            e.focus_window(id);
        }
        other => return Err(format!("unknown `group {other}`")),
    }
    Ok(None)
}
