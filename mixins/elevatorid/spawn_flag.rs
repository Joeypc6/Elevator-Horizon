// ElevatorBlockBase.isValidSpawn with mobSpawn on: `is_valid_spawn` is generic, so it's answered where it reads the flags.
fn elevator_spawn_flag(value: StateData, id: horizon_world::registry::BlockStateId) -> StateData {
    hz_elevator_spawn::flagged(value, id)
}
