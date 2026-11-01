// Meteor Horizon Lite: a native Horizon mod (horizon-mixins bundle).
//
// This file is both the bundle's Cargo library — so `cargo test` checks the
// logic with no game around it, as Horizon's own native mods do — and the
// module the mixin tool adds to `crates/horizon-client/src/game.rs` as
// `game::official_meteor_horizon`.
//
// It deliberately uses nothing but `std`: a bundle's crate may only name
// crates already present in Horizon's Cargo.lock, and none are needed here.
//
// Everything in this file is pure logic: module registry, keybinds, the saved
// settings model, the ClickGUI's layout and hit-testing, the Anchor Macro's
// step planner, the Auto Totem's slot choice and the ESP target filter. The
// small `*...rs` helpers beside it do the parts that must touch the game.

use std::cell::RefCell;

// ---------------------------------------------------------------- identity --

/// The mod's id: its settings folder, its log name, and the `mod_id` the
/// settings and command hooks receive.
pub const ID: &str = "meteor_horizon";
pub const NAME: &str = "Meteor Horizon Lite";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Player ESP, Anchor Macro, X-Ray, Auto Totem and a \
customizable ClickGUI, each with its own keybind and a global toggle.";

/// `key.keyboard.<name>` is Horizon's key naming (vanilla's translation keys).
pub const KEY_GLOBAL: &str = "meteor.global";
pub const KEY_GUI: &str = "meteor.gui";

pub const DEFAULT_KEY_GLOBAL: &str = "key.keyboard.right.shift";
pub const DEFAULT_KEY_GUI: &str = "key.keyboard.right.control";

// ----------------------------------------------------------------- palette --

/// ARGB, the same shape `Game::native_fill` and `native_text` take.
pub const BG: u32 = 0xF0_12_12_16;
pub const BG_ALT: u32 = 0xFF_1B_1B_22;
pub const BG_HOVER: u32 = 0xFF_26_26_30;
pub const INK: u32 = 0xFF_FF_FF_FF;
pub const INK_DIM: u32 = 0xFF_9A_9A_A6;
pub const ACCENT: u32 = 0xFF_8B_5C_F6;
pub const ACCENT_ON: u32 = 0xFF_38_D9_9A;
pub const DANGER: u32 = 0xFF_E5_48_4D;

// -------------------------------------------------------------- categories --

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Combat,
    Render,
    Movement,
    Player,
    Misc,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Combat,
        Category::Render,
        Category::Movement,
        Category::Player,
        Category::Misc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Category::Combat => "Combat",
            Category::Render => "Render",
            Category::Movement => "Movement",
            Category::Player => "Player",
            Category::Misc => "Misc",
        }
    }
}

// ----------------------------------------------------------------- modules --

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    Esp,
    AnchorMacro,
    XRay,
    AutoTotem,
}

impl Module {
    pub const ALL: [Module; 4] = [
        Module::Esp,
        Module::AnchorMacro,
        Module::XRay,
        Module::AutoTotem,
    ];

    /// The stable id used in settings keys and in `/meteor <id>`.
    pub fn id(self) -> &'static str {
        match self {
            Module::Esp => "esp",
            Module::AnchorMacro => "anchor_macro",
            Module::XRay => "xray",
            Module::AutoTotem => "auto_totem",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Module::Esp => "Player ESP",
            Module::AnchorMacro => "Anchor Macro",
            Module::XRay => "X-Ray",
            Module::AutoTotem => "Auto Totem",
        }
    }

    pub fn category(self) -> Category {
        match self {
            Module::Esp => Category::Render,
            Module::AnchorMacro => Category::Combat,
            Module::XRay => Category::Render,
            Module::AutoTotem => Category::Combat,
        }
    }

    pub fn from_id(id: &str) -> Option<Module> {
        Module::ALL.into_iter().find(|m| m.id() == id)
    }

    /// The keybind's name in the settings page (`NativeMod::keys`).
    pub fn key_name(self) -> &'static str {
        match self {
            Module::Esp => "meteor.esp",
            Module::AnchorMacro => "meteor.anchor_macro",
            Module::XRay => "meteor.xray",
            Module::AutoTotem => "meteor.auto_totem",
        }
    }

    pub fn default_key(self) -> &'static str {
        match self {
            Module::Esp => "key.keyboard.g",
            Module::AnchorMacro => "key.keyboard.v",
            Module::XRay => "key.keyboard.x",
            Module::AutoTotem => "key.keyboard.t",
        }
    }
}

