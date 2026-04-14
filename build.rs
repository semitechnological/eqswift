use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let udl = "src/eq_swift.udl";
    println!("cargo:rerun-if-changed={udl}");
    println!("cargo:rerun-if-env-changed=UNIFFI_BINDGEN");

    uniffi_build::generate_scaffolding(udl).expect("generate Rust scaffolding");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let swift_out_dir = manifest_dir.join("swift/Generated");
    let _ = fs::create_dir_all(&swift_out_dir);

    let bindgen = env::var("UNIFFI_BINDGEN").unwrap_or_else(|_| "uniffi-bindgen".to_string());
    let status = Command::new(bindgen)
        .args(["generate", udl, "--language", "swift", "--out-dir"])
        .arg(&swift_out_dir)
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) => {
            println!("cargo:warning=UniFFI Swift generation exited with {status}");
        }
        Err(err) => {
            println!("cargo:warning=Skipping Swift generation: {err}");
        }
    }
}
