fn kit_guide_tooltip_elevatorid(stack: &horizon_client::items::ItemStack, lines: &mut Vec<(horizon_client::text::Text, u32)>) {
    elevatorid_optional::if_kit!("elevatorid", {
    crate::game::hz_kit_guide_elevatorid::tooltip(stack, lines);
} else { let _ = (stack, lines); () })
}