/// The rows `Options > Mods` lists for this mod: name, label, default key.
pub fn listed_keys() -> Vec<[String; 3]> {
    let mut keys = vec![
        [
            KEY_GLOBAL.to_owned(),
            "Global toggle".to_owned(),
            DEFAULT_KEY_GLOBAL.to_owned(),
        ],
        [
            KEY_GUI.to_owned(),
            "ClickGUI".to_owned(),
            DEFAULT_KEY_GUI.to_owned(),
        ],
    ];
    for m in Module::ALL {
        keys.push([
            m.key_name().to_owned(),
            m.label().to_owned(),
            m.default_key().to_owned(),
        ]);
    }
    keys
}

// ---------------------------------------------------------------- settings --

/// ESP colours and shape. Colours are ARGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EspSettings {
    pub argb: u32,
    pub fill: bool,
    pub thickness: f32,
    /// Blocks; targets farther than this are not drawn.
    pub distance: f32,
    pub nametags: bool,
}

impl Default for EspSettings {
    fn default() -> Self {
        Self {
            argb: 0xFF_8B_5C_F6,
            fill: true,
            thickness: 1.5,
            distance: 128.0,
            nametags: true,
        }
    }
}

/// The X-Ray whitelist, as block ids without the `minecraft:` namespace.
#[derive(Clone, Debug, PartialEq)]
pub struct XraySettings {
    pub argb: u32,
    pub blocks: Vec<String>,
}

impl Default for XraySettings {
    fn default() -> Self {
        Self {
            argb: 0x90_38_D9_9A,
            blocks: DEFAULT_XRAY_BLOCKS.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

pub const DEFAULT_XRAY_BLOCKS: [&str; 19] = [
    "coal_ore",
    "deepslate_coal_ore",
    "iron_ore",
    "deepslate_iron_ore",
    "copper_ore",
    "deepslate_copper_ore",
    "gold_ore",
    "deepslate_gold_ore",
    "redstone_ore",
    "deepslate_redstone_ore",
    "lapis_ore",
    "deepslate_lapis_ore",
    "diamond_ore",
    "deepslate_diamond_ore",
    "emerald_ore",
    "deepslate_emerald_ore",
    "nether_gold_ore",
    "nether_quartz_ore",
    "ancient_debris",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorSettings {
    /// Milliseconds between the macro's steps; the anti-double-fire guard.
    pub delay_ms: u32,
    /// Detonate instead of leaving the anchor charged.
    pub detonate: bool,
    /// Charge the anchor with glowstone before detonating.
    pub charge: bool,
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            delay_ms: 120,
            detonate: true,
            charge: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TotemSettings {
    /// Milliseconds between offhand refills, so a full sweep cannot spin.
    pub delay_ms: u32,
    /// Only refill when health is at or below this many half-hearts.
    pub health: f32,
    /// Refill whenever the offhand is empty, regardless of health.
    pub always: bool,
}

impl Default for TotemSettings {
    fn default() -> Self {
        Self {
            delay_ms: 250,
            health: 10.0,
            always: true,
        }
    }
}

/// The whole saved state. `read` pulls one value, `stored` writes them all.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub global_on: bool,
    pub enabled: [bool; 4],
    pub esp: EspSettings,
    pub xray: XraySettings,
    pub anchor: AnchorSettings,
    pub totem: TotemSettings,
    pub gui_category: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global_on: true,
            enabled: [false; 4],
            esp: EspSettings::default(),
            xray: XraySettings::default(),
            anchor: AnchorSettings::default(),
            totem: TotemSettings::default(),
            gui_category: 1,
        }
    }
}

fn read_bool(v: Option<String>, fallback: bool) -> bool {
    match v {
        Some(s) => match s.trim() {
            "true" => true,
            "false" => false,
            _ => fallback,
        },
        None => fallback,
    }
}

fn read_f32(v: Option<String>, fallback: f32) -> f32 {
    v.and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|f| f.is_finite())
        .unwrap_or(fallback)
}

fn read_u32(v: Option<String>, fallback: u32) -> u32 {
    v.and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(fallback)
}

/// ARGB written as 8 hex digits, e.g. `ff8b5cf6`.
fn read_argb(v: Option<String>, fallback: u32) -> u32 {
    v.and_then(|s| u32::from_str_radix(s.trim().trim_start_matches('#'), 16).ok())
        .unwrap_or(fallback)
}

pub fn argb_hex(argb: u32) -> String {
    format!("{argb:08x}")
}

impl Config {
    /// Reads every value through `get`, which is
    /// `|name| game.native_setting(ID, name)` in the game.
    pub fn read(mut get: impl FnMut(&str) -> Option<String>) -> Self {
        let d = Config::default();
        let mut enabled = [false; 4];
        for (i, m) in Module::ALL.into_iter().enumerate() {
            enabled[i] = read_bool(get(&format!("module.{}", m.id())), d.enabled[i]);
        }
        let blocks = match get("xray.blocks") {
            Some(s) => {
                let list: Vec<String> = s
                    .split(',')
                    .map(|b| b.trim().trim_start_matches("minecraft:").to_owned())
                    .filter(|b| !b.is_empty())
                    .collect();
                if list.is_empty() {
                    d.xray.blocks.clone()
                } else {
                    list
                }
            }
            None => d.xray.blocks.clone(),
        };
        Self {
            global_on: read_bool(get("global"), d.global_on),
            enabled,
            esp: EspSettings {
                argb: read_argb(get("esp.color"), d.esp.argb),
                fill: read_bool(get("esp.fill"), d.esp.fill),
                thickness: read_f32(get("esp.thickness"), d.esp.thickness).clamp(0.5, 6.0),
                distance: read_f32(get("esp.distance"), d.esp.distance).clamp(8.0, 512.0),
                nametags: read_bool(get("esp.nametags"), d.esp.nametags),
            },
            xray: XraySettings {
                argb: read_argb(get("xray.color"), d.xray.argb),
                blocks,
            },
            anchor: AnchorSettings {
                delay_ms: read_u32(get("anchor.delay"), d.anchor.delay_ms).clamp(0, 5000),
                detonate: read_bool(get("anchor.detonate"), d.anchor.detonate),
                charge: read_bool(get("anchor.charge"), d.anchor.charge),
            },
            totem: TotemSettings {
                delay_ms: read_u32(get("totem.delay"), d.totem.delay_ms).clamp(0, 5000),
                health: read_f32(get("totem.health"), d.totem.health).clamp(0.0, 20.0),
                always: read_bool(get("totem.always"), d.totem.always),
            },
            gui_category: (read_u32(get("gui.category"), d.gui_category as u32) as usize)
                .min(Category::ALL.len() - 1),
        }
    }

