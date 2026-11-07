// The mod's row in Options > Mods, and its one key.
fn elevator_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "elevatorid".into(),
        title: "Elevator 1.16.6".into(),
        description: "Blocks that lift you to the next one of their colour.".into(),
        keys: vec![["guide".to_string(), "Open Guide for Items".to_string(), "key.keyboard.w".to_string()]],
    });
}
