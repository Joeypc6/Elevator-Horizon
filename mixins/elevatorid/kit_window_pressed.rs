fn jei_window_pressed(game: &crate::game::Game<'_>, screen: &crate::game::native_hooks::NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    if game.player.is_none() || *canceled {
        return;
    }
    crate::game::hz_kit_window_elevatorid::press(game, screen.mouse, button, canceled);
    if !*canceled {
        crate::game::hz_kit_creative_elevatorid::press(game, screen.mouse, button, canceled);
    }
} else { () })
}
