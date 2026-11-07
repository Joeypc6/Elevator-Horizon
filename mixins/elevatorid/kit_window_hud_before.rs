fn jei_hud_before(game: &mut crate::game::Game<'_>) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    crate::game::hz_kit_window_elevatorid::hud_before(game);
} else { () })
}
