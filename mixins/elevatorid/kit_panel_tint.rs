fn jei_panel_tint(game: &Game<'_>, batch: &mut HudBatch) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    crate::game::hz_kit_window_elevatorid::tint(game, batch);
} else { let _ = (game, batch); () })
}
