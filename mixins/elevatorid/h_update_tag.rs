fn elevator_update_tag(entity: &horizon_client::containers::ModBlockEntity, tag: &mut Option<horizon_world::nbt::Compound>) {
    crate::game::elevatorid::update_tag(entity, tag);
}