    /// Everything to persist, as settings names and values.
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        let mut out = Vec::with_capacity(24);
        out.push(("global", self.global_on.to_string()));
        for (i, m) in Module::ALL.into_iter().enumerate() {
            match m {
                Module::Esp => out.push(("module.esp", self.enabled[i].to_string())),
                Module::AnchorMacro => {
                    out.push(("module.anchor_macro", self.enabled[i].to_string()))
                }
                Module::XRay => out.push(("module.xray", self.enabled[i].to_string())),
                Module::AutoTotem => out.push(("module.auto_totem", self.enabled[i].to_string())),
            }
        }
        out.push(("esp.color", argb_hex(self.esp.argb)));
        out.push(("esp.fill", self.esp.fill.to_string()));
        out.push(("esp.thickness", format!("{:.2}", self.esp.thickness)));
        out.push(("esp.distance", format!("{:.1}", self.esp.distance)));
        out.push(("esp.nametags", self.esp.nametags.to_string()));
        out.push(("xray.color", argb_hex(self.xray.argb)));
        out.push(("xray.blocks", self.xray.blocks.join(",")));
        out.push(("anchor.delay", self.anchor.delay_ms.to_string()));
        out.push(("anchor.detonate", self.anchor.detonate.to_string()));
        out.push(("anchor.charge", self.anchor.charge.to_string()));
        out.push(("totem.delay", self.totem.delay_ms.to_string()));
        out.push(("totem.health", format!("{:.1}", self.totem.health)));
        out.push(("totem.always", self.totem.always.to_string()));
        out.push(("gui.category", self.gui_category.to_string()));
        out
    }

    pub fn is_on(&self, m: Module) -> bool {
        self.global_on && self.enabled[Module::ALL.iter().position(|x| *x == m).unwrap()]
    }

    /// Turning a module on is refused while the global toggle is off, so the
    /// per-module state is never mistaken for "running".
    pub fn set_on(&mut self, m: Module, on: bool) {
        let i = Module::ALL.iter().position(|x| *x == m).unwrap();
        self.enabled[i] = on;
    }

    pub fn toggle(&mut self, m: Module) -> bool {
        let i = Module::ALL.iter().position(|x| *x == m).unwrap();
        self.enabled[i] = !self.enabled[i];
        self.enabled[i]
    }

    /// The global toggle clears nothing: it only gates `is_on`.
    pub fn set_global(&mut self, on: bool) {
        self.global_on = on;
    }

    pub fn xray_wants(&self, block_id: &str) -> bool {
        let bare = block_id.trim_start_matches("minecraft:");
        self.xray.blocks.iter().any(|b| b == bare)
    }
}

// ---------------------------------------------------------------- clickgui --

pub const PANEL_W: f32 = 430.0;
pub const PANEL_H: f32 = 268.0;
pub const CAT_W: f32 = 92.0;
pub const ROW_H: f32 = 20.0;
pub const HEADER_H: f32 = 26.0;
pub const PAD: f32 = 6.0;

/// What a click at the pointer landed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    None,
    Header,
    Close,
    Search,
    Category(usize),
    Toggle(Module),
    Settings(Module),
}

