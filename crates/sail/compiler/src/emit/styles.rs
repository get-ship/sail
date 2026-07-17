//! Compile-time CSS rule stripping.
//!
//! R3 (bundle-perf): rules that exist verbatim in `@ship/sail-runtime`'s
//! `suiBaseSheet` are duplicated across nearly every shadow component's
//! `<style>` block. The runtime adopts `suiBaseSheet` into every shadow
//! root, so the per-component copies are dead weight in the bundle.
//!
//! This module recognises the exact author-form text of those rules and
//! removes them from the `styles:` string before emit. Conservative by
//! design — only literal-form matches are stripped; anything with
//! different whitespace, ordering, or extra declarations is left alone
//! so authors keep control of edge cases.
//!
//! See `packages/sail/runtime/src/styles/index.ts` (`BASE_CSS`) for the
//! source-of-truth list of rules adopted at runtime; the patterns
//! below must mirror that set.
//!
//! Patterns currently stripped (all variants tolerate inner whitespace
//! and an optional trailing semicolon):
//!
//! - `:host([hidden]) { display: none; }`
//! - `:host([hidden]) { display: none !important; }`
//! - `@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }`
//!   (single-line, single inner rule with `*` selector)

use super::writer::js_string;

/// Emit the idempotent runtime install of a view/component's lifted `<style>`
/// CSS at the top of `container`. Returns the JS statements to inline at the
/// start of a mount body, or an empty string when `css` is blank.
///
/// The `data-sail-style` key is an FNV-1a 32-bit hash of the content — a
/// re-mount of the same view finds the existing `<style>` and skips
/// re-inserting. The `<style>` lives inside `container` so it serializes into
/// the SSR first-paint and applies on the client (the browser treats any
/// in-document `<style>` as an active stylesheet, regardless of location). No
/// marker-attribute query, no `querySelector` — the server DOM does not
/// implement `querySelector`, so a hand-rolled `firstChild` walk is the only
/// isomorphic option (and the dedupe only matters on re-mount, hit O(1)).
///
/// `typed` selects the flavor: `true` emits TS type annotations + casts (the
/// synthetic emit produces type-checked `.ts`); `false` emits plain JS (the
/// production view emit). One source of truth for both mount paths.
pub(crate) fn styles_install_js(css: &str, typed: bool) -> String {
    let css = css.trim();
    if css.is_empty() {
        return String::new();
    }
    let key = fnv1a32_hex(css.as_bytes());
    // Type annotations + the `as Element` cast differ between the typed (TS)
    // and plain-JS flavors; everything else is identical.
    let (el_ann, node_ann, n_ref, n_assign) = if typed {
        (": Element | null", ": Node | null", "(__n as Element)", "__n as Element")
    } else {
        ("", "", "__n", "__n")
    };
    let mut b = String::new();
    b.push_str(&format!("const __suiStylesKey = \"{key}\";\n"));
    b.push_str(&format!("const __suiStyles = {};\n", js_string(css)));
    b.push_str("{\n");
    b.push_str(&format!("  let __existing{el_ann} = null;\n"));
    b.push_str(&format!("  let __n{node_ann} = container.firstChild;\n"));
    b.push_str("  while (__n) {\n");
    b.push_str(&format!("    if (__n.nodeType === 1 && {n_ref}.tagName === 'STYLE' &&\n"));
    b.push_str(&format!("        {n_ref}.getAttribute(\"data-sail-style\") === __suiStylesKey) {{\n"));
    b.push_str(&format!("      __existing = {n_assign};\n"));
    b.push_str("      break;\n");
    b.push_str("    }\n");
    b.push_str("    __n = __n.nextSibling;\n");
    b.push_str("  }\n");
    b.push_str("  if (!__existing) {\n");
    b.push_str("    const __el = document.createElement(\"style\");\n");
    b.push_str("    __el.setAttribute(\"data-sail-style\", __suiStylesKey);\n");
    b.push_str("    __el.textContent = __suiStyles;\n");
    b.push_str("    container.insertBefore(__el, container.firstChild);\n");
    b.push_str("  }\n");
    b.push_str("}\n");
    b
}

