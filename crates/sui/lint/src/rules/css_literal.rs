//! `sui-css-literal` — hardcoded color values inside a `.sui` `<style>`
//! block must be `var(--token)`, never a raw hex / rgb / hsl / named
//! color.
//!
//! Why this rule exists:
//! - The css_token rule (sibling) catches typos *inside* `var(--xyz)`
//!   references. It does not stop someone from skipping the registry
//!   entirely and writing `color: #fff`.
//! - The audit found `var(--card, #fff)` fallback patterns everywhere.
//!   Those literals are dead weight that mask real token-load-order
//!   bugs: if the token fails to resolve, the fallback flashes a
//!   light surface in a dark UI instead of throwing.
//! - When every author has to think twice before reaching for `#abc`,
//!   the design system stops being optional.
//!
//! Scope (intentionally narrow at introduction):
//! - Only the color-bearing declarations: `color`, `background`,
//!   `background-color`, `border-color`, `outline-color`, `fill`,
//!   `stroke`. (Border shorthand still catches its color slot when
//!   `border: 1px solid #fff` is written.)
//! - Reject: `#abc[def[01]]?`, `rgb(...)`, `rgba(...)`, `hsl(...)`,
//!   `hsla(...)`, `oklch(...)`, `oklab(...)`, `color(...)`, and the
//!   ~150 named CSS colors.
//! - Allow: `transparent`, `currentColor`, `inherit`, `initial`,
//!   `unset`, `none`, and anything inside a `var(--token)` reference
//!   (the css_token rule handles those — but a `var(--x, RAW)`
//!   fallback IS checked, since the fallback bypasses the token).
//!
//! Out of scope for v1 (deferred):
//! - Spacing literals (`padding: 12px` should be `var(--space-3)`).
//!   Primitive size-matrix declarations legitimately use px, so the
//!   rule needs more context to avoid false positives.
//! - Typography literals (`font-size: 13px`, etc.).
//! - linear-gradient(...) color stops.
//!
//! Severity is `Error`: `sui-compiler::check::css_token` promotes
//! every error-severity finding to a build failure. The rule was
//! introduced alongside a one-shot migration that cleared the prior
//! drift (80 violations across 14 primitives — `var(--card, #fff)`
//! fallback patterns and a handful of raw color-mix sources). After
//! the cleanup, Error became safe to enable without breaking compile.

use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-css-literal";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for decl in scan_color_declarations(block.content) {
            let value_no_var = strip_var_references(decl.value);
            for literal in find_color_literals(&value_no_var) {
                let abs_offset = block.content_offset + decl.value_offset + literal.offset;
                let (line, col) = byte_offset_to_line_col(source, abs_offset);
                let message = format!(
                    "raw color `{}` in `{}` — use a design-system token (`var(--…)`)",
                    literal.text, decl.property
                );
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Error,
                    message,
                    suggest: Some(
                        "Pick the closest tone token from \
                         crates/sui/compiler/tokens.json — typically \
                         --foreground, --background, --muted-foreground, \
                         --border, --destructive, etc. Custom color \
                         channels should be declared locally as \
                         `--my-channel: …;` derived from existing tokens \
                         (e.g. via color-mix(in oklch, var(--primary) 50%, transparent))."
                            .to_string(),
                    ),
                });
            }
        }
    }
    findings
}

/// One `<style>` block.
struct StyleBlock<'a> {
    content_offset: usize,
    content: &'a str,
}

/// A single themable declaration inside a style block: `property: value`.
struct ColorDecl<'a> {
    property: &'a str,
    /// Byte offset of `value` (the first char after the colon, skipping
    /// whitespace) into the style block's content.
    value_offset: usize,
    /// The value text, terminated at the next `;` or `}`.
    value: &'a str,
}

/// One color literal found inside a value.
struct ColorLiteral<'a> {
    /// Byte offset of the literal into the *value*.
    offset: usize,
    text: &'a str,
}

