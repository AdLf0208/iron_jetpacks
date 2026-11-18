# Iron Jetpacks — a Horizon port

**Iron Jetpacks** is a Minecraft mod. Adds fully customizable FE powered jetpacks!. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [BlakeBr0/IronJetpacks](https://github.com/BlakeBr0/IronJetpacks) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Iron Jetpacks ([source](https://github.com/BlakeBr0/IronJetpacks)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `ironjetpacks.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Item** `ironjetpacks:jetpack`, the 26.1 single jetpack item fixed to the
- **Flight**: while the jetpack is in the chest slot and the jump key is
- **Fuel (superseded by slice 2)**: slice 1 kept a per-player tank (800000 FE,
- **Controls**: jump = ascend, sneak = descend (only used by the hover rules),
- **Content**: the jetpack, the five component items (`strap`,
- **Tank on the item.** `item_energy_buffer` (API 307) declares the iron
- **Engine component.** `component_types` (API 65) registers
- **Commands surfaced.** The client `commands` hook (API 35) lists

## Not yet ported

- **One item, one tier.** 26.1 uses one `jetpack` item with a `jetpack_id`
- **No HUD / item bar for the item-backed tank.** Slice 2 stores FE in the
- **Hover is not toggled.** The engine now has a real component and
- **Jump key: host sample plus guest flags.** The pin's `player_tick` swaps
- **Horizontal flight deferred.** `moveRelative` with `speedSide` /
- **Hover and descend are runtime-only.** Sneak is read and the hover
- **Recipes are flattened.** The upstream 26.1 recipes are generated per tier
- **Worn equipment colour.** The 26.1 `equipment/jetpack.json` has

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `ironjetpacks.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Iron Jetpacks was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/BlakeBr0/IronJetpacks

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run ironjetpacks` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/ironjetpacks/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
