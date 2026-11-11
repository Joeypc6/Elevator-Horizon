# Elevator for Horizon

An unofficial port of [Elevator](https://github.com/VsnGamer/ElevatorMod) 1.16.6 (Fabric, MIT, by VsnGamer) to
[Horizon](https://github.com/horizon-rust/horizon), as one native mod.

- `mixins/elevatorid`: the mod. `elevator.rs` is the logic, `data/elevatorid-content.hmod` its blocks, items, recipes and textures.
- `tools/make_content.py`: makes the content from the original jar. `tools/make_bundle.py` writes the bundle's `horizon-mixin.toml` and helpers.

## What it does

Sixteen coloured blocks. Stand on one and jump to go to the next above, sneak to go to the next below. They can be camouflaged with
another block (with its light, tint, shape and collision), made directional, and dyed. `elevatorid-common.toml` in the config folder
holds the Java mod's options: `sameColor`, `range`, `activationRange`, `precisionTarget`, `mobSpawn`, `resetPitchNormal`,
`resetPitchDirectional`, `useXP` and `XPPointsAmount`.

In multiplayer the server finds the destination itself from the way the player asked to go: the next elevator within `range`, never one further or past it, whatever a client sends.

With JEI for Horizon in the build, its recipes and the W guide work with no other mod needed.

## What differs from the Java mod

The Java mod was rewritten in Rust for Horizon's hooks, so no original code runs.

- The guide pages are new: the Java mod has none.
- A camouflage is drawn by swapping the block's state where the mesher reads it, so it doesn't rely on a model of its own.

## Tested

On Horizon commit `6560641` (Minecraft 26.3), on Windows: single player, LAN with one guest and a dedicated server. Rides, directional
blocks, camouflage (with a light source, tinted grass and an AE2 glass block), mobs landing on a slab camouflage, every option, the JEI window
and the guide. **Not tested:** the sound by ear, and exact tint colours (by screenshot only).

Also built on Horizon commit `fdc87ae` (Nightly 2026-11-08) as `native-fdc87ae-83d10460ef68`, with AE2, Jade, JEI (mod kit 14), Iron
Furnaces, Hostile Neural Networks, Tinkers' Construct, Ender IO, Powah and Apothic Spawners: all ten compile together, and the W guide's
page with its 3D scene of elevators opens in game. Kit 14 draws recipe fluids as their textures, never as buckets.

## Build

This is a Horizon native mod bundle: Rust mixins compiled into the game. Native mods are unsandboxed code that runs with your rights, so
read what you build.

```bash
horizon-launcher native-build "path/to/folder/with/the/bundle" --trust-native-code
```

## Licence

MIT, as the original. See `LICENSE`.
