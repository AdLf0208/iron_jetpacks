# Validation

## Current: the SDK package (2026-11-18, SDK `0b14f8a`)

| Check | Command | Result |
|---|---|---|
| runtime + glue tests | `cargo test --offline` | **29 passed**, 0 failed (`_shared/evidence/ironjetpacks/sdk-0b14f8a/unit.log`) |
| fmt | `cargo fmt --check` | clean (`lint.log`) |
| clippy | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`lint.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `ironjetpacks.hmod` **33,900 B**, SHA-256 `81da8179…3b0f`, 44 entries; rebuilt byte-identically (`build.log`, `rebuild.log`) |
| registry check | `cargo horizon-mod check ironjetpacks.hmod` | `ironjetpacks 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)`, **EXIT 0** (`check.log`) |
| fits | `cargo horizon-mod fits ironjetpacks.hmod --build nightly` | `0b14f8a made for`, **EXIT 0** (`check.log`) |
| packed runtime | `unzip -p ironjetpacks.hmod src/runtime.rs \| sha256sum` | `6b83a735…8378` = the retired file (`runtime-carry.log`) |
| e2e | `tools/e2e.sh` | **not run at this pin**: needs a launcher-built game; the 0.1.0 run below stands |

All steps ran through `ports/tools/build-slot.sh` with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; the SDK layout needs no staging.

## Historical: the retired `.hnmod` (pin `1d63e0a`)

Iron Jetpacks slice 2, campaign pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**
(horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`, schema 6; `game.rs`
SHA-256 `64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f`),
Minecraft 26.3 resources from
`/home/adlf/Desktop/horizon/.local/horizon-client/26.3`. Migrated from
`41f822b6` on 2026-11-10.

| Check | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all -- --check` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | exit 0, 0 warnings |
| runtime tests | `cargo test --workspace --locked --offline` | 29 passed, 0 failed |
| staged stage | `python3 tools/horizon.py stage` | exit 0, `stage_changed: true`, pin `1d63e0a5` |
| staged check | `python3 tools/horizon.py check` | exit 0, 0 warnings |
| staged test | `python3 tools/horizon.py test` | `game::ironjetpacks::tests`: 29 passed, 1240 filtered |
| staged build | `python3 tools/horizon.py build` | `cargo_exit: 0`, fast profile |
| content | `cargo-horizon-mod horizon-mod build/check` | 16,186 B, SHA-256 `c8436418…85b3`, 37 entries |
| native bundle | `python3 tools/package.py` | 25,141 B, SHA-256 `7fedd33d…4691`, 14 entries; reproducible |
| e2e (default) | `tools/e2e.sh` on the 02:44 fast binary + shared ctl | **PASS**, `EXIT:0` (`_shared/evidence/ironjetpacks/e2e-default.log`) |
| e2e (counterfactual) | `NO_JETPACK=1 tools/e2e.sh` | **FAIL**, `EXIT:1` at the flight assertion (`e2e-negative.log`) |

All compile/test steps went through `ports/tools/build-slot.sh` (the `unit` step
was forwarded to the second machine; compile steps used the slot caches), with
`CARGO_INCREMENTAL=0`, no bare `cargo` and no `--target-dir`. The e2e ran in a
local lease against a fast binary built from these exact sources (copied out of
the slot immediately after the build so it could not be replaced by another
port's build). The exact outputs, PASS line and gaps are in
[notes.md](notes.md); the logs are in `ports/_shared/evidence/ironjetpacks/`.
