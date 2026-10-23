# Default key bindings

Generated with `gridpilot keymap --markdown` from the default config.
`mod` is `ctrl+alt` and `grid` is `ctrl+cmd`; change either in `[modifiers]` and every binding follows.
Edit `[bindings]` to add or remove keys (`"mod+m" = "none"`).

### `system` — Reload, key cheat sheet, themes, gaps, exec and introspection

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+b` | Browser | `exec open -a Safari` |
| `ctrl+alt+return` | Terminal | `exec open -na Terminal` |
| `ctrl+alt+shift+f` | File manager | `exec open ~` |
| `ctrl+alt+shift+backspace` | Toggle gaps | `gaps toggle` |
| `ctrl+alt+k` | Show key bindings | `keys` |
| `ctrl+alt+shift+r` | Reload config | `reload` |
| `ctrl+alt+shift+space` | Next theme | `theme next` |

### `grid` — Divvy-style grid overlay and placement shortcuts

| Keys | Action | Command |
|---|---|---|
| `ctrl+cmd+down` | Bottom half | `grid shortcut "Bottom half"` |
| `ctrl+cmd+j` | Bottom left | `grid shortcut "Bottom left"` |
| `ctrl+cmd+k` | Bottom right | `grid shortcut "Bottom right"` |
| `ctrl+cmd+c` | Center | `grid shortcut "Center"` |
| `ctrl+cmd+left` | Left half | `grid shortcut "Left half"` |
| `ctrl+cmd+1` | Left third | `grid shortcut "Left third"` |
| `ctrl+cmd+4` | Left two thirds | `grid shortcut "Left two thirds"` |
| `ctrl+cmd+return` | Maximize | `grid shortcut "Maximize"` |
| `ctrl+cmd+2` | Middle third | `grid shortcut "Middle third"` |
| `ctrl+cmd+right` | Right half | `grid shortcut "Right half"` |
| `ctrl+cmd+3` | Right third | `grid shortcut "Right third"` |
| `ctrl+cmd+5` | Right two thirds | `grid shortcut "Right two thirds"` |
| `ctrl+cmd+up` | Top half | `grid shortcut "Top half"` |
| `ctrl+cmd+u` | Top left | `grid shortcut "Top left"` |
| `ctrl+cmd+i` | Top right | `grid shortcut "Top right"` |
| `ctrl+alt+space` | Show the grid overlay | `grid toggle` |

### `focus` — Directional focus, swapping and window cycling

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+down` | Focus window down | `focus down` |
| `ctrl+alt+left` | Focus window left | `focus left` |
| `alt+tab` | Focus next window | `focus next` |
| `alt+shift+tab` | Focus previous window | `focus prev` |
| `ctrl+alt+right` | Focus window right | `focus right` |
| `ctrl+alt+up` | Focus window up | `focus up` |
| `ctrl+alt+period` | Focus next display | `focus-display next` |
| `ctrl+alt+comma` | Focus previous display | `focus-display prev` |
| `ctrl+alt+shift+down` | Swap window down | `swap down` |
| `ctrl+alt+shift+left` | Swap window left | `swap left` |
| `ctrl+alt+shift+right` | Swap window right | `swap right` |
| `ctrl+alt+shift+up` | Swap window up | `swap up` |

