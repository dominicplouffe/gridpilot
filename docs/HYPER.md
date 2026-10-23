# Caps Lock as your `mod` key

GridPilot's `mod` key, Omarchy's SUPER, is **Caps Lock**, out of the box.
Hold Caps Lock and press a key: Caps+F full screen, Caps+← focus left,
Caps+Shift+← swap, Caps+3 workspace 3, Caps+K all keys.

There is nothing to install. While GridPilot runs:

- Caps Lock no longer types capitals, and a tap on its own does nothing.
- Holding Caps Lock counts as holding **Control + Option + Command**
  (whatever `mod` is set to), so ⌃⌥⌘ by hand works too, for example on an
  external keyboard.
- Any key you press while holding Caps Lock goes to GridPilot, never to
  the app, even if it isn't bound to anything.

When GridPilot quits (menu bar → Quit, Ctrl-C, or even a crash) Caps Lock
goes back to normal.

## How it works

macOS ships a tool called `hidutil` that can remap keys for the current
login session. GridPilot uses it to turn Caps Lock into F18, a key no app
uses, and watches for F18 with a keyboard event tap (the same permission
system GridPilot already uses to move windows). Your other key remaps,
including ones made in System Settings → Keyboard → Modifier Keys, are kept
and restored.

## Turning it off

In the settings page (`gridpilot settings` → General), untick **Caps Lock is
the main modifier**, or in `~/.config/gridpilot/gridpilot.toml`:

```toml
[general]
capslock_mod = false
```

Then hold ⌃⌥⌘ (or whatever `mod` is) instead.

## If Caps Lock is stuck

If Caps Lock still does nothing after GridPilot stopped, run:

```sh
gridpilot capslock off
```

Logging out or restarting the Mac also resets it.

## If macOS blocks the keyboard hook

GridPilot prints `Caps Lock can't act as mod: macOS refused the keyboard
hook` and leaves Caps Lock alone. Allow GridPilot, or the terminal you
started it from, in System Settings → Privacy & Security → **Accessibility**
and **Input Monitoring**, then restart GridPilot.

## Advanced: using another remapper

If you already use Hyperkey, Raycast's Hyper Key or Karabiner-Elements to
turn Caps Lock into ⌃⌥⌘, set `capslock_mod = false` so the two don't fight
over the key. Leave Shift out of that Hyper key: GridPilot uses Shift as its
second layer (`mod+shift+←` swaps).
