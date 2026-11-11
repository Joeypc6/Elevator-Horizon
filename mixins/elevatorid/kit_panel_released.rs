fn jei_window_released(screen: &crate::game::native_hooks::NativeScreenView<'_>) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    crate::game::hz_kit_window_elevatorid::release(screen.mouse);
} else { let _ = screen; () })
}
