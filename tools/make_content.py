"""Builds Elevator's blocks and items as a Horizon content mod, from the Fabric jar for 26.3.

The jar's data is already in 26.3's form, so most files are copied as they are. What changes:

- each elevator gets a block declaration: the beacon's behaviour (a see-through block with a block
  entity), Elevator's strength and sound, and its four state properties;
- the blockstate becomes a multipart: the block's look while not camouflaged (the mod draws the
  camouflage itself) and the arrow while directional;
- `c:dyes/<colour>` is written out: Fabric gives that tag, Horizon doesn't, and redyeing needs it.

Usage: python make_content.py <jar> <out .hmod>
"""

import io
import json
import sys
import zipfile

ID = "elevatorid"
COLOURS = [
    "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
    "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black",
]
# ElevatorBlockBase: facing to the arrow model's turn.
ARROW_TURN = {"south": 0, "west": 90, "north": 180, "east": 270}


def block_declaration():
    return {
        "template": "minecraft:beacon",
        # Block.Properties.of().strength(0.8F).sound(SoundType.WOOL).noOcclusion()
        "properties": {
            "hardness": 0.8,
            "explosion_resistance": 0.8,
            "sounds": "minecraft:white_wool",
        },
        "state_properties": {
            "camo": {"type": "boolean"},
            "directional": {"type": "boolean"},
            "facing": {"type": "enum", "values": ["north", "south", "west", "east"]},
            # ElevatorBlockBase.LIGHT: what its camouflage gives off (the hook on the state table reads it).
            "light": {"type": "enum", "values": [str(n) for n in range(16)]},
            "show_arrow": {"type": "boolean"},
        },
        "default_state": {
            "camo": "false",
            "directional": "false",
            "facing": "north",
            "light": "0",
            "show_arrow": "true",
        },
    }


def blockstate(colour):
    parts = [{"when": {"camo": "false"}, "apply": {"model": f"{ID}:block/elevator_{colour}"}}]
    for facing, turn in ARROW_TURN.items():
        apply = {"model": f"{ID}:block/arrow"}
        if turn:
            apply["y"] = turn
        parts.append({"when": {"directional": "true", "show_arrow": "true", "facing": facing}, "apply": apply})
    return {"multipart": parts}


def guide(files):
    """The W guide's pages, in the form the kit reads (AE2's): an index and Markdown."""
    items = {f"{ID}:elevator_{c}": "elevator.md" for c in COLOURS}
    files[f"data/{ID}/ae2guide/index.json"] = json.dumps({
        "items": items,
        "titles": {"elevator.md": "Elevator", "camouflage.md": "Camouflage", "directional.md": "Directional Elevators"},
        "parents": {"camouflage.md": "elevator.md", "directional.md": "elevator.md"},
        "positions": {"elevator.md": "0", "camouflage.md": "10", "directional.md": "20"},
        "icons": {"elevator.md": f"{ID}:elevator_white", "camouflage.md": f"{ID}:elevator_white", "directional.md": f"{ID}:elevator_white"},
        "categories": {},
    }, indent=2).encode()
    files[f"data/{ID}/ae2guide/elevator.md"] = "\n".join([
        "# Elevator",
        "",
        "Elevators take you straight up or down to the next elevator in the same column.",
        "",
        "Stand on one and **jump** to go up, or **sneak** to go down. You may be up to four blocks above it. "
        "The next elevator above or below is used, within 384 blocks by default, as long as nothing solid stands on top of it.",
        "",
        "Using an elevator makes a sound and costs nothing, unless the server turned on `useXP` in `elevatorid-common.toml`. "
        "With `sameColor` on, an elevator only leads to one of its own colour.",
        "",
        "Elevators come in 16 colours. Any elevator with a dye makes one of that dye's colour.",
        "",
        '<RecipeFor id="elevator_white" />',
        "",
        '<RecipeFor id="elevator_black" />',
        "",
        "Right-click one with an empty hand for its options. "
        "It can look like another block: see [Camouflage](camouflage.md). "
        "It can also turn you as you arrive: see [Directional Elevators](directional.md).",
        "",
    ]).encode()
    files[f"data/{ID}/ae2guide/camouflage.md"] = "\n".join([
        "# Camouflage",
        "",
        "Right-click an elevator with a block to make it look like that block. "
        "It also takes that block's shape, so a slab makes it a half-height elevator, and you arrive on top of the slab.",
        "",
        "It works with blocks that are drawn as normal blocks and have a collision shape: not air, flowers or chests, "
        "and not another elevator. The block in your hand isn't used up.",
        "",
        "A redstone block as its camouflage gives a redstone signal, as the block does.",
        "",
        "To take it off, sneak and right-click with an empty hand, or use **Remove camouflage** in its options.",
        "",
    ]).encode()
    files[f"data/{ID}/ae2guide/directional.md"] = "\n".join([
        "# Directional Elevators",
        "",
        "Tick **Directional** in an elevator's options and arriving there turns you to the direction it is set to, "
        "and levels your view.",
        "",
        "The letters N, E, S and W set the direction; the current one is green. They are placed around the middle as you "
        "stand, so the top one is the way you are facing. An arrow on top of the block shows the direction; **Hide arrow** "
        "takes it away.",
        "",
    ]).encode()


