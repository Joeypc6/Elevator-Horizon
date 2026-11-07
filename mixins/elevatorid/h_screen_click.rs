fn elevator_screen_click(game: &crate::game::Game<'_>, block: &str, at: [f32; 2], right: bool, button: &mut Option<i32>) {
    crate::game::elevatorid::screen_click(game, block, at, right, button);
}
