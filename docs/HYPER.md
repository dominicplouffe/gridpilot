# Setting up the Hyper key

GridPilot's `mod` key, Omarchy's SUPER, is a **Hyper key**: Caps Lock
remapped so that holding it presses **Control + Option + Command**. No app
uses that combination, so GridPilot can take every `mod+…` key without
stealing app or system shortcuts.

**Leave Shift out of the Hyper key.** GridPilot uses `mod+shift+…` as its
second layer (`mod+←` focuses, `mod+shift+←` swaps). If your Hyper key
already includes Shift, every key press would look like the Shift layer.

Pick one of these tools.

## Hyperkey (free, simplest)

1. Install [Hyperkey](https://hyperkey.app).
2. Choose Caps Lock as the Hyper key.
3. Turn **off** the option to include Shift in the Hyper key.
4. Optionally make a quick tap of Caps Lock send Escape.

## Raycast

Settings → Advanced → **Hyper Key**: choose Caps Lock, and turn off the
option to include Shift.

## Karabiner-Elements

Add this rule under *Complex Modifications* (paste it as a new rule, or put
it in `~/.config/karabiner/assets/complex_modifications/gridpilot.json`
wrapped in `{"title": "GridPilot", "rules": [ … ]}`):

```json
{
  "description": "Caps Lock → Hyper for GridPilot (ctrl+opt+cmd), Escape when tapped",
  "manipulators": [
    {
      "type": "basic",
      "from": { "key_code": "caps_lock", "modifiers": { "optional": ["any"] } },
      "to": [{ "key_code": "left_command", "modifiers": ["left_control", "left_option"] }],
      "to_if_alone": [{ "key_code": "escape" }]
    }
  ]
}
```

## Check it works

Run `gridpilot run`, hold Caps Lock and press `K`: the key cheat sheet
should open. `gridpilot keymap` prints every binding as GridPilot sees it,
e.g. `ctrl+alt+cmd+left  Focus window left`.

## No Hyper key

Everything still works by pressing `ctrl+alt+cmd` by hand. To use a
different modifier, change one line in `~/.config/gridpilot/gridpilot.toml`:

```toml
[modifiers]
mod = "ctrl+alt"      # or "hyper", or any combination
```

Then run `gridpilot check` to catch bindings that clash with each other,
and `gridpilot reload`.

If you'd rather keep Shift in your Hyper key, set
`hyper = "ctrl+alt+shift+cmd"` and move the `mod+shift+…` actions (swap,
move to workspace, …) to other keys in `[bindings]`. `gridpilot check`
can't spot this case for you: to GridPilot `mod+x` and `mod+shift+x` are
different keys, it's your keyboard that sends them both as one.
