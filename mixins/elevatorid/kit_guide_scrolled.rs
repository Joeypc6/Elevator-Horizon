fn kit_guide_scrolled_elevatorid(game: &crate::game::Game<'_>, scroll: [f64; 2], canceled: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    if game.player.is_some() {
        crate::game::hz_kit_guide_elevatorid::scroll(scroll[1], canceled);
    }
} else { () })
}
