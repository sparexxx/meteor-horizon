# Meteor Horizon Lite — verified API dossier and blockers

Status: **no `.hnmod` exists and none could be produced in this session.** The source is
written against the real API below, but it was never compiled. See "Blockers".

---

## 1. Blockers (all verified, not assumed)

### Blocker A — the Horizon source is not in the workspace

```
glob **/*           -> meteor_horizon/PROBE.txt   (1 file)
glob **/Cargo.toml  -> 0 results
list .              -> .freebuff/, meteor_horizon/
```

There is no clone, no git history, no `native_hooks.rs` on disk. Horizon native mods are
mixin bundles that name **real functions in Horizon's source** and bind **real argument
types**, plus pin an exact `horizon_revision`. Without the tree these cannot be authored or
validated, and `horizon_revision` cannot be discovered.

### Blocker B — no process can be executed (three paths tested)

| Path | Command | Result |
|---|---|---|
| SYNC shell | `pwd && ls -la` | exit 34, empty stdout |
| SYNC shell | `echo hello` | exit 34, empty stdout |
| SYNC shell | `true` | exit 34, empty stdout |
| SYNC shell | `echo ok` (cwd=meteor_horizon) | exit 34, empty stdout |
| BACKGROUND shell | `echo marker > /tmp/bg_probe.txt` | pid issued, file never created |
| BACKGROUND shell | `echo "terminal-works" > meteor_horizon/probe2.txt` | pid issued, `probe2.txt` never appeared |
| BACKGROUND logs | read stdout.log / stderr.log | both **empty** — the process never ran |
| SYNC shell | `git --version` | exit 34, empty stdout |
| SYNC shell | `/bin/sh -c 'echo via-sh'` | exit 34, empty stdout |
| BACKGROUND shell | `printf 'bg-ok' > bgtest.txt; sleep 1; ...` | pid 16720 issued, `bgtest.txt` **does not exist** |
| File-change hooks | `run_file_change_hooks` | "not supported in SDK mode" |
| Interactive | `terminal_list` | no terminals |

Note what fails: `git --version` and `true` fail identically to `echo hello`, and so does
invoking `/bin/sh` directly, which means the failure is **before bash ever runs** — the
process spawn itself fails, not the command. `write_file` / `read_files` work perfectly on
the same project root, so the workspace path and permissions are fine; only the executor is
dead.

Host is Windows (paths under `C:\Users\KANOJI~1\...`). The tool spawns **Git Bash** on
Windows, so a missing or unregistered `bash.exe` is the leading suspect: spawn fails, exit
34, no output, background never runs.

Consequences: **no `git clone`, no `cargo check`, no `cargo test`, no packaging.** The
`.hnmod` is produced by running the mixin tool over the game tree; with no shell that
cannot happen. Nothing below has been compiled.

`write_file` and `read_files` do work — that is how this file was created and verified.

---

## 2. The real mod format (read from source, not invented)

Sources actually read:
- `horizon/native-hooks.json` — the authoritative hook catalog
- `horizon/crates/horizon-client-render/src/gfx/native_render.rs` — render hooks
- `horizon-mods/native/keystrokes/mixins/keystrokes/{horizon-mixin.toml,runtime.rs}`
- `horizon-mods/native/zoom/mixins/zoom/horizon-mixin.toml`
- `horizon-mods/native/minimap/mixins/minimap/horizon-mixin.toml`
- `horizon-mixins/README.md` — the injection tool contract

A native mod is a **horizon-mixins bundle**, not a standalone binary:

```
meteor_horizon/
  horizon-mixin.toml     # [mod], [compatibility], [[module]], [[injection]]
  runtime.rs             # the module added to the game as game::official_<mod>
  key.rs, hud.rs, ...    # one small helper per hook
```

A helper is **compiled as a local function inside the target function**, which is why it can
touch private fields and game types.

### Verified manifest shape (from official Keystrokes/Zoom/Minimap)

