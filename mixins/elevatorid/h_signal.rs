fn elevator_signal(pos: horizon_core::position::BlockPos, face: horizon_core::geometry::Direction, signal: &mut i32) {
    let _ = face;
    crate::game::elevatorid::block_signal([pos.x, pos.y, pos.z], signal);
}
