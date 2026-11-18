# Iron Jetpacks — re-pin audit (Horizon `83ad384f` / mixins `e8cd0aa`)

Read-only audit, 2026-11-05. No port code, manifests, content or builds were
changed; this file is the only write.

| | |
|---|---|
| New Horizon pin | `83ad384f48d0742746d518737511882680aab5b7` (427 hooks, 159 helpers), `ports/_shared/pins/horizon-83ad384f` |
| New mixins pin | `e8cd0aa38cffef49cb2e088ef716baa107148c02` |
| Port still pinned at | `c9d474dd…` / `d9c83d7…` (`compatibility.json`, `mixins/ironjetpacks/horizon-mixin.toml`) |
| Sources read | `ports/ironjetpacks/notes.md`, ARCHITECTURE item 31, upstream 26.1 `PlayerTickHandler`/`JetpackUtils` notes; pin `native_hooks.rs`, `native_motion.rs`, `native_entity_motion.rs`, `native_item_energy.rs`, `native_components.rs`, `native_server.rs`, `horizon-world/src/item_defaults.rs`, `horizon-protocol/src/packets/play.rs`, `native-hooks.json` |
| Method | Each deferred gap was searched in `native-hooks.json` and the pin's client/world sources; classified **usable** (writable out / cancel / catalogued helper), **hook-only** (listen) or **absent**. |

## Summary

| | |
|---|---:|
| Deferred feature groups audited | **8** |
| Fully realizable at this pin | **6** |
| Partially realizable (host half only) | **2** |
| Still blocked as a whole feature | **0** |
| Highest-value next step | Slice 2: move the tank onto the **item** (`item_energy_buffer` + `native_item_energy_*`), surface `/ironjetpacks fuel` with the client **`commands`** hook, and read the guest's jump from **`player_input`**. These three close the shipped slice's real gaps (non-persistent tank, dead command, host-only jump) without touching the renderer. Then slice 3: the custom `jetpack_id` component (API 65) and the 14 tiers. |

The big change at this pin is that the mod's **core data model** (one item +
tier component + per-item energy) is now expressible: previously the port had
to fix the item to iron and keep fuel in a server map. The remaining hard
limit is server-side player motion for **LAN guests** (see below).

## Deferred feature audit

Legend: **class** = usable / hook-only / absent; **verdict** = realizable now /
partial / blocked. Line numbers are in the pin root above.

