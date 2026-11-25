fn main() {
    validate_chip_feature();
    linker_be_nice();
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

/// Ensures that exactly one chip feature is enabled.
///
/// Cargo features are additive, so without this check two chips could be
/// selected at once and produce a confusing error from `esp-hal` (or, worse,
/// a target triple mismatch).
fn validate_chip_feature() {
    // Only chips with a supported LEDC driver are listed: the servo and the
    // motor are driven through LEDC. `esp-hal` 1.2 does not implement LEDC for
    // the ESP32-C5, ESP32-C61 and ESP32-S31, and the ESP32-H2/ESP32-P4 have no
    // Wi-Fi radio.
    const CHIPS: &[&str] = &[
        "esp32", "esp32c2", "esp32c3", "esp32c6", "esp32s2", "esp32s3",
    ];

    let selected: Vec<&str> = CHIPS
        .iter()
        .copied()
        .filter(|chip| std::env::var(format!("CARGO_FEATURE_{}", chip.to_uppercase())).is_ok())
        .collect();

    match selected.as_slice() {
        [_] => {}
        [] => panic!(
            "No ESP32 chip feature selected. Enable one of: {}.",
            CHIPS.join(", ")
        ),
        _ => panic!(
            "Multiple ESP32 chip features selected: {}. Enable exactly one chip feature.",
            selected.join(", ")
        ),
    }
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                what if what.starts_with("_defmt_") => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                what if what.starts_with("esp_rtos_") => {
                    eprintln!();
                    eprintln!(
                        "💡 `esp-radio` has no scheduler enabled. Make sure you have initialized `esp-rtos` or provided an external scheduler."
                    );
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!(
                        "💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests"
                    );
                    eprintln!();
                }
                "free"
                | "malloc"
                | "calloc"
                | "get_free_internal_heap_size"
                | "malloc_internal"
                | "realloc_internal"
                | "calloc_internal"
                | "free_internal" => {
                    eprintln!();
                    eprintln!(
                        "💡 Did you forget the `esp-alloc` dependency or didn't enable the `compat` feature on it?"
                    );
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    // RISC-V targets are linked with `rust-lld`, while Xtensa targets are
    // linked through GCC, which needs linker flags to be forwarded with `-Wl,`.
    #[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3"))]
    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
    #[cfg(not(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3")))]
    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