/// Properties whose value is treated as a color and validated.
const COLOR_PROPERTIES: &[&str] = &[
    "color",
    "background",
    "background-color",
    "border-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "outline-color",
    "fill",
    "stroke",
    // Border shorthand: catches `border: 1px solid #fff` as a value
    // string. The rule's color-literal scanner is property-agnostic
    // once invoked — it scans the entire value for hex/rgb/etc., so
    // `border` works.
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
    "box-shadow",
    "text-shadow",
];

fn is_color_property(name: &str) -> bool {
    COLOR_PROPERTIES.contains(&name)
}

/// Walk `<style>...</style>` blocks. Mirror of the helper in
/// `css_token.rs` — kept duplicated rather than introducing a shared
/// AST module while the two rules' position-mapping needs are still
/// in flux.
fn scan_style_blocks(source: &str) -> Vec<StyleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while let Some(rel) = find_subslice(&bytes[i..], b"<style") {
        let tag_start = i + rel;
        let after_tag = tag_start + b"<style".len();
        if !matches!(
            bytes.get(after_tag).copied(),
            Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        ) {
            i = after_tag;
            continue;
        }
        let Some(open_end_rel) = find_subslice(&bytes[after_tag..], b">") else {
            break;
        };
        if open_end_rel > 0 && bytes.get(after_tag + open_end_rel - 1) == Some(&b'/') {
            i = after_tag + open_end_rel + 1;
            continue;
        }
        let content_start = after_tag + open_end_rel + 1;
        let Some(close_rel) = find_subslice(&bytes[content_start..], b"</style>") else {
            break;
        };
        let content_end = content_start + close_rel;
        out.push(StyleBlock {
            content_offset: content_start,
            content: &source[content_start..content_end],
        });
        i = content_end + b"</style>".len();
    }
    out
}

/// Find every declaration whose property is in `COLOR_PROPERTIES`.
/// Cheap scanner: walk character by character, recognize a property
/// name by being followed by `:`. Skip inside string literals.
fn scan_color_declarations(content: &str) -> Vec<ColorDecl<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip whitespace and `;`/`{`/`}` separators.
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || matches!(bytes[i], b';' | b'{' | b'}')) {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Skip CSS comments.
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        // Read identifier (potential property name).
        let name_start = i;
        while i < bytes.len() && is_ident_byte(bytes[i]) {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = &content[name_start..i];
        // Whitespace then ':'.
        let mut k = i;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= bytes.len() || bytes[k] != b':' {
            // Not a declaration; could be a selector or function name.
            // Skip to next separator.
            while i < bytes.len() && !matches!(bytes[i], b';' | b'{' | b'}') {
                i += 1;
            }
            continue;
        }
        // Skip the colon + whitespace.
        let mut v = k + 1;
        while v < bytes.len() && bytes[v].is_ascii_whitespace() {
            v += 1;
        }
        // Read value to next `;` or `}` (balancing parens so `;` inside
        // a `var(...)` fallback doesn't terminate early).
        let value_start = v;
        let mut depth = 0i32;
        while v < bytes.len() {
            match bytes[v] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b';' | b'}' if depth <= 0 => break,
                _ => {}
            }
            v += 1;
        }
        let value = content[value_start..v].trim_end();
        if is_color_property(name) {
            out.push(ColorDecl {
                property: name,
                value_offset: value_start,
                value,
            });
        }
        i = v;
    }
    out
}

