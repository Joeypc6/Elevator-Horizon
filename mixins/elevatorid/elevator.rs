// Elevator (VsnGamer, MIT): a block that takes you to the next one of its kind above or below, jumping or sneaking.
// A port of `ElevatorBlockBase`, `ElevatorBlockEntityBase`, `ElevatorHandler`, `TeleportPacket` and `ElevatorScreen`.

use super::Game;
elevatorid_optional::kit_use!(guide as kit_guide);
elevatorid_optional::if_mod_items!("jei", {
elevatorid_optional::kit_use!(layer as kit_layer);
});
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::BlockStateId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::OnceLock;

type Pos = [i32; 3];

const PREFIX: &str = "elevatorid:elevator_";
const NET_TELEPORT: &str = "elevatorid:teleport";
const SOUND_TELEPORT: &str = "elevatorid:teleport";
const SOUND_CAMOUFLAGE: &str = "elevatorid:camouflage";
/// SoundSource.BLOCKS.
const SOUND_BLOCKS: usize = 4;
const SET_BLOCK: i32 = 3;

// ElevatorScreen's menu buttons.
const BUTTON_DIRECTIONAL: i32 = 0;
const BUTTON_ARROW: i32 = 1;
const BUTTON_RESET: i32 = 2;
/// Then one for each of `FACINGS`.
const BUTTON_FACING: i32 = 3;
/// Direction's 2D values: south, west, north, east.
const FACINGS: [&str; 4] = ["south", "west", "north", "east"];

#[derive(Default)]
struct Shared {
    /// (player, from, to): `TeleportPacket`, for the server's tick, where every level can be reached.
    teleports: Vec<(u128, Pos, Pos)>,
    /// Entities changed inside a hook that holds them: told to their viewers at the tick's end.
    sync: Vec<Pos>,
    /// Camouflaged as a redstone block: what a held state's getSignal gives.
    powered: HashSet<Pos>,
    /// The client's keys last tick (ElevatorHandler.lastSneaking, lastJumping).
    last_sneak: bool,
    last_jump: bool,
    /// The pointer over the open screen, from its origin.
    mouse: [f32; 2],
    /// What the mesher was last told (`camo_table`).
    camos: Vec<([i32; 3], u32)>,
}

thread_local! {
    static STATE: RefCell<Shared> = RefCell::new(Shared::default());
}

fn java(s: &str) -> JavaString {
    JavaString::from(s)
}

fn is_elevator(block: &str) -> bool {
    block.starts_with(PREFIX)
}

fn colour(block: &str) -> &str {
    block.strip_prefix(PREFIX).unwrap_or("")
}

// ---- Config (Config.GENERAL)

struct Config {
    /// ElevatorBlockBase.isValidSpawn: off, nothing spawns on an elevator.
    mob_spawn: bool,
    same_color: bool,
    range: i32,
    activation_range: i32,
    precision_target: bool,
    reset_pitch_normal: bool,
    reset_pitch_directional: bool,
    use_xp: bool,
    xp_points: i32,
}

const CONFIG_DEFAULT: &str = "\
# Elevator's options, as the Java mod's.
[General]
sameColor = false
range = 384
activationRange = 6
precisionTarget = true
mobSpawn = false
resetPitchNormal = false
resetPitchDirectional = true
useXP = false
XPPointsAmount = 1
";

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let text = Game::native_config_dir().ok().and_then(|dir| {
            let file = dir.join("elevatorid-common.toml");
            match std::fs::read_to_string(&file) {
                Ok(text) => Some(text),
                Err(_) => {
                    let _ = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&file, CONFIG_DEFAULT));
                    None
                }
            }
        });
        let text = text.unwrap_or_else(|| CONFIG_DEFAULT.to_owned());
        let read = |key: &str| {
            text.lines().find_map(|line| {
                let (k, v) = line.split_once('=')?;
                (k.trim() == key).then(|| v.trim().to_owned())
            })
        };
        let flag = |key: &str, default: bool| read(key).and_then(|v| v.parse().ok()).unwrap_or(default);
        let int = |key: &str, default: i32| read(key).and_then(|v| v.parse().ok()).unwrap_or(default);
        Config {
            mob_spawn: flag("mobSpawn", false),
            same_color: flag("sameColor", false),
            range: int("range", 384).clamp(1, 4096),
            activation_range: int("activationRange", 6).clamp(1, 1024),
            precision_target: flag("precisionTarget", true),
            reset_pitch_normal: flag("resetPitchNormal", false),
            reset_pitch_directional: flag("resetPitchDirectional", true),
            use_xp: flag("useXP", false),
            xp_points: int("XPPointsAmount", 1).max(0),
        }
    })
}

// ---- The held state (`held_id`, the BlockState codec's Name and Properties)

/// The camouflage as state text (`minecraft:oak_stairs[facing=north]`). An empty compound is none: an update tag
/// only adds keys, so taking it off has to be said.
fn held_text(nbt: &Compound) -> Option<String> {
    held_text_of(nbt.get(&java("held_id"))?)
}

