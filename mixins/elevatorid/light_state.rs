// ElevatorBlockBase.LIGHT: each state gives off what its `light` says (a content block's light is one number otherwise).
fn elevator_light_state(b: &NewBlock, combo: &[(String, String)], row: &mut J) {
    if !b.decl.id.starts_with("elevatorid:elevator_") {
        return;
    }
    let light: i64 = combo.iter().find(|(k, _)| k == "light").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
    row.set("emission", J::Num(light.into()));
}
