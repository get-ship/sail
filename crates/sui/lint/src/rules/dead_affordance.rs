//! `sui-dead-affordance` — a declared `<prop name=…>` or `<event name=…>` in a
//! `.sui` that is never read anywhere: not interpolated (`{name()}`), not bound
//! (`attr:name` / `prop:name` / `class:name` / `bind:name`), not read by the
//! `<style>` block's reflected-attribute selectors (`:host([name…])` /
//! `[name…]`), and not referenced in the paired `<stem>.controller.ts`.
//!
//! A dead affordance is a lie in the component's public API: it shows up in the
//! prop table / autocomplete and the registry (`primitives.json`), but setting
//! it does nothing. The next author treats the scaffold as architecture.
//!
//! ## What counts as "read" (deliberately generous — false positives are worse
//! than false negatives for a tail this wide, ~380 reflect props). A prop is
//! live if ANY holds: (1) interpolation `{name}` / `{name()}`; (2) a directive
//! binding `attr:`/`prop:`/`class:`/`bind:name`; (3) a bare `name(` accessor
//! call in the body; (4) a reflected-attribute CSS selector `[name…]` in the
//! `<style>` block — the `reflect` idiom, e.g. `sui-skeleton` styles purely via
//! `:host([shape=…])` with no interpolation/controller and is NOT dead; (5) the
//! paired `<stem>.controller.ts` (read from disk) mentions the name.
//!
//! **Severity: Warning.** The tail is large and the heuristics can still miss an
//! exotic read path, so this surfaces candidates without failing a compile.
//! Promotion to Error is a later sweep once the tail is triaged.

use std::path::{Path, PathBuf};

use crate::prop_extract::extract_props;
use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-dead-affordance";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Split the source into a `<style>` region (for CSS attr-selector reads) and
    // the rest (template body). Comments AND the `<prop>`/`<event>` declaration
    // tags themselves are stripped, so neither a docblock mention nor the
    // declaration's own `name="x"` counts as a read of `x`.
    let style = extract_style(source);
    let body = strip_declarations(&strip_comments(source));

    // Paired controller text, if the file exists on disk. Synthetic paths (no
    // real file) read as `None` and are simply treated as "no controller".
    let controller = read_paired_controller(path);
    let controller = controller.as_deref().unwrap_or("");

    for affordance in declared_affordances(source) {
        let name = &affordance.name;
        if is_read(name, &body, &style, controller) {
            continue;
        }
        let (line, col) = byte_offset_to_line_col(source, affordance.decl_offset);
        let kind = affordance.kind;
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: format!(
                "declared {kind} `{name}` is never read — no `{{{name}()}}` \
                 interpolation, no `attr:`/`prop:`/`class:`/`bind:` binding, no \
                 reflected-attribute CSS selector, and no reference in the paired \
                 controller. A dead affordance is a lie in the component's public \
                 API: it appears in the prop table and registry but does nothing."
            ),
            suggest: Some(format!(
                "Either wire `{name}` into the template/controller, or remove the \
                 `<{kind}>` declaration. If it's read by something this lint can't \
                 see, mark it `reflect` and style via `:host([{name}…])`."
            )),
        });
    }

    findings
}

/// A declared affordance — a `<prop>` or `<event>` with a `name=`.
struct Affordance {
    name: String,
    kind: &'static str,
    /// Byte offset of the opening `<` of the declaration.
    decl_offset: usize,
}

/// Collect `<prop name=…>` (via the shared extractor) and `<event name=…>`
/// declarations with their source offsets.
fn declared_affordances(source: &str) -> Vec<Affordance> {
    let mut out = Vec::new();
    // Props — reuse the shared extractor for the name set, then locate each
    // declaration's offset for the finding's caret.
    for prop in extract_props(source) {
        if let Some(off) = decl_offset(source, "<prop", &prop.name) {
            out.push(Affordance { name: prop.name, kind: "prop", decl_offset: off });
        }
    }
    // Events — `<event name="…">`. No declarations exist in the tree yet, but
    // the affordance contract covers both; scan for completeness.
    for (name, off) in scan_named_tags(source, "<event") {
        out.push(Affordance { name, kind: "event", decl_offset: off });
    }
    out
}

