# Working on the Iron Jetpacks port

This is a **flat SDK package** (mod API 0.2): `horizon-mod.toml` + `Cargo.toml` + `src/lib.rs`
(`impl mod_api::Hooks`) + `src/runtime.rs` (the pure rules and their 29 tests) + `assets/` +
`data/`. Horizon retired the `.hnmod` container in the mod-API merge (#16941), so there is no
`compatibility.json`, `mixins/<id>/`, `horizon-mixin.toml`, plan/injection schema or
`rust-toolchain.toml` here, and none of them should be recreated. The retired 0.1.0 tree is kept
at `../_shared/evidence/ironjetpacks/retired-hnmod-0.1.0/`.

> **Build slots (v2):** every build/test goes through the campaign runner
> (`ports/tools/build-slot.sh` from the campaign root, `../tools/build-slot.sh` from this port).
> It **blocks in the kernel** until a slot is free, so **never poll and never background a
> build**; a timeout exits **75**. It does **not** `cd` into the port — wrap the command
> (`bash -c 'cd ports/ironjetpacks && …'`) and never pass `--target-dir`. Read
> `../_shared/build-slots-usage.md` before running anything.

```sh
ports/tools/build-slot.sh run --job sdk-ij --port ports/ironjetpacks --step test \
    -- bash -c 'cd ports/ironjetpacks && cargo test --offline'
ports/tools/build-slot.sh run --job sdk-ij --port ports/ironjetpacks --step build \
    -- bash -c 'cd ports/ironjetpacks && rm -f Cargo.lock && cargo horizon-mod build'
ports/tools/build-slot.sh run --job sdk-ij --port ports/ironjetpacks --step check \
    -- bash -c 'cd ports/ironjetpacks && cargo horizon-mod check ironjetpacks.hmod'
```

- `cargo horizon-mod` fetches the pinned SDK into `.horizon/sdk/` (gitignored, never packed).
  `build` packs the `.hmod`; it does **not** compile against the game — that is
  `horizon-launcher native-build . --instance <instance> --trust-native-code`, which is what
  `tools/e2e.sh`'s `GAME=` must point at.
- **`src/runtime.rs` is byte-identical to the retired bundle's `runtime.rs` and must stay so**
  (SHA-256 `6b83a735…8378`): the flight rules, the command parser and the guest-flag decoder live
  there. New rule → a test in `runtime.rs`; new glue → a private fn in `lib.rs`.
- Only the API the SDK publishes: read `.horizon/sdk/api/crates/horizon-mod-vocab/generated/` and
  `surface.json` (or `cargo horizon-mod hooks <text>`), never guess a name. Handle reads are
  inherent methods on the handle (`player.entity().velocity(host)`, `living.equipment(host, slot)`),
  not host-area methods. Record an API gap in `notes.md` instead of faking one.
- **Delete `Cargo.lock` before packaging**: it is packed and changes the artifact hash. `cargo
  fmt --check`, never `--all` (the fetched SDK is a path dependency).
- The delivered package is `ironjetpacks.hmod` **in the port root**, never in `target/`;
  `LICENSE` (upstream BlakeBr0 notice + port line) and `LICENSE-NOTICE.md` travel inside it.
- Keep the port separate from Horizon and horizon-mixins; never modify the pin checkouts, never
  commit binaries, exported sources or Minecraft assets, and never put Mojang files in the tree.
- `python3 ports/tools/check-skills.py --notes ports/ironjetpacks` must stay green, and `notes.md`
  records every claim with its command and output.
