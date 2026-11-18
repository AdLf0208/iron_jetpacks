# Iron Jetpacks — port notes (slice 2: iron tier, item-backed tank)

Ported from **Iron Jetpacks 8.0.11** (`BlakeBr0/IronJetpacks`, **MIT**) for
Minecraft Java 1.21.1 NeoForge, rewritten against Horizon **26.3** at pin
`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` / horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (migrated from `41f822b6` on 2026-11-10; `game.rs` SHA-256
`64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f`).

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
The licence is MIT (rank 0), so derivatives are allowed; publication still
needs the maintainer's approval.

Status: **migrated to the flat SDK package (`0.2.0`) and verified in-game at
`0.1.0`** — one `ironjetpacks:jetpack` item with an item-backed 800,000 FE tank,
a pure 29-test flight model, eight hook methods, and a fail-capable
high-Overworld e2e (PASS: charged jetpack 12.7282 blocks vs 1.17676 for every
disabled control). The retired `.hnmod` container is gone; see
[`## Migration to the SDK format`](#migration-to-the-sdk-format-2026-11-18-sdk-0b14f8a)
for the current package, the hook→method map and the traps.

## Version note

The pack ships Iron Jetpacks **8.0.11 for 1.21.1**, but the upstream
repository's default branch is **26.1** (`601f96aaa8b1d389d7b5d587ca68dc6b68fe9a15`,
2026-09-02, "Update Neoforge version"; `gradle.properties`:
`minecraft_version=26.1.2`, `neo_version=26.1.2.71`). The 26.1 branch is the
source of the behaviour and content below; the 1.21.1 jar was not opened.
The 26.1 tree has one `ironjetpacks:jetpack` item whose tier is a
`jetpack_id` data component; the port fixes that item to the **iron** tier
(tier 2), because the content format has no way to register a custom data
component from a content-only `.hmod` (the engine switch is registered from
the native bundle instead).

## Upstream semantics (file:line in the 26.1 checkout)

Clone: `ports/_shared/upstream/ironjetpacks/` (deleted after this note).
Files read: `handler/PlayerTickHandler.java`, `util/JetpackUtils.java`,
`registry/Jetpack.java`, `registry/JetpackRegistry.java`,
`lib/ModJetpacks.java`, `config/ModConfigs.java`,
`client/handler/InputHandler.java`, `client/handler/KeybindHandler.java`,
`client/handler/JetpackClientHandler.java`, `init/ModItems.java`,
`init/ModDataComponentTypes.java`, `init/ModCreativeModeTabs.java`,
`crafting/DynamicRecipeManager.java`, `item/JetpackItem.java`, the item
models/textures/lang and the equipment asset.

| Behaviour | Evidence |
|---|---|
| Equipped jetpack is the **chest slot**; a Curios back slot is the fallback | `JetpackUtils.java:56-61` (`getEquippedJetpack`) |
| The engine must be on (`JETPACK_ENGINE` component, default false) | `PlayerTickHandler.java:24`; `JetpackUtils.java:82-85` |
| Flight trigger: ascend key held, or hover mode while airborne | `PlayerTickHandler.java:27` |
| `motionY + accel`, with `accelVert` × 2.5 while `motionY < 0.3` | `PlayerTickHandler.java:30-33` |
| Ascent cap `speedVert`, × 0.4 in water | `PlayerTickHandler.java:33,57` |
| Throttle multiplies the ascent; sprint multiplies it by `sprintSpeedVert` when `motionY >= 0` | `PlayerTickHandler.java:52-53,57` |
| Hover + ascend + descend: `min(motionY + accel, -speedHoverSlow)` | `PlayerTickHandler.java:59-60` |
| Hover + ascend: cap `speedHoverAscend` | `PlayerTickHandler.java:62` |
| Hover, no ascend: `min(motionY + accel, -hoverSpeed)`, where hoverSpeed is `speedHoverDescend` while descend is held and `speedHoverSlow` otherwise | `PlayerTickHandler.java:31,65-66` |
| Fuel is extracted **before** the flight check, so the tick that empties the tank does not thrust | `PlayerTickHandler.java:40-51` |
| Sprint burns `usage * sprintFuel`; creative players and creative jetpacks burn nothing | `PlayerTickHandler.java:35-45,51` |
| Fall protection: `fallDistance = 0` on every thrusting tick | `PlayerTickHandler.java:90-96` |
| Horizontal flight: `moveRelative` with `speedSide`/`sprintSpeed`, half while crouching | `PlayerTickHandler.java:69-88` |
| Hover cancels elytra flight | `PlayerTickHandler.java:47-49` |
| Iron tier stats: capacity 800000, usage 120, speedVert 0.41, accelVert 0.12, hover ascend 0.27 / descend 0.25 / slow 0.075, sprint vertical 1.05, sprint fuel 2.1 | `lib/ModJetpacks.java:30` (`IRON.setStats(...)`) |
| Inputs are the vanilla keys: jump/ascend, shift/descend, WASD, sprint; a custom `ascend`/`descend` key is unbound by default | `KeybindHandler.java:106-112` |
| The client sends the seven input booleans to the server each change | `KeybindHandler.java:114-131`; `network/payloads/UpdateInputPayload.java` |
| Jetpack items are unbreakable, stack to 1, and take their tier's armor modifiers | `item/JetpackItem.java:32-36`; `registry/Jetpack.java:140-154` |
| Iron tier armor points 3, toughness 0, knockback resistance 0 | `lib/ModJetpacks.java:18,30`; `registry/Jetpack.java:140-154` |
| Recipes are built dynamically: strap + coil → cell, + furnace → thruster, 3 cells → capacitor, then the jetpack; the non-lowest tiers are upgrade recipes over the previous tier | `crafting/DynamicRecipeManager.java:41-55,180-224` |
| Coil tier: iron (tier 2) uses the advanced coil | `JetpackRegistry.java:84-98`; `lib/ModJetpacks.java:12,30` |
| Creative tab lists strap, coils, cells, thrusters, capacitors and jetpacks | `init/ModCreativeModeTabs.java:17-45` |

