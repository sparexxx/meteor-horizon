// The mod's row in Options > Mods, with every keybind listed on its page.
fn official_meteor_horizon_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::official_meteor_horizon as m;

    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        // `[name, label, default key]`; the page lists these first and
        // `native_mod_key` reads whatever the player rebound them to.
        keys: m::listed_keys(),
    });
}
