// The mesher reads each block from here, its neighbours too: a camouflage is the block itself from now on, so its
// biome tint, shading, render layer and what its faces hide are the block's own.
fn elevator_mesh_camos(value: (), pad: &mut Padded, region: &[[Option<&Chunk>; 3]; 3], sy: i32) {
    let () = value;
    hz_elevator::swap(pad, region, sy);
}