## Slice 1 behaviour (history, pin `c9d474d`)

- **Item** `ironjetpacks:jetpack`, the 26.1 single jetpack item fixed to the
  iron tier. It is a chest item (right-click equips it), unbreakable, stacks
  to 1, armor 3, and uses the upstream item model/texture and worn equipment
  asset.
- **Flight**: while the jetpack is in the chest slot and the jump key is
  held, the player rises. The vertical velocity is
  `min(motionY + accel, speedVert) * throttle * sprint` with the upstream
  acceleration/cap/water/sprint rules. The thrust ticks also zero the fall
  distance.
- **Fuel (superseded by slice 2)**: slice 1 kept a per-player tank (800000 FE,
  120 FE/tick, 252 FE/tick while sprinting) in a server-side map and used the
  no-jetpack phase as its only disabled control.
- **Controls**: jump = ascend, sneak = descend (only used by the hover rules),
  sprint = the sprint multiplier.
- **Content**: the jetpack, the five component items (`strap`,
  `advanced_coil`, `cell`, `thruster`, `capacitor`) and the iron-tier recipe
  chain (strap + advanced coil → cell → thruster/capacitor → jetpack). The
  models, textures and lang are the 26.1 upstream assets; the recipes are the
  upstream dynamic shapes with their tags resolved to the vanilla member
  items. The `cucumber:item_colors` tints became `minecraft:constant` tints
  of the iron colour `0xD8D8D8`.

## Slice 2 behaviour (2026-11-07, pin `41f822b6`)

Slice 2 closes the three gaps the re-pin audit named (REPIN-AUDIT.md items
2, 3 and 5) without changing the content model:

- **Tank on the item.** `item_energy_buffer` (API 307) declares the iron
  tier's 800,000 FE tank for `ironjetpacks:jetpack`; `player_tick` extracts
  the thrust tick's `spent` with `native_item_energy_extract` and writes the
  returned one-item stack back to the chest slot. The amount lives in the
  stack's `minecraft:custom_data` as `horizon_energy`, so it is saved and sent
  with the item; the empty tank drops the key.
- **Engine component.** `component_types` (API 65) registers
  `ironjetpacks:engine`; `/ironjetpacks engine [on|off]` flips it on the worn
  stack. Upstream's default (off) is preserved: a fresh jetpack has no engine
  component and does not fly.
- **Commands surfaced.** The client `commands` hook (API 35) lists
  `ironjetpacks`, so `/ironjetpacks fuel [amount]` and `/ironjetpacks engine
  [on|off]` reach `register_commands`/`perform_command` from player chat. The
  query answers `N / 800000 FE, engine on|off`.
- **Guest input.** `player_input` (API 343) decodes jump/shift/sprint flags
  (`16`/`32`/`64`) into the same `Input`; the host keeps the `tick` hook's
  raw Space sample.
- **Flight model unchanged** apart from surfacing `spent`: the same `step`
  rules, and the e2e now checks the item stack itself rather than a map.

## Hooks (pin `41f822b6`)

| Hook | Helper | What it does |
|---|---|---|
| `tick` (v1, HEAD, API 1) | `tick.rs` | Samples the window's Space key; `player_tick` cannot see the host's held keys. |
| `player_tick` (v1, HEAD, API 31) | `player_tick.rs` | Reads the chest item, the player's motion/water/sneak/sprint and the sampled jump key; applies `step`; extracts the spent FE from the item, writes the vertical velocity, marks it for `SetEntityMotion`, and zeroes `fall_distance` on thrust ticks. |
| `register_commands` (v1, HEAD, API 8) | `register.rs` | Declares the four level-2 usages: `/ironjetpacks fuel`, `/ironjetpacks fuel <amount:integer>`, `/ironjetpacks engine`, `/ironjetpacks engine <on:boolean>`. |
| `perform_command` (v1, HEAD, API 8) | `perform.rs` | Queries/sets the worn item's tank and toggles the engine component. |
| `item_energy_buffer` (v1, HEAD, API 307) | `energy_buffer.rs` | Gives `ironjetpacks:jetpack` the iron tier's 800,000 FE buffer. |
| `component_types` (v1, HEAD, API 65) | `component_types.rs` | Registers `ironjetpacks:engine`. |
| `commands` (v1, HEAD, API 35) | `commands.rs` | Adds `ironjetpacks` to the client chat tree so the command is listed and completed. |
| `player_input` (v1, HEAD, API 343) | `player_input.rs` | Decodes a guest's jump/shift/sprint flags for the flight model. |

