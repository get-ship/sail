//! Analysis facade — the read surface the LSP server and agent (MCP) tools
//! consume. One Rust entry point over the compiler's existing intelligence:
//! `check_all` (multi-diagnostics), `SymbolTable` (scope names + kinds +
//! declaration offsets), and an IR walk for tag/event dependencies.
//!
//! No TypeScript, no WASM — the controller `.ts` is read structurally by the
//! `SymbolTable` byte-walker (the same one the type-strengthening checks use).
//! Positions are 1-based, matching the rest of the compiler; the LSP transport
//! converts to 0-based protocol coordinates.

use std::collections::BTreeSet;
use std::path::Path;

use crate::check::controller_parse::{read_paired_controller, ParsedController};
use crate::check::symbol_table::SymbolTable;
use crate::diagnostics::{line_col, Diagnostic};
use crate::ir::*;

/// A name in scope for a `.sui` template, with where it came from and (when
/// controller-declared) where to jump to.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Symbol {
    pub name: String,
    /// Binding shape: `data` | `prop` | `signal` | `ref` | `derived` |
    /// `action` | `value` | `on` | `use`.
    pub kind: String,
    /// Origin section: `data` | `prop` | `scope` | `derived` | `on` | `use`.
    pub source: String,
    /// 1-based declaration position in the controller. `(0, 0)` when the
    /// symbol is declared in the `.sui` itself (`<data>` / `<prop>`).
    pub line: usize,
    pub col: usize,
}

/// Hover result for an identifier under the cursor.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Hover {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub line: usize,
    pub col: usize,
}

/// Everything the tooling needs about one `.sui` source in a single pass.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Analysis {
    pub diagnostics: Vec<Diagnostic>,
    pub symbols: Vec<Symbol>,
    /// Custom-element tags the template instantiates (e.g. `sui-button`).
    pub tags: Vec<String>,
    /// `on:` event names the template handles (e.g. `click`).
    pub events: Vec<String>,
    /// Resolved paired-controller path, when one exists on disk.
    pub controller_path: Option<String>,
}

/// Full analysis of a `.sui` source: diagnostics + scope symbols + the tag /
/// event dependency surface. `source_path` locates the paired controller.
pub fn analyze(source: &str, source_path: Option<&Path>) -> Analysis {
    let diagnostics = crate::check_all(source, source_path);
    let Ok(program) = crate::parse::parse(source) else {
        // Without an AST we can still return the parse diagnostic.
        return Analysis {
            diagnostics,
            symbols: Vec::new(),
            tags: Vec::new(),
            events: Vec::new(),
            controller_path: None,
        };
    };
    let controller = read_paired_controller(source_path);
    let table = SymbolTable::build(&program, controller.as_ref());
    let symbols = symbols_from_table(&table, controller.as_ref());
    let (tags, events) = collect_tags_events(&program);
    Analysis {
        diagnostics,
        symbols,
        tags,
        events,
        controller_path: controller.as_ref().map(|c| c.path.display().to_string()),
    }
}

/// Resolve the symbol under a 1-based `(line, col)` cursor. Finds the root of
/// the member chain at the cursor (`items` for a cursor on `items().length`)
/// and looks it up in the scope table.
pub fn hover_at(source: &str, source_path: Option<&Path>, line: usize, col: usize) -> Option<Hover> {
    let program = crate::parse::parse(source).ok()?;
    let controller = read_paired_controller(source_path);
    let table = SymbolTable::build(&program, controller.as_ref());
    let offset = offset_of(source, line, col);
    let root = root_ident_at(source.as_bytes(), offset)?;
    symbols_from_table(&table, controller.as_ref())
        .into_iter()
        .find(|s| s.name == root)
        .map(|s| Hover { name: s.name, kind: s.kind, source: s.source, line: s.line, col: s.col })
}