/// `@property` registration for every prop/signal `(--x)` binding this
/// component's `sx` reads matched (see `check::sx_vars`). Emitted as its
/// OWN install call, never appended to `program.styles` — that string gets
/// wrapped in a `@scope (...) to (...) { ... }` donut at runtime
/// (`injectScopedStyles`), and `@property` is a top-level-only at-rule: a
/// browser silently drops one nested inside another at-rule/block. The
/// runtime's `registerCustomProperties` helper installs this CSS as its own
/// unscoped, deduped-by-content `<style>` in `<head>` instead.
///
/// `initial-value` is required by the CSS spec for every `syntax` value
/// except `"*"` — a non-numeric binding registers with `syntax: '*'` and no
/// `initial-value`, which the spec permits (`"*"` matches any token
/// sequence, so there's no type to default).
pub(crate) fn register_properties_install_js(
    bindings: &[crate::check::sx_vars::SxVarBinding],
) -> String {
    if bindings.is_empty() {
        return String::new();
    }
    let mut css = String::new();
    for b in bindings {
        if b.numeric {
            css.push_str(&format!(
                "@property --{name} {{ syntax: '<number>'; inherits: true; initial-value: 0; }}",
                name = b.css_name,
            ));
        } else {
            css.push_str(&format!(
                "@property --{name} {{ syntax: '*'; inherits: true; }}",
                name = b.css_name,
            ));
        }
    }
    let key = fnv1a32_hex(css.as_bytes());
    format!(
        "registerCustomProperties({}, {});\n",
        js_string(&format!("sail-props-{key}")),
        js_string(&css),
    )
}

/// FNV-1a 32-bit hash, hex string. A tiny dep-free helper to key the
/// `data-sail-style` dedupe attribute.
fn fnv1a32_hex(bytes: &[u8]) -> String {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

/// Remove shared base-sheet rules from `styles`. Returns the stripped
/// CSS. Idempotent — running it twice produces the same result.
///
/// Operates on byte indices so the matchers can fast-path against
/// ASCII patterns, but copies multi-byte UTF-8 by char-boundary so
/// non-ASCII content (like the rail's `◀` / `▶` toggle glyphs)
/// survives intact. The earlier `bytes[i] as char` loop truncated
/// each UTF-8 byte to a Latin-1 code unit, so `◀` (e2 97 80) showed
/// up downstream as `â—€` and esbuild serialized that as the byte
/// escape `\xE2\x97\x80` in the bundled JS — visible as mojibake in
/// the rendered shadow CSS.
pub fn strip_shared_rules(styles: &str) -> String {
    let mut out = String::with_capacity(styles.len());
    let bytes = styles.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = match_shared_rule(bytes, i) {
            // Eat trailing whitespace up to and including the next \n,
            // so stripping a whole-line rule doesn't leave a blank line.
            let mut j = end;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'\n' {
                j += 1;
            }
            // Also drop the leading whitespace on the rule's own line.
            // Walk `out` back to the previous newline (or start) and
            // truncate any indent we already pushed.
            let trim_to = out
                .rfind('\n')
                .map(|n| n + 1)
                .unwrap_or(0);
            if out[trim_to..].chars().all(|c| c == ' ' || c == '\t') {
                out.truncate(trim_to);
            }
            i = j;
            continue;
        }
        // Forward one full UTF-8 character so multi-byte sequences
        // aren't sliced byte-by-byte into Latin-1.
        let next_boundary = (i + 1..=bytes.len())
            .find(|&j| styles.is_char_boundary(j))
            .unwrap_or(bytes.len());
        out.push_str(&styles[i..next_boundary]);
        i = next_boundary;
    }
    out
}

/// If `bytes[start..]` begins with a known shared-sheet rule, return
/// the byte index just past the rule's closing `}`. Otherwise `None`.
fn match_shared_rule(bytes: &[u8], start: usize) -> Option<usize> {
    if let Some(end) = match_host_hidden(bytes, start) {
        return Some(end);
    }
    if let Some(end) = match_reduced_motion(bytes, start) {
        return Some(end);
    }
    None
}

fn match_host_hidden(bytes: &[u8], start: usize) -> Option<usize> {
    // Selector: `:host([hidden])`
    let selector = b":host([hidden])";
    if !bytes.get(start..)?.starts_with(selector) {
        return None;
    }
    let mut i = start + selector.len();
    i = skip_ws(bytes, i)?;
    if bytes.get(i)? != &b'{' {
        return None;
    }
    i += 1;
    i = skip_ws(bytes, i)?;
    // Body: `display: none` with optional `!important` and optional `;`.
    let display = b"display:";
    if !bytes.get(i..)?.starts_with(display) {
        return None;
    }
    i += display.len();
    i = skip_ws(bytes, i)?;
    let none_kw = b"none";
    if !bytes.get(i..)?.starts_with(none_kw) {
        return None;
    }
    i += none_kw.len();
    i = skip_ws(bytes, i)?;
    if bytes.get(i)? == &b'!' {
        let imp = b"!important";
        if !bytes.get(i..)?.starts_with(imp) {
            return None;
        }
        i += imp.len();
        i = skip_ws(bytes, i)?;
    }
    if bytes.get(i)? == &b';' {
        i += 1;
        i = skip_ws(bytes, i)?;
    }
    if bytes.get(i)? != &b'}' {
        return None;
    }
    Some(i + 1)
}

