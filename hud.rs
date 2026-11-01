// The ClickGUI and the on-screen module list, drawn in GUI pixels.
//
// Only three verified helpers are used: `native_fill`, `native_text` and
// `native_text_width`, in the same way Horizon's own Coordinates HUD draws its
// box.
fn meteor_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::official_meteor_horizon as m;

    if !game.native_on(m::ID) {
        return;
    }
    m::ensure_loaded(|name| game.native_setting(m::ID, name));

    let (open, pos, category, selected, search) = m::with(|live| {
        (
            live.gui.open,
            live.gui.pos,
            live.gui.category,
            live.gui.selected,
            live.gui.search.clone(),
        )
    });

    // The corner is only claimed while the GUI is open.
    if !open {
        // A small watermark when anything is running, so the state is never a
        // mystery, and nothing at all when the client is idle.
        let running = m::with(|live| {
            live.cfg
                .global_on
                .then(|| m::Module::ALL.iter().filter(|x| live.cfg.is_on(**x)).count())
                .unwrap_or(0)
        });
        if running == 0 {
            return;
        }
        let label = format!("Meteor {}", m::VERSION);
        let w = game.native_text_width(&label) + 6.0;
        let (x, y) = corners.place(horizon_client::extras::Corner::BottomRight, w, 12.0);
        game.native_fill(batch, [x, y, w, 12.0], m::BG);
        game.native_text(batch, &label, [x + 3.0, y + 2.0], m::INK_DIM, true);
        return;
    }

    // The panel is positioned from its own saved corner, not a HUD corner.
    let _ = &corners;

    // ---- panel frame ----
    let panel = m::panel_rect(pos);
    game.native_fill(batch, panel, m::BG);
    // Header strip.
    game.native_fill(batch, [pos[0], pos[1], m::PANEL_W, m::HEADER_H], m::ACCENT);
    game.native_text(
        batch,
        &format!("METEOR {}  -  {}", m::VERSION, m::NAME),
        [pos[0] + m::PAD, pos[1] + 7.0],
        m::INK,
        true,
    );

    // Close button.
    let close = m::close_rect(pos);
    game.native_fill(batch, close, m::DANGER);
    game.native_text(batch, "x", [close[0] + 6.0, close[1] + 3.0], m::INK, false);

    // ---- categories ----
    for (i, cat) in m::Category::ALL.into_iter().enumerate() {
        let r = m::category_rect(pos, i);
        let active = i == category;
        game.native_fill(batch, r, if active { m::ACCENT } else { m::BG_ALT });
        game.native_text(
            batch,
            cat.label(),
            [r[0] + 4.0, r[1] + 4.0],
            if active { m::INK } else { m::INK_DIM },
            !active,
        );
    }

    // ---- module rows ----
    let visible = m::with(|live| live.gui.visible());
    if visible.is_empty() {
        game.native_text(
            batch,
            "no module matches",
            [pos[0] + m::CAT_W + m::PAD + 4.0, pos[1] + m::HEADER_H + m::PAD + 4.0],
            m::INK_DIM,
            true,
        );
    }
    for (i, module) in visible.into_iter().enumerate() {
        let r = m::row_rect(pos, i);
        let on = m::with(|live| live.cfg.is_on(module));

        game.native_fill(batch, r, if on { m::BG_HOVER } else { m::BG_ALT });
        // The state pill on the right.
        let pill = [r[0] + r[2] - 30.0, r[1] + 3.0, 26.0, r[3] - 6.0];
        game.native_fill(batch, pill, if on { m::ACCENT_ON } else { m::BG });
        game.native_text(
            batch,
            if on { "ON" } else { "OFF" },
            [pill[0] + 6.0, pill[1] + 1.0],
            if on { m::BG } else { m::INK_DIM },
            false,
        );
        game.native_text(
            batch,
            module.label(),
            [r[0] + 5.0, r[1] + 4.0],
            if on { m::INK } else { m::INK_DIM },
            !on,
        );
    }

    // ---- the selected module's settings ----
    if let Some(module) = selected {
        let top = pos[1] + m::HEADER_H + m::PAD;
        let mut y = top;
        let left = pos[0] + m::CAT_W + m::PAD;
        game.native_text(
            batch,
            &format!("{} settings", module.label()),
            [left, y],
            m::ACCENT,
            true,
        );
        y += 12.0;
        for line in module_settings_lines(module) {
            game.native_text(batch, &line, [left + 4.0, y], m::INK_DIM, false);
            y += 10.0;
        }
    }

    // ---- search box ----
    let box_ = m::search_rect(pos);
    game.native_fill(batch, box_, m::BG_ALT);
    let shown = if search.is_empty() {
        "type to search...".to_owned()
    } else {
        format!("{search}_")
    };
    game.native_text(
        batch,
        &shown,
        [box_[0] + 4.0, box_[1] + 4.0],
        if search.is_empty() { m::INK_DIM } else { m::INK },
        false,
    );
}

/// The lines the settings panel shows for `module`, read from the live config.
fn module_settings_lines(module: crate::game::official_meteor_horizon::Module) -> Vec<String> {
    use crate::game::official_meteor_horizon as m;

    m::with(|live| {
        let cfg = live.cfg;
        match module {
            m::Module::Esp => vec![
                format!("colour   #{}", m::argb_hex(cfg.esp.argb)),
                format!("range    {:.0} blocks", cfg.esp.distance),
                format!("thickness {:.1}px", cfg.esp.thickness),
                format!("fill     {}", if cfg.esp.fill { "on" } else { "off" }),
                format!("nametags {}", if cfg.esp.nametags { "on" } else { "off" }),
            ],
            m::Module::XRay => vec![
                format!("colour   #{}", m::argb_hex(cfg.xray.argb)),
                format!("blocks   {} whitelisted", cfg.xray.blocks.len()),
            ],
            m::Module::AnchorMacro => vec![
                format!("delay    {}ms", cfg.anchor.delay_ms),
                format!("charge   {}", if cfg.anchor.charge { "on" } else { "off" }),
                format!("detonate {}", if cfg.anchor.detonate { "on" } else { "off" }),
            ],
            m::Module::AutoTotem => vec![
                format!("delay    {}ms", cfg.totem.delay_ms),
                format!("always   {}", if cfg.totem.always { "on" } else { "off" }),
                format!("health   {:.0}", cfg.totem.health),
            ],
        }
    })
}
