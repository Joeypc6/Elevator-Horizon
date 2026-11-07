fn jei_creative_width() -> core::ops::ControlFlow<Option<f32>> {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    match crate::game::hz_kit_creative_elevatorid::screen_width() {
        Some(width) => core::ops::ControlFlow::Break(Some(width)),
        None => core::ops::ControlFlow::Continue(()),
    }
} else { core::ops::ControlFlow::Continue(()) })
}
