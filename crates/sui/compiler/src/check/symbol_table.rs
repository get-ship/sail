//! Cross-check symbol table built from a paired controller.
//!
//! The byte-walking scope-bound check answers "is this name declared?"
//! but stops short of "what kind of binding is it?". The
//! type-strengthening checks need the second answer: an arrow-shaped
//! scope action behaves differently from a ref-shaped scope handle,
//! and a derived getter is only a callable when invoked with `()`.
//!
//! This module owns the kind classifier so each downstream check
//! (`scope_member`, `bind_this_target`, `on_callable`) can ask
//! `kind_of_scope("foo")` instead of re-walking the controller source.

// FILE_CAP_OVERRIDE: one cohesive concern — a single SymbolTable builder + the
// ScopeKind classifier. The collect_* fns are its construction internals
// (section keys, use keys, scope kinds); splitting them scatters one
// table-build across files for no clarity gain.
use std::collections::BTreeMap;

use crate::check::controller_parse::ParsedController;
use crate::check::js_scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};
use crate::ir::Program;

/// Author-declared shape of a scope key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// `key: ref(initial)` — DOM-handle slot; written by the runtime,
    /// read like a signal.
    Ref,
    /// `key: signal(initial)` — readable + writable cell.
    Signal,
    /// `key: derived(() => ...)` — read-only computed getter.
    Derived,
    /// `key: () => ...` or `key: (e) => ...` — bound action / handler.
    Action,
    /// Anything else (plain values, expressions we can't classify).
    Other,
}

impl ScopeKind {
    pub fn label(self) -> &'static str {
        match self {
            ScopeKind::Ref => "ref",
            ScopeKind::Signal => "signal",
            ScopeKind::Derived => "derived",
            ScopeKind::Action => "action",
            ScopeKind::Other => "value",
        }
    }

    pub fn is_callable(self) -> bool {
        matches!(self, ScopeKind::Action)
    }
}

/// Symbol table merged across the program's `<prop>` directives and
/// the paired controller's top-level sections.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub props: BTreeMap<String, KeyEntry>,
    pub derived: BTreeMap<String, KeyEntry>,
    pub on: BTreeMap<String, KeyEntry>,
    /// `use:` keys — opaque type, treated as Other.
    pub use_keys: BTreeMap<String, KeyEntry>,
    /// Scope-factory return keys, with kinds.
    pub scope: BTreeMap<String, ScopeEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    /// Byte offset into the controller source (or 0 if the key is
    /// declared in the `.sui`).
    pub offset: usize,
    /// Source kind label — surfaced in hover.
    pub kind_label: &'static str,
}

#[derive(Debug, Clone)]
pub struct ScopeEntry {
    pub offset: usize,
    pub kind: ScopeKind,
}

impl SymbolTable {
    pub fn build(program: &Program, controller: Option<&ParsedController>) -> Self {
        let mut t = Self::default();
        for p in &program.props {
            let cm = crate::emit::util::camel_of(&p.name);
            t.props.insert(cm, KeyEntry { offset: 0, kind_label: "prop" });
        }
        let Some(c) = controller else {
            return t;
        };
        for sec in &c.sections {
            match sec.name.as_str() {
                "derived" => collect_section_keys(c, sec.value_span, "derived", &mut t.derived),
                "on" => collect_section_keys(c, sec.value_span, "on", &mut t.on),
                "use" => collect_use_keys(c, sec.value_span, &mut t.use_keys),
                "setup" => collect_scope_kinds(c, sec.value_span, &mut t.scope),
                _ => {}
            }
        }
        t
    }

    pub fn kind_of_scope(&self, name: &str) -> Option<ScopeKind> {
        self.scope.get(name).map(|e| e.kind)
    }