### `tiling` — Float, fullscreen, pop-out, split, resize and layout cycling

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+c` | Centre floating window | `center` |
| `ctrl+alt+w` | Close window | `close` |
| `ctrl+alt+t` | Toggle floating/tiling | `float toggle` |
| `ctrl+alt+f` | Full screen | `fullscreen full` |
| `ctrl+alt+cmd+f` | Maximize inside gaps | `fullscreen maximize` |
| `ctrl+alt+l` | Next workspace layout | `layout next` |
| `ctrl+alt+shift+l` | Previous workspace layout | `layout prev` |
| `ctrl+alt+m` | Minimize window | `minimize` |
| `ctrl+alt+o` | Pop window out (float and pin) | `pop` |
| `ctrl+alt+shift+equal` | Grow height | `resize height +100` |
| `ctrl+alt+shift+cmd+equal` | Grow height a lot | `resize height +300` |
| `ctrl+alt+shift+minus` | Shrink height | `resize height -100` |
| `ctrl+alt+shift+cmd+minus` | Shrink height a lot | `resize height -300` |
| `ctrl+alt+equal` | Grow width | `resize width +100` |
| `ctrl+alt+cmd+equal` | Grow width a lot | `resize width +300` |
| `ctrl+alt+minus` | Shrink width | `resize width -100` |
| `ctrl+alt+cmd+minus` | Shrink width a lot | `resize width -300` |
| `ctrl+alt+j` | Toggle split direction | `split toggle` |
| `ctrl+alt+home` | Restore window width | `width restore` |
| `ctrl+alt+cmd+home` | Save window width | `width save` |

### `workspaces` — Numbered workspaces per display, moving windows and workspaces between displays

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+cmd+down` | Move window to display down | `move-to-display down` |
| `ctrl+alt+shift+cmd+left` | Move window to display left | `move-to-display left` |
| `ctrl+alt+shift+cmd+right` | Move window to display right | `move-to-display right` |
| `ctrl+alt+shift+cmd+up` | Move window to display up | `move-to-display up` |
| `ctrl+alt+shift+1` | Move window to workspace 1 | `move-to-workspace 1` |
| `ctrl+alt+shift+cmd+1` | Move window silently to workspace 1 | `move-to-workspace 1 --silent` |
| `ctrl+alt+shift+0` | Move window to workspace 10 | `move-to-workspace 10` |
| `ctrl+alt+shift+cmd+0` | Move window silently to workspace 10 | `move-to-workspace 10 --silent` |
| `ctrl+alt+shift+2` | Move window to workspace 2 | `move-to-workspace 2` |
| `ctrl+alt+shift+cmd+2` | Move window silently to workspace 2 | `move-to-workspace 2 --silent` |
| `ctrl+alt+shift+3` | Move window to workspace 3 | `move-to-workspace 3` |
| `ctrl+alt+shift+cmd+3` | Move window silently to workspace 3 | `move-to-workspace 3 --silent` |
| `ctrl+alt+shift+4` | Move window to workspace 4 | `move-to-workspace 4` |
| `ctrl+alt+shift+cmd+4` | Move window silently to workspace 4 | `move-to-workspace 4 --silent` |
| `ctrl+alt+shift+5` | Move window to workspace 5 | `move-to-workspace 5` |
| `ctrl+alt+shift+cmd+5` | Move window silently to workspace 5 | `move-to-workspace 5 --silent` |
| `ctrl+alt+shift+6` | Move window to workspace 6 | `move-to-workspace 6` |
| `ctrl+alt+shift+cmd+6` | Move window silently to workspace 6 | `move-to-workspace 6 --silent` |
| `ctrl+alt+shift+7` | Move window to workspace 7 | `move-to-workspace 7` |
| `ctrl+alt+shift+cmd+7` | Move window silently to workspace 7 | `move-to-workspace 7 --silent` |
| `ctrl+alt+shift+8` | Move window to workspace 8 | `move-to-workspace 8` |
| `ctrl+alt+shift+cmd+8` | Move window silently to workspace 8 | `move-to-workspace 8 --silent` |
| `ctrl+alt+shift+9` | Move window to workspace 9 | `move-to-workspace 9` |
| `ctrl+alt+shift+cmd+9` | Move window silently to workspace 9 | `move-to-workspace 9 --silent` |
| `ctrl+alt+shift+period` | Move workspace to next display | `move-workspace-to-display next` |
| `ctrl+alt+shift+comma` | Move workspace to previous display | `move-workspace-to-display prev` |
| `ctrl+alt+1` | Switch to workspace 1 | `workspace 1` |
| `ctrl+alt+0` | Switch to workspace 10 | `workspace 10` |
| `ctrl+alt+2` | Switch to workspace 2 | `workspace 2` |
| `ctrl+alt+3` | Switch to workspace 3 | `workspace 3` |
| `ctrl+alt+4` | Switch to workspace 4 | `workspace 4` |
| `ctrl+alt+5` | Switch to workspace 5 | `workspace 5` |
| `ctrl+alt+6` | Switch to workspace 6 | `workspace 6` |
| `ctrl+alt+7` | Switch to workspace 7 | `workspace 7` |
| `ctrl+alt+8` | Switch to workspace 8 | `workspace 8` |
| `ctrl+alt+9` | Switch to workspace 9 | `workspace 9` |
| `ctrl+alt+tab` | Next workspace | `workspace next` |
| `ctrl+alt+shift+tab` | Previous workspace | `workspace prev` |
| `ctrl+alt+backslash` | Former workspace | `workspace previous` |

### `scratchpad` — A summonable stash of floating windows

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+s` | Move window to scratchpad | `scratchpad move` |
| `ctrl+alt+shift+grave` | Move window to scratchpad | `scratchpad move` |
| `ctrl+alt+s` | Toggle scratchpad | `scratchpad toggle` |
| `ctrl+alt+grave` | Toggle scratchpad | `scratchpad toggle` |

### `groups` — Tabbed window groups inside a tile

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+cmd+1` | Switch to group window 1 | `group index 1` |
| `ctrl+alt+cmd+2` | Switch to group window 2 | `group index 2` |
| `ctrl+alt+cmd+3` | Switch to group window 3 | `group index 3` |
| `ctrl+alt+cmd+4` | Switch to group window 4 | `group index 4` |
| `ctrl+alt+cmd+5` | Switch to group window 5 | `group index 5` |
| `ctrl+alt+cmd+down` | Move window into group down | `group join down` |
| `ctrl+alt+cmd+left` | Move window into group left | `group join left` |
| `ctrl+alt+cmd+right` | Move window into group right | `group join right` |
| `ctrl+alt+cmd+up` | Move window into group up | `group join up` |
| `ctrl+alt+shift+g` | Move window out of group | `group leave` |
| `ctrl+alt+cmd+tab` | Next window in group | `group next` |
| `ctrl+alt+shift+cmd+tab` | Previous window in group | `group prev` |
| `ctrl+alt+g` | Toggle window grouping | `group toggle` |
