fn elevator_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    crate::game::elevatorid::right_click(game, pos, face, canceled);
}