    /// Iterator of every (name, label) pair for completion.
    pub fn all_root_names(&self) -> Vec<(String, &'static str)> {
        let mut out: Vec<(String, &'static str)> = Vec::new();
        out.extend(self.props.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.derived.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.on.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.use_keys.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.scope.iter().map(|(k, v)| (k.clone(), v.kind.label())));
        out
    }
}

fn collect_section_keys(
    c: &ParsedController,
    span: crate::diagnostics::Span,
    label: &'static str,
    into: &mut BTreeMap<String, KeyEntry>,
) {
    let bytes = c.source.as_bytes();
    if span.start >= span.end || bytes[span.start] != b'{' {
        return;
    }
    let start = span.start + 1;
    let end = span.end.saturating_sub(1).min(bytes.len());
    walk_object_keys(bytes, start, end, |name, offset| {
        into.insert(name.to_string(), KeyEntry { offset, kind_label: label });
    });
}

fn collect_use_keys(
    c: &ParsedController,
    span: crate::diagnostics::Span,
    into: &mut BTreeMap<String, KeyEntry>,
) {
    let bytes = c.source.as_bytes();
    let mut i = span.start;
    let end = span.end.min(bytes.len());
    i = skip_trivia(bytes, i);
    if i < end && bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let after = skip_trivia(bytes, close);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            i = skip_trivia(bytes, after + 2);
        }
    }
    if i < end && bytes[i] == b'(' {
        i = skip_trivia(bytes, i + 1);
    }
    if i < end && bytes[i] == b'{' {
        let close = skip_balanced(bytes, i, end);
        walk_object_keys(bytes, i + 1, close.saturating_sub(1), |name, offset| {
            into.insert(name.to_string(), KeyEntry { offset, kind_label: "use" });
        });
    }
}

fn collect_scope_kinds(
    c: &ParsedController,
    span: crate::diagnostics::Span,
    into: &mut BTreeMap<String, ScopeEntry>,
) {
    let bytes = c.source.as_bytes();
    let start = span.start;
    let end = span.end.min(bytes.len());
    let return_kw = b"return";
    let mut i = start;
    let mut last_return_obj: Option<usize> = None;
    while i + return_kw.len() < end {
        if bytes[i..].starts_with(return_kw) {
            let after = skip_trivia(bytes, i + return_kw.len());
            if after < end && bytes[after] == b'{' {
                last_return_obj = Some(after);
            }
        }
        i += 1;
    }
    let obj_start = if let Some(s) = last_return_obj {
        s
    } else {
        // Expression body: skip params + `=>` + optional `(`.
        let mut k = skip_trivia(bytes, start);
        if k >= end {
            return;
        }
        if bytes[k] == b'(' {
            k = skip_balanced(bytes, k, end);
        } else if is_ident_start(bytes[k]) {
            while k < end && is_ident_cont(bytes[k]) {
                k += 1;
            }
        } else {
            return;
        }
        k = skip_trivia(bytes, k);
        if k + 1 >= end || bytes[k] != b'=' || bytes[k + 1] != b'>' {
            return;
        }
        k = skip_trivia(bytes, k + 2);
        while k < end && bytes[k] == b'(' {
            k = skip_trivia(bytes, k + 1);
        }
        if k >= end || bytes[k] != b'{' {
            return;
        }
        k
    };
    let obj_end = skip_balanced(bytes, obj_start, end);
    let factory_body_start = start;
    let factory_body_end = obj_start;
    walk_object_entries(
        bytes,
        obj_start + 1,
        obj_end.saturating_sub(1),
        |name, key_offset, value_start, value_end| {
            let mut kind = classify_scope_value(bytes, value_start, value_end);
            // Shorthand entries (e.g. `{ foo }`) classify as Other based
            // on the immediate value span; resolve them by looking up
            // the binding in the factory body before the return object.
            if kind == ScopeKind::Other && is_shorthand(bytes, name, value_start, value_end) {
                kind = classify_local_binding(bytes, name, factory_body_start, factory_body_end);
            }
            into.insert(name.to_string(), ScopeEntry { offset: key_offset, kind });
        },
    );
}

/// Returns true when the value-span is just the key ident (shorthand
/// property-init like `{ foo }`).
fn is_shorthand(bytes: &[u8], name: &str, start: usize, end: usize) -> bool {
    let s = skip_trivia(bytes, start);
    if s >= end || s + name.len() > end {
        return false;
    }
    if !std::str::from_utf8(&bytes[s..s + name.len()])
        .map(|w| w == name)
        .unwrap_or(false)
    {
        return false;
    }
    // Nothing after the ident other than trivia / commas inside the slice.
    let after = skip_trivia(bytes, s + name.len());
    after >= end
}