/// Find the byte offset of the `<tag` whose interior declares `name="value"`
/// (or `name='value'`). Used for caret placement only.
fn decl_offset(source: &str, tag: &str, name: &str) -> Option<usize> {
    let mut search = 0;
    while let Some(rel) = source[search..].find(tag) {
        let start = search + rel;
        let after = start + tag.len();
        if let Some(close_rel) = source[after..].find('>') {
            let inner = &source[after..after + close_rel];
            if tag_name_attr(inner).as_deref() == Some(name) {
                return Some(start);
            }
            search = after + close_rel + 1;
        } else {
            break;
        }
    }
    None
}

/// Scan `<tag name="…">` declarations, returning `(name, opening-`<`-offset)`.
fn scan_named_tags(source: &str, tag: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut search = 0;
    while let Some(rel) = source[search..].find(tag) {
        let start = search + rel;
        let after = start + tag.len();
        // Guard against `<eventual>`-style lookalikes: require a delimiter.
        let next = source.as_bytes().get(after).copied();
        if !matches!(next, Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')) {
            search = after;
            continue;
        }
        if let Some(close_rel) = source[after..].find('>') {
            let inner = &source[after..after + close_rel];
            if let Some(name) = tag_name_attr(inner) {
                out.push((name, start));
            }
            search = after + close_rel + 1;
        } else {
            break;
        }
    }
    out
}