```toml
[mod]
id = "keystrokes"
version = "2.0.0"
priority = 100

[compatibility]
horizon_revision = "6f8c7a0e5d531d3834c4d0be4659a1db8e7e5c2a"

[[module]]
id = "runtime"
file = "crates/horizon-client/src/game.rs"
name = "official_keystrokes"
source = "runtime.rs"

[[injection]]
id = "hud"
kind = "HEAD"
target = "horizon_client::game::native_hooks::Game<'_>::native_hud"
helper = "hud.rs"
bindings = [
  { name = "game", ty = "&mut crate::game::Game<'_>", value = "&mut *self" },
  { name = "batch", ty = "&mut horizon_client::hud::HudBatch", value = "&mut *batch" },
  { name = "corners", ty = "&mut horizon_client::extras_hud::Corners", value = "&mut *corners" },
]
```

Prefer the **named-hook form** (horizon-mixins 0.6.0+), which survives hook movement:

```toml
[[injection]]
id = "hud"
hook = "hud"          # a name from native-hooks.json
hook_version = 1
kind = "HEAD"
helper = "hud.rs"
bindings = [ ... ]
```

Injection kinds: `HEAD`, `BEFORE_CALL`, `RETURN`, `MODIFY_ARG`, `MODIFY_ARGS`,
`OVERWRITE`, `AFTER_CALL`, `AFTER_ASSIGN`, `MODIFY_EXPRESSION_VALUE`, `MODIFY_CONSTANT`,
`MODIFY_VARIABLE`, `WRAP_OPERATION`. An `OVERWRITE` conflicts with every other hook on its
function.

---

## 3. Exact binding types, copied from official mods

These are verbatim from the official bundles — safe to reuse.

**`key`** (`Game<'_>::native_key`) — client keyboard, press *and* release:
```toml
bindings = [
  { name = "game", ty = "&mut crate::game::Game<'_>", value = "&mut *self" },
  { name = "key", ty = "&str", value = "key" },
  { name = "pressed", ty = "bool", value = "pressed" },
]
```

**`wheel`** (`native_wheel`):
```toml
bindings = [
  { name = "game", ty = "&crate::game::Game<'_>", value = "&*self" },
  { name = "steps", ty = "f32", value = "steps" },
  { name = "claimed", ty = "&mut bool", value = "&mut *claimed" },
]
```

**`mods`** / **`settings`** / **`settings_click`** — mod listing and the Options > Mods page:
```toml
# mods
{ name = "mods", ty = "&mut Vec<crate::game::native_hooks::NativeMod>", value = "&mut *mods" }
# settings
{ name = "game", ty = "&mut crate::game::Game<'_>", value = "&mut *self" }
{ name = "mod_id", ty = "&str", value = "mod_id" }
{ name = "buttons", ty = "&mut Vec<String>", value = "&mut *buttons" }
# settings_click
{ name = "button", ty = "usize", value = "button" }
```

**`hud`** (`native_hud`) — 2D screen drawing:
```toml
{ name = "batch", ty = "&mut horizon_client::hud::HudBatch", value = "&mut *batch" }
{ name = "corners", ty = "&mut horizon_client::extras_hud::Corners", value = "&mut *corners" }
```

**`zoom`** / **`minimap`** — same shape, output is an `Option<NativeX>` handed back:
```toml
{ name = "zoom", ty = "&mut Option<crate::game::native_hooks::NativeZoom>", value = "&mut *zoom" }
{ name = "minimap", ty = "&mut Option<crate::game::native_hooks::NativeMinimap>", value = "&mut *minimap" }
```

---

## 4. The finding that changes the plan: ESP is possible

The earlier conclusion (from docs prose) that Player ESP was unreachable was **wrong**.
`native-hooks.json` and `native_render.rs` expose a real render hook, API 58:

```rust
pub fn native_world_drawn(world: &mut WorldDrawn<'_>)   // horizon_client_render::gfx::native_render

pub struct WorldDrawn<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub color: &'a wgpu::Texture,
    pub color_view: &'a wgpu::TextureView,
    pub color_format: wgpu::TextureFormat,
    pub size: [u32; 2],
    pub depth: &'a wgpu::TextureView,     // reversed: 0 == sky
    pub depth_size: [u32; 2],
    pub view_proj: Mat4,                  // camera-relative W2C, no TAA jitter
    pub jittered: Mat4,                   // what the world was drawn with
    pub camera: DVec3,
    pub entities: &'a [Vertex],
    pub spans: &'a [EntitySpan],
    pub atlas: Option<&'a wgpu::Texture>,
    pub quad_indices: &'a wgpu::Buffer,
}

pub struct RenderNeeds { pub entity_spans: bool, pub world_depth: bool }
pub fn native_render_needs(needs: &mut RenderNeeds)
```

