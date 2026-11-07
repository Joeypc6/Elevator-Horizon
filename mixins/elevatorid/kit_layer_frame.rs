fn kit_layer_frame_elevatorid(game: &mut crate::game::Game<'_>) {
    elevatorid_optional::if_kit_jei!("elevatorid", {
    if game.native_on(crate::game::jei::ID) {
        crate::game::hz_kit_layer_elevatorid::frame(game);
    }
} else { () })
}
