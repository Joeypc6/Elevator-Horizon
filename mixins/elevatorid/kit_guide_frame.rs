fn kit_guide_frame_elevatorid(game: &mut crate::game::Game<'_>) {
    elevatorid_optional::if_kit!("elevatorid", {
    if game.player.is_some() {
        crate::game::hz_kit_guide_elevatorid::frame_hook(game);
    }
} else { let _ = game; () })
}
