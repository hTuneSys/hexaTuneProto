// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let config =
        cbindgen::Config::from_file(format!("{}/cbindgen.toml", crate_dir)).unwrap_or_default();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .map(|bindings| {
            bindings.write_to_file("include/hexa_tune_proto.h");
        })
        .ok();
}