fn held_text_of(tag: &Tag) -> Option<String> {
    let Tag::Compound(held) = tag else { return None };
    let Some(Tag::String(name)) = held.get(&java("Name")).or_else(|| held.get(&java("id"))) else { return None };
    let name = name.to_string_lossy();
    if name.is_empty() {
        return None;
    }
    let mut props = Vec::new();
    if let Some(Tag::Compound(p)) = held.get(&java("Properties")).or_else(|| held.get(&java("properties"))) {
        for (k, v) in p {
            if let Tag::String(v) = v {
                props.push(format!("{}={}", k.to_string_lossy(), v.to_string_lossy()));
            }
        }
    }
    Some(if props.is_empty() { name } else { format!("{name}[{}]", props.join(",")) })
}

fn split_state(text: &str) -> (&str, Vec<(&str, &str)>) {
    let Some((name, rest)) = text.split_once('[') else { return (text, Vec::new()) };
    let props = rest.trim_end_matches(']').split(',').filter_map(|p| p.split_once('=')).collect();
    (name, props)
}

/// The state as `BlockState.CODEC` writes it; an empty compound is none.
fn held_nbt(game: &Game<'_>, held: Option<BlockStateId>) -> Tag {
    // (The registry's own writes `id` and `properties`; a Java world's blocks have these names.)
    let Some(Tag::Compound(own)) = held.and_then(|s| game.pack.assets.registry.state(s)).map(|s| s.to_nbt()) else {
        return Tag::Compound(Compound::new());
    };
    let mut out = Compound::new();
    if let Some(name) = own.get(&java("id")).or_else(|| own.get(&java("Name"))) {
        out.insert(java("Name"), name.clone());
    }
    if let Some(props) = own.get(&java("properties")).or_else(|| own.get(&java("Properties"))) {
        out.insert(java("Properties"), props.clone());
    }
    Tag::Compound(out)
}

fn state_text(game: &Game<'_>, state: BlockStateId) -> Option<String> {
    held_text_of(&held_nbt(game, Some(state)))
}

fn held_tag(text: Option<&str>) -> Tag {
    let mut held = Compound::new();
    if let Some(text) = text {
        let (name, props) = split_state(text);
        held.insert(java("Name"), Tag::String(java(name)));
        if !props.is_empty() {
            let mut p = Compound::new();
            for (k, v) in props {
                p.insert(java(k), Tag::String(java(v)));
            }
            held.insert(java("Properties"), Tag::Compound(p));
        }
    }
    Tag::Compound(held)
}

fn parse(game: &Game<'_>, text: &str) -> Option<BlockStateId> {
    horizon_client::block_sprites::parse_state(&game.pack.assets.registry, text)
}

fn held_state(game: &Game<'_>, nbt: &Compound) -> Option<BlockStateId> {
    held_text(nbt).and_then(|t| parse(game, &t))
}

/// `hz_state` for the client's collisions, which have no registry to read a state's name with; -1 is none. The
/// number holds for this run of the game: each entity's tick puts it right after a load.
fn write_held(game: &Game<'_>, entity: &mut ModBlockEntity, text: Option<&str>) {
    let id = text.and_then(|t| parse(game, t)).map_or(-1, |s| s.0 as i32);
    entity.nbt.insert(java("held_id"), held_tag(text));
    entity.nbt.insert(java("hz_state"), Tag::Int(id));
}

// ---- The block entity

pub fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && is_elevator(block) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub fn block_entity_ticks(block: &str, ticks: &mut bool) {
    if is_elevator(block) {
        *ticks = true;
    }
}

pub fn block_entity_container(block: &str, out: &mut Option<ModContainer>) {
    if is_elevator(block) {
        *out = Some(ModContainer {
            // (A menu needs one here: nothing can reach it, see `screen`.)
            slots: 1,
            faces: std::array::from_fn(|_| Vec::new()),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: Some("elevatorid:elevator".to_owned()),
        });
    }
}

pub(in crate::game) fn block_entity_tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
    if !is_elevator(&entity.id) {
        return;
    }
    let text = held_text(&entity.nbt);
    let want = text.as_deref().and_then(|t| parse(game, t)).map_or(-1, |s| s.0 as i32);
    let has = match entity.nbt.get(&java("hz_state")) {
        Some(Tag::Int(n)) => *n,
        _ => -1,
    };
    if want != has {
        entity.nbt.insert(java("hz_state"), Tag::Int(want));
        STATE.with_borrow_mut(|s| s.sync.push(pos));
    }
    let lit = text.as_deref().is_some_and(|t| split_state(t).0 == "minecraft:redstone_block");
    STATE.with_borrow_mut(|s| {
        if lit {
            s.powered.insert(pos);
        } else {
            s.powered.remove(&pos);
        }
    });
}

pub fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    if is_elevator(&entity.id) {
        *tag = Some(entity.nbt.clone());
    }
}

/// The mesher draws the camouflage as the block itself (`camo_table`). What's left to draw is the arrow, which sits on
/// any elevator: its own model with the camouflaged state, which is only the arrow.
pub(in crate::game) fn submit(
    game: &Game<'_>,
    block: &str,
    state: BlockStateId,
    draw: &mut super::native_block_entity_draw::BlockEntityDraw,
) {
    if !is_elevator(block) {
        return;
    }
    let on = |key: &str| game.native_block_state_property(state, key) == Some("true");
    if on("camo") && on("directional") && on("show_arrow") {
        let facing = game.native_block_state_property(state, "facing").unwrap_or("north").to_owned();
        game.native_draw_block(draw, block, &[("camo", "true"), ("directional", "true"), ("show_arrow", "true"), ("facing", &facing)]);
    }
}

