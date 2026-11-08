"""Writes the Elevator bundle's horizon-mixin.toml and its one-line hook helpers, and packs the content.

Usage: python make_bundle.py
"""

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "mixins" / "elevatorid"
BUILD = ROOT.parent / "native-build" / "main" / "elevatorid"
JAR = ROOT.parent / "Ported" / "elevatorid-fabric-26.3-1.16.6.jar"
HORIZON = "6560641ba51a452f6bc2407baa4eb1b3be517ca8"
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
VERSION = "1.16.7"
M = "crate::game::elevatorid"

GAME_MUT = ("game", "&mut crate::game::Game<'_>", "&mut *self")
GAME = ("game", "&crate::game::Game<'_>", "&*self")

# (id, hook, bindings, body)
HOOKS = [
    ("be", "block_entity", [("block", "&str", "block"), ("entity", "&mut Option<(String, horizon_world::nbt::Compound)>", "&mut *entity")], "block_entity(block, entity)"),
    ("be-ticks", "block_entity_ticks", [("block", "&str", "block"), ("ticks", "&mut bool", "&mut *ticks")], "block_entity_ticks(block, ticks)"),
    ("be-container", "block_entity_container", [("block", "&str", "block"), ("container", "&mut Option<horizon_client::containers::ModContainer>", "&mut *container")], "block_entity_container(block, container)"),
    ("be-tick", "block_entity_tick", [GAME_MUT, ("pos", "[i32; 3]", "pos"), ("entity", "&mut horizon_client::containers::ModBlockEntity", "&mut *entity")], "block_entity_tick(game, pos, entity)"),
    ("update-tag", "block_entity_update_tag", [("entity", "&horizon_client::containers::ModBlockEntity", "&*entity"), ("tag", "&mut Option<horizon_world::nbt::Compound>", "&mut *tag")], "update_tag(entity, tag)"),
    ("submit", "block_entity_submit", [GAME, ("pos", "[i32; 3]", "pos"), ("block", "&str", "block"), ("state", "horizon_world::registry::BlockStateId", "state"), ("entity", "&horizon_client::containers::ModBlockEntity", "&*entity"), ("draw", "&mut crate::game::native_block_entity_draw::BlockEntityDraw", "&mut *draw")], "submit(game, block, state, draw)"),
    ("right-click", "right_click_block", [GAME_MUT, ("pos", "[i32; 3]", "pos"), ("face", "[i32; 3]", "face"), ("canceled", "&mut bool", "&mut *canceled")], "right_click(game, pos, face, canceled)"),
    ("menu-button", "menu_button_v2", [GAME_MUT, ("pos", "[i32; 3]", "pos"), ("entity", "&mut horizon_client::containers::ModBlockEntity", "&mut *entity"), ("id", "i32", "id"), ("carried", "&mut Option<horizon_client::items::ItemStack>", "&mut *carried")], "menu_button(game, pos, entity, id)"),
    ("screen", "screen", [GAME, ("block", "&str", "block"), ("screen", "&mut Option<crate::game::native_hooks::NativeScreen>", "&mut *screen")], "screen(game, block, screen)"),
    ("screen-draw", "screen_draw", [GAME, ("block", "&str", "block"), ("data", "&[i32]", "data"), ("batch", "&mut horizon_client::hud::HudBatch", "&mut *batch"), ("origin", "[f32; 2]", "origin")], "screen_draw(game, block, batch, origin)"),
    ("screen-click", "screen_click", [GAME, ("block", "&str", "block"), ("at", "[f32; 2]", "at"), ("right", "bool", "right"), ("button", "&mut Option<i32>", "&mut *button")], "screen_click(game, block, at, right, button)"),
    ("screen-tooltip", "screen_tooltip", [("block", "&str", "block"), ("at", "[f32; 2]", "at"), ("lines", "&mut Vec<String>", "&mut *lines")], "screen_tooltip(block, at)"),
    ("net-from-client", "net_from_client", [GAME_MUT, ("player", "u128", "player"), ("channel", "&str", "channel"), ("data", "&[u8]", "data")], "from_client(player, channel, data)"),
    ("tick", "tick", [GAME_MUT], "client_tick(game)"),
    ("kit-frame", "frame", [GAME_MUT], "kit_frame(game)"),
    ("signal", "block_signal", [("pos", "horizon_core::position::BlockPos", "pos"), ("face", "horizon_core::geometry::Direction", "face"), ("signal", "&mut i32", "&mut *signal")], "block_signal([pos.x, pos.y, pos.z], signal)"),
    ("signal-source", "is_signal_source", [("level", "&crate::game::GameLevel<'_>", "&*self"), ("state", "horizon_world::registry::BlockStateId", "state"), ("source", "&mut bool", "&mut *source")], "is_signal_source(&level.facts.get(state).name, source)"),
]

