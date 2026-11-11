fn jei_hud_after(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    crate::game::hz_kit_window_elevatorid::hud_after(game, batch);
} else { let _ = (game, batch); () })
}