/// The mesher lists block entities by what it draws, and a camouflage's block mostly has none: the arrows go on the
/// list of those no renderer knows of, and `submit` draws them.
pub(in crate::game) fn arrows_to_draw(game: &Game<'_>, value: Vec<([i32; 3], u32)>) -> Vec<([i32; 3], u32)> {
    let mut out = value;
    STATE.with_borrow(|s| {
        for (p, _) in &s.camos {
            let state = game.world.block(p[0], p[1], p[2]);
            if game.native_block_state_property(state, "directional") == Some("true")
                && game.native_block_state_property(state, "show_arrow") == Some("true")
                && !out.iter().any(|(at, _)| at == p)
            {
                out.push((*p, state.0));
            }
        }
    });
    out
}

/// The client's camouflages, for the mesher: each tick the loaded elevators' `hz_state`, and the sections of what changed
/// are drawn again.
fn camo_table(game: &mut Game<'_>) {
    let mut now: Vec<([i32; 3], u32)> = Vec::new();
    let camera = game.player.as_ref().map(|p| (p.position.x, p.position.z));
    let Some((x, z)) = camera else { return };
    for p in game.world.block_entities.mods_near(x, z, 1024.0) {
        if let Some(horizon_client::containers::BlockEntity::Mod(m)) = game.world.block_entities.get(&(p.0, p.1, p.2))
            && is_elevator(&m.id)
            && let Some(Tag::Int(id)) = m.nbt.get(&java("hz_state"))
            && *id >= 0
            && is_elevator(&game.res.facts().get(game.world.block(p.0, p.1, p.2)).name)
        {
            now.push(([p.0, p.1, p.2], *id as u32));
        }
    }
    now.sort_unstable();
    let was = STATE.with_borrow_mut(|s| {
        if s.camos == now {
            None
        } else {
            Some(std::mem::replace(&mut s.camos, now.clone()))
        }
    });
    let Some(was) = was else { return };
    horizon_client::mesher::Mesher::hz_elevator_set_camos(&now);
    // What changed, either way, and the sections touching it (neighbours cull against it).
    let min_y = game.world.shape.min_y;
    let changed = was.iter().filter(|e| !now.contains(e)).chain(now.iter().filter(|e| !was.contains(e)));
    let mut dirty = Vec::new();
    for ([x, y, z], _) in changed {
        for (dx, dy, dz) in [(0, 0, 0), (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
            dirty.push(((x + dx).div_euclid(16), (y + dy - min_y).div_euclid(16), (z + dz).div_euclid(16)));
        }
    }
    for section in dirty {
        if section.1 >= 0 {
            game.world.dirty_sections.insert(section);
        }
    }
}

/// `ElevatorBlockBase.getShape`: what the pointer selects is the camouflage's.
pub(in crate::game) fn outline(game: &Game<'_>, pos: Pos) -> Option<Vec<[f64; 6]>> {
    let Some(horizon_client::containers::BlockEntity::Mod(m)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2])) else {
        return None;
    };
    if !is_elevator(&m.id) {
        return None;
    }
    let state = held_state(game, &m.nbt)?;
    // (In the world's coordinates, as `outline_boxes` answers.)
    let base = [f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2])];
    Some(game.res.facts().get(state).outline.iter().map(|b| std::array::from_fn(|i| b[i] + base[i % 3])).collect())
}

pub fn is_signal_source(name: &str, source: &mut bool) {
    if is_elevator(name) {
        *source = true;
    }
}

pub fn block_signal(pos: Pos, signal: &mut i32) {
    if STATE.with_borrow(|s| s.powered.contains(&pos)) {
        *signal = 15;
    }
}

// ---- Using the block (`ElevatorBlockBase.useItemOn`)

/// `ElevatorBlockEntityBase.isValidState`.
fn valid_camouflage(game: &Game<'_>, state: BlockStateId) -> bool {
    let facts = game.res.facts().get(state);
    let name: &str = &facts.name;
    if is_elevator(name) || name == "minecraft:air" || name == "minecraft:cave_air" || name == "minecraft:void_air" {
        return false;
    }
    if !matches!(facts.render_shape, horizon_items::blocks::RenderShape::Model) {
        return false;
    }
    // "Only blocks with a collision box": carpets and snow have thin ones, and are no `legacy_solid`.
    facts.full_collision || facts.legacy_solid || name.ends_with("_carpet") || name == "minecraft:snow"
}

