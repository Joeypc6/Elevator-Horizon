fn jei_screen_tip(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<()> {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    if crate::game::hz_kit_window_elevatorid::defer_tooltip(game) {
        return core::ops::ControlFlow::Break(());
    }
} else { let _ = game; () });
    core::ops::ControlFlow::Continue(())
}
