fn elevator_arrows(value: Vec<([i32; 3], u32)>, game: &crate::game::Game<'_>) -> Vec<([i32; 3], u32)> {
    crate::game::elevatorid::arrows_to_draw(game, value)
}
