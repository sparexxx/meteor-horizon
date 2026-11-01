// /meteor, so every function is reachable without a keybind.
//   /meteor                 status
//   /meteor on | off        the global toggle
//   /meteor list            module ids and states
//   /meteor <module id>     toggle one module
fn meteor_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::official_meteor_horizon as m;

    if !game.native_on(m::ID) {
        return;
    }

    let mut words = command.split_whitespace();
    if words.next() != Some("meteor") {
        return;
    }
    *ran = true;

    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    let arg = words.next().unwrap_or("").to_lowercase();
    match arg.as_str() {
        "" => {
            let line = m::with(|live| {
                let mut on: Vec<&'static str> = Vec::new();
                for module in m::Module::ALL {
                    if live.cfg.is_on(module) {
                        on.push(module.label());
                    }
                }
                let state = if live.cfg.global_on { "ON" } else { "OFF" };
                if on.is_empty() {
                    format!("Meteor Horizon Lite {state}: no module running")
                } else {
                    format!("Meteor Horizon Lite {state}: {}", on.join(", "))
                }
            });
            game.native_chat(&line);
        }
        "on" | "off" => {
            let want = arg == "on";
            m::with(|live| live.cfg.set_global(want));
            m::save(|name, value| game.native_set_setting(m::ID, name, value));
            game.native_chat(if want {
                "Meteor Horizon Lite: ON"
            } else {
                "Meteor Horizon Lite: OFF (setting kept)"
            });
        }
        "list" => {
            let line = m::with(|live| {
                let mut parts = Vec::new();
                for module in m::Module::ALL {
                    parts.push(format!(
                        "{}={}",
                        module.id(),
                        if live.cfg.is_on(module) { "on" } else { "off" }
                    ));
                }
                parts.join(" ")
            });
            game.native_chat(&format!("modules: {line}"));
        }
        other => match m::Module::from_id(other) {
            Some(module) => {
                let on = m::with(|live| live.cfg.toggle(module));
                m::save(|name, value| game.native_set_setting(m::ID, name, value));
                game.native_chat(&format!(
                    "{} {}",
                    module.label(),
                    if on { "enabled" } else { "disabled" }
                ));
            }
            None => game.native_chat(
                "usage: /meteor [on|off|list|esp|anchor_macro|xray|auto_totem]",
            ),
        },
    }
}