/// `handBlock.getStateForPlacement(new FakeUseContext(player, hand, hit))`: the game's own, with the click and the
/// player's look. FakeUseContext stands the block where the click was, so what the neighbours allow doesn't come into it
/// beyond what the game's own asks.
fn placed_state(game: &mut Game<'_>, item: &str, pos: Pos, face: Pos) -> Option<BlockStateId> {
    use horizon_core::geometry::Direction as D;
    let direction = match face {
        [0, 1, 0] => D::Up,
        [0, -1, 0] => D::Down,
        [0, 0, -1] => D::North,
        [0, 0, 1] => D::South,
        [-1, 0, 0] => D::West,
        _ => D::East,
    };
    let player = game.native_acting_player()?;
    let ctx = horizon_world::level::PlaceContext {
        clicked_pos: horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]),
        face: direction,
        hit: horizon_core::geometry::Vec3::new(
            f64::from(pos[0]) + 0.5 + f64::from(face[0]) * 0.5,
            f64::from(pos[1]) + 0.5 + f64::from(face[1]) * 0.5,
            f64::from(pos[2]) + 0.5 + f64::from(face[2]) * 0.5,
        ),
        inside: false,
        yaw: player.yaw,
        pitch: player.pitch,
        sneaking: false,
        item: item.to_owned(),
        can_use_game_master_blocks: false,
    };
    // BlockItem.place's answer without placing anything, with Block.byItem in it; a block nothing could be placed for
    // here (the place beside the click is taken) is its usual state.
    let placed = game.with_level(|behaviours, level, ticks| behaviours.placement_state(level, ticks, &ctx));
    placed.map(|(_, state)| state).or_else(|| game.native_default_state(item))
}

fn sneaking(game: &Game<'_>) -> bool {
    let body = game.player.as_ref().is_some_and(|p| p.pose == horizon_client::pose::Pose::Crouching);
    if game.lan_acting.is_some() {
        return body;
    }
    body || game.has_shift_down()
}

fn set_camouflage(game: &mut Game<'_>, pos: Pos, held: Option<BlockStateId>) {
    let parsed = held.map_or(-1, |s| s.0 as i32);
    let tag = held_nbt(game, held);
    let text = held.and_then(|s| state_text(game, s));
    let text = text.as_deref();
    game.native_with_mod_block_entity(pos, |e| {
        e.nbt.insert(java("held_id"), tag);
        e.nbt.insert(java("hz_state"), Tag::Int(parsed));
    });
    // getLightEmission: what any block gives off, a mod's too.
    let light = held.map_or(0, |s| game.res.facts().get(s).emission);
    game.native_set_block_property_with(pos, "light", &light.to_string(), SET_BLOCK);
    // The signal first: setting the property is what tells the neighbours to look.
    let lit = text.is_some_and(|t| split_state(t).0 == "minecraft:redstone_block");
    STATE.with_borrow_mut(|s| {
        if lit {
            s.powered.insert(pos);
        } else {
            s.powered.remove(&pos);
        }
    });
    game.native_set_block_property_with(pos, "camo", if text.is_some() { "true" } else { "false" }, SET_BLOCK);
    game.native_block_entity_sync(pos);
    play(game, SOUND_CAMOUFLAGE, pos);
}

fn play(game: &mut Game<'_>, sound: &str, pos: Pos) {
    let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 0.5, f64::from(pos[2]) + 0.5];
    if !game.with_level(|_, level, _| level.native_play_sound(sound, at, SOUND_BLOCKS, 1.0, 1.0)) {
        log::warn!("Elevator: the sound {sound} could not be played");
    }
}

pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: Pos, face: Pos, canceled: &mut bool) {
    if !is_elevator(&game.native_block(pos)) {
        return;
    }
    let sneak = sneaking(game);
    let held: Option<ItemStack> = game.held().cloned();
    // A crouching player with something in hand places or uses that: useItemOn isn't asked.
    if sneak && held.is_some() {
        return;
    }
    let camo = {
        let key = (pos[0], pos[1], pos[2]);
        match game.world.block_entities.get(&key) {
            Some(horizon_client::containers::BlockEntity::Mod(m)) => held_state(game, &m.nbt),
            _ => None,
        }
    };
    // The block in hand: tile.setCamoAndUpdate, which refuses what it already holds.
    if let Some(stack) = &held
        && let Some(state) = placed_state(game, &stack.item, pos, face)
        && camo != Some(state)
        && valid_camouflage(game, state)
    {
        set_camouflage(game, pos, Some(state));
        *canceled = true;
        return;
    }
    if sneak && camo.is_some() {
        set_camouflage(game, pos, None);
        *canceled = true;
    }
    // Otherwise the block's menu opens, as Horizon does for a block entity with one.
}

// ---- The screen (`ElevatorScreen`, `FacingControllerWrapper`, `FacingButton`)

const FACING_ORIGIN: [f32; 2] = [120.0, 20.0];
const CHECK_DIRECTIONAL: [f32; 2] = [8.0, 25.0];
const CHECK_ARROW: [f32; 2] = [8.0, 50.0];
const RESET: [f32; 4] = [8.0, 75.0, 110.0, 20.0];
const CHECK_SIZE: f32 = 17.0;

/// The player's facing's place in `Collections.rotate(slots, playerFacing.get2DDataValue())`.
fn facing_rects(game: &Game<'_>) -> [[f32; 4]; 4] {
    let yaw = game.native_player().map_or(0.0, |p| p.yaw);
    let turn = ((yaw / 90.0 + 0.5).floor() as i32 & 3) as usize;
    let slots = [[20.0, 0.0], [40.0, 20.0], [20.0, 40.0], [0.0, 20.0]];
    std::array::from_fn(|i| {
        let [x, y]: [f32; 2] = slots[(i + 4 - turn) % 4];
        [FACING_ORIGIN[0] + x, FACING_ORIGIN[1] + y, 20.0, 20.0]
    })
}

fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}

fn check_rect(at: [f32; 2], game: &Game<'_>, label: &str) -> [f32; 4] {
    // Checkbox.builder: its box and its message are one widget.
    [at[0], at[1], CHECK_SIZE + 4.0 + game.native_text_width(label), CHECK_SIZE]
}

fn lang(key: &str, english: &str) -> String {
    horizon_client::lang::get_or(key, english).to_string()
}

fn viewed(game: &Game<'_>) -> Option<(Pos, bool, bool, String, bool)> {
    let pos = game.native_screen_pos()?;
    let directional = game.native_block_property(pos, "directional").as_deref() == Some("true");
    let arrow = game.native_block_property(pos, "show_arrow").as_deref() != Some("false");
    let facing = game.native_block_property(pos, "facing").unwrap_or_else(|| "north".to_owned());
    let camo = game.native_block_property(pos, "camo").as_deref() == Some("true");
    Some((pos, directional, arrow, facing, camo))
}

pub(in crate::game) fn screen(_game: &Game<'_>, block: &str, out: &mut Option<super::native_hooks::NativeScreen>) {
    if is_elevator(block) {
        *out = Some(super::native_hooks::NativeScreen {
            width: 200.0,
            height: 100.0,
            slots: vec![None],
            // ElevatorContainer has no player slots: Horizon's are put out of sight and out of reach.
            inventory_y: -100_000.0,
            inventory_x: -100_000.0,
            own_labels: true,
            own_background: true,
        });
    }
}

fn draw_check(game: &Game<'_>, batch: &mut HudBatch, origin: [f32; 2], at: [f32; 2], label: &str, selected: bool, mouse: [f32; 2]) {
    let hover = inside(check_rect(at, game, label), mouse);
    let sprite = match (selected, hover) {
        (true, true) => "gui/sprites/widget/checkbox_selected_highlighted",
        (true, false) => "gui/sprites/widget/checkbox_selected",
        (false, true) => "gui/sprites/widget/checkbox_highlighted",
        (false, false) => "gui/sprites/widget/checkbox",
    };
    let (x, y) = (origin[0] + at[0], origin[1] + at[1]);
    batch.sprite_scaled(&game.gui.atlas, sprite, x, y, CHECK_SIZE, CHECK_SIZE, u32::MAX);
    game.native_text(batch, label, [x + CHECK_SIZE + 4.0, y + 4.0], 0xE0E0E0, true);
}

pub(in crate::game) fn screen_draw(game: &Game<'_>, block: &str, batch: &mut HudBatch, origin: [f32; 2]) {
    if !is_elevator(block) {
        return;
    }
    let [ox, oy] = origin;
    let mouse = STATE.with_borrow(|s| s.mouse);
    game.native_sprite(batch, "elevatorid:gui/elevator_gui", [0.0, 0.0, 200.0, 100.0], [ox, oy, 200.0, 100.0]);
    game.native_text(batch, &lang("screen.elevatorid.elevator", "Elevator Options"), [ox + 8.0, oy + 8.0], 0xE0E0E0, true);
    let Some((_, directional, arrow, facing, camo)) = viewed(game) else { return };
    draw_check(game, batch, origin, CHECK_DIRECTIONAL, &lang("screen.elevatorid.elevator.directional", "Directional"), directional, mouse);
    if directional {
        draw_check(game, batch, origin, CHECK_ARROW, &lang("screen.elevatorid.elevator.hide_arrow", "Hide arrow"), !arrow, mouse);
        let english = ["S", "W", "N", "E"];
        for (i, rect) in facing_rects(game).into_iter().enumerate() {
            let active = FACINGS[i] != facing;
            let (x, y) = (ox + rect[0], oy + rect[1]);
            if active && inside(rect, mouse) {
                game.native_fill(batch, [x, y, rect[2], rect[3]], 0x80FF_FFFF);
            }
            let label = lang(&format!("screen.elevatorid.elevator.directional_{}", FACINGS[i]), english[i]);
            let w = game.native_text_width(&label);
            game.native_text(batch, &label, [x + (rect[2] - w) / 2.0, y + (rect[3] - 8.0) / 2.0], if active { 0xE0E0E0 } else { 0x00FF00 }, true);
        }
    }
    // The Remove camouflage button.
    let hover = inside(RESET, mouse);
    let sprite = match (camo, hover) {
        (false, _) => "gui/sprites/widget/button_disabled",
        (true, true) => "gui/sprites/widget/button_highlighted",
        (true, false) => "gui/sprites/widget/button",
    };
    let (x, y) = (ox + RESET[0], oy + RESET[1]);
    horizon_client::ui::nine_slice(batch, &game.gui.atlas, sprite, x, y, RESET[2], RESET[3], u32::MAX);
    let label = lang("screen.elevatorid.elevator.reset_camo", "Remove camouflage");
    let w = game.native_text_width(&label);
    game.native_text(batch, &label, [x + (RESET[2] - w) / 2.0, y + 6.0], if camo { 0xFFFFFF } else { 0xA0A0A0 }, true);
}