/// Read a `name="…"` (or `name='…'`) attribute from a tag interior.
fn tag_name_attr(inner: &str) -> Option<String> {
    let idx = find_attr(inner, "name")?;
    let rest = &inner[idx + 4..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let after = &rest[1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

/// Find `name` as a standalone attribute key (preceded by whitespace or the
/// start of the interior), so `surname` doesn't match `name`.
fn find_attr(inner: &str, attr: &str) -> Option<usize> {
    let bytes = inner.as_bytes();
    let mut search = 0;
    while let Some(rel) = inner[search..].find(attr) {
        let at = search + rel;
        let prev = if at == 0 { None } else { bytes.get(at - 1).copied() };
        let ok_prev = prev.is_none_or(|c| c.is_ascii_whitespace());
        let after = at + attr.len();
        let next = bytes.get(after).copied();
        let ok_next = matches!(next, Some(b'=' | b' ' | b'\t' | b'\n' | b'\r') | None);
        if ok_prev && ok_next {
            return Some(at);
        }
        search = after;
    }
    None
}

/// Is `name` read anywhere — template body, style attr-selectors, or controller?
///
/// A prop is declared in kebab-case (`<prop name="described-by">`) but the
/// template body reads it via the camelCase SUI accessor (`{describedBy()}`).
/// We try the literal kebab name first, then fall back to the camel form —
/// without the fallback, `sui-input`'s `described-by` / `error-message` look
/// dead even though `{describedBy()}` / `{errorMessage()}` clearly read them.
fn is_read(name: &str, body: &str, style: &str, controller: &str) -> bool {
    // 1+2+3: interpolation, directive binding, or accessor call in the body.
    if read_in_body(name, body) {
        return true;
    }
    // Same scan with the camelCase form. Skipped when the name has no `-`
    // (no transformation happened) so the common case is a single scan.
    let camel = camelize(name);
    if camel != name && read_in_body(&camel, body) {
        return true;
    }
    // 4: reflected-attribute CSS selector `[name` in the style block.
    if read_in_style(name, style) {
        return true;
    }
    // 5: identifier mention in the paired controller. The controller is
    // TypeScript and accesses props through the camelCase SUI accessor
    // (`state.$signals.describedBy()`), so we try the literal name first
    // then the camel form — same reasoning as the body scan above.
    if !controller.is_empty() && contains_identifier(controller, name) {
        return true;
    }
    if !controller.is_empty()
        && camel != name
        && contains_identifier(controller, &camel)
    {
        return true;
    }
    false
}

/// Convert a kebab-case prop name to its camelCase SUI accessor form.
/// `described-by` → `describedBy`, `error-message` → `errorMessage`,
/// `aria-label` → `ariaLabel`. Names without a `-` are returned unchanged.
/// The boundary is `-`; runs of `-` are not specially handled (a prop named
/// `x--y` is not a SUI convention, so the degenerate output is acceptable).
fn camelize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_boundary = false;
    for ch in s.chars() {
        if ch == '-' {
            at_boundary = true;
        } else if at_boundary {
            out.extend(ch.to_uppercase());
            at_boundary = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// `{name}` / `{name()}` interpolation, `attr:name=`/`prop:name=`/`class:name=`/
/// `bind:name=` directive bindings, or a bare `name(` accessor call — anywhere
/// in the comment-stripped template body. Matched on identifier boundaries so
/// `loading` doesn't satisfy `load`.
fn read_in_body(name: &str, body: &str) -> bool {
    for prefix in ["attr:", "prop:", "class:", "bind:"] {
        let needle = format!("{prefix}{name}");
        if has_token_after(body, &needle) {
            return true;
        }
    }
    contains_identifier(body, name)
}

/// `[name=` / `[name]` / `[name~=` etc. attribute selector in the style block.
/// The `reflect` idiom mirrors the prop to a host attribute that CSS reads.
fn read_in_style(name: &str, style: &str) -> bool {
    if style.is_empty() {
        return false;
    }
    let needle = format!("[{name}");
    let bytes = style.as_bytes();
    let mut search = 0;
    while let Some(rel) = style[search..].find(&needle) {
        let at = search + rel + needle.len();
        // The char after the attribute name must terminate it: `]`, `=`, `~`,
        // `|`, `^`, `$`, `*`, or whitespace. Guards `[namespace]` vs `[name]`.
        match bytes.get(at) {
            Some(b']' | b'=' | b'~' | b'|' | b'^' | b'$' | b'*')
            | Some(b' ' | b'\t' | b'\n' | b'\r') => return true,
            _ => {}
        }
        search = search + rel + needle.len();
    }
    false
}

/// True when `needle` is followed by a non-identifier byte (or EOS) — i.e.
/// `attr:name` matches `attr:name=` but not `attr:nameplate`.
fn has_token_after(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let mut search = 0;
    while let Some(rel) = haystack[search..].find(needle) {
        let at = search + rel + needle.len();
        let next = bytes.get(at).copied();
        if !matches!(next, Some(c) if is_ident_byte(c)) {
            return true;
        }
        search = search + rel + needle.len();
    }
    false
}

/// `name` appears as a standalone identifier — both sides are non-identifier
/// bytes (or boundaries). Catches `{name}`, `name()`, `props.name`, etc.
fn contains_identifier(haystack: &str, name: &str) -> bool {
    let bytes = haystack.as_bytes();
    let mut search = 0;
    while let Some(rel) = haystack[search..].find(name) {
        let at = search + rel;
        let before = if at == 0 { None } else { bytes.get(at - 1).copied() };
        let after = bytes.get(at + name.len()).copied();
        let ok_before = before.is_none_or(|c| !is_ident_byte(c));
        let ok_after = after.is_none_or(|c| !is_ident_byte(c));
        if ok_before && ok_after {
            return true;
        }
        search = at + name.len();
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Extract the inner text of the first `<style>…</style>` block, or "".
fn extract_style(source: &str) -> String {
    let Some(open) = source.find("<style") else {
        return String::new();
    };
    let Some(gt) = source[open..].find('>') else {
        return String::new();
    };
    let body_start = open + gt + 1;
    let Some(close_rel) = source[body_start..].find("</style>") else {
        return source[body_start..].to_string();
    };
    source[body_start..body_start + close_rel].to_string()
}

/// Strip HTML comments out of the source so comment contents never count as a
/// read. Only the non-comment regions are kept (read detection doesn't rely on
/// byte offsets, so collapsing comments to nothing is fine here).
fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(open) = rest.find("<!--") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 4..];
        match after.find("-->") {
            Some(close) => rest = &after[close + 3..],
            None => return out, // unterminated — drop the tail
        }
    }
    out.push_str(rest);
    out
}

/// Remove `<prop …>` and `<event …>` declaration tags so a declaration's own
/// `name="x"` attribute is never mistaken for a read of `x`.
fn strip_declarations(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    'outer: loop {
        // Find the next declaration tag opener.
        let mut best: Option<usize> = None;
        for tag in ["<prop", "<event"] {
            if let Some(at) = rest.find(tag) {
                // Require a delimiter so `<property>` / `<eventual>` don't match.
                let after = at + tag.len();
                let next = rest.as_bytes().get(after).copied();
                if matches!(next, Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/'))
                    && best.is_none_or(|b| at < b)
                {
                    best = Some(at);
                }
            }
        }
        let Some(start) = best else { break 'outer };
        out.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(close) => rest = &rest[start + close + 1..],
            None => return out, // unterminated tag — drop the tail
        }
    }
    out.push_str(rest);
    out
}

/// Read the paired `<stem>.controller.ts` next to a `*.sui` template, if any.
fn read_paired_controller(sui_path: &Path) -> Option<String> {
    let stem = sui_path.file_stem()?.to_str()?;
    let dir = sui_path.parent()?;
    let controller: PathBuf = dir.join(format!("{stem}.controller.ts"));
    std::fs::read_to_string(controller).ok()
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(src: &str) -> Vec<Finding> {
        check(&PathBuf::from("/fake/foo.sui"), src)
    }

    // --- camelize helper ---

    #[test]
    fn camelize_kebab_to_camel() {
        assert_eq!(camelize("described-by"), "describedBy");
        assert_eq!(camelize("error-message"), "errorMessage");
        assert_eq!(camelize("aria-label"), "ariaLabel");
        assert_eq!(camelize("tone"), "tone");
        assert_eq!(camelize(""), "");
        // Multi-segment (e.g. "tone-primary-border" — current SUI primitives
        // don't use these, but the helper is general).
        assert_eq!(camelize("tone-primary-border"), "tonePrimaryBorder");
    }

    // --- Kebab prop read via camelCase accessor ---

    #[test]
    fn kebab_prop_read_via_camel_accessor_is_live() {
        // The literal text is a kebab prop, but the template reads it via
        // the camelCase SUI accessor. Should NOT be flagged.
        let src = r#"
            <prop name="described-by" type="string" default="" />
            <prop name="error-message" type="string" default="" />
            <div aria-describedby={describedBy()}>
              {#if invalid() && errorMessage()}
                <span role="alert">{errorMessage()}</span>
              {/if}
            </div>
        "#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn kebab_prop_with_no_camel_accessor_still_flags() {
        // Genuinely dead — kebab prop declared, no camel accessor either.
        let src = r#"
            <prop name="described-by" type="string" default="" />
            <div>hello</div>
        "#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("described-by"), "{}", f[0].message);
    }

    #[test]
    fn prop_without_kebab_still_works() {
        // A prop with no kebab (no transformation) — original behaviour.
        let src = r#"
            <prop name="tone" type="string" default="" />
            <div>{tone()}</div>
        "#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Existing behaviour preserved ---

    #[test]
    fn truly_dead_prop_still_flags() {
        let src = r#"
            <prop name="orphan" type="string" default="" />
            <div>hello</div>
        "#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("orphan"));
    }

    #[test]
    fn prop_read_in_attr_directive_is_live() {
        // `attr:foo` directive counts as a read in the body scan.
        let src = r#"
            <prop name="foo" type="string" default="" />
            <div attr:foo={foo()}></div>
        "#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }
}