/// Every scope name available for completion, with its binding kind.
pub fn completions_at(source: &str, source_path: Option<&Path>) -> Vec<Symbol> {
    let Ok(program) = crate::parse::parse(source) else {
        return Vec::new();
    };
    let controller = read_paired_controller(source_path);
    let table = SymbolTable::build(&program, controller.as_ref());
    symbols_from_table(&table, controller.as_ref())
}

// ── internals ─────────────────────────────────────────────────────────────

fn symbols_from_table(table: &SymbolTable, controller: Option<&ParsedController>) -> Vec<Symbol> {
    let ctrl_src = controller.map(|c| c.source.as_str()).unwrap_or("");
    let mut out: Vec<Symbol> = Vec::new();

    // `<data>` / `<prop>` live in the .sui — no controller offset (0).
    for (name, e) in &table.data {
        out.push(Symbol { name: name.clone(), kind: e.kind_label.to_string(), source: "data".into(), line: 0, col: 0 });
    }
    for (name, e) in &table.props {
        out.push(Symbol { name: name.clone(), kind: e.kind_label.to_string(), source: "prop".into(), line: 0, col: 0 });
    }
    // Controller-declared keys carry a byte offset into the controller source.
    for (name, e) in &table.derived {
        let (l, c) = pos(ctrl_src, e.offset);
        out.push(Symbol { name: name.clone(), kind: "derived".into(), source: "derived".into(), line: l, col: c });
    }
    for (name, e) in &table.on {
        let (l, c) = pos(ctrl_src, e.offset);
        out.push(Symbol { name: name.clone(), kind: "action".into(), source: "on".into(), line: l, col: c });
    }
    for (name, e) in &table.use_keys {
        let (l, c) = pos(ctrl_src, e.offset);
        out.push(Symbol { name: name.clone(), kind: "use".into(), source: "use".into(), line: l, col: c });
    }
    for (name, e) in &table.scope {
        let (l, c) = pos(ctrl_src, e.offset);
        out.push(Symbol { name: name.clone(), kind: e.kind.label().to_string(), source: "scope".into(), line: l, col: c });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Controller byte offset → 1-based (line, col). Offset 0 means "declared in
/// the .sui", reported as `(0, 0)`.
fn pos(ctrl_src: &str, offset: usize) -> (usize, usize) {
    if offset == 0 || ctrl_src.is_empty() {
        return (0, 0);
    }
    line_col(ctrl_src, offset)
}

/// Recursively collect custom-element tags + `on:` event names.
fn collect_tags_events(program: &Program) -> (Vec<String>, Vec<String>) {
    let mut tags: BTreeSet<String> = BTreeSet::new();
    let mut events: BTreeSet<String> = BTreeSet::new();
    walk(&program.template, &mut tags, &mut events);
    (tags.into_iter().collect(), events.into_iter().collect())
}

fn walk(instrs: &[Instr], tags: &mut BTreeSet<String>, events: &mut BTreeSet<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                // Custom elements contain a hyphen (HTML rule). Native tags
                // (div, span) aren't dependencies worth reporting.
                if el.tag.contains('-') {
                    tags.insert(el.tag.clone());
                }
                for a in &el.attrs {
                    if let AttrInstr::On { event, .. } = a {
                        events.insert(event.clone());
                    }
                }
                walk(&el.children, tags, events);
            }
            Instr::If(i) => {
                for b in &i.branches {
                    walk(&b.body, tags, events);
                }
                if let Some(e) = &i.else_body {
                    walk(e, tags, events);
                }
            }
            Instr::For(f) => {
                walk(&f.body, tags, events);
                if let Some(e) = &f.else_body {
                    walk(e, tags, events);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk(&c.body, tags, events);
                }
                if let Some(d) = &m.default {
                    walk(d, tags, events);
                }
            }
            Instr::WhileLoading(wl) => walk(&wl.body, tags, events),
            Instr::WhileError(we) => walk(&we.body, tags, events),
            Instr::Let(l) => walk(&l.body, tags, events),
            _ => {}
        }
    }
}