pub(in crate::game) fn screen_click(game: &Game<'_>, block: &str, at: [f32; 2], right: bool, button: &mut Option<i32>) {
    if !is_elevator(block) || right {
        return;
    }
    let Some((_, directional, _, facing, camo)) = viewed(game) else { return };
    if inside(check_rect(CHECK_DIRECTIONAL, game, &lang("screen.elevatorid.elevator.directional", "Directional")), at) {
        *button = Some(BUTTON_DIRECTIONAL);
    } else if directional && inside(check_rect(CHECK_ARROW, game, &lang("screen.elevatorid.elevator.hide_arrow", "Hide arrow")), at) {
        *button = Some(BUTTON_ARROW);
    } else if camo && inside(RESET, at) {
        *button = Some(BUTTON_RESET);
    } else if directional {
        for (i, rect) in facing_rects(game).into_iter().enumerate() {
            if FACINGS[i] != facing && inside(rect, at) {
                *button = Some(BUTTON_FACING + i as i32);
            }
        }
    }
}

pub fn screen_tooltip(block: &str, at: [f32; 2]) {
    if is_elevator(block) {
        STATE.with_borrow_mut(|s| s.mouse = at);
    }
}

pub(in crate::game) fn menu_button(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, id: i32) {
    if !is_elevator(&entity.id) {
        return;
    }
    let toggled = |game: &Game<'_>, key: &str, default: bool| {
        let now = game.native_block_property(pos, key).map_or(default, |v| v == "true");
        if now { "false" } else { "true" }
    };
    match id {
        BUTTON_DIRECTIONAL => {
            let value = toggled(game, "directional", false);
            game.native_set_block_property_with(pos, "directional", value, SET_BLOCK);
        }
        BUTTON_ARROW => {
            let value = toggled(game, "show_arrow", true);
            game.native_set_block_property_with(pos, "show_arrow", value, SET_BLOCK);
        }
        BUTTON_RESET => {
            if held_text(&entity.nbt).is_some() {
                write_held(game, entity, None);
                STATE.with_borrow_mut(|s| {
                    s.powered.remove(&pos);
                    s.sync.push(pos);
                });
                game.native_set_block_property_with(pos, "light", "0", SET_BLOCK);
                game.native_set_block_property_with(pos, "camo", "false", SET_BLOCK);
                play(game, SOUND_CAMOUFLAGE, pos);
            }
        }
        n if (BUTTON_FACING..BUTTON_FACING + 4).contains(&n) => {
            game.native_set_block_property_with(pos, "facing", FACINGS[(n - BUTTON_FACING) as usize], SET_BLOCK);
        }
        _ => {}
    }
}

// ---- Going up and down (`ElevatorHandler`, `TeleportPacket`)

fn elevator_at(game: &Game<'_>, pos: Pos) -> Option<String> {
    let name: &str = &game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name;
    is_elevator(name).then(|| name.to_owned())
}

/// `BlockState.isSuffocating`, what stands above an elevator to bar it: a full, solid cube. An elevator is one
/// too (see-through only because a camouflage is drawn in it).
fn valid_pos(game: &Game<'_>, pos: Pos) -> bool {
    let above = game.res.facts().get(game.world.block(pos[0], pos[1] + 1, pos[2]));
    !(above.full_collision && (above.solid_render || is_elevator(&above.name)))
}

fn key_held(game: &Game<'_>, mapping: &str) -> bool {
    let pad = game.pad.holds_action(&game.settings.controller, mapping);
    let key = horizon_client::keybinds::key_code(game.settings.keys.get(mapping)).is_some_and(|code| game.keys.contains(&code));
    pad || key
}

/// `ElevatorHandler.handleInput`, each client tick: a key's press, not its hold.
pub(in crate::game) fn client_tick(game: &mut Game<'_>) {
    camo_table(game);
    let playing = game.player.is_some() && matches!(game.screen, super::Screen::Playing);
    let sneak = playing && key_held(game, "key.sneak");
    let jump = playing && key_held(game, "key.jump");
    let (sneak_press, jump_press) = STATE.with_borrow_mut(|s| {
        let pressed = (sneak && !s.last_sneak, jump && !s.last_jump);
        s.last_sneak = sneak;
        s.last_jump = jump;
        pressed
    });
    if sneak_press {
        try_teleport(game, -1);
    }
    if jump_press {
        try_teleport(game, 1);
    }
}

fn origin_elevator(game: &Game<'_>) -> Option<Pos> {
    let p = game.player.as_ref()?;
    let mut pos = [p.position.x.floor() as i32, p.position.y.floor() as i32, p.position.z.floor() as i32];
    for _ in 0..config().activation_range {
        if elevator_at(game, pos).is_some() {
            return valid_pos(game, pos).then_some(pos);
        }
        pos[1] -= 1;
    }
    None
}

/// The y of the first elevator `matches` accepts going `step` (1 up, -1 down) from `from_y`, within `range` blocks
/// and the world's band; `None` past them. Used by the client to ask and by the server to decide.
fn next_elevator_y(from_y: i32, step: i32, range: i32, mut matches: impl FnMut(i32) -> bool) -> Option<i32> {
    let step = step.signum();
    if step == 0 {
        return None;
    }
    let mut y = i64::from(from_y);
    for _ in 0..range.max(0) {
        y += i64::from(step);
        if !(-2048..=4096).contains(&y) {
            return None;
        }
        if matches(y as i32) {
            return Some(y as i32);
        }
    }
    None
}

