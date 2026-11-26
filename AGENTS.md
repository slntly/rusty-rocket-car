# AGENTS.md

Guidance for AI agents and contributors working on **Rusty Rocket Car** — an
ESP32-based RC car project consisting of a `no_std` Rust firmware
(`rrc-firmware/`) and a Flutter mobile controller (`rrc-app/`).

The car exposes a Wi-Fi access point (`192.168.4.1`), hands out addresses over
DHCP, and accepts a small UDP control protocol. Steering is driven by a servo
and throttle by an H-bridge motor; both outputs are smoothed and are forced to
safe values by an independent failsafe task.

## Prerequisites

- Rust **1.95+** (the toolchain file pins stable and installs `rust-src`).
- [`espflash`](https://github.com) 4.x for flashing/monitoring.
- RISC-V targets:
  `rustup target add riscv32imc-unknown-none-elf riscv32imac-unknown-none-elf`.
- For Xtensa chips (ESP32, ESP32-S2, ESP32-S3), the `esp` toolchain and the
  Xtensa GCC toolchain (`espup install`, then source `export-esp.sh`) are
  required. Invoke Cargo with `+esp` for those targets.
- Flutter SDK (Dart `^3.14.0`) for `rrc-app/`.

## Build, flash, monitor

### Firmware (`rrc-firmware/`)

All commands below run in `rrc-firmware/`.

No chip feature is enabled by default. Always pass exactly one chip feature
**plus** its matching target triple (otherwise `build.rs` panics):

| Chip feature | Target triple                 | Toolchain |
| ------------ | ----------------------------- | --------- |
| `esp32`      | `xtensa-esp32-none-elf`       | `+esp`    |
| `esp32c2`    | `riscv32imc-unknown-none-elf` | stable    |
| `esp32c3`    | `riscv32imc-unknown-none-elf` | stable    |
| `esp32c6`    | `riscv32imac-unknown-none-elf`| stable    |
| `esp32s2`    | `xtensa-esp32s2-none-elf`     | `+esp`    |
| `esp32s3`    | `xtensa-esp32s3-none-elf`     | `+esp`    |

Examples:

```sh
# ESP32-C3 (RISC-V)
cargo build --features esp32c3 --target riscv32imc-unknown-none-elf
cargo run --features esp32c3 --target riscv32imc-unknown-none-elf # flashes & opens defmt monitor

# ESP32-C3 SuperMini (caps Wi-Fi TX power)
cargo run --features esp32c3,supermini --target riscv32imc-unknown-none-elf

# ESP32-C6 (RISC-V)
cargo build --features esp32c6 --target riscv32imac-unknown-none-elf

# ESP32-S3 (Xtensa)
cargo +esp build --features esp32s3 --target xtensa-esp32s3-none-elf
```

Unsupported chips: ESP32-C5, ESP32-C61 and ESP32-S31 (no LEDC driver in
`esp-hal` 1.2) and ESP32-H2/ESP32-P4 (no Wi-Fi radio). Do not add them to the
feature list or the `build.rs` validation without checking the HAL first.

`DEFMT_LOG` (default `info`) and other user/runtime settings belong in
`.cargo/esp-config.toml`; compilation settings belong in `.cargo/config.toml`.

### Mobile App (`rrc-app/`)

All commands below run in `rrc-app/`.

```sh
flutter pub get
flutter run
flutter build apk
```

## Verification checklist

Before handing work back, run the narrowest set that covers the change:

- **Firmware (`rrc-firmware/`):**
  ```sh
  cargo fmt --check
  cargo clippy --features <chip> --target <target>
  cargo build --features <chip> --target <target>
  ```
- **Mobile App (`rrc-app/`):**
  ```sh
  dart format --output=none --set-exit-if-changed .
  flutter analyze
  ```

There are currently no host tests and no CI configuration. Do not claim that
firmware was flashed or exercised on hardware unless that actually happened.

## Code conventions

### Architecture

- Firmware is `#![no_std]`, `#![no_main]`, async and Embassy-based. The entry
  point is `src/bin/main.rs`; reusable code lives in the library crate.
- **Configuration is runtime data, not compile-time constants.** Build the
  `WiFiConfig`/`ServoConfig`/`MotorConfig` values in `main()`, move them into
  their `StaticCell`s, then hand `&'static` references to the tasks through the
  capacity-1 channels in `src/config.rs`. `config.rs` is explicitly *not* the
  configuration file — keep user-tunable values in `main()` unless the task
  requires a new shared channel or struct.
- Tasks are `#[embassy_executor::task] async fn` named `<subsystem>_task` and
  are spawned from `main()` with `.expect("Failed to spawn <name>")`.
- The failsafe owns all stop decisions. Hardware drivers must not inspect Wi-Fi
  or packet timing themselves; they call `failsafe::is_stopped()` every tick and
  substitute the safe target (servo center, zero throttle). Keep the transition
  smoothed rather than cutting outputs.
- The control path must never block. Use `Watch` for the steering/throttle
  samples: it always holds only the newest value, so a slow or stuck consumer
  can neither freeze the system nor fall behind.
- Naming for statics: `*_CH` for `Channel`, `*_WATCH` for `Watch`, `*_SIGNAL`
  for `Signal`, and `*_CELL` for `StaticCell`.

### Style

- `cargo fmt` is authoritative; there is no custom `rustfmt.toml`.
- Imports are grouped by origin with short comments (`// Embassy`, `// ESP-HAL`,
  `// Logging`, `// Config`, `// Modules`) in `main.rs`, and use `crate::...`
  paths inside modules.
- Use `defmt` (`info!`, `warn!`, `error!`) for all output. Never use
  `std::println!`; this is bare-metal.
- Use `///` doc comments for public items and for non-obvious behavior. Inline
  comments should explain *why* (hardware quirks, timing, safety), not restate
  the code.
- Error handling: use `expect("...")` with an actionable message at
  initialization boundaries instead of bare `unwrap()`. In long-running tasks,
  log recoverable errors and continue (optionally with a short backoff) rather
  than panicking.
- Control math uses `f32` and explicit `.clamp(...)` where values leave the
  packet or hardware range. The wire protocol is two signed bytes (`int8` in
  range `[-100, 100]` for throttle, steering), sent to `192.168.4.1:8888` over
  UDP; `Packet` is `#[repr(C)]` and `defmt::Format`.
- Driver tasks tick every 20 ms (`TICK_MS`) and share the `hardware::smooth()`
  helper. Keep these timings consistent unless the change is intentional.
- `main.rs` sets `#![deny(clippy::mem_forget)]` and
  `#![deny(clippy::large_stack_frames)]`. Avoid putting large buffers on task
  or async stacks; prefer statics/`StaticCell`.
- The heap is deliberately capped at 64 KiB (the smallest reclaimed DRAM across
  supported chips). Do not increase it casually or add allocation-heavy code.

## Git conventions

This project follows [Conventional Commits](https://conventionalcommits.org):

```
<type>(<optional scope>): <summary>

<optional body>

<optional footer>
```

- **Types:** `feat`, `fix`, `refactor`, `docs`, `test`, `build`, `ci`, `perf`,
  `chore`. Use `feat` for user-visible capability, `fix` for corrected
  behavior, `refactor` for restructuring without behavior change.
- **Scope:** optional lowercase subsystem or component name in parentheses
  (e.g., `feat(servo): ...`, `fix(udp): ...`). Omit it when unnecessary or when
  a change spans multiple areas.
- **Summary:** imperative mood, lowercase, no trailing period, aim for ≤ 72
  characters. Wrap identifiers in backticks.
- **Body:** separate with a blank line. Explain *why* and any behavioral
  consequences; wrap around 72 columns. Use `-` bullets for several related
  points.
- **One logical change per commit.** Do not mix formatting, renames, or
  dependency bumps with behavior changes.
- **Breaking changes:** add `!` before `:` (e.g., `feat!: ...` or
  `feat(udp)!: ...`) and a `BREAKING CHANGE:` footer.
- Do not add emoji, "Generated with ...", or tool attribution to commit
  messages.
- Do not create commits, amend history, or rewrite branches unless explicitly
  asked. Never commit build artifacts (`target/` is ignored).

## Working agreements

- Keep changes focused; avoid unrelated cleanup in the same change.
- Preserve the existing module boundaries and task ownership model. New
  hardware or network behavior generally belongs in a dedicated task plus a
  channel/signal in `config.rs`, not in `main()`.
- Do not edit `Cargo.lock` or `pubspec.lock` by hand; let Cargo and Flutter
  update them.
- Keep `LICENSE` intact.
- If you add build or flashing documentation, update `README.md`
  rather than duplicating everything here.