SPECIAL_HELPERS = {
    "server_tick.rs": """fn elevator_server_tick(game: &mut crate::game::Game<'_>, phase: &crate::game::native_lifecycle::ServerTickPhase) {
    if matches!(phase, crate::game::native_lifecycle::ServerTickPhase::ServerTickEnd { .. }) {
        crate::game::elevatorid::server_tick(game);
        crate::game::elevatorid::sweep_powered(game);
    }
}
""",
    "embedded_content.rs": """// Elevator is one file on the registry: its blocks, recipes and textures are the content mod in this bundle's data/
// (made by tools/make_content.py from the jar). It is written out once and loaded as a dev mod is.
fn elevator_embedded_content(root: &std::path::Path, enabled: &std::collections::BTreeSet<String>, dev: &[std::path::PathBuf]) -> core::ops::ControlFlow<std::path::PathBuf> {
    static CONTENT: &[u8] = horizon_client::native_data!("elevatorid", "elevatorid-content.hmod");
    let dir = std::env::current_dir().unwrap_or_default().join(".horizon-cache");
    let file = dir.join("elevatorid-embedded-content.hmod");
    if dev.contains(&file) {
        return core::ops::ControlFlow::Continue(());
    }
    let same = std::fs::read(&file).is_ok_and(|on_disk| on_disk == CONTENT);
    if !same && std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&file, CONTENT)).is_err() {
        log::warn!("Horizon: Elevator's content could not be written to {}", file.display());
        return core::ops::ControlFlow::Continue(());
    }
    let mut with = dev.to_vec();
    with.push(file);
    core::ops::ControlFlow::Break(content_root(root, enabled, &with))
}
""",
    "mesh_camos.rs": """// The mesher reads each block from here, its neighbours too: a camouflage is the block itself from now on, so its
// biome tint, shading, render layer and what its faces hide are the block's own.
fn elevator_mesh_camos(value: (), pad: &mut Padded, region: &[[Option<&Chunk>; 3]; 3], sy: i32) {
    let () = value;
    hz_elevator::swap(pad, region, sy);
}
""",
    "list.rs": """// The mod's row in Options > Mods, and its one key.
fn elevator_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "elevatorid".into(),
        title: "Elevator 1.16.7".into(),
        description: "Blocks that lift you to the next one of their colour.".into(),
        keys: vec![["guide".to_string(), "Open Guide for Items".to_string(), "key.keyboard.w".to_string()]],
    });
}
""",
    "spawn_flag.rs": """// ElevatorBlockBase.isValidSpawn with mobSpawn on: `is_valid_spawn` is generic, so it's answered where it reads the flags.
fn elevator_spawn_flag(value: StateData, id: horizon_world::registry::BlockStateId) -> StateData {
    hz_elevator_spawn::flagged(value, id)
}
""",
    "spawn_table.rs": """// The elevators' states a mob may spawn on. Never set (the default), and every other block pays one load.
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
""",
    "mesh_table.rs": """// The elevators with a camouflage in each section, from the game's thread.
use super::*;
use std::collections::HashMap;
use std::sync::RwLock;

static SECTIONS: RwLock<Option<HashMap<(i32, i32, i32), Vec<([i32; 3], u32)>>>> = RwLock::new(None);

impl Mesher<'_> {
    /// Every camouflage now: (position, state).
    pub fn hz_elevator_set_camos(camos: &[([i32; 3], u32)]) {
        let Ok(mut table) = SECTIONS.write() else { return };
        let mut by_section: HashMap<(i32, i32, i32), Vec<([i32; 3], u32)>> = HashMap::new();
        for (p, state) in camos {
            by_section.entry((p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16))).or_default().push((*p, *state));
        }
        *table = Some(by_section);
    }
}

pub(super) fn swap(pad: &mut Padded, region: &[[Option<&Chunk>; 3]; 3], sy: i32) {
    let Ok(table) = SECTIONS.read() else { return };
    let Some(table) = table.as_ref() else { return };
    if table.is_empty() {
        return;
    }
    let Some(center) = region[1][1] else { return };
    let base_y = center.min_y() + sy * 16;
    let section_y = base_y.div_euclid(16);
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let Some(list) = table.get(&(center.x + dx, section_y + dy, center.z + dz)) else { continue };
                for (p, state) in list {
                    let (x, y, z) = (p[0] - center.x * 16, p[1] - base_y, p[2] - center.z * 16);
                    if (-B..16 + B).contains(&x) && (-B..16 + B).contains(&y) && (-B..16 + B).contains(&z) {
                        pad.blocks[pi(x, y, z)] = *state;
                    }
                }
            }
        }
    }
}
""",
    "arrows.rs": """fn elevator_arrows(value: Vec<([i32; 3], u32)>, game: &crate::game::Game<'_>) -> Vec<([i32; 3], u32)> {
    crate::game::elevatorid::arrows_to_draw(game, value)
}
""",
    "light_state.rs": """// ElevatorBlockBase.LIGHT: each state gives off what its `light` says (a content block's light is one number otherwise).
fn elevator_light_state(b: &NewBlock, combo: &[(String, String)], row: &mut J) {
    if !b.decl.id.starts_with("elevatorid:elevator_") {
        return;
    }
    let light: i64 = combo.iter().find(|(k, _)| k == "light").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
    row.set("emission", J::Num(light.into()));
}
""",
    "outline.rs": """fn elevator_outline(game: &crate::game::Game<'_>, pos: [i32; 3]) -> core::ops::ControlFlow<Vec<[f64; 6]>> {
    match crate::game::elevatorid::outline(game, pos) {
        Some(boxes) => core::ops::ControlFlow::Break(boxes),
        None => core::ops::ControlFlow::Continue(()),
    }
}
""",
    "collision.rs": """// ElevatorBlockBase.getCollisionShape: the camouflage's. `hz_state` is its state, which the entity keeps up to date.
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
""",
}