/// The ClickGUI's own state: open or not, where it is, what it shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Gui {
    pub open: bool,
    pub category: usize,
    pub search: String,
    pub selected: Option<Module>,
    pub scroll: f32,
    /// GUI-pixel offset of the panel's top-left corner.
    pub pos: [f32; 2],
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            open: false,
            category: 1,
            search: String::new(),
            selected: None,
            scroll: 0.0,
            pos: [60.0, 60.0],
        }
    }
}

/// The panel's rectangle: `[x, y, width, height]`.
pub fn panel_rect(pos: [f32; 2]) -> [f32; 4] {
    [pos[0], pos[1], PANEL_W, PANEL_H]
}

/// Each category button's rectangle, down the left column.
pub fn category_rect(pos: [f32; 2], index: usize) -> [f32; 4] {
    [
        pos[0] + PAD,
        pos[1] + HEADER_H + PAD + index as f32 * (ROW_H + 2.0),
        CAT_W - PAD,
        ROW_H,
    ]
}

/// A module row in the visible list, top to bottom.
pub fn row_rect(pos: [f32; 2], index: usize) -> [f32; 4] {
    [
        pos[0] + CAT_W + PAD,
        pos[1] + HEADER_H + PAD + index as f32 * ROW_H,
        PANEL_W - CAT_W - 2.0 * PAD,
        ROW_H - 2.0,
    ]
}

pub fn close_rect(pos: [f32; 2]) -> [f32; 4] {
    [pos[0] + PANEL_W - 22.0, pos[1] + 4.0, 18.0, 18.0]
}

pub fn search_rect(pos: [f32; 2]) -> [f32; 4] {
    [
        pos[0] + CAT_W + PAD,
        pos[1] + PANEL_H - 24.0,
        PANEL_W - CAT_W - 2.0 * PAD,
        18.0,
    ]
}

pub fn contains(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0]
        && at[0] <= rect[0] + rect[2]
        && at[1] >= rect[1]
        && at[1] <= rect[1] + rect[3]
}

impl Gui {
    /// The modules the list shows: the open category, narrowed by the search
    /// text (matched against the label and the id, case-insensitively).
    pub fn visible(&self) -> Vec<Module> {
        let needle = self.search.trim().to_lowercase();
        let category = Category::ALL[self.category.min(Category::ALL.len() - 1)];
        Module::ALL
            .into_iter()
            .filter(|m| m.category() == category)
            .filter(|m| {
                needle.is_empty()
                    || m.label().to_lowercase().contains(&needle)
                    || m.id().contains(&needle)
            })
            .collect()
    }

    /// Which control a click at `at` hits. Search is treated as a text box.
    pub fn hit(&self, at: [f32; 2]) -> Hit {
        if !self.open {
            return Hit::None;
        }
        if contains(close_rect(self.pos), at) {
            return Hit::Close;
        }
        if contains(search_rect(self.pos), at) {
            return Hit::Search;
        }
        for i in 0..Category::ALL.len() {
            if contains(category_rect(self.pos, i), at) {
                return Hit::Category(i);
            }
        }
        for (i, m) in self.visible().into_iter().enumerate() {
            let r = row_rect(self.pos, i);
            if contains(r, at) {
                // The right-hand strip toggles; the rest opens the settings.
                if at[0] >= r[0] + r[2] - 34.0 {
                    return Hit::Toggle(m);
                }
                return Hit::Settings(m);
            }
        }
        if contains(panel_rect(self.pos), at) {
            return Hit::Header;
        }
        Hit::None
    }

    /// Applies a left click and reports the module whose state changed, if any.
    pub fn click(&mut self, at: [f32; 2], cfg: &mut Config) -> Option<Module> {
        match self.hit(at) {
            Hit::Close => {
                self.open = false;
                None
            }
            Hit::Category(i) => {
                self.category = i;
                cfg.gui_category = i;
                self.scroll = 0.0;
                None
            }
            Hit::Toggle(m) => {
                cfg.toggle(m);
                Some(m)
            }
            Hit::Settings(m) => {
                self.selected = if self.selected == Some(m) { None } else { Some(m) };
                None
            }
            Hit::Header => {
                self.pos = at;
                None
            }
            Hit::Search | Hit::None => None,
        }
    }

    pub fn type_char(&mut self, c: char) {
        if self.search.chars().count() < 24 && !c.is_control() {
            self.search.push(c);
        }
    }

    pub fn backspace(&mut self) {
        self.search.pop();
    }
}

// ------------------------------------------------------------- anchor macro --