The velocity write is the pin's `PlayerMotion.velocity` (`motion.rs:278`); the
client is driven by `PlayerHealth.sync_velocity`, which the server sends as
`ClientboundSetEntityMotion` on the next tick (the server copy follows the
client's moves, so a server-only velocity would not move the local player).

## Pure engine (`runtime.rs`, 29 tests)

`step(tier, state, input, motion_y)` mirrors `PlayerTickHandler`:
engine gate, trigger, fuel extraction/cutoff, acceleration, water cap,
throttle, sprint multiplier, and the four hover/ascend branches. `parse`
handles the fuel and engine commands (query, set, clamp, toggle, bad input),
and `input_from_flags` decodes the guest bits. Tests cover every branch,
including the empty-tank tick, the spent-energy accounting and the creative
no-drain case.

## Content (`.hmod`)

Built from the 26.1 branch's own MIT assets:

- `data/ironjetpacks/horizon/item/{jetpack,strap,advanced_coil,cell,thruster,capacitor}.json`
- `data/ironjetpacks/recipe/{strap,advanced_coil,cell,thruster,capacitor,jetpack}.json`
- `data/ironjetpacks/horizon/creative_tab/jetpacks.json`
- `assets/ironjetpacks/items/*.json`, `models/item/*.json`
- the real 16×16 item textures, plus the worn `equipment/jetpack.json` and
  `textures/entity/equipment/humanoid/jetpack{,_overlay}.png`
- `assets/ironjetpacks/lang/en_us.json` (the upstream file with the four
  tier-format names resolved to the iron names)

`cargo-horizon-mod check`: `ironjetpacks 8.0.11 (mod, API 0.2, both)`,
**16186 bytes**, SHA-256 `c84364186d51a16971dad112b15ade70922475dbf929362e534c5c8835c885b3`,
37 entries, unsigned, "no code: content only". (The first build used
`minecraft:unbreakable`, which the pin's content loader refuses because no
vanilla item has that component; the item now uses a large
`minecraft:max_damage` and `damage_on_hurt: false` instead.)

## Native bundle (`.hnmod`)

`python3 tools/package.py` (deterministic, source-only) produces
`target/ironjetpacks-0.1.0.hnmod`: **12,496 bytes**, SHA-256
`ccc4af5f536748d2aa85db521b31d779909bd3cb40447c078ac7972fcfabda07`,
**12 entries** (`horizon-native.json` + the 11 `mixins/ironjetpacks/*.rs|.toml`).
The campaign-wide `ports/_shared/artifact-audit.md` row still lists an older
8,123-byte `3fb21a52…` bundle from before the slice-2 helpers; the current
sources are the 12,496-byte bundle above.

## Deviations and gaps

1. **One item, one tier.** 26.1 uses one `jetpack` item with a `jetpack_id`
   component and a synced registry of 14 tiers; the pin's content format
   cannot register that component, so the port ships the iron tier only and
   the item id itself is the tier. The creative tab lists only the iron-tier
   components (one advanced coil, one cell/thruster/capacitor).
2. **No HUD / item bar for the item-backed tank.** Slice 2 stores FE in the
   stack's own `minecraft:custom_data` (`horizon_energy`) and the commands
   report/set it, so it survives with the item; the pin's `item_bar` (API 307)
   and the HUD are not wired, so the player sees the charge only through
   `/ironjetpacks fuel`. The tank has no GUI either. Deferred.
3. **Hover is not toggled.** The engine now has a real component and
   `/ironjetpacks engine [on|off]`; its default (off) matches upstream. Hover,
   throttle and HUD still have no keybind/component plumbing, so hover stays
   off in game even though the runtime and its tests cover the branches.
4. **Jump key: host sample plus guest flags.** The pin's `player_tick` swaps
   the host's keys out, so the host's Space is sampled in `tick` by player
   name. A LAN guest's jump/shift/sprint now come from `player_input` flags
   (16/32/64); there is still no server-side player velocity write for a
   guest (REPIN-AUDIT.md: `native_entity_set_velocity` refuses players and
   `native_push_player` is client-local), so guest thrust remains one-sided
   until that pin gap closes.
5. **Horizontal flight deferred.** `moveRelative` with `speedSide` /
   `sprintSpeed` is not implemented; only vertical thrust is in this slice.
   Sprint's vertical multiplier and fuel multiplier are implemented.
6. **Hover and descend are runtime-only.** Sneak is read and the hover
   branches are unit-tested, but with hover permanently off, sneak does
   nothing in-game. The hover/descend path is ready for the toggle slice.
7. **Recipes are flattened.** The upstream 26.1 recipes are generated per tier
   with component-bearing `cell`/`thruster`/`capacitor` ingredients and a
   `JetpackUpgradeRecipe` over the previous tier. The port ships one iron-tier
   chain with the base-jetpack shape (`MCM / MSM / T T`), tags resolved to
   their vanilla members (`#c:ingots/iron` → `minecraft:iron_ingot`, etc.).
   The recipe chain is craftable but not identical to a multi-tier install.
8. **Worn equipment colour.** The 26.1 `equipment/jetpack.json` has
   `dyeable: {}`; 26.3 needs `color_when_undyed`, so the port sets it to the
   iron colour. The item model's `cucumber:item_colors` tints became
   `minecraft:constant` tints of the same colour. No texture was invented or
   recoloured on disk.
9. **No sounds, particles, HUD or keybinds.** `JetpackClientHandler`'s flame/
   smoke particles and looping sound, the HUD/item bar, the throttle/hover
   keybinds and the in-game HUD are deferred. The engine toggle exists only as
   the chat command, not as a keybind.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Verification results (2026-11-07, pin `41f822b6`)

| | |
|---|---|
| Pin | Horizon `41f822b659789780f58d32c63ddfb6887ea1b8bf`, horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57` |
| Class + tier | **content / logic** (`horizon/docs/modpacks.md`); approach = carried content + logic hooks (`tick`, `player_tick`, item energy, commands, guest input) |
| Licence | **MIT, rank 0**; local-only until the maintainer approves publishing (ARCHITECTURE.md §8) |
| Evidence | `ports/_shared/evidence/ironjetpacks/{stage,check,test,build,e2e-build,e2e-default,e2e-game}.log` |

### Runtime and staged ladder

`HORIZON_MIXINS=…/pins/horizon-mixins-5a64af9 python3 tools/horizon.py <step>
--horizon …/pins/horizon-41f822b` with `CARGO_INCREMENTAL=0`, through
`ports/tools/build-slot.sh`.

| Step | Evidence |
|---|---|
| fmt | `rustfmt --check --edition 2024` is clean for `runtime.rs`; `support.rs` still has whitespace-only rustfmt diffs, left untouched so the verified binary and the sources are identical (formatting is a follow-up) |
| clippy | the staged `check` (`cargo clippy --all-targets … -D warnings`) recorded 0 warnings; the 18:46 e2e rebuild recompiled the current `support.rs` with 0 warnings |
| stage | exit 0, `stage_changed: true`, revision `41f822b6…`, 83.1 s (`stage.log`); the manifest carries 8 injections + 2 modules |
| check | exit 0, **0 `warning:` lines**, `Finished dev profile` (`check.log`, 13,139 B) |
| test | `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 1105 filtered out`, **0 `warning:` lines** (`test.log`) |
| build | `cargo_exit: 0`, fast profile (`build.log`, `cargo_seconds: 740.7`) |
| e2e rebuild | `build-slot.sh e2e` rebuilt the fast game from the current mixins at 18:46: `Finished fast profile … in 7m 26s`, **0 `warning:` lines** (`e2e-build.log`) |

The 29 tests are the pure `runtime.rs` suite (flight branches, spent-energy
accounting, fuel/engine command parsing, guest flags), run as
`game::ironjetpacks::tests`. The `check.log`/`test.log` files are the 17:02
ladder; `mixins/ironjetpacks/support.rs` (game glue, not part of the
standalone test crate) was last edited at 18:27, so its compile evidence is
the 18:46 e2e rebuild (`e2e-build.log`, 0 warnings) plus the PASS run itself.

### Artifacts

| Artifact | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/ironjetpacks.hmod` | 16,186 B | `c84364186d51a16971dad112b15ade70922475dbf929362e534c5c8835c885b3` | 37 |
| `target/ironjetpacks-0.1.0.hnmod` | 12,496 B | `ccc4af5f536748d2aa85db521b31d779909bd3cb40447c078ac7972fcfabda07` | 12 |

The `.hnmod` is the deterministic `python3 tools/package.py` bundle
(`horizon-native.json` + `mixins/ironjetpacks/*`). Content did not change, so
the 26.1 assets and the 16,186-byte `.hmod` were not rebuilt.

### In-game e2e (`tools/e2e.sh`) — default run, exit 0

`build-slot.sh e2e --job G6-ironjetpacks --port ports/ironjetpacks -- env
CTL=<shared 41f822b6 ctl> tools/e2e.sh`. Headless, scratch home, creative
world switched to survival, high-Overworld stone floor at y=99 (feet 100),
same held jump key in every phase. Full log: `e2e-default.log`; game log:
`e2e-game.log`.

The ctl transcript facts, quoted from the log:

- `ctl inventory` after `/ironjetpacks fuel 800000` (sent with `ctl chat`):
  `{"components":"{\"minecraft:custom_data\":{horizon_energy:800000}}","count":1,"item":"ironjetpacks:jetpack","part":"chest","slot":501}`
  — the charge is **on the chest stack**, not in a map.
- `ctl chat "/ironjetpacks fuel"` →
  `Iron Jetpacks: 0 / 800000 FE, engine off` (fresh),
  `Iron Jetpacks: 800000 / 800000 FE, engine off` (charged),
  `Iron Jetpacks: 800000 / 800000 FE, engine on` (after the toggle),
  `Iron Jetpacks: 795080 / 800000 FE, engine on` (post-flight),
  `Iron Jetpacks: 0 / 800000 FE, engine on` (drained).
- `ctl chat "/ironjetpacks fuel 800000"` →
  `Iron Jetpacks: fuel set to 800000 / 800000`;
  `ctl chat "/ironjetpacks engine"` → `Iron Jetpacks: engine on`.
- Post-flight `ctl inventory`:
  `{"components":"{\"ironjetpacks:engine\":1b,\"minecraft:custom_data\":{horizon_energy:795080}}",...}`
  — the item itself lost 4,920 FE.
- Input edge: `vy while the key was held: 0.23851632033527775` then
  `released: fuel 793880 then 793880, vy -0.0784000015258789` — the tank
  stops draining and the thrust stops when Space is released.
- Empty tank with the engine on: `fuel set to 0 / 800000`, then the chest
  stack has no `horizon_energy` key, and the held jump rises only **1.17676**.

Phase peaks: no-jetpack **1.17676**, fresh empty tank **1.17676**,
engine-off with a full tank **1.17676**, charged jetpack with the engine on
**12.7282** (feet 101→112.73 over ten 0.1 s samples, vy 0.2385). Health after
landing 9.0; the thrust ticks zero `fall_distance`.

`PASS: jetpack rose 12.7282 blocks with the engine on and a charged item tank;
the disabled control (no jetpack) rose 1.17676; the fresh empty tank and the
engine-off control rose 1.17676 and 1.17676; the tank drained on the item to
795080 (stack horizon_energy 795080, matching the query), held at 793880
while the key was released, and an empty tank with the engine on rose only
1.17676`

`EXIT:0`.

### Fail-capability

The default run is differential on the same held jump and the same
measurement: removing the item (no-jetpack), leaving the tank empty, or
leaving the engine off each drops the peak from 12.7282 to 1.17676, and the
script fails (`FAIL: …`) if any of those phases exceeds 1.6 or if the charged
phase does not exceed 2.0. A hook that thrust without the item, ignored the
engine component or ignored the empty tank would therefore fail the run.

The separate `NO_JETPACK=1` counterfactual leg did **not** complete: its
18:49 run was killed after the cached rebuild and left a 146-byte stub, so
there is no counterfactual log in the evidence directory. The three in-run
controls above are the recorded fail-capable evidence; a future re-run can
add the counterfactual leg.

### Embedded content — one `.hnmod` deliverable (2026-11-10, pin `1d63e0a`)

The registry keeps one `file` per version, and a split content+code port cannot
be listed (`ports/_shared/horizon-mods/AGENT_PLAYBOOK.md` §1.0). The port now
ships its content **inside** the native bundle, the AE2 pattern proved on
`charginggadgets` (ARCHITECTURE.md §8, "Embedded content works"):

- `mixins/ironjetpacks/data/ironjetpacks-content.hmod` — the built
  `content/ironjetpacks.hmod`, byte for byte;
- `mixins/ironjetpacks/embedded_content.rs` — `native_data!` compiles the file
  in, writes it to `.horizon-cache/ironjetpacks-embedded-content.hmod` when the
  copy on disk differs, appends it to the dev list and calls
  `content_root(root, enabled, &with)`;
- one `[[injection]] id = "embedded-content"` on
  `horizon_client::game::mods::content_root`, `kind = "HEAD"`,
  `cancellable = true`, bindings `root`/`enabled`/`dev`;
- `tools/package.py` now packs `mixins/<id>/data/` and refuses a data file that
  differs from `content/<id>.hmod` (`data/` travels as `mixins/<id>/data/…`,
  `docs/native-mods.md` "Data files").

Install one file: `ironjetpacks.hnmod` carries the item, its assets and its
recipes **and** the eight hooks. `content/ironjetpacks.hmod` stays as the input
of the embedding (not a second deliverable).

| Artifact | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `ironjetpacks.hnmod` | 25,141 B | `7fedd33d9f96de8c61b6b1b2f9bcc3d218481e07ab5d8c59861b77ac3ed94691` | 14 |
| …of which `mixins/ironjetpacks/data/ironjetpacks-content.hmod` | 16,186 B | `c8436418…c885b3` (unchanged content) | — |

`cargo-horizon-mod horizon-mod check ironjetpacks.hnmod --kind native-mod` →
`ironjetpacks 0.1.0 (native-mod, format native-1/1d63e0a5…/5a64af93…, for 26.3)`,
25,141 bytes, same SHA-256, unsigned, **exit 0**.

Ladder at `1d63e0a` (through the slot registry): `cargo fmt --check`, `cargo
clippy --all-targets --locked --offline -D warnings` and `cargo test --locked
--offline` clean (29 passed); `stage` → `check` → `test` (`cargo_exit: 0`,
`game::ironjetpacks::tests`), `stage_changed: false`.

**In-game proof from one file** (`e2e-embedded.log`, `EMBEDDED=1`, exit 0): the
game is launched with **no `--dev-mod`**, the bundle writes
`16186 B at …/.horizon-cache/ironjetpacks-embedded-content.hmod, identical to
content/ironjetpacks.hmod`, and the port's own scenario passes —
`PASS: jetpack rose 12.7184 blocks with the engine on and a charged item tank;
the disabled control (no jetpack) rose 1.24919; … the tank drained on the item
to 795080 (stack horizon_energy 795080, matching the query), held at 793880
while the key was released, and an empty tank with the engine on rose only
1.17676`, `EXIT:0`.

**Control** (same run, before the main leg): `.horizon-cache` is a plain file in
both the scratch root and the game folder, so the helper cannot write. The same
binary then answers
`{"chat":["Unknown item 'ironjetpacks:jetpack'", …]}` to
`item replace … armor.chest with ironjetpacks:jetpack`, holds no jetpack, and a
vanilla stick still lands — the ids come from the embedded file, not from the
hook code.

**Fail-capable selftest** (`e2e-embedded-selftest.log`, `NO_JETPACK=1`):
`FAIL: the jetpack rose only 1.17676 blocks`, `EXIT:1`.

`python3 ports/tools/check-e2e-logs.py ironjetpacks --strict` → 4 logs, all
with a matching exit status (0); `python3 ports/tools/upstream_ready.py
ironjetpacks` → **READY**.

### Remaining gaps

1. One item fixed to iron (no `jetpack_id` component / 14 tiers).
2. No HUD or item bar for the item tank; charge visible via the command only.
3. Hover/throttle/HUD keybinds not wired (hover off; engine has a command).
4. Guest input flags are unit-tested; a guest's in-game flight was not tested
   (no second client). Server-side guest velocity write is still absent
   (REPIN-AUDIT.md), so guest thrust stays one-sided.
5. Horizontal flight deferred; vertical only.
6. Recipes flattened to the iron-tier chain.
7. Worn-equipment colour adapted to `color_when_undyed`.
8. No sounds, particles or GUI.

### Skills

- superpowers: `using-superpowers` (gate), `systematic-debugging` (log and
  pin-API diagnosis), `verification-before-completion` (every number above
  is quoted from the copied logs); `test-driven-development` governed the
  slice-2 tests when they were added.
- Beautiful Rust: `rust-best-practices` (fmt, `clippy -D warnings`, tests;
  typed `Tier`/`State`/`Input`/`Step`/`Action`, `Option` for thrust, no
  unused API).

## Pin migration (2026-11-10): `41f822b6` → `1d63e0a5`

Mode A of `ports/_shared/horizon-mods/brief-artifact-current.md`. The stamped
catalog is byte-identical across the pins, so the migration edits were
`compatibility.json` (`horizon_revision` + `game.rs` `2488718…` → `64813cf4…`),
`mixins/ironjetpacks/horizon-mixin.toml` `[compatibility]`, and
`tools/horizon.py`'s defaults (`ports/_shared/pins/horizon-1d63e0a` /
`horizon-mixins-5a64af9`). No runtime, helper or binding changed.