/// The elevator a ride from `from` goes to, `step` 1 up or -1 down: the next one in range, of the same colour when
/// the option asks, with room to stand.
fn destination(game: &Game<'_>, from: Pos, step: i32) -> Option<Pos> {
    let from_block = elevator_at(game, from)?;
    let y = next_elevator_y(from[1], step, config().range, |y| {
        let to = [from[0], y, from[2]];
        elevator_at(game, to).is_some_and(|to_block| valid_pos(game, to) && (!config().same_color || colour(&from_block) == colour(&to_block)))
    })?;
    Some([from[0], y, from[2]])
}

fn try_teleport(game: &mut Game<'_>, step: i32) {
    let Some(from) = origin_elevator(game) else { return };
    let Some(to) = destination(game, from, step) else { return };
    // The server takes only the origin and the direction from this; it finds the destination itself.
    let message = format!("{} {} {} {} {} {}", from[0], from[1], from[2], to[0], to[1], to[2]);
    let _ = game.native_net_send_to_server(NET_TELEPORT, message.as_bytes());
}

/// A player's message: kept for the tick, one each.
pub fn from_client(player: u128, channel: &str, data: &[u8]) {
    if channel != NET_TELEPORT || data.len() > 96 {
        return;
    }
    let text = String::from_utf8_lossy(data);
    let n: Vec<i32> = text.split(' ').filter_map(|w| w.parse().ok()).collect();
    if let [a, b, c, d, e, f] = n[..] {
        STATE.with_borrow_mut(|s| {
            if !s.teleports.iter().any(|(p, _, _)| *p == player) {
                s.teleports.push((player, [a, b, c], [d, e, f]));
            }
        });
    }
}

/// `native_server_tick`, at its end.
pub(in crate::game) fn server_tick(game: &mut Game<'_>) {
    // Block.Properties.isValidSpawn: the option's answer, whatever the state's light. Once, when it's on.
    if config().mob_spawn {
        static TOLD: std::sync::Once = std::sync::Once::new();
        TOLD.call_once(|| {
            let states = game.res.facts().states.iter().map(|f| is_elevator(&f.name)).collect();
            horizon_client::mobs::data::MobData::hz_elevator_set_spawnable(states);
        });
    }
    let sync = STATE.with_borrow_mut(|s| std::mem::take(&mut s.sync));
    for pos in sync {
        game.native_block_entity_sync(pos);
    }
    let teleports = STATE.with_borrow_mut(|s| std::mem::take(&mut s.teleports));
    if teleports.is_empty() {
        return;
    }
    let back = game.dim;
    for (uuid, from, to) in teleports {
        let Some(player) = game.native_server_players().into_iter().find(|p| p.uuid == uuid) else { continue };
        let Some(dim) = game.levels_to_tick().into_iter().find(|d| d.id() == player.dimension) else { continue };
        if game.enter_level(None, dim).is_ok() {
            teleport(game, &player, from, to);
        }
    }
    let _ = game.enter_level(None, back);
}

/// Whether a client's origin is one it may ride from: near its player (by blocks, in i64 so far-off numbers can't
/// overflow), and its destination in the same column. Only the direction of `to` is used after this.
fn origin_near(at: [f64; 3], from: Pos, to: Pos, activation_range: i32) -> bool {
    let at = at.map(|v| if v.is_finite() { v.floor() as i64 } else { i64::MAX / 4 });
    let distance: i64 = (0..3).map(|i| (at[i] - i64::from(from[i])).abs()).sum();
    distance <= i64::from(activation_range) && from[0] == to[0] && from[2] == to[2] && from[1] != to[1]
}

