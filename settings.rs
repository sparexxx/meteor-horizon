// The mod's settings page in Options > Mods. The button order here is the
// contract `click.rs` relies on: index `i` in `official_meteor_horizon_page`
// is index `i` in `official_meteor_horizon_click`.
fn official_meteor_horizon_page(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use crate::game::official_meteor_horizon as m;

    if mod_id != m::ID {
        return;
    }
    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    let on = |b: bool| if b { "ON" } else { "OFF" };
    m::with(|live| {
        let cfg = &live.cfg;
        buttons.push(format!("Client: {}", on(cfg.global_on)));
        buttons.push(format!("Player ESP: {}", on(cfg.enabled[0])));
        buttons.push(format!("X-Ray: {}", on(cfg.enabled[2])));
        buttons.push(format!("Anchor Macro: {}", on(cfg.enabled[1])));
        buttons.push(format!("Auto Totem: {}", on(cfg.enabled[3])));
        buttons.push(format!("ESP fill: {}", on(cfg.esp.fill)));
        buttons.push(format!("ESP nametags: {}", on(cfg.esp.nametags)));
        buttons.push(format!("ESP range: {:.0} blocks", cfg.esp.distance));
        buttons.push(format!("X-Ray colour: {}", m::argb_hex(cfg.xray.argb)));
        buttons.push(format!("Anchor detonate: {}", on(cfg.anchor.detonate)));
        buttons.push(format!("Anchor charge: {}", on(cfg.anchor.charge)));
        buttons.push(format!("Totem ignores health: {}", on(cfg.totem.always)));
        buttons.push(format!("Totem health: {:.0}", cfg.totem.health));
    });
}
