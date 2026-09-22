//! `sail-lsp` binary entry — thin shell over the library.
fn main() -> anyhow::Result<()> {
    sail_lsp::run()
}