def binding_lines(bindings):
    return "".join(f'  {{ name = "{n}", ty = "{t}", value = "{v}" }},\n' for n, t, v in bindings)


def main():
    out = [
        "[mod]",
        'id = "elevatorid"',
        f'version = "{VERSION}"',
        "",
        "# The Horizon commit this bundle was built and tested on.",
        "[compatibility]",
        f'horizon_revision = "{HORIZON}"',
        "",
        "[[module]]",
        'id = "elevator"',
        'file = "crates/horizon-client/src/game.rs"',
        'name = "elevatorid"',
        'source = "elevator.rs"',
        "",
    ]
    for hook_id, hook, bindings, body in HOOKS:
        helper = f"h_{hook_id.replace('-', '_')}.rs"
        args = ", ".join(f"{n}: {t}" for n, t, _ in bindings)
        (BUNDLE / helper).write_text(f"fn elevator_{hook_id.replace('-', '_')}({args}) {{\n    {M}::{body};\n}}\n", encoding="utf8")
        out += ["[[injection]]", f'id = "elevator-{hook_id}"', 'kind = "HEAD"', f'hook = "{hook}"', "hook_version = 1", f'helper = "{helper}"', "bindings = [", binding_lines(bindings).rstrip("\n"), "]", ""]
    for name, text in SPECIAL_HELPERS.items():
        (BUNDLE / name).write_text(text, encoding="utf8")
    out += ["[[injection]]", 'id = "elevator-server-tick"', 'kind = "HEAD"', 'hook = "server_tick"', "hook_version = 1", 'helper = "server_tick.rs"', "bindings = [", binding_lines([GAME_MUT, ("phase", "&crate::game::native_lifecycle::ServerTickPhase", "&phase")]).rstrip("\n"), "]", ""]
    out += ["[[module]]", 'id = "mesh-table"', 'file = "crates/horizon-client-render/src/mesher.rs"', 'name = "hz_elevator"', 'source = "mesh_table.rs"', ""]
    out += ["[[module]]", 'id = "spawn-table"', 'file = "crates/horizon-client/src/mobs/data.rs"', 'name = "hz_elevator_spawn"', 'source = "spawn_table.rs"', ""]
    out += ["[[injection]]", 'id = "elevator-spawn-flag"', 'kind = "RETURN"', 'target = "horizon_client::mobs::data::MobData::state"', 'helper = "spawn_flag.rs"', "bindings = [", binding_lines([("value", "StateData", "$value"), ("id", "horizon_world::registry::BlockStateId", "id")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-list"', 'kind = "HEAD"', 'hook = "mods"', "hook_version = 1", 'helper = "list.rs"', "bindings = [", binding_lines([("mods", "&mut Vec<crate::game::native_hooks::NativeMod>", "&mut *mods")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-mesh-camos"', 'kind = "RETURN"', 'target = "horizon_client_render::mesher::Padded::fill"', 'helper = "mesh_camos.rs"', "bindings = [", binding_lines([("value", "()", "$value"), ("pad", "&mut Padded", "&mut *self"), ("region", "&[[Option<&Chunk>; 3]; 3]", "region"), ("sy", "i32", "sy")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-arrows"', 'kind = "RETURN"', 'target = "horizon_client::game::native_block_entity_draw::Game<\'_>::untemplated_mod_block_entities"', 'helper = "arrows.rs"', "bindings = [", binding_lines([("value", "Vec<([i32; 3], u32)>", "$value"), GAME]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-light-state"', 'kind = "HEAD"', 'target = "horizon_client::mod_content::Plan<\'_>::override_gameplay"', 'helper = "light_state.rs"', "bindings = [", binding_lines([("b", "&NewBlock", "b"), ("combo", "&[(String, String)]", "combo"), ("row", "&mut J", "&mut *row")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-embedded-content"', 'kind = "HEAD"', "cancellable = true", 'target = "horizon_client::game::mods::content_root"', 'helper = "embedded_content.rs"', "bindings = [", binding_lines([("root", "&std::path::Path", "root"), ("enabled", "&std::collections::BTreeSet<String>", "enabled"), ("dev", "&[std::path::PathBuf]", "dev")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-outline"', 'kind = "HEAD"', "cancellable = true", 'target = "horizon_client::game::Game<\'_>::outline_boxes"', 'helper = "outline.rs"', "bindings = [", binding_lines([GAME, ("pos", "[i32; 3]", "pos")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "elevator-collision"', 'kind = "HEAD"', "cancellable = true", 'target = "horizon_client_render::collisions::<SceneCollisions<\'_,\'_> as BlockCollisionWorld>::context_shape"', 'helper = "collision.rs"', "bindings = [", binding_lines([("scene", "&mut SceneCollisions<'_, '_>", "&mut *self"), ("p", "BlockPos", "p"), ("context", "&CollisionContext<'_>", "context")]).rstrip("\n"), "]", ""]
    (BUNDLE / "horizon-mixin.toml").write_text("\n".join(out), encoding="utf8")
    # The JEI window, Mob Drops, World Gen, the creative panel and the W guide: this mod's own copy.
    subprocess.run([sys.executable, str(ROOT.parent / "jei-kit" / "apply.py"), "elevatorid", str(BUNDLE)], check=True)
    (BUNDLE / "data").mkdir(exist_ok=True)
    subprocess.run([sys.executable, str(ROOT / "tools" / "make_content.py"), str(JAR), str(BUNDLE / "data" / "elevatorid-content.hmod")], check=True)
    # The dev build folder: pinned, with no hooks number, so the pin applies.
    target = BUILD / "mixins" / "elevatorid"
    target.mkdir(parents=True, exist_ok=True)
    for old in target.rglob("*"):
        if old.is_file():
            old.unlink()
    for f in BUNDLE.rglob("*"):
        if f.is_file():
            to = target / f.relative_to(BUNDLE)
            to.parent.mkdir(parents=True, exist_ok=True)
            to.write_bytes(f.read_bytes())
    (BUILD / "horizon-native.json").write_text(json.dumps({"game": "26.3", "horizon": HORIZON, "id": "elevatorid", "mixins": MIXINS, "schema": 1, "version": VERSION}, indent=2) + "\n", encoding="utf8")
    print("bundle written:", len(HOOKS) + 4, "injections")


if __name__ == "__main__":
    main()