/// Match the single-line reduced-motion rule the runtime's base sheet
/// covers: `@media (prefers-reduced-motion: reduce) { * { transition:
/// none !important; animation: none !important; } }`.
///
/// Conservative: requires the entire rule on a single line and rejects
/// any extra inner rules. Anything more elaborate (per-element bodies,
/// extra declarations, multi-rule blocks) stays in the component.
fn match_reduced_motion(bytes: &[u8], start: usize) -> Option<usize> {
    let head = b"@media (prefers-reduced-motion: reduce)";
    if !bytes.get(start..)?.starts_with(head) {
        return None;
    }
    let mut i = start + head.len();
    i = skip_ws_nl(bytes, i)?;
    if bytes.get(i)? != &b'{' {
        return None;
    }
    i += 1;
    i = skip_ws_nl(bytes, i)?;
    if bytes.get(i)? != &b'*' {
        return None;
    }
    i += 1;
    i = skip_ws(bytes, i)?;
    if bytes.get(i)? != &b'{' {
        return None;
    }
    i += 1;
    i = skip_ws(bytes, i)?;
    // Accept `transition: none !important; animation: none !important;`
    // or the same two declarations in either order, with optional final
    // semicolon. Tracked via a 2-bit set.
    let mut seen = 0u8;
    for _ in 0..2 {
        let prop = if bytes.get(i..)?.starts_with(b"transition") {
            i += b"transition".len();
            1u8
        } else if bytes.get(i..)?.starts_with(b"animation") {
            i += b"animation".len();
            2u8
        } else {
            return None;
        };
        if seen & prop != 0 {
            return None;
        }
        seen |= prop;
        i = skip_ws(bytes, i)?;
        if bytes.get(i)? != &b':' {
            return None;
        }
        i += 1;
        i = skip_ws(bytes, i)?;
        if !bytes.get(i..)?.starts_with(b"none") {
            return None;
        }
        i += b"none".len();
        i = skip_ws(bytes, i)?;
        if !bytes.get(i..)?.starts_with(b"!important") {
            return None;
        }
        i += b"!important".len();
        i = skip_ws(bytes, i)?;
        // Required between the two decls; optional on the second.
        if bytes.get(i)? == &b';' {
            i += 1;
            i = skip_ws(bytes, i)?;
        }
    }
    if seen != 0b11 {
        return None;
    }
    if bytes.get(i)? != &b'}' {
        return None;
    }
    i += 1;
    i = skip_ws_nl(bytes, i)?;
    if bytes.get(i)? != &b'}' {
        return None;
    }
    Some(i + 1)
}

/// Advance past spaces and tabs (no newlines — rules in the wild are
/// authored on a single line, and tolerating multi-line splits would
/// risk eating real content).
fn skip_ws(bytes: &[u8], i: usize) -> Option<usize> {
    let mut i = i;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    Some(i)
}

/// Advance past spaces, tabs, and newlines. Used by the @media matcher
/// where the at-rule's outer brace conventionally spans lines.
fn skip_ws_nl(bytes: &[u8], i: usize) -> Option<usize> {
    let mut i = i;
    while i < bytes.len()
        && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r')
    {
        i += 1;
    }
    Some(i)
}