/// Replace every `var(...)` expression with same-length spaces so the
/// rest of the scanner can ignore it but byte offsets are preserved.
fn strip_var_references(value: &str) -> String {
    let mut out: Vec<u8> = value.as_bytes().to_vec();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if &bytes[i..i + 4] == b"var(" {
            // Within var(...), the css_token rule handles --name typos.
            // The FALLBACK after the comma can hold a literal — we DO
            // want to catch that. So we strip the leading `var(--name`
            // up to the first comma, leaving the fallback exposed to
            // the literal scanner. If no comma, strip the whole expr.
            let start = i;
            let mut depth = 0i32;
            let mut comma_at: Option<usize> = None;
            let mut j = i;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            j += 1;
                            break;
                        }
                    }
                    b',' if depth == 1 && comma_at.is_none() => comma_at = Some(j),
                    _ => {}
                }
                j += 1;
            }
            let end = j;
            match comma_at {
                Some(c) => {
                    // Blank out `var(--name,` and the trailing `)` so the
                    // fallback text reads as bare value; positions stay aligned.
                    out[start..=c].fill(b' ');
                    if end > 0 && end - 1 < out.len() && out[end - 1] == b')' {
                        out[end - 1] = b' ';
                    }
                }
                None => {
                    out[start..end].fill(b' ');
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

/// Find every color literal in a value string. Returns offsets relative
/// to the value's start.
fn find_color_literals(value: &str) -> Vec<ColorLiteral<'_>> {
    let mut out = Vec::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Hex literal: `#` followed by 3, 4, 6, or 8 hex digits.
        if bytes[i] == b'#' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            let len = j - i - 1;
            if matches!(len, 3 | 4 | 6 | 8) {
                out.push(ColorLiteral {
                    offset: i,
                    text: &value[i..j],
                });
            }
            i = j.max(i + 1);
            continue;
        }
        // Function color (`rgb(`, `rgba(`, `hsl(`, `hsla(`, `oklch(`,
        // `oklab(`, `color(`).
        if let Some(end) = try_match_color_fn(bytes, i) {
            out.push(ColorLiteral {
                offset: i,
                text: &value[i..end],
            });
            i = end;
            continue;
        }
        // Named color (`red`, `white`, etc.). Only enter when bytes[i]
        // is an identifier start at a word boundary — otherwise we'd
        // loop forever on whitespace/punct after a non-ident neighbor.
        let at_word_boundary = i == 0 || !is_ident_byte(bytes[i - 1]);
        if at_word_boundary && is_ident_byte(bytes[i]) {
            let name_start = i;
            while i < bytes.len() && is_ident_byte(bytes[i]) {
                i += 1;
            }
            let name = &value[name_start..i];
            if name.len() >= 3 && color_names().contains(name) {
                out.push(ColorLiteral {
                    offset: name_start,
                    text: name,
                });
            }
            continue;
        }
        i += 1;
    }
    out
}

fn try_match_color_fn(bytes: &[u8], i: usize) -> Option<usize> {
    let starts = [
        &b"rgba("[..],
        &b"rgb("[..],
        &b"hsla("[..],
        &b"hsl("[..],
        &b"oklch("[..],
        &b"oklab("[..],
        &b"color("[..],
    ];
    let head = bytes.get(i..)?;
    for s in starts {
        if head.len() >= s.len() && head[..s.len()] == *s {
            // Find matching `)`.
            let mut depth = 0i32;
            let mut j = i + s.len() - 1;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(j + 1);
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            return None;
        }
    }
    None
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|&i| &haystack[i..i + needle.len()] == needle)
}