/// Walk the factory body for a top-level `const NAME = …` /
/// `function NAME …` / `async function NAME …` declaration and
/// classify the right-hand side. Returns Other if no binding is found
/// or the binding's RHS isn't a recognised shape.
fn classify_local_binding(
    bytes: &[u8],
    name: &str,
    start: usize,
    end: usize,
) -> ScopeKind {
    let nb = name.as_bytes();
    let mut i = start;
    while i < end {
        // Match `function NAME` / `async function NAME`.
        if bytes[i..].starts_with(b"function") {
            let mut j = i + 8;
            j = skip_trivia(bytes, j);
            if j + nb.len() <= end
                && &bytes[j..j + nb.len()] == nb
                && (j + nb.len() == end || !is_ident_cont(bytes[j + nb.len()]))
            {
                return ScopeKind::Action;
            }
        }
        // Match `const NAME = …` / `let NAME = …`.
        let kw_len = if bytes[i..].starts_with(b"const") {
            5
        } else if bytes[i..].starts_with(b"let") {
            3
        } else {
            0
        };
        if kw_len > 0 {
            let mut j = i + kw_len;
            // Must be a non-ident boundary before kw and after.
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let post_ok = j < end && !is_ident_cont(bytes[j]);
            if prev_ok && post_ok {
                j = skip_trivia(bytes, j);
                if j + nb.len() <= end
                    && &bytes[j..j + nb.len()] == nb
                    && (j + nb.len() >= end || !is_ident_cont(bytes[j + nb.len()]))
                {
                    let mut k = skip_trivia(bytes, j + nb.len());
                    // Skip optional `: TYPE`.
                    if k < end && bytes[k] == b':' {
                        k += 1;
                        // Walk until `=` at top level (skip nested angles/brackets).
                        let mut depth: i32 = 0;
                        while k < end {
                            let c = bytes[k];
                            if depth == 0 && c == b'=' {
                                break;
                            }
                            match c {
                                b'<' | b'(' | b'{' | b'[' => depth += 1,
                                b'>' | b')' | b'}' | b']' => depth -= 1,
                                _ => {}
                            }
                            k += 1;
                        }
                    }
                    if k < end && bytes[k] == b'=' {
                        k = skip_trivia(bytes, k + 1);
                        // Use existing classifier on the RHS slice.
                        let rhs_end = end;
                        return classify_scope_value(bytes, k, rhs_end);
                    }
                }
            }
        }
        i += 1;
    }
    ScopeKind::Other
}

/// If `bytes[start..]` begins with `kw` (followed by a non-ident
/// boundary or end-of-slice), return the index past the keyword.
/// Otherwise None. Used to peel `async` / `function` prefixes off
/// scope-entry RHS values during classification.
fn strip_kw(bytes: &[u8], start: usize, end: usize, kw: &[u8]) -> Option<usize> {
    if start + kw.len() > end {
        return None;
    }
    if &bytes[start..start + kw.len()] != kw {
        return None;
    }
    let after = start + kw.len();
    if after < end && is_ident_cont(bytes[after]) {
        return None;
    }
    Some(after)
}

