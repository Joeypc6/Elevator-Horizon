// ElevatorBlockBase.getCollisionShape: the camouflage's. `hz_state` is its state, which the entity keeps up to date.
fn elevator_collision(scene: &mut SceneCollisions<'_, '_>, p: BlockPos, context: &CollisionContext<'_>) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    use crate::containers::BlockEntity;
    let Some(BlockEntity::Mod(m)) = scene.world.block_entities.get(&(p.x, p.y, p.z)) else {
        return core::ops::ControlFlow::Continue(());
    };
    if !m.id.starts_with("elevatorid:elevator_") {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(horizon_world::nbt::Tag::Int(id)) = m.nbt.get(&horizon_world::nbt::JavaString::from("hz_state")) else {
        return core::ops::ControlFlow::Continue(());
    };
    if *id < 0 {
        return core::ops::ControlFlow::Continue(());
    }
    let state = horizon_world::registry::BlockStateId(*id as u32);
    let binding = scene.binding;
    core::ops::ControlFlow::Break(binding.collision_shape_with_context(scene, state, p, context).map_err(io::Error::other))
}
