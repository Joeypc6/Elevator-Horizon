fn ae2_jei_event(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    event: &crate::game::native_hooks::NativePanelEvent,
) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    crate::game::hz_kit_layer_elevatorid::event(game, mod_id, event);
} else { () })
}
