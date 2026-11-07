fn kit_guide_pressed_elevatorid(game: &crate::game::Game<'_>, screen: &crate::game::native_hooks::NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    if game.player.is_none() {
        return;
    }
    let _ = button;
    crate::game::hz_kit_guide_elevatorid::click(game, screen.mouse, canceled);
} else { () })
}
