// The elevators' states a mob may spawn on. Never set (the default), and every other block pays one load.
use super::*;
use std::sync::OnceLock;

static SPAWNABLE: OnceLock<Vec<bool>> = OnceLock::new();

impl MobData {
    pub fn hz_elevator_set_spawnable(states: Vec<bool>) {
        let _ = SPAWNABLE.set(states);
    }
}

pub(super) fn flagged(mut value: StateData, id: horizon_world::registry::BlockStateId) -> StateData {
    if let Some(states) = SPAWNABLE.get()
        && states.get(id.0 as usize).copied().unwrap_or(false)
    {
        value.flags |= VALID_SPAWN | VALID_SPAWN_LEAVES | VALID_SPAWN_ICE;
    }
    value
}
