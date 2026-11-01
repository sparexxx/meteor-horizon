// Once a tick: refresh the cached world snapshot.
//
// The expensive work (finding players, scanning for ores) happens here at 20 Hz
// and never in the render pass, so a frame never walks the world. The renderer
// then only reads the cache. Everything gathered here is dropped on a world
// change by world_lifecycle.rs, so a dimension switch cannot leave stale
// positions behind.
fn meteor_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::official_meteor_horizon as m;

    if !game.native_on(m::ID) {
        return;
    }
    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    // Not in a world: nothing to cache, and the previous world's targets are
    // meaningless.
    let Some(player) = game.native_player() else {
        m::with(|live| {
            live.targets.clear();
            live.xray_hits.clear();
        });
        return;
    };

    let (esp_on, xray_on) = m::with(|live| {
        (live.cfg.is_on(m::Module::Esp), live.cfg.is_on(m::Module::XRay))
    });

    if esp_on {
        let (local, settings) = m::with(|live| {
            (
                [player.x, player.y, player.z],
                live.cfg.esp,
            )
        });
        // The helper `collect_targets` lives in runtime.rs so it can be unit
        // tested; the world query that feeds it is the one game-facing call
        // that has to be checked against native_hooks.rs (see README).
        let gathered = collect_targets(game);
        let kept: Vec<m::Target> = m::esp_targets(&gathered, local, &settings, 64)
            .into_iter()
            .cloned()
            .collect();
        m::with(|live| live.targets = kept);
    } else {
        m::with(|live| live.targets.clear());
    }

    if xray_on {
        let hits = collect_ores(game, &player);
        m::with(|live| live.xray_hits = hits);
    } else {
        m::with(|live| live.xray_hits.clear());
    }

    // ---- SEAM 1: the world queries -------------------------------------
    // These two are the only places a game-world read is needed, and both need
    // one signature confirmed from the source, which this session could not
    // read past the fetch limit:
    //
    //   * other players: `native_hooks.rs` has no listed client-side entity
    //     iterator. The official mods all read only `native_player()`. The
    //     likely answers are a `Game::native_entities()`/`native_players()`
    //     helper, or reading the client's entity table directly from the mixin
    //     (a helper is compiled inside its target function, so private fields
    //     are reachable).
    //   * ores: `native_block` (API 20) reads one block; there is no listed
    //     bulk scan, so a 16x16x16 walk per tick around the player is the
    //     shape to write once that helper's signature is known.
    //
    // Both return nothing until then, so ESP and X-Ray draw nothing rather
    // than drawing wrong boxes. The filtering, sorting and capping they feed
    // are implemented and unit-tested in runtime.rs.
    fn collect_targets(_game: &crate::game::Game<'_>) -> Vec<crate::game::official_meteor_horizon::Target> {
        Vec::new()
    }

    fn collect_ores(
        _game: &crate::game::Game<'_>,
        _player: &crate::game::native_hooks::NativePlayer,
    ) -> Vec<[i32; 3]> {
        Vec::new()
    }
}
