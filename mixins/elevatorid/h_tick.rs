fn elevator_tick(game: &mut crate::game::Game<'_>) {
    crate::game::elevatorid::client_tick(game);
}