/// Rewrite shadow-idiom selectors for LIGHT-DOM emission. The runtime wraps a
/// light component's styles in `@scope ([data-component="tag"]) to
/// ([data-component])` — a donut that stops at nested component boundaries.
/// Model A: the component's top template element is the root (class `sail-root`),
/// the host a transparent pass-through, so inside the sheet:
///   `:host`        → `.sail-root`             (the root IS that element)
///   `:host(<sel>)` → `:scope<sel> .sail-root` (host state → the root element)
///   `::slotted(<sel>)` → `slot > <sel>`  (projected children stay real
///                                         children inside the <slot> wrapper,
///                                         which renders display:contents)
/// Everything else passes through untouched; the transform is purely textual
/// and boundary-safe (ASCII scanning, balanced-paren unwrap).
pub fn light_dom_selectors(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if css[i..].starts_with(":host") {
            let after = i + ":host".len();
            if bytes.get(after) == Some(&b'(') {
                if let Some(close) = balanced_paren_end(bytes, after) {
                    // `:host(<state>)` — the state (a reflected prop like
                    // `[variant='solid']`) lives on the host; it styles the
                    // root element (`.sail-root`), a descendant of the host.
                    out.push_str(":scope");
                    out.push_str(&css[after + 1..close]);
                    out.push_str(" .sail-root");
                    i = close + 1;
                    continue;
                }
            }
            // Bare `:host` — the model-A root IS the component's top template
            // element, carrying the `sail-root` class.
            out.push_str(".sail-root");
            i = after;
            continue;
        }
        if css[i..].starts_with("::slotted(") {
            let open = i + "::slotted".len();
            if let Some(close) = balanced_paren_end(bytes, open) {
                out.push_str("slot > ");
                out.push_str(&css[open + 1..close]);
                i = close + 1;
                continue;
            }
        }
        // Advance one full UTF-8 char (comments can carry multibyte text).
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        out.push_str(&css[i..n]);
        i = n;
    }
    out
}

/// Index of the `)` matching the `(` at `open`, honoring nesting.
fn balanced_paren_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_canonical_form() {
        let css = ":host { display: block; }\n  :host([hidden]) { display: none; }\nbutton { color: red; }\n";
        let out = strip_shared_rules(css);
        assert!(!out.contains(":host([hidden])"), "rule not stripped:\n{out}");
        assert!(out.contains(":host { display: block; }"));
        assert!(out.contains("button { color: red; }"));
    }

    #[test]
    fn strips_important_form() {
        let css = ":host([hidden]) { display: none !important; }\n";
        assert_eq!(strip_shared_rules(css), "");
    }

    #[test]
    fn strips_extra_whitespace() {
        let css = ":host([hidden])   {  display:   none ; }\n";
        assert_eq!(strip_shared_rules(css), "");
    }

    #[test]
    fn leaves_unrelated_rules() {
        let css = ":host([hidden][open]) { display: flex; }\n";
        assert_eq!(strip_shared_rules(css), css);
    }

    #[test]
    fn leaves_extended_body() {
        let css = ":host([hidden]) { display: none; visibility: hidden; }\n";
        assert_eq!(strip_shared_rules(css), css);
    }

    #[test]
    fn strips_reduced_motion_rule() {
        let css = "  @media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }\n";
        assert_eq!(strip_shared_rules(css), "");
    }

    #[test]
    fn strips_reduced_motion_multiline() {
        let css = "  @media (prefers-reduced-motion: reduce) {\n    * { transition: none !important; animation: none !important; }\n  }\n";
        assert_eq!(strip_shared_rules(css), "");
    }

    #[test]
    fn light_dom_rewrites_host_and_slotted() {
        // Model A: `:host` → the root element `.sail-root`; a host state
        // `:host(<sel>)` reads the host attr and styles the root element.
        let css = ":host { display: inline-flex; }\n:host([tone='primary']) .badge { color: red; }\n:host(:not([size])) { --h: 1px; }\n::slotted([slot='leading']) { flex: none; }\n.badge { border: 0; }";
        let out = light_dom_selectors(css);
        assert!(out.contains(".sail-root { display: inline-flex; }"), "{out}");
        assert!(out.contains(":scope[tone='primary'] .sail-root .badge"), "{out}");
        assert!(out.contains(":scope:not([size]) .sail-root"), "{out}");
        assert!(out.contains("slot > [slot='leading'] { flex: none; }"), "{out}");
        assert!(out.contains(".badge { border: 0; }"), "{out}");
        assert!(!out.contains(":host"), "no :host survives: {out}");
    }

    #[test]
    fn strips_reduced_motion_swapped_order() {
        let css = "@media (prefers-reduced-motion: reduce) { * { animation: none !important; transition: none !important } }\n";
        assert_eq!(strip_shared_rules(css), "");
    }

    #[test]
    fn leaves_richer_reduced_motion() {
        let css = "@media (prefers-reduced-motion: reduce) { .panel { transition: none; transform: none !important; } }\n";
        assert_eq!(strip_shared_rules(css), css);
    }

    #[test]
    fn idempotent() {
        let css = ":host { display: block; }\n:host([hidden]) { display: none; }\n";
        let once = strip_shared_rules(css);
        let twice = strip_shared_rules(&once);
        assert_eq!(once, twice);
    }
}
