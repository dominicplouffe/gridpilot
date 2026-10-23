# Default key bindings

Generated with `gridpilot keymap --markdown` from the default config.
**Caps** means hold Caps Lock (or ⌃⌥⌘), see [HYPER.md](HYPER.md). In the config, the same keys are written `mod+…`; the Chord column shows the exact modifiers.
Edit `[bindings]` or use `gridpilot settings` to change them.

### `system` — Reload, key cheat sheet, themes, gaps, exec and introspection

| Press | Action | Command | Chord |
|---|---|---|---|
| Caps + ⇧ + B | Browser | `exec open -a Safari` | `ctrl+alt+shift+cmd+b` |
| Caps + Return | Terminal | `exec open -na Terminal` | `ctrl+alt+cmd+return` |
| Caps + E | File manager | `exec open ~` | `ctrl+alt+cmd+e` |
| Caps + ⇧ + Delete | Toggle gaps | `gaps toggle` | `ctrl+alt+shift+cmd+backspace` |
| Caps + K | Show key bindings | `keys` | `ctrl+alt+cmd+k` |
| Caps + ⇧ + R | Reload config | `reload` | `ctrl+alt+shift+cmd+r` |
| Caps + ⇧ + Space | Next theme | `theme next` | `ctrl+alt+shift+cmd+space` |

### `grid` — Divvy-style grid overlay and placement shortcuts

| Press | Action | Command | Chord |
|---|---|---|---|
| ⌃⌘ + ↓ | Bottom half | `grid shortcut "Bottom half"` | `ctrl+cmd+down` |
| ⌃⌘ + J | Bottom left | `grid shortcut "Bottom left"` | `ctrl+cmd+j` |
| ⌃⌘ + K | Bottom right | `grid shortcut "Bottom right"` | `ctrl+cmd+k` |
| ⌃⌘ + C | Center | `grid shortcut "Center"` | `ctrl+cmd+c` |
| ⌃⌘ + ← | Left half | `grid shortcut "Left half"` | `ctrl+cmd+left` |
| ⌃⌘ + 1 | Left third | `grid shortcut "Left third"` | `ctrl+cmd+1` |
| ⌃⌘ + 4 | Left two thirds | `grid shortcut "Left two thirds"` | `ctrl+cmd+4` |
| ⌃⌘ + Return | Maximize | `grid shortcut "Maximize"` | `ctrl+cmd+return` |
| ⌃⌘ + 2 | Middle third | `grid shortcut "Middle third"` | `ctrl+cmd+2` |
| ⌃⌘ + → | Right half | `grid shortcut "Right half"` | `ctrl+cmd+right` |
| ⌃⌘ + 3 | Right third | `grid shortcut "Right third"` | `ctrl+cmd+3` |
| ⌃⌘ + 5 | Right two thirds | `grid shortcut "Right two thirds"` | `ctrl+cmd+5` |
| ⌃⌘ + ↑ | Top half | `grid shortcut "Top half"` | `ctrl+cmd+up` |
| ⌃⌘ + U | Top left | `grid shortcut "Top left"` | `ctrl+cmd+u` |
| ⌃⌘ + I | Top right | `grid shortcut "Top right"` | `ctrl+cmd+i` |
| Caps + Space | Show the grid overlay | `grid toggle` | `ctrl+alt+cmd+space` |

### `focus` — Directional focus, swapping and window cycling

| Press | Action | Command | Chord |
|---|---|---|---|
| Caps + ↓ | Focus window down | `focus down` | `ctrl+alt+cmd+down` |
| Caps + ← | Focus window left | `focus left` | `ctrl+alt+cmd+left` |
| ⌥ + Tab | Focus next window | `focus next` | `alt+tab` |
| ⌥⇧ + Tab | Focus previous window | `focus prev` | `alt+shift+tab` |
| Caps + → | Focus window right | `focus right` | `ctrl+alt+cmd+right` |
| Caps + ↑ | Focus window up | `focus up` | `ctrl+alt+cmd+up` |
| Caps + . | Focus next display | `focus-display next` | `ctrl+alt+cmd+period` |
| Caps + , | Focus previous display | `focus-display prev` | `ctrl+alt+cmd+comma` |
| Caps + ⇧ + ↓ | Swap window down | `swap down` | `ctrl+alt+shift+cmd+down` |
| Caps + ⇧ + ← | Swap window left | `swap left` | `ctrl+alt+shift+cmd+left` |
| Caps + ⇧ + → | Swap window right | `swap right` | `ctrl+alt+shift+cmd+right` |
| Caps + ⇧ + ↑ | Swap window up | `swap up` | `ctrl+alt+shift+cmd+up` |

### `tiling` — Float, fullscreen, pop-out, split, resize and layout cycling