/// What the Anchor Macro wants done next. The helper executes it through the
/// game's interaction API; the decision itself lives here, so it is testable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorStep {
    Idle,
    /// Place obsidian where the anchor should sit.
    PlaceObsidian,
    /// Place a respawn anchor on the obsidian.
    PlaceAnchor,
    /// Use glowstone on the anchor to charge it.
    Charge,
    /// Hit the charged anchor to detonate it.
    Detonate,
    /// Wait this many milliseconds, then ask again.
    Wait(u32),
    /// Nothing can be done: no obsidian, no anchor, no glowstone, or no room.
    Missing(&'static str),
}

/// The macro's phase, kept across ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorPhase {
    Ready,
    PlacedObsidian,
    PlacedAnchor,
    Charged,
}

/// The world facts the planner needs, gathered by the helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorWorld {
    /// The anchor to work on is within reach.
    pub anchor_in_reach: bool,
    /// The anchor there is already charged.
    pub anchor_charged: bool,
    /// The target block position is air, so obsidian can go there.
    pub spot_free: bool,
    pub has_obsidian: bool,
    pub has_anchor: bool,
    pub has_glowstone: bool,
    /// Milliseconds since the last action; the anti-spam guard.
    pub since_last_ms: u32,
}

/// Decides the next step. `charge`/`detonate` come from the module's settings.
pub fn anchor_next(phase: AnchorPhase, w: AnchorWorld, s: AnchorSettings) -> (AnchorPhase, AnchorStep) {
    if w.since_last_ms < s.delay_ms {
        return (phase, AnchorStep::Wait(s.delay_ms - w.since_last_ms));
    }
    match phase {
        AnchorPhase::Ready => {
            if w.anchor_in_reach {
                // Something is already there: skip straight to charging.
                return if w.anchor_charged {
                    (AnchorPhase::Charged, AnchorStep::Detonate)
                } else if s.charge {
                    (AnchorPhase::PlacedAnchor, AnchorStep::Charge)
                } else {
                    (AnchorPhase::PlacedAnchor, AnchorStep::Detonate)
                };
            }
            if !w.spot_free {
                return (phase, AnchorStep::Missing("no free spot for the anchor"));
            }
            if !w.has_obsidian {
                return (phase, AnchorStep::Missing("no obsidian"));
            }
            (AnchorPhase::PlacedObsidian, AnchorStep::PlaceObsidian)
        }
        AnchorPhase::PlacedObsidian => {
            if !w.has_anchor {
                return (phase, AnchorStep::Missing("no respawn anchor"));
            }
            (AnchorPhase::PlacedAnchor, AnchorStep::PlaceAnchor)
        }
        AnchorPhase::PlacedAnchor => {
            if w.anchor_charged {
                return (AnchorPhase::Charged, AnchorStep::Detonate);
            }
            if !s.charge {
                return (phase, AnchorStep::Missing("anchor is not charged"));
            }
            if !w.has_glowstone {
                return (phase, AnchorStep::Missing("no glowstone"));
            }
            (AnchorPhase::Charged, AnchorStep::Charge)
        }
        AnchorPhase::Charged => {
            if !s.detonate {
                return (AnchorPhase::Ready, AnchorStep::Idle);
            }
            if !w.anchor_in_reach {
                return (AnchorPhase::Ready, AnchorStep::Idle);
            }
            (AnchorPhase::Ready, AnchorStep::Detonate)
        }
    }
}

// -------------------------------------------------------------- auto totem --

/// One carried stack, as the offhand check sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The slot index in the player's inventory.
    pub index: usize,
    /// `minecraft:totem_of_undying`, or another item id.
    pub item: &'static str,
    pub count: u32,
}

pub const TOTEM: &str = "minecraft:totem_of_undying";

/// Whether the offhand already holds a totem: the common case, and the one
/// that must cause no inventory traffic at all.
pub fn offhand_has_totem(offhand: Option<Slot>) -> bool {
    matches!(offhand, Some(s) if s.item == TOTEM && s.count > 0)
}

/// The slot to move into the offhand, or `None` when nothing should happen.
///
/// Rules: never move when the offhand already has a totem; never move when
/// the health gate is not met and `always` is off; pick the first totem stack;
/// refuse when the inventory is empty of totems.
pub fn totem_source(offhand: Option<Slot>, inventory: &[Slot], health: f32, s: TotemSettings) -> Option<usize> {
    if offhand_has_totem(offhand) {
        return None;
    }
    if !s.always && health > s.health {
        return None;
    }
    inventory
        .iter()
        .find(|slot| slot.item == TOTEM && slot.count > 0)
        .map(|slot| slot.index)
}

// --------------------------------------------------------------------- esp --

/// One drawable target, resolved from the world by the helper.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub health: f32,
}

