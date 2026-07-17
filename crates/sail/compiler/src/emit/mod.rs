//! Emit pipeline. Takes IR + lowered metadata and produces a JS
//! ESM module, a CEM JSON entry (component mode), and a directive
//! schema JSON object.
//!
//! See `js.rs` for the module-level layout and instruction emission,
//! `cem.rs` for the custom-elements-manifest entry, and `schema.rs`
//! for the directive declarations bundle.

pub mod cem;
pub mod element;
pub mod dts;
pub mod expr_rewrite;
pub mod js;
pub mod meta;
pub mod schema;
pub mod scope_factory;
pub mod static_html;
pub mod synthetic;
pub mod styles;
pub mod template_emit;
pub mod util;
pub mod writer;