| Press | Action | Command | Chord |
|---|---|---|---|
| Caps + C | Centre floating window | `center` | `ctrl+alt+cmd+c` |
| Caps + W | Close window | `close` | `ctrl+alt+cmd+w` |
| Caps + T | Toggle floating/tiling | `float toggle` | `ctrl+alt+cmd+t` |
| Caps + F | Full screen | `fullscreen full` | `ctrl+alt+cmd+f` |
| Caps + ⇧ + F | Maximize inside gaps | `fullscreen maximize` | `ctrl+alt+shift+cmd+f` |
| Caps + L | Next workspace layout | `layout next` | `ctrl+alt+cmd+l` |
| Caps + ⇧ + L | Previous workspace layout | `layout prev` | `ctrl+alt+shift+cmd+l` |
| Caps + M | Minimize window | `minimize` | `ctrl+alt+cmd+m` |
| Caps + O | Pop window out (float and pin) | `pop` | `ctrl+alt+cmd+o` |
| Caps + ⇧ + = | Grow height | `resize height +100` | `ctrl+alt+shift+cmd+equal` |
| Caps + ⇧ + - | Shrink height | `resize height -100` | `ctrl+alt+shift+cmd+minus` |
| Caps + = | Grow width | `resize width +100` | `ctrl+alt+cmd+equal` |
| Caps + - | Shrink width | `resize width -100` | `ctrl+alt+cmd+minus` |
| Caps + J | Toggle split direction | `split toggle` | `ctrl+alt+cmd+j` |
| Caps + Home | Restore window width | `width restore` | `ctrl+alt+cmd+home` |
| Caps + ⇧ + Home | Save window width | `width save` | `ctrl+alt+shift+cmd+home` |

### `workspaces` — Numbered workspaces per display, moving windows and workspaces between displays

| Press | Action | Command | Chord |
|---|---|---|---|
| Caps + ⇧ + . | Move window to next display | `move-to-display next` | `ctrl+alt+shift+cmd+period` |
| Caps + ⇧ + , | Move window to previous display | `move-to-display prev` | `ctrl+alt+shift+cmd+comma` |
| Caps + ⇧ + 1 | Move window to workspace 1 | `move-to-workspace 1` | `ctrl+alt+shift+cmd+1` |
| Caps + ⇧ + 0 | Move window to workspace 10 | `move-to-workspace 10` | `ctrl+alt+shift+cmd+0` |
| Caps + ⇧ + 2 | Move window to workspace 2 | `move-to-workspace 2` | `ctrl+alt+shift+cmd+2` |
| Caps + ⇧ + 3 | Move window to workspace 3 | `move-to-workspace 3` | `ctrl+alt+shift+cmd+3` |
| Caps + ⇧ + 4 | Move window to workspace 4 | `move-to-workspace 4` | `ctrl+alt+shift+cmd+4` |
| Caps + ⇧ + 5 | Move window to workspace 5 | `move-to-workspace 5` | `ctrl+alt+shift+cmd+5` |
| Caps + ⇧ + 6 | Move window to workspace 6 | `move-to-workspace 6` | `ctrl+alt+shift+cmd+6` |
| Caps + ⇧ + 7 | Move window to workspace 7 | `move-to-workspace 7` | `ctrl+alt+shift+cmd+7` |
| Caps + ⇧ + 8 | Move window to workspace 8 | `move-to-workspace 8` | `ctrl+alt+shift+cmd+8` |
| Caps + ⇧ + 9 | Move window to workspace 9 | `move-to-workspace 9` | `ctrl+alt+shift+cmd+9` |
| Caps + 1 | Switch to workspace 1 | `workspace 1` | `ctrl+alt+cmd+1` |
| Caps + 0 | Switch to workspace 10 | `workspace 10` | `ctrl+alt+cmd+0` |
| Caps + 2 | Switch to workspace 2 | `workspace 2` | `ctrl+alt+cmd+2` |
| Caps + 3 | Switch to workspace 3 | `workspace 3` | `ctrl+alt+cmd+3` |
| Caps + 4 | Switch to workspace 4 | `workspace 4` | `ctrl+alt+cmd+4` |
| Caps + 5 | Switch to workspace 5 | `workspace 5` | `ctrl+alt+cmd+5` |
| Caps + 6 | Switch to workspace 6 | `workspace 6` | `ctrl+alt+cmd+6` |
| Caps + 7 | Switch to workspace 7 | `workspace 7` | `ctrl+alt+cmd+7` |
| Caps + 8 | Switch to workspace 8 | `workspace 8` | `ctrl+alt+cmd+8` |
| Caps + 9 | Switch to workspace 9 | `workspace 9` | `ctrl+alt+cmd+9` |
| Caps + Tab | Next workspace | `workspace next` | `ctrl+alt+cmd+tab` |
| Caps + ⇧ + Tab | Previous workspace | `workspace prev` | `ctrl+alt+shift+cmd+tab` |
| Caps + \ | Former workspace | `workspace previous` | `ctrl+alt+cmd+backslash` |

### `scratchpad` — A summonable stash of floating windows

| Press | Action | Command | Chord |
|---|---|---|---|
| Caps + ⇧ + S | Move window to scratchpad | `scratchpad move` | `ctrl+alt+shift+cmd+s` |
| Caps + ⇧ + ` | Move window to scratchpad | `scratchpad move` | `ctrl+alt+shift+cmd+grave` |
| Caps + S | Toggle scratchpad | `scratchpad toggle` | `ctrl+alt+cmd+s` |
| Caps + ` | Toggle scratchpad | `scratchpad toggle` | `ctrl+alt+cmd+grave` |

### `groups` — Tabbed window groups inside a tile

| Press | Action | Command | Chord |
|---|---|---|---|
| Caps + ⇧ + [ | Move window into group on the left | `group join left` | `ctrl+alt+shift+cmd+leftbracket` |
| Caps + ⇧ + ] | Move window into group on the right | `group join right` | `ctrl+alt+shift+cmd+rightbracket` |
| Caps + ⇧ + G | Move window out of group | `group leave` | `ctrl+alt+shift+cmd+g` |
| Caps + ] | Next window in group | `group next` | `ctrl+alt+cmd+rightbracket` |
| Caps + [ | Previous window in group | `group prev` | `ctrl+alt+cmd+leftbracket` |
| Caps + G | Toggle window grouping | `group toggle` | `ctrl+alt+cmd+g` |