/// Set of CSS color keywords whose use as a color value is forbidden
/// by this rule. We treat `transparent`, `currentColor`, `inherit`,
/// `initial`, `unset`, and `none` as allowed because they are
/// semantically tokenless (no equivalent design-system token).
fn color_names() -> &'static HashSet<&'static str> {
    static CACHE: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        const NAMES: &[&str] = &[
            "aliceblue", "antiquewhite", "aqua", "aquamarine", "azure",
            "beige", "bisque", "black", "blanchedalmond", "blue",
            "blueviolet", "brown", "burlywood", "cadetblue", "chartreuse",
            "chocolate", "coral", "cornflowerblue", "cornsilk", "crimson",
            "cyan", "darkblue", "darkcyan", "darkgoldenrod", "darkgray",
            "darkgreen", "darkgrey", "darkkhaki", "darkmagenta",
            "darkolivegreen", "darkorange", "darkorchid", "darkred",
            "darksalmon", "darkseagreen", "darkslateblue", "darkslategray",
            "darkslategrey", "darkturquoise", "darkviolet", "deeppink",
            "deepskyblue", "dimgray", "dimgrey", "dodgerblue", "firebrick",
            "floralwhite", "forestgreen", "fuchsia", "gainsboro",
            "ghostwhite", "gold", "goldenrod", "gray", "green",
            "greenyellow", "grey", "honeydew", "hotpink", "indianred",
            "indigo", "ivory", "khaki", "lavender", "lavenderblush",
            "lawngreen", "lemonchiffon", "lightblue", "lightcoral",
            "lightcyan", "lightgoldenrodyellow", "lightgray", "lightgreen",
            "lightgrey", "lightpink", "lightsalmon", "lightseagreen",
            "lightskyblue", "lightslategray", "lightslategrey",
            "lightsteelblue", "lightyellow", "lime", "limegreen", "linen",
            "magenta", "maroon", "mediumaquamarine", "mediumblue",
            "mediumorchid", "mediumpurple", "mediumseagreen",
            "mediumslateblue", "mediumspringgreen", "mediumturquoise",
            "mediumvioletred", "midnightblue", "mintcream", "mistyrose",
            "moccasin", "navajowhite", "navy", "oldlace", "olive",
            "olivedrab", "orange", "orangered", "orchid", "palegoldenrod",
            "palegreen", "paleturquoise", "palevioletred", "papayawhip",
            "peachpuff", "peru", "pink", "plum", "powderblue", "purple",
            "rebeccapurple", "red", "rosybrown", "royalblue", "saddlebrown",
            "salmon", "sandybrown", "seagreen", "seashell", "sienna",
            "silver", "skyblue", "slateblue", "slategray", "slategrey",
            "snow", "springgreen", "steelblue", "tan", "teal", "thistle",
            "tomato", "turquoise", "violet", "wheat", "white", "whitesmoke",
            "yellow", "yellowgreen",
        ];
        NAMES.iter().copied().collect()
    })
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

    #[test]
    fn hex_color_in_color_property_reports() {
        let src = "<style>:host { color: #fff; }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("#fff"), "{}", f[0].message);
    }

    #[test]
    fn six_digit_hex_reports() {
        let src = "<style>:host { background: #aabbcc; }</style>";
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn token_reference_passes() {
        let src = "<style>:host { color: var(--foreground); }</style>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn var_fallback_with_literal_reports() {
        // The whole point of the rule: catch `var(--x, #fff)` slop.
        let src = "<style>:host { color: var(--card, #fff); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("#fff"));
    }

    #[test]
    fn rgb_function_reports() {
        let src = "<style>:host { background-color: rgb(255, 0, 0); }</style>";
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn named_color_reports() {
        let src = "<style>:host { color: red; }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("red"));
    }

    #[test]
    fn transparent_keyword_passes() {
        let src = "<style>:host { background: transparent; }</style>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn current_color_passes() {
        let src = "<style>:host { fill: currentColor; }</style>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn spacing_property_is_ignored() {
        // Out of scope for v1.
        let src = "<style>:host { padding: 12px; }</style>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn border_shorthand_with_hex_reports() {
        let src = "<style>:host { border: 1px solid #fff; }</style>";
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn box_shadow_with_hex_reports() {
        let src = "<style>:host { box-shadow: 0 1px 2px #000; }</style>";
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn color_mix_with_token_only_passes() {
        // color-mix is a function, but its arguments are var()s — the
        // var-stripping pass blanks the inner colors, so the rule sees
        // a bare `color-mix(in oklch, , 50%, )` and emits nothing.
        let src = "<style>:host { background: color-mix(in oklch, var(--primary), transparent 50%); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn ident_name_inside_var_does_not_match_color_name() {
        // `var(--purple-100)` contains `purple` as a substring but we
        // strip var()s before scanning. Sanity check.
        let src = "<style>:host { color: var(--purple-tint); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn comment_block_is_skipped() {
        let src = "<style>/* color: red; */ :host { color: var(--foreground); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn position_points_at_literal() {
        let src = "<style>\n  :host { color: #fff; }\n</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
        // `  :host { color: ` is 17 chars; `#` is col 18.
        assert_eq!(f[0].col, 18);
    }
}
