fn elevator_signal_source(level: &crate::game::GameLevel<'_>, state: horizon_world::registry::BlockStateId, source: &mut bool) {
    crate::game::elevatorid::is_signal_source(&level.facts.get(state).name, source);
}