**Concurrent write (recorded, not fought).** Another job added the embedded
content half to this port at 01:03–01:04 (`mixins/ironjetpacks/embedded_content.rs`,
`mixins/ironjetpacks/data/ironjetpacks-content.hmod`, a ninth `embedded-content`
injection and its `[[module]]`). The ladder, build, package and e2e below are all
**after** that change (mixin toml 01:04, fast binary 02:44, bundle 01:37/04:0x),
so they describe the current sources, not just the eight-injection version.

Ladder (through `ports/tools/build-slot.sh`, `CARGO_INCREMENTAL=0`, no bare
cargo, no `--target-dir`):

| command | exit | evidence |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | runner `unit` step (command `cd ports/ironjetpacks` so the forwarded copy works) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | 0 warnings |
| `cargo test --workspace --locked --offline` | 0 | 29/29 |
| `tools/horizon.py stage` | 0 | `stage_changed: true`, pin `1d63e0a5`, 152.6 s |
| `tools/horizon.py check` | 0 | **0 warnings** (`target/horizon-check.log`), 152.2 s |
| `tools/horizon.py test` | 0 | `game::ironjetpacks::tests`: 29 passed, 1240 filtered |
| `tools/horizon.py build` (fast) | 0 | `cargo_exit: 0`, `target/horizon-build.log`, 1586.8 s |