| # | Deferred feature (notes / ARCHITECTURE) | New-pin symbol(s) (file:line) | Class | Verdict | Exact next step |
|---|---|---|---|---|---|
| 1 | **One item, one tier** — 26.1 uses one `jetpack` item with a `jetpack_id` data component and a 14-tier registry; content-only could not register the component | `native_component_types` (`native_hooks.rs:2913`, API 65), `native_component` / `native_set_component` / `native_remove_component` (`native_components.rs:26`, `:49`, `:74`), `native_modify_default_components` (`horizon-world/src/item_defaults.rs:77`, API 79), `native_items` (`native_hooks.rs:2942`, API 145), `native_prepare_item_craft` (`:5659`) to stamp the result. | usable | **realizable now** | Register `ironjetpacks:jetpack_id` in `component_types`; declare the tier stats in the runtime; either declare 14 code items (one default component each) or keep one item and stamp the component per craft. Read it with `native_component` and index the tier table. This replaces the “fixed to iron” deviation. |
| 2 | **Tank in a server-side map** — not saved across relog, no item bar | `native_item_energy_buffer` (`native_hooks.rs:3093`, API 307), `native_item_energy` (`native_item_energy.rs:156`), `native_item_energy_insert` (`:168`), `native_item_energy_extract` (`:180`); the amount lives in `custom_data/horizon_energy` and is saved/sent with the stack. `native_item_bar` (`:6458`) for the bar. The three `native_item_energy*` calls are `pub(in crate::game)` (documented under API 307 but absent from the catalog's helper array); an injected module in `crate::game` can call them. | usable | **realizable now** | Give `ironjetpacks:jetpack` a buffer (capacity/limits per tier); drain the chest stack each thrust tick with `native_item_energy_extract`, put the returned single item back, and read it for the HUD/bar. No static fuel map; the tank follows the item across relog. |
| 3 | **`/ironjetpacks fuel` not surfaced** — the old pin's client tree answered “Unknown or incomplete command” | `native_commands` (`:2666`, API 35, client list), `native_command` (`:2658`), `native_register_commands` (`:3899`), `native_perform_command` (`:3909`); `chat.rs:60` builds the client tree with `with_native_commands` (`chat.rs:42`). | usable | **realizable now** | Add a client `commands` injection returning `["ironjetpacks"]` (the server registration already exists). Keep `perform_command` for the actual run. E2E: `ctl chat "/ironjetpacks fuel"` answers the query, and `"/ironjetpacks fuel 123"` sets it. |
| 4 | **Engine/hover toggles + keybinds** — engine defaults off, hover off; throttle/HUD keys absent | `native_key` (`:2569`, client keys), `native_input_key` (`:2575`), `mod_key` helper, own persisted state; `native_set_setting`/`native_setting` (`:9205`/`:9200`) for a saved toggle. | usable (host/client) | **realizable now** | Sample the toggle keys in `tick` (the same pattern as Space) and keep the per-player state in the existing store. Wire the engine gate and hover branches the runtime already models. Guest custom keys are not in `player_input`; use a command/menu for guests or document the host-only limit. |
| 5 | **Jump key is the window's key; LAN guest jump input absent** | `native_player_input` (`:6359`, API 343): a guest's flags after they are applied; flags are forward 1, backward 2, left 4, right 8, **jump 16**, shift 32, sprint 64 (`horizon-protocol/src/packets/play.rs:2582`). | usable | **realizable now** | Keep the host's `tick` Space sample; for guests, decode `flags & 16` (and 32/64 for descend/sprint) in `player_input` and feed the same `Input`. This removes the “single-player only” limit for jump/sneak/sprint. |
| 6 | **Horizontal flight** (`moveRelative` with `speedSide`/`sprintSpeed`) | Host: the existing `PlayerMotion.velocity` write in `player_tick` (`support.rs:120-123`) can set x/z as well as y and `sync_velocity` sends it. Guests: `native_entity_set_velocity` (`native_entity_motion.rs:51`) **returns false for a player**; `native_push_player` (`native_motion.rs:31`) is **client-local only**; `native_player_velocity` (`:5307`) is **listen-only**. | usable for host; absent for guests | **partial** | Implement x/z for the host with the existing `motion.velocity` + `sync_velocity` path (same as vertical). Do not fake guests with `entity_set_velocity` (it refuses) or `push_player` (it is the local window's player). Record the guest half as the pin's player-write gap. |
| 7 | **Sounds, particles, HUD** — flame/smoke, looping sound, HUD, item decorations | `native_play_local_sound` (`:8782`, client-local), `native_flame_particle` (`:8427`, client-local), `native_hud` (`:2475`, `HudBatch`/`Corners`), `native_item_bar` (`:6458`), `native_tooltip` (`:6413`). | usable (client-local) | **partial** | Implement the host's HUD (fuel bar/percentage) and the local flame/smoke + sound first. For guests, these helpers do not run on the server; a server→client packet path would need a custom payload (`native_net_send_to_player`) and a matching client handler — out of this slice. |
| 8 | **Flattened recipes / upgrade chain** (26.1 builds per-tier dynamic recipes and a `JetpackUpgradeRecipe` over the previous tier) | `native_prepare_item_craft` (`:5659`, replace the result), `native_ingredient`/`native_ingredient_test` (catalog; component-aware matching), `native_item_crafted` (`:7525`), `native_recipes_of_type`/`native_recipe_*` helpers. | usable | **realizable now** | Generate the per-tier chain in code (the six base items already exist in content) and use `prepare_item_craft` to stamp the tier component/energy; use `ingredient_test` for the previous-tier upgrade. Keep the current base recipes as the control. Content models/lang for the extra tiers still need files. |

Accepted, not counted: the worn-equipment `dyeable` → `color_when_undyed` colour
adaptation (done), and the lack of a full 26.1 GUI (a HUD/inventory screen is
separate work).

## Requested-hook checks

- **`player_velocity` / `push_player`** — the critical check for this port:
  - `native_player_velocity` (`native_hooks.rs:5307`) fires **after** the
    server synced changed motion; it is a **notification**, not a writer.
  - `native_push_player` (`native_motion.rs:31`) adds to the **local player's**
    velocity on a client, clamped ±4, and does nothing on a server.
  - `native_entity_set_velocity` (`native_entity_motion.rs:51`) explicitly
    **returns false for a player** (“its client moves it”).
  Therefore there is still **no server-side helper to push another player**
  (a LAN guest). Vertical thrust works for the host because the port writes
  the host's `PlayerMotion.velocity` and `sync_velocity` sends it; the same
  path can carry horizontal x/z for the host, but not for guests. Do not
  spend a slice trying `player_velocity` as a writer.
- **`living_equipment_change` / `living_equipment_listened`**
  (`native_hooks.rs:7188`, `:7174`): usable, but **not needed** — the port
  reads the chest slot directly (`game.armor.get(1)` in `support.rs`). It could
  replace the per-tick chest check with an equip cache, but that is an
  optimisation, not a deferred feature.
- **`raid_state_change`** (`native_hooks.rs:5909`): **hook-only**; no jetpack
  feature is raid-related. Not used.
- **`player_name_entity`** (`native_hooks.rs:5227`): **hook-only**; no jetpack
  feature names entities. Not used.

## Pin state / repin prerequisites

The port is still on `c9d474d` / `d9c83d7`. Before the slices above:

1. `compatibility.json`: new `horizon_revision`, `game.rs` hash `996432b9…`,
   new mixins revision; `mixin_schema` stays 6.
2. `mixins/ironjetpacks/horizon-mixin.toml`: same 40-hex revision.
3. `tools/horizon.py`: embedded tests use `--lib` (the old `--bin
   horizon-client` runs 0 tests at this pin); `horizon-ctl` is in
   `horizon-client-lib` (`-p horizon-client-lib --bin horizon-ctl`).
4. One-time `cargo fetch --locked` for `zune-jpeg`; then
   `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2`.
5. Keep the `tick`/`player_tick`/`register_commands`/`perform_command`
   injections; all four hooks still exist at v1. Add the new `commands` and
   `player_input` injections for items 3 and 5.

## Evidence

- `native-hooks.json`: 427 hooks / 159 helpers; `player_velocity`,
  `raid_state_change` and `player_name_entity` all have `contract: "listen"`
  with no out field; `item_energy_buffer` is `replace`/`out: buffer`;
  `component_types` is `collect`/`out: types`.
- `native_entity_set_velocity` doc: “False for a player (its client moves
  it)” (`native_entity_motion.rs:51-55`); `native_push_player` doc:
  “For a client mod” (`native_motion.rs:29-31`).
- `native_player_input` flags documented in the protocol at
  `horizon-protocol/src/packets/play.rs:2582`: jump = 16, shift = 32,
  sprint = 64.
- Read-only: nothing under `horizon/`, `horizon-mixins/` or
  `ports/_shared/pins/` was modified.