This is called **after** sky, terrain, entities, particles, weather and the world post pass,
before the hand, HUD and screens. It gives a live `wgpu` device + queue + encoder, the
camera-relative view-projection matrix, the camera position and the world depth buffer.
That is everything a real 3D ESP box needs, and it also makes true X-Ray viable through
geometry rather than the API-46 block-rim limit.

So the renderer-mixin approach you chose is correct and supported. Two requirements follow:
- `native_render_needs` must set `world_depth: true` when ESP wants depth-tested (or
  depth-reading) boxes, and `entity_spans: true` to receive per-entity runs.
- The helper needs its own WGSL shader + pipeline, built once and cached, not per frame.

Full hook catalog highlights (`native-hooks.json`, all `version: 1`):

| Need | Hook |
|---|---|
| client keys (press/release) | `key` |
| mouse wheel | `wheel` |
| 2D HUD / ClickGUI drawing | `hud` |
| 3D world overlay (ESP, X-Ray) | `world_drawn`, `render_needs` |
| per-frame logic | `frame`, `tick` |
| player/entity spawn, despawn | `entity_join_level`, `entity_leave_level` |
| chunk load/unload | `chunk_load`, `chunk_unload` |
| world open/close (dimension change) | `world_lifecycle` |
| inventory/offhand (Auto Totem) | `inventory_tick`, `on_equip`, `item_bar` |
| screen mouse press/release (ClickGUI) | `screen_mouse_button_pressed_pre`, `screen_mouse_button_released_pre` |
| Options > Mods list + settings page | `mods`, `settings`, `settings_click` |
| block highlight rims | `visuals` |

Feature mapping against these:
- **Player ESP** -> `render_needs` + `world_drawn` (3D boxes) — supported.
- **X-Ray** -> `world_drawn` geometry, or `visuals` rims for a lighter version — supported,
  the latter limited to ~8 requests / 32 blocks.
- **Auto Totem** -> `inventory_tick` + `on_equip` — supported, no GUI needed.
- **Customizable ClickGUI** -> `hud` (draw) + `screen_mouse_button_pressed_pre` (click) —
  supported. Note Mojang-adjacent risk: this is a client UI, not banned automation.
- **Keybinds** -> `key` with `pressed` covering release too — supported.
- **Config** -> plain files via the settings accessor / `native_config_dir()` (API 26).

---

## 5. What must be fixed on your side

Single hard blocker: **the command executor cannot spawn any process.** Ranked causes:

1. **Git Bash not installed / not on PATH.** Freebuff spawns `bash` on Windows; if
   `C:\Program Files\Git\bin\bash.exe` is absent the spawn fails exactly like this.
   Fix: install Git for Windows, then restart Freebuff Desktop.
2. **Security software blocking process spawn** from the app (EDR/AV).
3. **Stale app state** — restart Freebuff Desktop.

Verify from your own terminal (outside Freebuff):

```bash
bash --version        # must print a version; if not, cause 1
where bash            # Windows: should list a path
```

Once Freebuff's own terminal prints anything for `echo hello`, the rest is automatic.

## 6. Exact build procedure once a shell exists

```bash
# 1. the source
git clone https://github.com/horizon-rust/horizon.git
cd horizon
git rev-parse HEAD            # <- the horizon_revision to pin

# 2. toolchain: Rust 1.90+ (docs), the mixin CLI
cargo fetch --locked
cargo build --locked -p horizon-mixins-cli

# 3. place the bundle and apply + typecheck it
#    (bundle goes in the launcher's native-mods dir, or use the official helper)
python3 native/tools/native.py check meteor_horizon

# 4. package
python3 native/tools/native.py package meteor_horizon
#    -> native/target/meteor_horizon-<version>.hnmod
```

The `.hnmod` is verified with the registry check before it counts as a release:

```bash
cargo horizon-mod check native/target/meteor_horizon-0.1.0.hnmod
```

Install: put the `.hnmod` in `horizon-native-mods/`, let the launcher build the game
(the first native install shows a trust notice and a red warning), then switch it on in
`Options > Mods`. Native mods run with the player's own rights and are **not sandboxed**.
A native mod plays in a game of its own, built for that instance.
