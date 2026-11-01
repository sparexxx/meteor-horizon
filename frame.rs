// Once a frame, with the frame's elapsed time.
//
// This is the clock the Anchor Macro and Auto Totem are paced by, so their
// delays are real milliseconds rather than tick counts. Only one action is
// taken per frame per module, which is what stops a macro from firing twice on
// one input.
fn meteor_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    use crate::game::official_meteor_horizon as m;

    if !game.native_on(m::ID) {
        return;
    }
    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    let ms = elapsed.as_millis().min(1_000) as u32;
    let clock = m::with(|live| {
        live.clock_ms = live.clock_ms.saturating_add(ms);
        live.clock_ms
    });

    let (anchor_on, totem_on, anchor_cfg, totem_cfg) = m::with(|live| {
        (
            live.cfg.is_on(m::Module::AnchorMacro),
            live.cfg.is_on(m::Module::AutoTotem),
            live.cfg.anchor,
            live.cfg.totem,
        )
    });

    // ---------------------------------------------------------- anchor ----
    if anchor_on {
        let since = clock.saturating_sub(m::with(|live| live.last_action_ms));
        // ---- SEAM 3 ------------------------------------------------------
        // `anchor_next` (runtime.rs, unit-tested) decides PlaceObsidian /
        // PlaceAnchor / Charge / Detonate from an `AnchorWorld` snapshot. The
        // snapshot needs, from the game: whether a respawn anchor is within
        // reach and whether it is charged, whether the target position is air,
        // and which of obsidian / anchor / glowstone the hotbar holds.
        //
        // The interaction itself is API 13's `native_use_item_on` (which HEARS
        // a right click; it is a notification, not an action) and API 14's
        // `native_place_block`. To *perform* an interaction the helper has to
        // call the client's own use/attack path; that call is the one
        // signature this session could not read, so nothing is fired here
        // rather than firing a guessed call at the world.
        let _ = (since, anchor_cfg);
    } else {
        m::with(|live| live.phase = m::AnchorPhase::Ready);
    }

    // ----------------------------------------------------------- totem ----
    if totem_on {
        let since = clock.saturating_sub(m::with(|live| live.last_totem_ms));
        if since >= totem_cfg.delay_ms {
            // ---- SEAM 4 --------------------------------------------------
            // `totem_source` (runtime.rs, unit-tested) answers "which slot
            // holds a totem" from an offhand slot, the carried stacks and the
            // health gate, and returns None whenever the offhand already holds
            // a totem — the case that must never touch the inventory.
            //
            // Reading the offhand and moving a stack is API 39's
            // `native_carried_stack` / `native_on_equip`, or API 9's
            // `native_count_items` / `native_take_items` / `native_give_items`.
            // Their exact parameter lists are not in the docs excerpt that was
            // readable here, so the move is not attempted blind.
            m::with(|live| live.last_totem_ms = clock);
        }
    }
}
