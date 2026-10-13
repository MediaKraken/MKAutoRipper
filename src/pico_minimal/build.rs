use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Put memory.x in our output directory and ensure it's available to the linker
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // Re-run this build script if memory.x changes
    println!("cargo:rerun-if-changed=memory.x");
}