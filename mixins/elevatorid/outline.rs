fn elevator_outline(game: &crate::game::Game<'_>, pos: [i32; 3]) -> core::ops::ControlFlow<Vec<[f64; 6]>> {
    match crate::game::elevatorid::outline(game, pos) {
        Some(boxes) => core::ops::ControlFlow::Break(boxes),
        None => core::ops::ControlFlow::Continue(()),
    }
}
