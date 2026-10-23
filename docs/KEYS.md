# Default key bindings

Generated with `gridpilot keymap --markdown` from the default config.
`mod` is `ctrl+alt+cmd`, which Caps Lock presses for you (see [HYPER.md](HYPER.md)), and `grid` is `ctrl+cmd`; change either in `[modifiers]` and every binding follows.
Edit `[bindings]` to add or remove keys (`"mod+m" = "none"`).

### `system` — Reload, key cheat sheet, themes, gaps, exec and introspection

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+cmd+b` | Browser | `exec open -a Safari` |
| `ctrl+alt+cmd+return` | Terminal | `exec open -na Terminal` |
| `ctrl+alt+cmd+e` | File manager | `exec open ~` |
| `ctrl+alt+shift+cmd+backspace` | Toggle gaps | `gaps toggle` |
| `ctrl+alt+cmd+k` | Show key bindings | `keys` |
| `ctrl+alt+shift+cmd+r` | Reload config | `reload` |
| `ctrl+alt+shift+cmd+space` | Next theme | `theme next` |

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
| `ctrl+alt+cmd+space` | Show the grid overlay | `grid toggle` |

### `focus` — Directional focus, swapping and window cycling

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+cmd+down` | Focus window down | `focus down` |
| `ctrl+alt+cmd+left` | Focus window left | `focus left` |
| `alt+tab` | Focus next window | `focus next` |
| `alt+shift+tab` | Focus previous window | `focus prev` |
| `ctrl+alt+cmd+right` | Focus window right | `focus right` |
| `ctrl+alt+cmd+up` | Focus window up | `focus up` |
| `ctrl+alt+cmd+period` | Focus next display | `focus-display next` |
| `ctrl+alt+cmd+comma` | Focus previous display | `focus-display prev` |
| `ctrl+alt+shift+cmd+down` | Swap window down | `swap down` |
| `ctrl+alt+shift+cmd+left` | Swap window left | `swap left` |
| `ctrl+alt+shift+cmd+right` | Swap window right | `swap right` |
| `ctrl+alt+shift+cmd+up` | Swap window up | `swap up` |

### `tiling` — Float, fullscreen, pop-out, split, resize and layout cycling

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+cmd+c` | Centre floating window | `center` |
| `ctrl+alt+cmd+w` | Close window | `close` |
| `ctrl+alt+cmd+t` | Toggle floating/tiling | `float toggle` |
| `ctrl+alt+cmd+f` | Full screen | `fullscreen full` |
| `ctrl+alt+shift+cmd+f` | Maximize inside gaps | `fullscreen maximize` |
| `ctrl+alt+cmd+l` | Next workspace layout | `layout next` |
| `ctrl+alt+shift+cmd+l` | Previous workspace layout | `layout prev` |
| `ctrl+alt+cmd+m` | Minimize window | `minimize` |
| `ctrl+alt+cmd+o` | Pop window out (float and pin) | `pop` |
| `ctrl+alt+shift+cmd+equal` | Grow height | `resize height +100` |
| `ctrl+alt+shift+cmd+minus` | Shrink height | `resize height -100` |
| `ctrl+alt+cmd+equal` | Grow width | `resize width +100` |
| `ctrl+alt+cmd+minus` | Shrink width | `resize width -100` |
| `ctrl+alt+cmd+j` | Toggle split direction | `split toggle` |
| `ctrl+alt+cmd+home` | Restore window width | `width restore` |
| `ctrl+alt+shift+cmd+home` | Save window width | `width save` |

### `workspaces` — Numbered workspaces per display, moving windows and workspaces between displays

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+cmd+period` | Move window to next display | `move-to-display next` |
| `ctrl+alt+shift+cmd+comma` | Move window to previous display | `move-to-display prev` |
| `ctrl+alt+shift+cmd+1` | Move window to workspace 1 | `move-to-workspace 1` |
| `ctrl+alt+shift+cmd+0` | Move window to workspace 10 | `move-to-workspace 10` |
| `ctrl+alt+shift+cmd+2` | Move window to workspace 2 | `move-to-workspace 2` |
| `ctrl+alt+shift+cmd+3` | Move window to workspace 3 | `move-to-workspace 3` |
| `ctrl+alt+shift+cmd+4` | Move window to workspace 4 | `move-to-workspace 4` |
| `ctrl+alt+shift+cmd+5` | Move window to workspace 5 | `move-to-workspace 5` |
| `ctrl+alt+shift+cmd+6` | Move window to workspace 6 | `move-to-workspace 6` |
| `ctrl+alt+shift+cmd+7` | Move window to workspace 7 | `move-to-workspace 7` |
| `ctrl+alt+shift+cmd+8` | Move window to workspace 8 | `move-to-workspace 8` |
| `ctrl+alt+shift+cmd+9` | Move window to workspace 9 | `move-to-workspace 9` |
| `ctrl+alt+cmd+1` | Switch to workspace 1 | `workspace 1` |
| `ctrl+alt+cmd+0` | Switch to workspace 10 | `workspace 10` |
| `ctrl+alt+cmd+2` | Switch to workspace 2 | `workspace 2` |
| `ctrl+alt+cmd+3` | Switch to workspace 3 | `workspace 3` |
| `ctrl+alt+cmd+4` | Switch to workspace 4 | `workspace 4` |
| `ctrl+alt+cmd+5` | Switch to workspace 5 | `workspace 5` |
| `ctrl+alt+cmd+6` | Switch to workspace 6 | `workspace 6` |
| `ctrl+alt+cmd+7` | Switch to workspace 7 | `workspace 7` |
| `ctrl+alt+cmd+8` | Switch to workspace 8 | `workspace 8` |
| `ctrl+alt+cmd+9` | Switch to workspace 9 | `workspace 9` |
| `ctrl+alt+cmd+tab` | Next workspace | `workspace next` |
| `ctrl+alt+shift+cmd+tab` | Previous workspace | `workspace prev` |
| `ctrl+alt+cmd+backslash` | Former workspace | `workspace previous` |

### `scratchpad` — A summonable stash of floating windows

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+cmd+s` | Move window to scratchpad | `scratchpad move` |
| `ctrl+alt+shift+cmd+grave` | Move window to scratchpad | `scratchpad move` |
| `ctrl+alt+cmd+s` | Toggle scratchpad | `scratchpad toggle` |
| `ctrl+alt+cmd+grave` | Toggle scratchpad | `scratchpad toggle` |

### `groups` — Tabbed window groups inside a tile

| Keys | Action | Command |
|---|---|---|
| `ctrl+alt+shift+cmd+leftbracket` | Move window into group on the left | `group join left` |
| `ctrl+alt+shift+cmd+rightbracket` | Move window into group on the right | `group join right` |
| `ctrl+alt+shift+cmd+g` | Move window out of group | `group leave` |
| `ctrl+alt+cmd+rightbracket` | Next window in group | `group next` |
| `ctrl+alt+cmd+leftbracket` | Previous window in group | `group prev` |
| `ctrl+alt+cmd+g` | Toggle window grouping | `group toggle` |
