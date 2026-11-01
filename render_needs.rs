// Asked as a frame is built: tell the renderer what the mod needs from it.
//
// `world_depth` keeps the world pass's depth buffer, which the ESP renderer
// reads to hide boxes behind terrain. It is a real cost, so it is only asked
// for while Player ESP is actually running.
fn meteor_render_needs(needs: &mut crate::gfx::native_render::RenderNeeds) {
    use crate::game::official_meteor_horizon as m;

    let esp_on = m::with(|live| live.cfg.is_on(m::Module::Esp));
    if esp_on {
        needs.world_depth = true;
    }
}
