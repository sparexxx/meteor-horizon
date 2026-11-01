// X-Ray: the world frame's blocked-outline requests.
//
// `NativeBlockRim` is the renderer's own type — `pos`, `argb`, `width` — the
// same one the block selection outline uses, so these are real depth-tested
// outlines on the world, not a HUD drawing. Horizon validates and bounds them
// itself: only the first eight rims are examined, so the whitelist is capped
// here rather than sending requests that would be dropped.
fn meteor_visuals(
    game: &crate::game::Game<'_>,
    rims: &mut Vec<crate::game::native_hooks::NativeBlockRim>,
    lights: &mut Vec<crate::game::native_hooks::NativeColoredLight>,
) {
    use crate::game::official_meteor_horizon as m;

    let _ = &mut *lights;

    if !game.native_on(m::ID) {
        return;
    }
    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    if !m::with(|live| live.cfg.is_on(m::Module::XRay)) {
        return;
    }

    // The hit list was gathered on the tick thread; this only reads it.
    let (hits, argb) = m::with(|live| (live.xray_hits.clone(), live.cfg.xray.argb));

    // Horizon examines the first eight rims; sending more is wasted work.
    for pos in hits.into_iter().take(8) {
        rims.push(crate::game::native_hooks::NativeBlockRim {
            pos,
            argb,
            width: 1.5,
        });
    }
}
