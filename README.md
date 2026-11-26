<div align="center">
  <img src="assets/icon/icon.png" alt="Rusty Rocket Car" width="96" height="96" />
</div>

# Rusty Rocket Car

`no_std` Rust firmware and Flutter mobile controller for ESP32-based RC cars.

## Overview

The project consists of two parts:

- **Firmware (`rrc-firmware/`):** Asynchronous Embassy-based firmware for ESP32 microcontrollers. 
  It spins up a Wi-Fi access point, assigns IP addresses via DHCP, and listens for control commands over UDP. 
  Steering is handled by a PWM servo and throttle by an H-bridge motor driver. Both outputs are smoothed, and 
  an independent failsafe task forces actuators to safe neutral positions if the connection drops.
- **Mobile App (`rrc-app/`):** Flutter application that sends real-time control packets over Wi-Fi.

## Prerequisites

- **Rust 1.95+** with the `rust-src` component installed
- **espflash 4.x** for flashing and viewing `defmt` logs
- **RISC-V targets (ESP32-C2, C3, C6):**
  ```sh
  rustup target add riscv32imc-unknown-none-elf riscv32imac-unknown-none-elf
  ```
- **Xtensa targets (ESP32, S2, S3):** Requires the `esp` toolchain and Xtensa GCC (`espup install`, then source `export-esp.sh`)
- **Flutter SDK** (to build and run the mobile client)

## Firmware

### Supported Hardware

| Chip | Cargo Feature | Target Triple | Toolchain |
| :--- | :--- | :--- | :--- |
| ESP32 | `esp32` | `xtensa-esp32-none-elf` | `+esp` |
| ESP32-C2 | `esp32c2` | `riscv32imc-unknown-none-elf` | `stable` |
| ESP32-C3 | `esp32c3` | `riscv32imc-unknown-none-elf` | `stable` |
| ESP32-C6 | `esp32c6` | `riscv32imac-unknown-none-elf` | `stable` |
| ESP32-S2 | `esp32s2` | `xtensa-esp32s2-none-elf` | `+esp` |
| ESP32-S3 | `esp32s3` | `xtensa-esp32s3-none-elf` | `+esp` |

> [!WARNING]
> ESP32-C5, C61, S31, H2, and P4 are unsupported due to missing LEDC drivers or Wi-Fi radios in the underlying HAL.

### Flashing

All firmware commands below are run from the `rrc-firmware/` directory. Always pass the chip feature along with its matching target triple:

```sh
# ESP32-C6 (RISC-V):
cargo run --features esp32c6 --target riscv32imac-unknown-none-elf

# ESP32-S3 (Xtensa):
cargo +esp run --features esp32s3 --target xtensa-esp32s3-none-elf
```

> [!NOTE]
> For compact SuperMini boards, enable the `supermini` feature to cap Wi-Fi TX power:
> ```sh
> # ESP32-C3 SuperMini
> cargo run --features esp32c3,supermini --target riscv32imc-unknown-none-elf
> ```

> See [Supported Hardware](#supported-hardware) for other boards.

## Mobile App

All app commands below are run from the `rrc-app/` directory.

```sh
flutter pub get        # update dependencies
flutter run            # run on a connected device
flutter build apk      # build an Android APK
```

Connect your phone to the car's Wi-Fi access point before launching the app. Both sticks automatically return to `0` upon release.

## Control Protocol

The app sends **2-byte UDP packets** to **`192.168.4.1:8888`** at a 100 Hz rate (every 10 ms):

| Byte | Channel | Range | Type |
| :--- | :--- | :--- | :--- |
| `0` | Throttle | `[-100, 100]` | Signed 8-bit integer (`int8`) |
| `1` | Steering | `[-100, 100]` | Signed 8-bit integer (`int8`) |

## Configuration

- **Hardware & Network:** Runtime parameters (Wi-Fi credentials, GPIO pin assignments, 
  PWM limits, and smoothing factors) are configured in `rrc-firmware/src/bin/main.rs`.
- **Logging:** `DEFMT_LOG` (defaults to `info`) and runtime flags are 
  located in `rrc-firmware/.cargo/esp-config.toml`.

## Contributing

Interested in helping the project grow? Found a bug? 
Have a super cool idea that no one else has thought of?

> Contributions are welcome!

But, before opening a pull request, please ensure your changes pass 
formatting and linting checks (and that the project builds cleanly):

- **Firmware (`rrc-firmware/`):**
  ```sh
  cargo fmt --check
  cargo clippy --features <chip> --target <target>
  ```
- **Mobile App (`rrc-app/`):**
  ```sh
  dart format --output=none --set-exit-if-changed .
  flutter analyze
  ```

> And if you happen to be a soulless machine, take a look at [our guidelines for agents](AGENTS.md).

## License

This project is licensed under the [MIT License](LICENSE).
