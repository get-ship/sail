// napi-build sets up the linker flags the node-API addon needs (e.g.
// `-undefined dynamic_lookup` on macOS) so the produced cdylib loads as a
// Node `.node` addon. Required by napi-rs; does nothing else.
fn main() {
    napi_build::setup();
}
