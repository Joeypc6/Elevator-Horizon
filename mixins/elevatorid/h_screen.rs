fn elevator_screen(game: &crate::game::Game<'_>, block: &str, screen: &mut Option<crate::game::native_hooks::NativeScreen>) {
    crate::game::elevatorid::screen(game, block, screen);
}
