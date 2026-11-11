fn kit_guide_shown_elevatorid(stack: &ItemStack) {
    elevatorid_optional::if_kit!("elevatorid", {
    crate::game::hz_kit_guide_elevatorid::hovered(&stack.item);
} else { let _ = stack; () })
}
