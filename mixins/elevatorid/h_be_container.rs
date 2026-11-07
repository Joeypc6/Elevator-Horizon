fn elevator_be_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    crate::game::elevatorid::block_entity_container(block, container);
}
