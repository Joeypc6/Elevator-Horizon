fn elevator_server_tick(game: &mut crate::game::Game<'_>, phase: &crate::game::native_lifecycle::ServerTickPhase) {
    if matches!(phase, crate::game::native_lifecycle::ServerTickPhase::ServerTickEnd { .. }) {
        crate::game::elevatorid::server_tick(game);
        crate::game::elevatorid::sweep_powered(game);
    }
}