/// `TeleportPacket.isBadTeleportPacket`, then `handle`. The client names its origin and a destination; the server
/// checks the origin and finds the destination itself from the direction (the next elevator within `range`), so a
/// modified client can't go further or skip one.
fn teleport(game: &mut Game<'_>, player: &horizon_client::mod_api::NativeServerPlayer, from: Pos, asked: Pos) {
    if player.health <= 0.0 || player.game_mode == 3 {
        return;
    }
    if !origin_near(player.position, from, asked, config().activation_range) || !valid_pos(game, from) {
        return;
    }
    let step = (i64::from(asked[1]) - i64::from(from[1])).signum() as i32;
    let Some(to) = destination(game, from, step) else { return };
    // XP: experience in points through the game's own command, as `giveExperiencePoints(-cost)`.
    if config().use_xp && player.game_mode != 1 {
        let cost = config().xp_points;
        let points = game.native_run_command_captured(&format!("experience query {} points", player.name)).result as i64;
        let levels = game.native_run_command_captured(&format!("experience query {} levels", player.name)).result as i64;
        if points - i64::from(cost) >= 0 || levels > 0 {
            game.native_run_command(&format!("experience add {} -{cost} points", player.name));
        } else {
            let line = lang("elevatorid.message.missing_xp", "You do not have enough XP to use elevators");
            game.native_action_bar(player.uuid, horizon_client::text::Text::legacy(&format!("§c{line}")));
            return;
        }
    }
    let (x, z) = if config().precision_target {
        (f64::from(to[0]) + 0.5, f64::from(to[2]) + 0.5)
    } else {
        (player.position[0], player.position[2])
    };
    // getBlockSupportShape: a camouflage's top, the block's own otherwise.
    let top = match game.world.block_entities.get(&(to[0], to[1], to[2])) {
        Some(horizon_client::containers::BlockEntity::Mod(m)) => held_state(game, &m.nbt)
            .map_or(1.0, |s| game.res.facts().get(s).outline.iter().map(|b| b[4]).fold(0.0, f64::max)),
        _ => 1.0,
    };
    let y = f64::from(to[1]) + top.max(0.0);
    let directional = game.native_block_property(to, "directional").as_deref() == Some("true");
    let yaw = if directional {
        match game.native_block_property(to, "facing").as_deref() {
            Some("south") => 0.0,
            Some("west") => 90.0,
            Some("east") => -90.0,
            _ => 180.0,
        }
    } else {
        player.yaw
    };
    let reset = if directional { config().reset_pitch_directional } else { config().reset_pitch_normal };
    let pitch = if reset { 0.0 } else { player.pitch };
    if game.native_teleport_player(player.uuid, &player.dimension, [x, y, z], yaw, pitch) {
        play(game, SOUND_TELEPORT, to);
    }
}

/// The powered camouflages still there (a broken block's entity no longer ticks to say).
pub(in crate::game) fn sweep_powered(game: &Game<'_>) {
    // (`elevator_at` reads the level in place: with another open, what stands there can't be told.)
    if game.levels_to_tick().len() > 1 {
        return;
    }
    let stale: Vec<Pos> = STATE.with_borrow(|s| s.powered.iter().copied().filter(|p| elevator_at(game, *p).is_none()).collect());
    if !stale.is_empty() {
        STATE.with_borrow_mut(|s| stale.iter().for_each(|p| {
            s.powered.remove(p);
        }));
    }
}

/// `native_frame`: the Elevator in the kit: its guide pages, and its place on JEI's pages (Elevator adds no category
/// of its own: its recipes are vanilla's, shown with everything else's).
pub(in crate::game) fn kit_frame(game: &mut Game<'_>) {
    if !game.native_on("elevatorid") {
        return;
    }
    kit_guide::register("elevatorid", "Elevator Guide", "Elevator guide");
    elevatorid_optional::if_mod!("jei", {
        kit_layer::register(kit_layer::Provider::new("elevatorid"));
    } else { () });
}

#[cfg(test)]
mod destination_tests {
    use super::{next_elevator_y, origin_near};

    #[test]
    fn out_of_range_destinations_are_refused() {
        // Elevators at 10 and 500, range 384: 500 is out of reach from 10.
        let at = |y| y == 10 || y == 500;
        assert_eq!(next_elevator_y(10, 1, 384, at), None);
        assert_eq!(next_elevator_y(10, 1, 490, at), Some(500));
        assert_eq!(next_elevator_y(500, -1, 384, at), None);
        // The world's band ends the search however large the range.
        assert_eq!(next_elevator_y(4090, 1, 4096, |y| y == 5000), None);
        assert_eq!(next_elevator_y(-2040, -1, 4096, |y| y == -3000), None);
        assert_eq!(next_elevator_y(i32::MAX, 1, i32::MAX, |_| true), None);
        assert_eq!(next_elevator_y(i32::MIN, -1, i32::MAX, |_| true), None);
    }

    #[test]
    fn the_next_elevator_is_taken_never_a_further_one() {
        let at = |y| [0, 5, 9, 20].contains(&y);
        assert_eq!(next_elevator_y(0, 1, 384, at), Some(5));
        assert_eq!(next_elevator_y(5, 1, 384, at), Some(9));
        assert_eq!(next_elevator_y(20, -1, 384, at), Some(9));
        assert_eq!(next_elevator_y(0, -1, 384, at), None);
        // Only the direction of a request counts: a step of 15 is still one elevator up.
        assert_eq!(next_elevator_y(0, 15, 384, at), Some(5));
        assert_eq!(next_elevator_y(0, 0, 384, at), None);
    }

    #[test]
    fn far_off_origins_and_columns_are_refused_without_overflow() {
        let me = [0.5, 64.0, 0.5];
        assert!(origin_near(me, [0, 63, 0], [0, 80, 0], 6));
        assert!(!origin_near(me, [0, 63, 0], [1, 80, 0], 6));
        assert!(!origin_near(me, [0, 63, 0], [0, 63, 0], 6));
        assert!(!origin_near(me, [0, 50, 0], [0, 80, 0], 6));
        assert!(!origin_near(me, [i32::MIN, i32::MIN, i32::MIN], [i32::MIN, i32::MAX, i32::MIN], 6));
        assert!(!origin_near(me, [i32::MAX, i32::MAX, i32::MAX], [i32::MAX, 0, i32::MAX], 1024));
        assert!(!origin_near([f64::NAN, 64.0, 0.5], [0, 63, 0], [0, 80, 0], 6));
    }
}
