//! Build script — emits canonical `BUILD_*` env vars consumed by
//! `build_info::build_info!()` so the compiler can stamp its identity
//! (git sha + dirty flag + build timestamp) into every artifact it
//! emits. Cargo scopes `cargo:rustc-env` per crate, so library
//! consumers cannot see these unless they pass through
//! `compiler_version()` defined in `src/lib.rs`.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_info::emit_build_env();
}
