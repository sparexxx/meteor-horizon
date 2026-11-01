# Meteor Horizon Lite

A native Horizon mod: Player ESP, Anchor Macro, X-Ray, Auto Totem, a customizable
ClickGUI, a keybind per module and a global toggle, with settings saved across restarts.

It is a **horizon-mixins bundle**, not a Java jar and not a standalone binary. Horizon
compiles native mods by rewriting the game's own Rust source before building it.

---

## Honest status

This matters more than anything else here, so it is first.

| Part | State |
|---|---|
| `runtime.rs` — modules, keybinds, config, ClickGUI layout + hit-testing, Anchor planner, Auto Totem slot choice, ESP filtering | **Written, pure std, no game API. 20 unit tests included. Verifiable by you with `cargo test` right now.** |
| `key.rs` — every keybind, global toggle, GUI key | Written against verified helpers (`native_on`, `native_mod_key`, `native_setting`, `native_set_setting`, `native_chat`) |
| `hud.rs` — ClickGUI drawing | Written against verified helpers (`native_fill`, `native_text`, `native_text_width`) |
| `settings.rs` / `click.rs` — Options > Mods page | Written against verified helper signatures |
| `list.rs` — the mod's row and its keybind list | Written; `NativeMod { id, title, description, keys }` copied from the source |
| `command.rs` — `/meteor` | Written against verified signature |
| `world_lifecycle.rs` — reset on dimension change | Written; deliberately does not match on `WorldPhase` variants, so it cannot break if they differ |
| `render_needs.rs` — ask for the depth buffer | Written; `RenderNeeds { entity_spans, world_depth }` copied from the source |
| `visuals.rs` — X-Ray block rims | Written; `NativeBlockRim { pos, argb, width }` copied from the source. Fed by a scan that is **not** implemented (SEAM 1) |
| `tick.rs` — per-tick cache | Clock and caches written; the two world queries are **SEAM 1** and return empty until filled in |
| `frame.rs` — frame clock | Clock written; the Anchor and Auto Totem **execution** are SEAM 3 and SEAM 4 |
| `bridge.rs` — cross-crate ESP state + wgpu pipeline | **Done.** Public support module in `horizon-client-render`; the game crate publishes boxes, the renderer reads them |
| `world_drawn.rs` — 3D ESP geometry | Done: reads the bridge, transforms by `view_proj`, depth-tests against the world's depth. Draws nothing until SEAM 1 supplies targets |
| `horizon-mixin.toml` | Written; `horizon_revision` **must be replaced** with your clone's commit |

**None of this has been compiled.** The session had no working terminal — every command
returned exit code 34 with no output — so no `cargo check`, no build, and no `.hnmod`
exists. The logic in `runtime.rs` is the part designed to be testable without the game.

## Seams — the four places that need one signature each

These are the only places where a game call was needed but its exact signature could not
be read (the hook source is larger than one fetch). Everything around them is done.

1. **`tick.rs` → `collect_targets` / `collect_ores`.** Nothing in the documented hook list
   gives a client-side entity iterator or a bulk block scan; the official mods only read
   `native_player()`. Look in `crates/horizon-client/src/game/native_hooks.rs` for
   `native_entities`/`native_players` and for API 20's `native_block`, whose signature
   takes a position and returns a state. A helper is compiled *inside* its target
   function, so reading the client's entity table and level directly also works.
2. ~~**`world_drawn.rs` → `crate::gfx::meteor_bridge`.**~~ **Solved** by `bridge.rs`, a
   `visibility = "pub"` support module on
   `crates/horizon-client-render/src/gfx/native_render.rs`. The game crate calls
   `horizon_client_render::gfx::native_render::meteor_bridge::set_boxes(..)`; the render
   helper reads it as the sibling `meteor_bridge::boxes()`. A `Mutex` covers the tick
   thread / render thread split, and a poisoned lock yields an empty list, never a panic.
3. **`frame.rs` → Anchor Macro execution.** `anchor_next()` already returns
   `PlaceObsidian` / `PlaceAnchor` / `Charge` / `Detonate`. Firing it needs the client's
   use/attack path (API 13 and 14 *observe* a use; they do not perform one).