/// 1-based `(line, col)` → byte offset into `source`.
fn offset_of(source: &str, target_line: usize, target_col: usize) -> usize {
    let bytes = source.as_bytes();
    let (mut line, mut col, mut i) = (1usize, 1usize, 0usize);
    while i < bytes.len() {
        if line == target_line && col == target_col {
            return i;
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    bytes.len()
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Given `pos` just after a closing `)` or `]`, return the index of its
/// matching opener (so the preceding identifier sits at `..opener`).
fn skip_group_left(bytes: &[u8], pos: usize) -> usize {
    let close = bytes[pos - 1];
    let open = if close == b')' { b'(' } else { b'[' };
    let mut depth = 0i32;
    let mut i = pos;
    while i > 0 {
        let c = bytes[i - 1];
        if c == close {
            depth += 1;
        } else if c == open {
            depth -= 1;
            if depth == 0 {
                return i - 1;
            }
        }
        i -= 1;
    }
    0
}

/// Root identifier of the member chain at `offset`. For a cursor anywhere in
/// `items().length` returns `items`; walks left across `.ident` segments to
/// the chain root. Returns `None` when the cursor isn't on an identifier.
fn root_ident_at(bytes: &[u8], offset: usize) -> Option<String> {
    let len = bytes.len();
    if len == 0 {
        return None;
    }
    // Find the ident token covering (or just left of) the offset.
    let mut end = offset.min(len);
    while end < len && is_ident(bytes[end]) {
        end += 1;
    }
    let mut start = end;
    while start > 0 && is_ident(bytes[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    // Walk left across `.ident` chains to the root, stepping over trailing
    // call/index groups so `items().length` and `rows[0].id` resolve to the
    // chain root (`items`, `rows`).
    loop {
        let mut p = start;
        while p > 0 && bytes[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p == 0 || bytes[p - 1] != b'.' {
            break;
        }
        p -= 1; // skip the dot
        while p > 0 && bytes[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        // Step over any `(...)` / `[...]` groups attached to the prior segment.
        while p > 0 && (bytes[p - 1] == b')' || bytes[p - 1] == b']') {
            p = skip_group_left(bytes, p);
            while p > 0 && bytes[p - 1].is_ascii_whitespace() {
                p -= 1;
            }
        }
        let seg_end = p;
        while p > 0 && is_ident(bytes[p - 1]) {
            p -= 1;
        }
        if p == seg_end {
            break; // not an ident before the dot (e.g. a literal)
        }
        start = p; // new root segment; loop checks for another `.ident` left of it
    }
    let mut e = start;
    while e < len && is_ident(bytes[e]) {
        e += 1;
    }
    std::str::from_utf8(&bytes[start..e]).ok().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_reports_tags_and_events() {
        let src = r#"<div><sui-button on:click={go}>Go</sui-button><span>{x}</span></div>"#;
        let a = analyze(src, None);
        assert!(a.tags.contains(&"sui-button".to_string()), "tags: {:?}", a.tags);
        assert!(!a.tags.iter().any(|t| t == "div" || t == "span"), "native tags leaked: {:?}", a.tags);
        assert!(a.events.contains(&"click".to_string()), "events: {:?}", a.events);
    }

    #[test]
    fn analyze_surfaces_diagnostics() {
        let a = analyze("<p>{new Date()}</p>", None);
        assert!(!a.diagnostics.is_empty(), "expected a diagnostic for `new`");
    }

    #[test]
    fn root_ident_walks_member_chain() {
        // cursor on `length` in `items().length` → root `items`.
        let src = "items().length";
        let on_length = src.find("length").unwrap();
        assert_eq!(root_ident_at(src.as_bytes(), on_length).as_deref(), Some("items"));
        // cursor on `items` itself → `items`.
        assert_eq!(root_ident_at(src.as_bytes(), 0).as_deref(), Some("items"));
    }

    #[test]
    fn root_ident_none_off_identifier() {
        assert_eq!(root_ident_at(b"  + ", 1), None);
    }
}