fn classify_scope_value(bytes: &[u8], start: usize, end: usize) -> ScopeKind {
    let mut i = skip_trivia(bytes, start);
    if i >= end {
        return ScopeKind::Other;
    }
    // Strip a leading `async` — `async (x) => …`, `async x => …`,
    // `async function …` all classify the same as their non-async
    // counterparts for `on:click={…}` purposes.
    if let Some(after) = strip_kw(bytes, i, end, b"async") {
        i = skip_trivia(bytes, after);
        if i >= end {
            return ScopeKind::Other;
        }
    }
    // Function expression: `function NAME?(args)…` / `function* NAME?(args)…`.
    // Pre-fix, `toggleStream: function toggleStream(): void {…}` classified
    // as Other and `on:click={toggleStream}` was rejected as "not callable",
    // forcing every author to either reshape as an arrow or wrap inline.
    if let Some(after) = strip_kw(bytes, i, end, b"function") {
        let mut j = skip_trivia(bytes, after);
        if j < end && bytes[j] == b'*' {
            j = skip_trivia(bytes, j + 1);
        }
        if j < end && is_ident_start(bytes[j]) {
            while j < end && is_ident_cont(bytes[j]) {
                j += 1;
            }
            j = skip_trivia(bytes, j);
        }
        if j < end && bytes[j] == b'<' {
            j = skip_generic_args(bytes, j, end);
            j = skip_trivia(bytes, j);
        }
        if j < end && bytes[j] == b'(' {
            return ScopeKind::Action;
        }
        return ScopeKind::Other;
    }
    // Arrow shapes: `(x) => …` / `(x): T => …` / `x => …` / `() => …` => Action.
    if bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let mut after = skip_trivia(bytes, close);
        // Optional return-type annotation: `(...): T => …`.
        if after < end && bytes[after] == b':' {
            after = skip_type_annotation(bytes, after + 1, end);
            after = skip_trivia(bytes, after);
        }
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return ScopeKind::Action;
        }
    }
    if is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = std::str::from_utf8(&bytes[s..i]).unwrap_or("");
        let mut after = skip_trivia(bytes, i);
        // `name => …` arrow shorthand.
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return ScopeKind::Action;
        }
        // Skip TS generic type-args, e.g. `ref<HTMLButtonElement>(null)`.
        if after < end && bytes[after] == b'<' {
            after = skip_generic_args(bytes, after, end);
            after = skip_trivia(bytes, after);
        }
        if after < end && bytes[after] == b'(' {
            return match word {
                "ref" => ScopeKind::Ref,
                "signal" => ScopeKind::Signal,
                "derived" | "computed" => ScopeKind::Derived,
                _ => ScopeKind::Other,
            };
        }
    }
    ScopeKind::Other
}

/// Walk a TS type annotation starting at the character after the
/// leading `:`. Stops when an `=>` token or top-level `=`/`,`/`)`/`}`
/// is reached. Tracks `<>`, `()`, `{}`, `[]` depth.
fn skip_type_annotation(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 {
            // `=>` is the arrow start.
            if c == b'=' && k + 1 < end && bytes[k + 1] == b'>' {
                return k;
            }
            if matches!(c, b',' | b';' | b'=' | b')' | b'}' | b']') {
                return k;
            }
        }
        match c {
            b'<' | b'(' | b'{' | b'[' => depth += 1,
            b'>' | b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return k;
        }
        k += 1;
    }
    end
}

/// Skip a TypeScript generic-args block starting at `bytes[start] == b'<'`.
/// Counts `<>` depth and returns the index immediately after the
/// matching `>`. If the block is malformed, returns `end`.
fn skip_generic_args(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth = 0i32;
    while k < end {
        match bytes[k] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            // Strings / comments unlikely inside type args; bail on them.
            b'"' | b'\'' | b'`' => return end,
            _ => {}
        }
        k += 1;
    }
    end
}

/// Visit `key: value` entries within an object literal slice. The
/// callback receives the key name, the key's byte offset, and the
/// value-span range.
fn walk_object_entries<F>(bytes: &[u8], start: usize, end: usize, mut f: F)
where
    F: FnMut(&str, usize, usize, usize),
{
    let mut i = start;
    while i < end {
        i = skip_trivia(bytes, i);
        if i >= end {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            key = std::str::from_utf8(&bytes[s..i])
                .unwrap_or("")
                .to_string();
            if i < end {
                i += 1;
            }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = std::str::from_utf8(&bytes[key_start..i])
                .unwrap_or("")
                .to_string();
        } else {
            i = next_entry(bytes, i, end);
            continue;
        }
        let after_key = i;
        i = skip_trivia(bytes, i);
        // Shorthand entry — value is the key itself (treat as "Other").
        if i < end && bytes[i] == b',' {
            f(&key, key_start, key_start, after_key);
            i += 1;
            continue;
        }
        if i >= end || bytes[i] != b':' {
            f(&key, key_start, key_start, after_key);
            i = next_entry(bytes, i, end);
            continue;
        }
        i += 1;
        i = skip_trivia(bytes, i);
        let value_start = i;
        let value_end = next_entry(bytes, i, end);
        f(&key, key_start, value_start, value_end);
        i = value_end;
        if i < end && bytes[i] == b',' {
            i += 1;
        }
    }
}

