fn kit_guide_render_post_elevatorid(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch, screen: &crate::game::native_hooks::NativeScreenView<'_>) {
    elevatorid_optional::if_kit!("elevatorid", {
    if game.player.is_none() {
        return;
    }
    crate::game::hz_kit_guide_elevatorid::screen_frame(screen.id, screen.menu);
    crate::game::hz_kit_guide_elevatorid::draw(game, batch, screen.mouse);
} else { () })
}
