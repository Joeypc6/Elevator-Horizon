fn jei_no_recipes_line(text: &str) -> core::ops::ControlFlow<()> {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    if crate::game::hz_kit_layer_elevatorid::chat_allowed(text) {
        core::ops::ControlFlow::Continue(())
    } else {
        core::ops::ControlFlow::Break(())
    }
} else { core::ops::ControlFlow::Continue(()) })
}