def fix_conditions(value):
    if isinstance(value, list):
        return [fix_conditions(v) for v in value]
    if isinstance(value, dict):
        return {("type" if k == "condition" else k): fix_conditions(v) for k, v in value.items()}
    return value


def main(jar, out):
    source = zipfile.ZipFile(jar)
    files = {}
    for name in source.namelist():
        if name.endswith("/") or not name.startswith(("assets/", "data/")):
            continue
        if name.startswith(f"assets/{ID}/blockstates/"):
            continue
        if name == f"assets/{ID}/models/arrow.json":
            # With the block models, where a blockstate looks for them.
            model = json.loads(source.read(name))
            model.pop("render_type", None)
            for element in model.get("elements", []):
                element.pop("__comment", None)
            files[f"assets/{ID}/models/block/arrow.json"] = json.dumps(model, indent=2).encode()
            continue
        if name.startswith(f"data/{ID}/loot_table/"):
            # A condition's kind is `type` to Horizon's loot tables; the jar's say `condition`.
            files[name] = json.dumps(fix_conditions(json.loads(source.read(name))), indent=2).encode()
            continue
        files[name] = source.read(name)
    for colour in COLOURS:
        block = f"elevator_{colour}"
        files[f"data/{ID}/horizon/block/{block}.json"] = json.dumps(block_declaration(), indent=2).encode()
        files[f"assets/{ID}/blockstates/{block}.json"] = json.dumps(blockstate(colour), indent=2).encode()
        files[f"data/c/tags/item/dyes/{colour}.json"] = json.dumps({"replace": False, "values": [f"minecraft:{colour}_dye"]}, indent=2).encode()
    # A menu has at least one slot here; ElevatorContainer has none, so this one is out of sight and takes nothing.
    guide(files)
    files[f"data/{ID}/horizon/menu/elevator.json"] = json.dumps({"slots": 1, "data_slots": 0}, indent=2).encode()
    files[f"data/{ID}/horizon/creative_tab/main.json"] = json.dumps({
        "title": "itemGroup.elevators_tab",
        "icon": f"{ID}:elevator_white",
        "items": [f"{ID}:elevator_{c}" for c in COLOURS],
    }, indent=2).encode()
    files["horizon-mod.toml"] = "\n".join([
        "# Elevator's blocks and items, from elevatorid-fabric-26.3-1.16.6.jar by elevator-horizon/tools/make_content.py.",
        f'id = "{ID}"',
        'name = "Elevator (blocks and items)"',
        'version = "1.16.6"',
        'description = "Elevator\'s blocks, recipes and textures."',
        'authors = ["VsnGamer"]',
        'api = "0.2"',
        'side = "both"',
        'capabilities = ["registries"]',
        "",
    ]).encode()
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as z:
        for name in sorted(files):
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            z.writestr(info, files[name])
    with open(out, "wb") as f:
        f.write(buffer.getvalue())
    print(len(files), "files,", len(buffer.getvalue()), "bytes")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
