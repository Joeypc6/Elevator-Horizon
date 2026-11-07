fn kit_guide_released_elevatorid(game: &crate::game::Game<'_>, canceled: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    crate::game::hz_kit_guide_elevatorid::release(game, canceled);
} else { () })
}
