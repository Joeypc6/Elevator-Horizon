fn kit_guide_char_elevatorid(game: &crate::game::Game<'_>, c: char, canceled: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    let _ = c;
    if game.player.is_some() && crate::game::hz_kit_guide_elevatorid::is_open() {
        *canceled = true;
    }
} else { () })
}