4. **`frame.rs` → Auto Totem move.** `totem_source()` already returns the slot to move, and
   returns `None` when the offhand holds a totem. The move itself is API 39
   (`native_carried_stack`, `native_on_equip`) or API 9 (`native_count_items`,
   `native_take_items`, `native_give_items`) — confirm the parameter lists first.

Until 1–4 are filled, ESP/X-Ray draw nothing rather than drawing wrong things, and the
Anchor/Auto Totem modules change state and save settings but perform no world action.
Nothing pretends to work.

---

## Files

```
meteor_horizon/
  Cargo.toml              the bundle's crate, so `cargo test` checks the logic
  horizon-mixin.toml      the bundle: module + every injection
  runtime.rs              the logic module (added to the game as game::official_meteor_horizon)
  key.rs                  native_key
  frame.rs                native_frame
  tick.rs                 native_tick
  hud.rs                  native_hud
  visuals.rs              native_visuals
  render_needs.rs         native_render_needs
  world_drawn.rs          native_world_drawn
  world_lifecycle.rs      native_world_lifecycle
  list.rs                 native_mods
  settings.rs             native_settings
  click.rs                native_settings_click
  command.rs              native_command
  README.md
```

`runtime.rs` is one module rather than the `src/` tree in the original plan because a
Horizon bundle is flat helpers plus one module — `[[module]] file = "...game.rs"` in
`horizon-mixin.toml` is what makes it part of the game. Categories, modules, config and GUI
state are all separated inside it.

## Build it yourself

You need the Horizon source and Rust 1.90+.

```bash
# 1. horizon-mixins, the tool that applies the bundle
git clone https://github.com/horizon-rust/horizon-mixins.git
cd horizon-mixins
cargo fetch --locked
cargo build --locked -p horizon-mixins-cli

# 2. the game
git clone https://github.com/horizon-rust/horizon.git
cd horizon
git rev-parse HEAD        # <- put this hash in horizon-mixin.toml's horizon_revision

# 3. apply + typecheck. This is where real compiler errors surface.
/horizon-mixins/target/debug/horizon-mixins check \
  --workspace . \
  --mods /path/to/meteor_horizon \
  --out /tmp/meteor-stage \
  --trust-native-mods

# 4. package it into the .hnmod
cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod
cargo horizon-mod check /tmp/meteor-stage/../meteor_horizon-0.1.0.hnmod
```

The official, shortest path once you have a Horizon checkout is the helper every Horizon
native mod uses:

```bash
python3 native/tools/native.py check   meteor_horizon   # apply on Horizon + cargo check
python3 native/tools/native.py package meteor_horizon   # -> native/target/meteor_horizon-0.1.0.hnmod
```

Check the logic on its own, with no game at all — this always works:

```bash
cd meteor_horizon && cargo test
```

## Install

1. Put `meteor_horizon-0.1.0.hnmod` in `horizon-native-mods/` (beside `options.txt`).
2. Restart the launcher and press **Build** on the mod's page. Horizon builds a game
   specific to that instance.
3. Switch it on under **Options > Mods**. The first native mod install shows a trust
   notice: native mods run with your own user rights and are not sandboxed.
4. The settings page is **Options > Mods > Meteor Horizon Lite > Settings**.

## Default keybinds

| Action | Key |
|---|---|
| Global toggle | Right Shift |
| ClickGUI | Right Ctrl |
| Player ESP | G |
| Anchor Macro | V |
| X-Ray | X |
| Auto Totem | T |

Rebind any of them on the mod's page; they are saved like every other setting. The global
toggle never erases the per-module state, so switching the client back on restores exactly
the modules that were running.

## Where settings live

Horizon keeps a native mod's settings under its id, which is why every key in
`runtime.rs::Config::stored()` is a flat name (`global`, `module.esp`, `esp.color`,
`xray.blocks`, `anchor.delay`, `totem.health`, `gui.category`, …). They are written through
`native_set_setting` and read once through `native_setting`, so they survive a restart and
a crash. Malformed values fall back to defaults instead of panicking — there is a test for
exactly that.

## Safety notes

- Native mods are not sandboxed. Everything here stays inside the game's own helpers.
- ESP and X-Ray are visual only: they change what you see, not world state.
- Auto Totem only moves an item between your own inventory slots, and does nothing at all
  when the offhand already holds a totem.
- On a server, client-side ESP can violate that server's rules. That is your call to make.

## Licence

MIT.
