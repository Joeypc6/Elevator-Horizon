fn kit_guide_key_elevatorid(game: &crate::game::Game<'_>, screen: &crate::game::native_hooks::NativeScreenView<'_>, key: &str, repeat: bool, canceled: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    if game.player.is_none() {
        return;
    }
    crate::game::hz_kit_guide_elevatorid::key(game, screen.menu, key, repeat, canceled);
} else { () })
}