/// Filters and sorts targets: the local player is excluded by the caller, and
/// anything past the configured range is dropped here. Sorted nearest first so
/// the nearest are drawn even when the list is capped.
pub fn esp_targets(targets: &[Target], local: [f64; 3], s: &EspSettings, cap: usize) -> Vec<&Target> {
    let max2 = (s.distance as f64) * (s.distance as f64);
    let mut kept: Vec<(f64, &Target)> = targets
        .iter()
        .filter_map(|t| {
            let d2 = (t.x - local[0]).powi(2) + (t.y - local[1]).powi(2) + (t.z - local[2]).powi(2);
            (d2 <= max2).then_some((d2, t))
        })
        .collect();
    kept.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    kept.truncate(cap);
    kept.into_iter().map(|(_, t)| t).collect()
}

pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// ARGB as the renderer wants it: linear RGBA, 0..1.
pub fn argb_to_rgba(argb: u32) -> [f32; 4] {
    let a = ((argb >> 24) & 0xFF) as f32 / 255.0;
    let r = ((argb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((argb >> 8) & 0xFF) as f32 / 255.0;
    let b = (argb & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}

/// Marks the renderer's cached GPU objects as belonging to a dead world.
pub fn reset_renderer() {
    with(|live| {
        live.renderer_epoch = live.renderer_epoch.wrapping_add(1);
        live.targets.clear();
        live.xray_hits.clear();
    });
}

/// The epoch the renderer should key its buffers on.
pub fn renderer_epoch() -> u32 {
    with(|live| live.renderer_epoch)
}

// ------------------------------------------------------------ live state ----

/// The mod's live state. `thread_local!` is how Horizon's own native mods keep
/// state, so it never touches the game's structs.
#[derive(Default)]
pub struct Live {
    pub cfg: Config,
    /// False until the settings were read once from disk.
    pub loaded: bool,
    pub gui: Gui,
    pub phase: AnchorPhase,
    pub last_action_ms: u32,
    pub last_totem_ms: u32,
    pub clock_ms: u32,
    /// Bumped when the world changes, so the renderer knows its GPU buffers
    /// belong to a world that is gone and must be rebuilt.
    pub renderer_epoch: u32,
    /// The ESP targets gathered this tick, so the renderer never scans.
    pub targets: Vec<Target>,
    /// The X-Ray block positions gathered this tick.
    pub xray_hits: Vec<[i32; 3]>,
}

impl Default for AnchorPhase {
    fn default() -> Self {
        AnchorPhase::Ready
    }
}

thread_local! {
    static LIVE: RefCell<Live> = RefCell::new(Live::default());
}

/// Runs `f` with the live state. The helpers use this instead of globals so
/// the state is per-thread and drops with the client.
pub fn with<R>(f: impl FnOnce(&mut Live) -> R) -> R {
    LIVE.with(|c| f(&mut c.borrow_mut()))
}

/// Loads the settings once, on the first hook that runs.
pub fn ensure_loaded(mut get: impl FnMut(&str) -> Option<String>) {
    with(|live| {
        if !live.loaded {
            live.cfg = Config::read(&mut get);
            live.gui.category = live.cfg.gui_category;
            live.gui.open = false;
            live.loaded = true;
        }
    });
}

/// Writes back everything the model holds.
pub fn save(mut set: impl FnMut(&str, &str)) {
    with(|live| {
        for (name, value) in live.cfg.stored() {
            set(name, &value);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_module_has_a_unique_id_and_key() {
        let mut ids = Vec::new();
        for m in Module::ALL {
            assert!(!ids.contains(&m.id()), "duplicate module id {}", m.id());
            ids.push(m.id());
            assert!(m.default_key().starts_with("key.keyboard."));
            assert_eq!(Module::from_id(m.id()), Some(m));
        }
    }

    #[test]
    fn listed_keys_cover_the_gui_global_and_every_module() {
        let keys = listed_keys();
        assert_eq!(keys.len(), 2 + Module::ALL.len());
        assert_eq!(keys[0][0], KEY_GLOBAL);
        assert_eq!(keys[1][0], KEY_GUI);
    }

    #[test]
    fn defaults_are_off_and_save_round_trips() {
        let cfg = Config::default();
        assert!(cfg.global_on);
        assert!(Module::ALL.iter().all(|m| !cfg.is_on(*m)));

        let stored = cfg.stored();
        let back = Config::read(|name| {
            stored
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.clone())
        });
        assert_eq!(back, cfg);
    }

    #[test]
    fn global_toggle_disables_modules_without_clearing_them() {
        let mut cfg = Config::default();
        assert!(cfg.toggle(Module::Esp));
        assert!(cfg.is_on(Module::Esp));

        cfg.set_global(false);
        assert!(!cfg.is_on(Module::Esp), "global off must gate the module");
        // The per-module state survives, so turning it back on restores it.
        assert!(cfg.enabled[0]);

        cfg.set_global(true);
        assert!(cfg.is_on(Module::Esp));
    }

    #[test]
    fn malformed_settings_fall_back_instead_of_panicking() {
        let cfg = Config::read(|name| match name {
            "esp.color" => Some("nonsense".to_owned()),
            "esp.thickness" => Some("-4".to_owned()),
            "module.esp" => Some("maybe".to_owned()),
            "gui.category" => Some("99".to_owned()),
            _ => None,
        });
        assert_eq!(cfg.esp.argb, EspSettings::default().argb);
        assert_eq!(cfg.esp.thickness, 0.5);
        assert!(!cfg.enabled[0]);
        assert!(cfg.gui_category < Category::ALL.len());
    }

    #[test]
    fn xray_whitelist_matches_with_or_without_namespace() {
        let cfg = Config::default();
        assert!(cfg.xray_wants("diamond_ore"));
        assert!(cfg.xray_wants("minecraft:diamond_ore"));
        assert!(!cfg.xray_wants("minecraft:stone"));
        assert!(!cfg.xray_wants(""));
    }

    #[test]
    fn gui_filters_by_category_then_search() {
        let mut gui = Gui::default();
        gui.category = 1; // Render
        let visible = gui.visible();
        assert!(visible.contains(&Module::Esp));
        assert!(visible.contains(&Module::XRay));
        assert!(!visible.contains(&Module::AutoTotem));

        gui.search = "x-r".to_owned();
        assert_eq!(gui.visible(), vec![Module::XRay]);

        gui.search = "zzzz".to_owned();
        assert!(gui.visible().is_empty());
    }

    #[test]
    fn gui_hit_testing_finds_rows_and_their_toggle_strip() {
        let mut gui = Gui::default();
        gui.open = true;
        gui.category = 1;
        let row = row_rect(gui.pos, 0);
        // Left part of the row opens settings.
        assert_eq!(gui.hit([row[0] + 4.0, row[1] + 4.0]), Hit::Settings(Module::Esp));
        // Right strip toggles.
        assert_eq!(
            gui.hit([row[0] + row[2] - 4.0, row[1] + 4.0]),
            Hit::Toggle(Module::Esp)
        );
        // The close button closes.
        let close = close_rect(gui.pos);
        assert_eq!(gui.hit([close[0] + 1.0, close[1] + 1.0]), Hit::Close);
        // Clicking the close button actually closes and changes no module.
        let mut cfg = Config::default();
        assert_eq!(gui.click([close[0] + 1.0, close[1] + 1.0], &mut cfg), None);
        assert!(!gui.open);
    }

    #[test]
    fn gui_toggle_click_changes_exactly_one_module() {
        let mut gui = Gui::default();
        gui.open = true;
        gui.category = 1;
        let mut cfg = Config::default();
        let row = row_rect(gui.pos, 0);
        let hit = [row[0] + row[2] - 4.0, row[1] + 4.0];
        assert_eq!(gui.click(hit, &mut cfg), Some(Module::Esp));
        assert!(cfg.enabled[0]);
        assert!(!cfg.enabled[2], "X-Ray must be untouched");
        // Clicking again turns it back off.
        assert_eq!(gui.click(hit, &mut cfg), Some(Module::Esp));
        assert!(!cfg.enabled[0]);
    }

    #[test]
    fn search_box_accepts_typing_and_backspace() {
        let mut gui = Gui::default();
        for c in "esp".chars() {
            gui.type_char(c);
        }
        assert_eq!(gui.search, "esp");
        gui.backspace();
        assert_eq!(gui.search, "es");
        // Control characters never land in the box.
        gui.type_char('\n');
        assert_eq!(gui.search, "es");
    }

    fn world(since: u32) -> AnchorWorld {
        AnchorWorld {
            anchor_in_reach: false,
            anchor_charged: false,
            spot_free: true,
            has_obsidian: true,
            has_anchor: true,
            has_glowstone: true,
            since_last_ms: since,
        }
    }

    #[test]
    fn anchor_runs_place_charge_detonate_in_order() {
        let s = AnchorSettings::default();
        let (p, step) = anchor_next(AnchorPhase::Ready, world(1000), s);
        assert_eq!(step, AnchorStep::PlaceObsidian);
        assert_eq!(p, AnchorPhase::PlacedObsidian);

        let (p, step) = anchor_next(p, world(1000), s);
        assert_eq!(step, AnchorStep::PlaceAnchor);
        assert_eq!(p, AnchorPhase::PlacedAnchor);

        let (p, step) = anchor_next(p, world(1000), s);
        assert_eq!(step, AnchorStep::Charge);
        assert_eq!(p, AnchorPhase::Charged);

        let mut reached = world(1000);
        reached.anchor_in_reach = true;
        reached.anchor_charged = true;
        let (p, step) = anchor_next(p, reached, s);
        assert_eq!(step, AnchorStep::Detonate);
        assert_eq!(p, AnchorPhase::Ready);
    }

    #[test]
    fn anchor_delay_blocks_a_second_action() {
        let s = AnchorSettings::default();
        let (_, step) = anchor_next(AnchorPhase::Ready, world(10), s);
        assert_eq!(step, AnchorStep::Wait(s.delay_ms - 10));
    }

    #[test]
    fn anchor_refuses_when_a_required_item_is_missing() {
        let s = AnchorSettings::default();
        let mut w = world(1000);
        w.has_obsidian = false;
        assert!(matches!(anchor_next(AnchorPhase::Ready, w, s).1, AnchorStep::Missing(_)));

        let mut w = world(1000);
        w.has_anchor = false;
        assert!(matches!(
            anchor_next(AnchorPhase::PlacedObsidian, w, s).1,
            AnchorStep::Missing(_)
        ));

        // An uncharged anchor with charge disabled is left alone.
        let mut w = world(1000);
        w.anchor_in_reach = true;
        w.anchor_charged = false;
        let s2 = AnchorSettings {
            charge: false,
            ..AnchorSettings::default()
        };
        assert_eq!(
            anchor_next(AnchorPhase::PlacedAnchor, w, s2).1,
            AnchorStep::Missing("anchor is not charged")
        );
    }

    #[test]
    fn anchor_reuses_an_anchor_already_in_reach() {
        let s = AnchorSettings::default();
        let mut w = world(1000);
        w.anchor_in_reach = true;
        w.anchor_charged = true;
        let (_, step) = anchor_next(AnchorPhase::Ready, w, s);
        assert_eq!(step, AnchorStep::Detonate, "must not re-place the anchor");
    }

    fn totem_slot(index: usize) -> Slot {
        Slot {
            index,
            item: TOTEM,
            count: 1,
        }
    }

    #[test]
    fn auto_totem_does_nothing_when_offhand_is_full() {
        let s = TotemSettings::default();
        let inv = [totem_slot(9)];
        // Even at one heart, an offhand totem means no inventory traffic.
        assert_eq!(totem_source(Some(totem_slot(40)), &inv, 1.0, s), None);
    }

    #[test]
    fn auto_totem_moves_the_first_totem_into_an_empty_offhand() {
        let s = TotemSettings::default();
        let inv = [
            Slot {
                index: 9,
                item: "minecraft:diamond",
                count: 64,
            },
            totem_slot(10),
            totem_slot(11),
        ];
        assert_eq!(totem_source(None, &inv, 1.0, s), Some(10));
    }

    #[test]
    fn auto_totem_handles_an_empty_inventory() {
        let s = TotemSettings::default();
        assert_eq!(totem_source(None, &[], 20.0, s), None);
    }

    #[test]
    fn auto_totem_respects_the_health_gate() {
        let s = TotemSettings {
            always: false,
            health: 10.0,
            delay_ms: 250,
        };
        let inv = [totem_slot(9)];
        assert_eq!(totem_source(None, &inv, 18.0, s), None, "too healthy to refill");
        assert_eq!(totem_source(None, &inv, 8.0, s), Some(9));
    }

    #[test]
    fn esp_drops_far_targets_and_sorts_nearest_first() {
        let s = EspSettings::default();
        let targets = [
            Target {
                name: "far".to_owned(),
                x: 400.0,
                y: 0.0,
                z: 0.0,
                health: 20.0,
            },
            Target {
                name: "near".to_owned(),
                x: 3.0,
                y: 0.0,
                z: 0.0,
                health: 20.0,
            },
            Target {
                name: "mid".to_owned(),
                x: 10.0,
                y: 0.0,
                z: 0.0,
                health: 20.0,
            },
        ];
        let kept = esp_targets(&targets, [0.0; 3], &s, 8);
        assert_eq!(
            kept.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            vec!["near", "mid"]
        );
    }

    #[test]
    fn esp_cap_keeps_the_nearest_targets() {
        let s = EspSettings::default();
        let targets: Vec<Target> = (1..=10)
            .map(|i| Target {
                name: format!("p{i}"),
                x: i as f64,
                y: 0.0,
                z: 0.0,
                health: 20.0,
            })
            .collect();
        let kept = esp_targets(&targets, [0.0; 3], &s, 3);
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].name, "p1", "nearest kept, farthest dropped");
    }

    #[test]
    fn argb_becomes_rgba() {
        assert_eq!(argb_to_rgba(0xFF_00_00_00), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(argb_to_rgba(0x00_FF_FF_FF), [1.0, 1.0, 1.0, 0.0]);
        let half = argb_to_rgba(0x80_80_80_80);
        assert!((half[3] - 128.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn distance_is_euclidean() {
        assert!((distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]) - 5.0).abs() < 1e-9);
    }
}
