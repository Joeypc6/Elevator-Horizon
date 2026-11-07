fn kit_guide_dragged_elevatorid(delta: [f32; 2], canceled: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    crate::game::hz_kit_guide_elevatorid::drag(delta, canceled);
} else { () })
}
