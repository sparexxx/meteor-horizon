// A world opening or closing, a dimension change, or the final save.
//
// Every cached thing the mod holds is frame- or world-scoped, so it is dropped
// here: the Anchor Macro's phase and timers, the ClickGUI's selection, and the
// renderer's buffers. Settings are deliberately untouched — those survive the
// world, the dimension and the game restart.
fn meteor_world_lifecycle(
    game: &mut crate::game::Game<'_>,
    context: &crate::game::native_lifecycle::NativeLifecycleContext,
    phase: crate::game::native_lifecycle::WorldPhase,
    error_text: &str,
) {
    use crate::game::official_meteor_horizon as m;

    // The phase and context are matched on elsewhere; here every phase means
    // the same thing, so nothing about their shape is assumed.
    let _ = (context, phase, error_text);

    if !game.native_on(m::ID) {
        return;
    }

    m::with(|live| {
        live.phase = m::AnchorPhase::Ready;
        live.last_action_ms = 0;
        live.last_totem_ms = 0;
        live.clock_ms = 0;
        live.gui.selected = None;
        live.gui.scroll = 0.0;
        live.gui.open = false;
    });

    // Drop the ESP boxes: they belong to the world that is going away. The
    // renderer rebuilds its vertex buffer every frame, so clearing the shared
    // list is all the cleanup it needs.
    horizon_client_render::gfx::native_render::meteor_bridge::clear();
    m::reset_renderer();
}