Packages (a second build reproduced each hash):

- `content/ironjetpacks.hmod` — 16,186 B, SHA-256
  `c84364186d51a16971dad112b15ade70922475dbf929362e534c5c8835c885b3`,
  37 entries; `horizon-mod check` exit 0.
- `ironjetpacks.hnmod` (port root) — 25,141 B, SHA-256
  `7fedd33d9f96de8c61b6b1b2f9bcc3d218481e07ab5d8c59861b77ac3ed94691`,
  14 entries (`horizon-native.json`, the 9-injection bundle, and the embedded
  content `.hmod`).

In-game e2e (default + counterfactual, on the fast binary built from these
sources at 02:44; shared `horizon-ctl` from `_shared/bin/horizon-1d63e0a5`):

- `_shared/evidence/ironjetpacks/e2e-default.log` — `PASS` + `EXIT:0`: the
  charged, engine-on jetpack rose **12.7184** blocks; the no-jetpack, fresh
  empty-tank and engine-off controls stayed at **1.17676**; the item tank drained
  to 795080 and the stack's `horizon_energy` matched; the empty armed tank rose
  1.17676.
- `_shared/evidence/ironjetpacks/e2e-negative.log` — `NO_JETPACK=1` removes the
  chest item before the same flight phase, which then rises only 1.17676 and
  **fails** at the `rose only … blocks` assertion: `FAIL` + `EXIT:1`.

