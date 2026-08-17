//! Build script — emits `BUILD_*` env vars.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_info::emit_build_env();
}
