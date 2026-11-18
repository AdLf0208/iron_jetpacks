# Iron Jetpacks for Horizon

A clean-room Rust port of **Iron Jetpacks 8.0.11** (`BlakeBr0/IronJetpacks`,
MIT) — the iron tier's powered vertical flight and the iron-tier item/recipe
chain. Wear the jetpack in the chest slot, switch its engine on, and hold jump
to rise; the item's own 800,000 FE tank drains per thrust tick and cuts the
lift when empty. `/ironjetpacks fuel [amount]` reads or sets the tank and
`/ironjetpacks engine [on|off]` flips the engine.

**Version 0.2.0 — the SDK package (mod API 0.2).** One flat `.hmod` carries the
code and the content. The retired `.hnmod` container and its mixin plan are not
part of this package; the history is in `notes.md` and the old tree is kept at
`../_shared/evidence/ironjetpacks/retired-hnmod-0.1.0/`.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).

## Layout

```
horizon-mod.toml           the only manifest: id, version, api 0.2, [native] pin
Cargo.toml                 one crate; the only dependency is horizon-mod-api
src/lib.rs                 the one #[mod_api::horizon_mod] impl Hooks
src/runtime.rs             pure flight model, command parser, guest flags (29 tests)
assets/ironjetpacks/       item definitions, models, textures, lang, equipment
data/ironjetpacks/         item behaviour, recipes, creative tab
tools/e2e.sh               fail-capable headless in-game check
ironjetpacks.hmod          the delivered package
notes.md                   semantics, verification, deviations, gaps
```

## Commands

```sh
# pure runtime + glue tests
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check                      # not --all: .horizon/sdk is not ours

# package (no Cargo.lock: it would be packed and change the hash)
rm -f Cargo.lock
cargo horizon-mod build                # -> target/horizon-mod/ironjetpacks.hmod
cp target/horizon-mod/ironjetpacks.hmod ./ironjetpacks.hmod
cargo horizon-mod check ironjetpacks.hmod
cargo horizon-mod fits ironjetpacks.hmod --build nightly
```

`cargo horizon-mod` fetches the pinned SDK into `.horizon/sdk/` (gitignored,
never packed). The game binary that runs the mod is built by the launcher:
`horizon-launcher native-build ports/ironjetpacks --instance <instance>
--trust-native-code`, and `tools/e2e.sh` takes that binary as `GAME=`.