`check-e2e-logs.py ironjetpacks --strict` → 2/2 ok. A transient
`e2e-build.log` (a build log, not a verdict log) was left in the evidence
directory by an earlier run and removed; another job's remote evidence pull
re-created it once, so it may reappear — it is not a verdict log.

## Review repairs (PR #567)

1. Staging is the orchestrator's (horizon-mods#686).
2. *"Fix the upstream description link: `(BlakeBr0/IronJetpacks)` is a relative
   link"* — the entry links `https://github.com/BlakeBr0/IronJetpacks` and
   credits BlakeBr0; the description is rewritten complete.
3. **[P1] "Restore the upstream MIT notice in the distributed port"** — the
   repository `LICENSE` now names BlakeBr0 beside the port's contributors, and
   `mixins/ironjetpacks/LICENSE` + `NOTICE.md` travel inside the `.hnmod` (and
   at the embedded content archive's root).
4. **[P2] "Let ordinary players turn on their jetpack"** — **fixed**:
   `/ironjetpacks engine` is no longer an operator command (only `fuel` is
   level 2), so a survival wearer can switch their own jetpack on.

### Verification after the repairs

- Artifact: `ironjetpacks.hnmod`, 27,399 B, SHA-256 `fec4ae09e4bcd6a121189805b12925bd9c865051a72ad7db9d4847e0be1be17e` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/ironjetpacks.hmod`).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked --offline`: clean, 29 passed; the embedded run (`python3 tools/horizon.py test --horizon ports/_shared/pins/horizon-1d63e0a`) is 29 passed, 0 failed (`target/horizon-test.log`).
- `build-slot.sh e2e --job … --port ports/ironjetpacks -- tools/e2e.sh`: PASS, log `ports/_shared/evidence/ironjetpacks/e2e-repair.log` ends `EXIT:0` (`check-e2e-logs.py ironjetpacks --strict` exit 0).
- `python3 ports/tools/upstream_ready.py ironjetpacks` and `check-skills.py --notes ports/ironjetpacks`: exit 0.

## Migration to the SDK format (2026-11-18, SDK `0b14f8a`)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`, and the
registry closed this port's submission (PR **#567**) with *"republish this as a native `.hmod`
with the SDK from the next Nightly"*. The port is one flat package now:

- **Pin:** Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (SDK build 2026-11-18),
  horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API **`0.2`**, launcher
  0.2.78. The SDK is fetched into `.horizon/sdk/` by `cargo horizon-mod` and never packed.
- **Version:** **0.2.0** — a resubmission is always a new version (the retired bundle was
  0.1.0).
- **PR number:** the migration brief says #876, but no such submission exists for this port;
  `ports/_shared/horizon-mods/README.md` records **#567** for `ironjetpacks`, and this file's own
  “Review repairs (PR #567)” section is that review. The artefact wins: the changelog below names
  **#567**.
- **Class + tier** unchanged: class **content**, tier **logic** (`docs/modpacks.md`): one
  chest-slot item with a tick-driven flight rule, no machine and no new deep API. The content
  half (items, models, textures, lang, recipes, creative tab) now travels **inside the same
  `.hmod`** under `assets/` + `data/`, so the AE2-style embedded-content injection is gone.

### File map

| Created | What it holds |
|---|---|
| `horizon-mod.toml` | the only manifest: `id = "ironjetpacks"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `license`, `homepage`/`issues`, the required `capabilities`, and `[native]` with the exact `game = "26.3"` / `horizon` / `mixins` |
| `Cargo.toml` | one crate; the only dependency is `horizon-mod-api` from `.horizon/sdk/`, and no `rust-version` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` (`register`, `component_types`, `client_tick_pre`, `player_input`, `player_tick_post`, `register_commands`, `run_mod_command`), the energy/custom-data glue and the `ITEM_ENERGY` provider |
| `src/runtime.rs` | the old `mixins/ironjetpacks/runtime.rs`, **byte-identical** (SHA-256 `6b83a735672c55425eda6c0be11eb8006b9e59fdedd06d378acd8e54f0a28378`, `cmp` clean, and the same hash inside the built package), 29 tests |
| `assets/ironjetpacks/…`, `data/ironjetpacks/…` | the carried content, moved out of `content/` to the package root (`cmp`-checked against the retired copy) |
| `LICENSE`, `LICENSE-NOTICE.md` | carried, and **repaired** (see “Upstream defects fixed” below) |
| `ironjetpacks.hmod` | the delivered package in the port root |
| `.gitignore` | `/target`, `/.horizon`, `/Cargo.lock` |

| Deleted (into `../_shared/evidence/ironjetpacks/retired-hnmod-0.1.0/`) | Why |
|---|---|
| `mixins/ironjetpacks/` (`horizon-mixin.toml`, `support.rs`, the eight hook helpers, `embedded_content.rs`, `data/*.hmod`) | the plan/injection schema and per-hook helper files are not part of any SDK package; the hook bodies are the `impl Hooks` methods now |
| `content/` (`horizon-mod.toml`, `assets/`, `data/`, the built `ironjetpacks.hmod`) | a second package; the flat package carries the content itself, so the old content `.hmod` is no longer a deliverable |
| `ironjetpacks.hnmod` | the retired container (0.1.0, 27,399 B `fec4ae09…`) |
| `compatibility.json` | the pin is `[native]` in `horizon-mod.toml` |
| `rust-toolchain.toml` (1.95.0) | the pinned Horizon commit and launcher decide the toolchain; a floor above the box's stable refuses the crate |
| `Cargo.lock` | not needed without a registry dependency, and packing one changes the artifact's hash (trap 4) |
| `tools/horizon.py`, `tools/package.py`, `tools/add-notices.py`, `tools/audit/`, `tools/__pycache__`, `.github/workflows/ci.yml` | the staged/mixin build machinery and the template workflow that reads the deleted files; the registry check is `cargo horizon-mod check` now |

Kept on purpose: `tools/e2e.sh` (the in-game scenario; its launch leg now expects a
launcher-built game, because a native package cannot be a `--dev-mod` folder), `notes.md`,
`README.md` (rewritten for the flat layout), `VALIDATION.md`, `REPIN-AUDIT.md`.

### Hook → `Hooks` method

| Retired injection (API) | SDK method (API 0.2) | Mapping notes |
|---|---|---|
| `tick` HEAD (1) — the window's raw Space | **`client_tick_pre`** | `host.local_player()` + `host.mapping("key.jump")`; the held state is stored by UUID, so the server tick's acting player reads its own key. The hook is the window's `Game` only, as the old one was. |
| `player_tick` HEAD (31) | **`player_tick_post`** with `HostSide` + `PlayerRef` | runs on both sides; only `HostSide::Server` flies. The player's state is acting, and `player.game_mode(host)`/`life` are handle reads. |
| `item_energy_buffer` HEAD (307) | **`Registrar::item_capability(&ITEM_ENERGY, …)`** in `register` | the handler is built per lookup from the `ItemPlace`; the amount still lives in the stack's `minecraft:custom_data` `horizon_energy` (`Int`), read and written through `ItemPlace::stack` / `ItemPlace::edit`. The empty tank removes the key, as before. |
| `component_types` HEAD (65) | **`component_types`** | pushes `ironjetpacks:engine`; the value stays `Byte` (`1b`/`0b`). |
| `register_commands` HEAD (8) | **`register_commands`** | four `Command` usages: `fuel` at level 2, `engine` at level 0. |
| `perform_command` HEAD (8) | **`run_mod_command`** (`Flow::Cancel`) | the invocation is parsed once by the server (`CommandInvocation`); the glue rebuilds the same line the runtime's `parse` already tests (literals + resolved `integer`/`boolean` arguments, boolean as `on`/`off`) and answers through `host.commands().feedback`. |
| `player_input` HEAD (343) | **`player_input`** | identical flags (`16`/`32`/`64`), now keyed by the guest's UUID instead of the name. |
| `commands` HEAD (35) — client chat tree | **deleted** | obsolete: the server's `register_commands` tree is what the client colours, completes and lists; `register_client_commands` is for commands the *client* runs, which this mod has none of. |
| `embedded_content` (content_root, non-API) | **deleted** | the package carries `assets/` + `data/` itself. |

`runtime.rs` is untouched: `step`, `parse`, `input_from_flags`, `IRON`, `Tier`, `State`,
`Input`, `Step`, `Flags`, `Action` and the flag constants are called exactly as before. The new
glue only converts between the SDK's types and the runtime's (`ItemStack` custom data ↔ `u32`,
`PlayerRef`/`LivingRef` handles, `CommandInvocation` → the parser's line).

### Build and check (verbatim)

`BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-ij --port ports/ironjetpacks --step …`
wraps each step (the runner does not `cd` into the port, and the SDK layout needs no staging):

```sh
bash -c 'cd ports/ironjetpacks && cargo test --offline'
# → test result: ok. 29 passed; 0 failed   (the runtime's own tests, before any glue: 29/29)
bash -c 'cd ports/ironjetpacks && cargo fmt --check && cargo clippy --all-targets --offline -- -D warnings'
# → Finished `dev` profile   (0 warnings)
bash -c 'cd ports/ironjetpacks && rm -f Cargo.lock && cargo horizon-mod build'
# → SDK 0b14f8a (build 0b14f8ac…) in ./.horizon/sdk
# → Built ./target/horizon-mod/ironjetpacks.hmod
cargo horizon-mod check ironjetpacks.hmod
# → ironjetpacks 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)
# → 33900 bytes, SHA-256 81da8179…, unsigned        CHECK_EXIT:0
cargo horizon-mod check ironjetpacks.hmod --json
# → {"ok":true,…,"kind":"native-mod","requires":[],"size":33900,"version":"0.2.0"}  exit 0
cargo horizon-mod fits ironjetpacks.hmod --build nightly
# → Iron Jetpacks: / 0b14f8a  made for              FITS_EXIT:0
```

**Package:** `ironjetpacks.hmod`, **33,901 B**, SHA-256
`81da8179a2848489b97bafd2ced8a29a271fcff4f2675f9551423e8b93163b0f`, **44 entries**
(`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `src/lib.rs`,
`src/runtime.rs`, `uses.json`, and the 36 carried asset/data files). Built twice from the same
sources: byte-identical. `unzip -p ironjetpacks.hmod src/runtime.rs | sha256sum` is
`6b83a735…` — the retired file, byte for byte.

`uses.json` (26,530 B) names the seven hooks the glue implements (`client_tick_pre`,
`component_types`, `player_input`, `player_tick_post`, `register_commands`, `run_mod_command`,
`register`) and the handles it reads (`PlayerRef::game_mode`, `LivingRef::equipment`/
`set_equipment`, `EntityRef::velocity`/`push`/`reset_fall_distance`, `ItemPlace::stack`/`edit`,
`ItemStack` components, `capability:horizon:item_energy`).

### Traps this migration cost

1. **Handle methods, not host-area methods.** `host.entity_velocity(…)`, `host.living_equipment(…)`,
   `host.player_game_mode(…)` do **not** resolve on `&mut dyn ServerHost`; the published surface is
   the handle's inherent method (`player.entity().velocity(host)`, `living.equipment(host,
   EquipmentSlot::Chest)`, `player.game_mode(host)`). Read `mod_api_generated_host.rs` /
   `surface.json` for the handle form, not `host/*.rs` (which declares the engine side).
   `mod_api::prelude::*` is then unused and fails `-D warnings`.
2. **`item_key` takes `&Identifier`, not `&str`.** `Identifier::parse_namespaced("ironjetpacks:jetpack")`
   first; `None` before the registry completes.
3. **`entity_set_velocity` refuses players** ("its client moves it"). The flight tick uses
   `EntityRef::push(host, [0, target − current, 0])`: `Entity.push` takes a player and adds the
   delta, so the resulting y velocity is the runtime's target and x/z are untouched. The server
   sends the change to the client as `SetEntityMotion` on the next tick. This is the SDK's
   supported route; a direct velocity write is still not exposed.
4. **`Cargo.lock` is packed.** Any `cargo test`/`cargo clippy`/`cargo check` writes one; delete it
   right before `cargo horizon-mod build`, and keep it in `.gitignore`. (A stray `cargo horizon-mod
   check --help` also regenerated one here — check for it before packaging.)
5. **`cargo fmt --check`, never `--all`.** `.horizon/sdk` is a path dependency; `--all` reports the
   fetched SDK's own formatting.
6. **No `rust-version`.** `rust-version = "1.99"` would refuse the box's 1.97.0; the scaffold
   declares none.
7. **The runner does not `cd`.** `--port` is a label; wrap every command in
   `bash -c 'cd ports/ironjetpacks && …'`.
8. **A native package cannot be a `--dev-mod` folder.** `tools/e2e.sh` therefore needs `GAME=` a
   launcher-built game (`horizon-launcher native-build … --trust-native-code`); there is no
   content-folder launch leg any more.
9. **`component_types` still matters.** Registering `ironjetpacks:engine` is a separate hook from
   `register`; the item component is written as `Byte`, matching the 0.1.0 stacks so old items keep
   their engine state.

### Upstream defects fixed

- **The MIT notice did not travel (PR #567 P1, re-checked).** The retired `LICENSE` carried
  "Copyright (c) 2026 Horizon Veinminer contributors" — another port's line — and no upstream
  copyright, although `notes.md` claimed BlakeBr0 was named; the upstream repository's
  `LICENSE` (26.1 branch, fetched for this migration) is `MIT License / Copyright (c) 2018
  BlakeBr0`. The package's `LICENSE` now carries **both** that line and the port contributors'
  line, and `LICENSE-NOTICE.md` records the jar's `license="MIT"` and the upstream text. A user
  installing the package sees the attribution the MIT terms require. Worth an upstream report:
  no — it was this campaign's packaging defect, now fixed here.

### Gaps after the migration

1. **No in-game run at `0b14f8a`.** This box has no `horizon-launcher`, and the only game builds
   are at `1d63e0a5`/`41f822b6`; a native package is compiled by the launcher, so the behaviour
   evidence is the 0.1.0 run (`e2e-default.log`, `e2e-negative.log`, checker 2/2) plus the
   type-checked glue here. The first slice at a launcher-capable box runs `tools/e2e.sh` with
   `GAME=<launcher-built game>` and records the transcript.
2. **No HUD/item bar** for the item tank; the charge is visible through `/ironjetpacks fuel`.
3. **Hover/throttle keybinds are not wired** (hover stays off; the engine has a command).
4. **One item fixed to the iron tier** — no `jetpack_id` component / 14 tiers.
5. **Horizontal flight deferred**; vertical only.
6. **Recipes flattened** to the iron-tier chain.
7. **Historical guest gap re-checked:** `EntityRef::push` accepts a player, so the old
   "no server-side guest velocity write" limit no longer blocks the mechanism; a guest's actual
   flight is still untested (no second client here).

### Changelog for the resubmission

> Supersedes #567, closed because Horizon retired `.hnmod` (#17992). Changes: one flat SDK package
> with `api = "0.2"` and `[native]` pinned to `0b14f8a` / `e59da90`; the eight mixin helpers are
> seven typed `impl mod_api::Hooks` methods (`client_tick_pre`, `player_tick_post`,
> `component_types`, `player_input`, `register_commands`, `run_mod_command`, `register`), the
> client `commands` hook and the embedded-content `content_root` injection are gone, and the
> item tank is an `ITEM_ENERGY` capability over the chest stack's `minecraft:custom_data`;
> the content ships inside the same `.hmod` (`assets/`, `data/`); the MIT notice now carries
> BlakeBr0's upstream copyright beside the port's. `runtime.rs` and its 29 tests unchanged.
> Version 0.2.0.
