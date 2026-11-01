// A click on the mod's settings page. The indices match settings.rs exactly.
fn official_meteor_horizon_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::official_meteor_horizon as m;

    if mod_id != m::ID {
        return;
    }
    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    // X-Ray colours to cycle through.
    const SHADES: [u32; 5] = [
        0x90_38_D9_9A,
        0x90_8B_5C_F6,
        0x90_FF_D1_66,
        0x90_4D_A6_FF,
        0x90_FF_7A_B8,
    ];

    m::with(|live| {
        let cfg = &mut live.cfg;
        match button {
            0 => cfg.global_on = !cfg.global_on,
            1 => cfg.set_on(m::Module::Esp, !cfg.enabled[0]),
            2 => cfg.set_on(m::Module::XRay, !cfg.enabled[2]),
            3 => cfg.set_on(m::Module::AnchorMacro, !cfg.enabled[1]),
            4 => cfg.set_on(m::Module::AutoTotem, !cfg.enabled[3]),
            5 => cfg.esp.fill = !cfg.esp.fill,
            6 => cfg.esp.nametags = !cfg.esp.nametags,
            7 => {
                // Steps through a few useful ranges rather than a slider.
                cfg.esp.distance = match cfg.esp.distance as u32 {
                    d if d < 64 => 64.0,
                    d if d < 96 => 96.0,
                    d if d < 128 => 128.0,
                    d if d < 192 => 192.0,
                    _ => 32.0,
                };
            }
            8 => {
                let at = SHADES
                    .iter()
                    .position(|c| *c == cfg.xray.argb)
                    .map(|i| (i + 1) % SHADES.len())
                    .unwrap_or(0);
                cfg.xray.argb = SHADES[at];
            }
            9 => cfg.anchor.detonate = !cfg.anchor.detonate,
            10 => cfg.anchor.charge = !cfg.anchor.charge,
            11 => cfg.totem.always = !cfg.totem.always,
            12 => {
                cfg.totem.health = match cfg.totem.health as u32 {
                    h if h < 6 => 6.0,
                    h if h < 10 => 10.0,
                    h if h < 14 => 14.0,
                    _ => 2.0,
                };
            }
            _ => return,
        }
    });

    m::save(|name, value| game.native_set_setting(m::ID, name, value));
}
