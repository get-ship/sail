//! Output buffer with a silent-walk depth counter.
//!
//! While `silent > 0`, `push_*` is a no-op. Phase E-prep needs this so
//! the walker can advance its key counter past un-rendered control-flow
//! branches without polluting the rendered HTML — keeps the registry
//! and walker key indices in lockstep.

pub(crate) struct Out {
    inner: String,
    silent: u32,
}

impl Out {
    pub(crate) fn new() -> Self {
        Self { inner: String::new(), silent: 0 }
    }

    pub(crate) fn push_str(&mut self, s: &str) {
        if self.silent == 0 {
            self.inner.push_str(s);
        }
    }

    pub(crate) fn push(&mut self, c: char) {
        if self.silent == 0 {
            self.inner.push(c);
        }
    }

    pub(crate) fn enter_silent(&mut self) {
        self.silent += 1;
    }

    pub(crate) fn exit_silent(&mut self) {
        debug_assert!(self.silent > 0, "exit_silent without matching enter");
        self.silent -= 1;
    }

    pub(crate) fn into_string(self) -> String {
        self.inner
    }
}