fn walk_object_keys<F>(bytes: &[u8], start: usize, end: usize, mut f: F)
where
    F: FnMut(&str, usize),
{
    walk_object_entries(bytes, start, end, |name, key_offset, _, _| {
        f(name, key_offset)
    });
}

/// Walk to the next top-level `,` or to `end`.
fn next_entry(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' {
                k += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                k += 1;
            }
            b'(' | b'{' | b'[' => {
                k = skip_balanced(bytes, k, end);
            }
            b',' => return k,
            _ => k += 1,
        }
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("test.controller.ts"), src.to_string()).expect("parsed")
    }

    fn empty_program() -> Program {
        Program {
            props: vec![],
            styles: String::new(),
            scope_types: String::new(),
            template: vec![],
            source_len: 0,
            span: crate::diagnostics::Span::point(0),
        }
    }

    #[test]
    fn classifies_scope_kinds() {
        let c = ctrl(
            r#"export default controller({
                setup: () => ({
                    el: ref(null),
                    n: signal(0),
                    doubled: derived(() => 1),
                    handle: (e) => 1,
                    plain: 42,
                }),
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("el"), Some(ScopeKind::Ref));
        assert_eq!(t.kind_of_scope("n"), Some(ScopeKind::Signal));
        assert_eq!(t.kind_of_scope("doubled"), Some(ScopeKind::Derived));
        assert_eq!(t.kind_of_scope("handle"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("plain"), Some(ScopeKind::Other));
    }

    #[test]
    fn classifies_async_and_function_expression_values() {
        // Pre-fix, `toggleStream: function toggleStream(): void {…}` and
        // `openModal: async (): Promise<void> => {…}` both classified as
        // Other, so `on:click={toggleStream}` was rejected as "not
        // callable", forcing arrow wrap. Real-world idioms (named
        // function expressions for stack-trace readability, async
        // handlers) should round-trip the on:-callable check.
        let c = ctrl(
            r#"export default controller({
                setup: () => ({
                    asyncArrow: async (): Promise<void> => {},
                    asyncShort: async e => e,
                    fnExpr: function foo(): void {},
                    fnAnon: function () { return 1 },
                    asyncFn: async function bar() {},
                    gen: function* () { yield 1 },
                }),
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("asyncArrow"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("asyncShort"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("fnExpr"),     Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("fnAnon"),     Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("asyncFn"),    Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("gen"),        Some(ScopeKind::Action));
    }

    #[test]
    fn classifies_shorthand_arrow_const() {
        let c = ctrl(
            r#"export default controller({
                setup: () => {
                    const inc = (e) => 1;
                    const typed = (e: KeyboardEvent): void => { return };
                    function dec() { return 2 }
                    const n = signal(0);
                    return { inc, typed, dec, n };
                },
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("inc"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("typed"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("dec"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("n"), Some(ScopeKind::Signal));
    }

    #[test]
    fn classifies_ref_with_ts_generic() {
        let c = ctrl(
            r#"export default controller({
                setup: () => ({
                    btn: ref<HTMLButtonElement>(null),
                    list: signal<string[]>([]),
                }),
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("btn"), Some(ScopeKind::Ref));
        assert_eq!(t.kind_of_scope("list"), Some(ScopeKind::Signal));
    }

    #[test]
    fn collects_derived_and_on_keys() {
        let c = ctrl(
            r#"export default controller({
                derived: { total: () => 1 },
                on: { increment: () => {} },
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert!(t.derived.contains_key("total"));
        assert!(t.on.contains_key("increment"));
    }
}
