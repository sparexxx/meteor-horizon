// Every keybind: the global toggle, the ClickGUI, and one per module.
//
// `native_key` fires for both the press and the release, which is why the
// `pressed` flag is checked: a toggle acts on the press only, so holding a key
// cannot flip a module back and forth, and the release is a no-op.
fn meteor_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::official_meteor_horizon as m;

    // Off in Options > Mods: no keys at all.
    if !game.native_on(m::ID) {
        return;
    }

    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    // The global toggle works even while the client is "off", or it could
    // never be turned back on.
    let global = game.native_mod_key(m::ID, m::KEY_GLOBAL, m::DEFAULT_KEY_GLOBAL);
    if key == global {
        if pressed {
            let now = m::with(|live| {
                let on = !live.cfg.global_on;
                live.cfg.set_global(on);
                on
            });
            m::save(|name, value| game.native_set_setting(m::ID, name, value));
            game.native_chat(if now {
                "Meteor Horizon Lite: ON"
            } else {
                "Meteor Horizon Lite: OFF (settings kept)"
            });
        }
        return;
    }

    // Everything else needs the client switched on.
    if !m::with(|live| live.cfg.global_on) {
        return;
    }

    // The ClickGUI key. It opens even without a world, so the modules can be
    // set up from the title screen.
    let gui = game.native_mod_key(m::ID, m::KEY_GUI, m::DEFAULT_KEY_GUI);
    if key == gui {
        if pressed {
            m::with(|live| {
                live.gui.open = !live.gui.open;
            });
        }
        return;
    }

    if !pressed {
        return;
    }

    for module in m::Module::ALL {
        let bound = game.native_mod_key(m::ID, module.key_name(), module.default_key());
        if key != bound {
            continue;
        }
        let on = m::with(|live| live.cfg.toggle(module));
        m::save(|name, value| game.native_set_setting(m::ID, name, value));
        game.native_chat(&format!(
            "Meteor Horizon Lite: {} {}",
            module.label(),
            if on { "enabled" } else { "disabled" }
        ));
        return;
    }
}
