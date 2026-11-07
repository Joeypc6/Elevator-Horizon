fn elevator_be(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::elevatorid::block_entity(block, entity);
}
