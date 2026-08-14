//! `sail-height-chain` — overflow scroll/clip without the matching min-size.
//!
//! Classic flex collapse: a child with `overflow:auto|hidden` that should
//! shrink inside a flex parent needs a zero min-size on the scroll axis.
//! Without it the box refuses to shrink below content size and the height
//! chain breaks (parent collapses or clips silent).
//!
//! ## Axis-aware contract
//!
//! | Overflow form                         | Required zero min-size   |
//! |---------------------------------------|--------------------------|
//! | `overflowY(…)` / `overflow-y-*`       | `minH(0)` / `min-h-0`    |
//! | `overflowX(…)` / `overflow-x-*`       | `minW(0)` / `min-w-0`    |
//! | bare `overflow(…)` / `overflow-*`     | either minH(0) or minW(0)|
//!
//! ## Viewport clip roots (skipped)
//!
//! Shells that already pin a definite viewport height (`h(screen)` /
//! `minH(screen)`) and only use `overflow(hidden|clip)` as an outer clip are
//! not flex-shrink scroll parents — flagging them is noise. Scroll owners
//! (`overflow(auto|scroll)`) are never skipped.
//!
//! **Severity:** `Warn` by default; **`Error`** when the package sets
//! `"layout": "error"` (studio strict profile) so check/dev fail closed.

use std::path::Path;

use crate::config;
use crate::report::{Finding, Severity};
use crate::rules::layout::{self, OpenTag};

pub const RULE_ID: &str = "sail-height-chain";

const CLIP_MODES: &[&str] = &["auto", "hidden", "scroll", "clip"];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let severity = if config::layout_is_error_for(path) {
        Severity::Error
    } else {
        Severity::Warn
    };
    let mut findings = Vec::new();
    for t in layout::scan_any_open_tags(source) {
        if let Some(f) = check_tag(path, source, &t, severity) {
            findings.push(f);
        }
    }
    findings
}

fn check_tag(path: &Path, source: &str, t: &OpenTag, severity: Severity) -> Option<Finding> {
    let sx = t
        .attrs
        .iter()
        .find(|(n, _)| n == "style")
        .map(|(_, v)| v.as_str())?;

    let axes = detect_overflow_axes(sx)?;
    if is_viewport_clip_root(sx, &axes) {
        return None;
    }
    if axes_satisfied(sx, &axes) {
        return None;
    }

    let (line, col) = layout::offset_to_line_col(source, t.start);
    let need = required_min_hint(&axes);
    Some(Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity,
        message: format!(
            "<{}> sets overflow without {need} — flex children that scroll or \
             clip need a zero min-size on the overflow axis or the height \
             chain collapses (parent measures content size; overflow never \
             engages).",
            t.tag
        ),
        suggest: Some(format!(
            "Add {need} on the same concern chain, e.g. \
             `style({{ overflow({{ y: auto }}).size({{ min-h: 0 }}).grow(1) }})` (Y axis) or \
             `style({{ overflow(auto).size({{ min-w: 0 }}) }})` (row / bare overflow)."
        )),
    })
}

/// Which zero min-sizes the overflow form(s) require.
#[derive(Debug, Clone, Copy, Default)]
struct OverflowAxes {
    /// Bare `overflow(…)` / unprefixed token — either minH or minW suffices.
    bare: bool,
    /// `overflowY` / `overflow-y-*` — needs minH(0).
    y: bool,
    /// `overflowX` / `overflow-x-*` — needs minW(0).
    x: bool,
}

impl OverflowAxes {
    fn any(self) -> bool {
        self.bare || self.y || self.x
    }

    fn only_clip_not_scroll(self, sx: &str) -> bool {
        // Used for viewport-root skip: only hidden/clip, no auto/scroll.
        if !self.any() {
            return false;
        }
        !has_scroll_overflow(sx)
    }
}

fn detect_overflow_axes(sx: &str) -> Option<OverflowAxes> {
    let mut axes = OverflowAxes::default();

    // Style API: overflowY(auto), overflowX(hidden), overflow(scroll), …
    // Product object: overflow({ y: auto }) / overflow({ x: hidden }) / overflow({ all: clip }).
    for mode in CLIP_MODES {
        if style_call(sx, "overflowY", mode)
            || style_call(sx, "overflowy", mode)
            || overflow_slot(sx, "y", mode)
        {
            axes.y = true;
        }
        if style_call(sx, "overflowX", mode)
            || style_call(sx, "overflowx", mode)
            || overflow_slot(sx, "x", mode)
        {
            axes.x = true;
        }
        if style_call(sx, "overflow", mode) || overflow_slot(sx, "all", mode) {
            axes.bare = true;
        }
    }

    // Token atoms with word-ish boundaries (no over-match of overflow-hidden-sm).
    for mode in CLIP_MODES {
        if token_atom(sx, &format!("overflow-y-{mode}"))
            || token_atom(sx, &format!("overflowy{mode}"))
        {
            axes.y = true;
        }
        if token_atom(sx, &format!("overflow-x-{mode}"))
            || token_atom(sx, &format!("overflowx{mode}"))
        {
            axes.x = true;
        }
        if token_atom(sx, &format!("overflow-{mode}")) {
            axes.bare = true;
        }
    }

    if axes.any() {
        Some(axes)
    } else {
        None
    }
}

fn has_scroll_overflow(sx: &str) -> bool {
    for mode in ["auto", "scroll"] {
        if style_call(sx, "overflow", mode)
            || style_call(sx, "overflowY", mode)
            || style_call(sx, "overflowy", mode)
            || style_call(sx, "overflowX", mode)
            || style_call(sx, "overflowx", mode)
            || overflow_slot(sx, "y", mode)
            || overflow_slot(sx, "x", mode)
            || overflow_slot(sx, "all", mode)
            || token_atom(sx, &format!("overflow-{mode}"))
            || token_atom(sx, &format!("overflow-y-{mode}"))
            || token_atom(sx, &format!("overflow-x-{mode}"))
        {
            return true;
        }
    }
    false
}

/// Viewport-pinned outer clip shells (e.g. studio chrome `minH(screen).h(screen).overflow(hidden)`).
fn is_viewport_clip_root(sx: &str, axes: &OverflowAxes) -> bool {
    if !axes.only_clip_not_scroll(sx) {
        return false;
    }
    // Explicit viewport / full-shell height — not a flex-shrink scroll parent.
    has_style_arg(sx, "h", "screen")
        || has_style_arg(sx, "minH", "screen")
        || has_style_arg(sx, "minh", "screen")
        || size_slot(sx, "h", "screen")
        || size_slot(sx, "min-h", "screen")
        || (has_style_arg(sx, "h", "full")
            && (has_style_arg(sx, "minH", "screen") || has_style_arg(sx, "minh", "screen")))
        || (size_slot(sx, "h", "full") && size_slot(sx, "min-h", "screen"))
        || token_atom(sx, "h-screen")
        || token_atom(sx, "min-h-screen")
}

fn axes_satisfied(sx: &str, axes: &OverflowAxes) -> bool {
    let has_h = has_zero_min_h(sx);
    let has_w = has_zero_min_w(sx);

    if axes.y && !has_h {
        return false;
    }
    if axes.x && !has_w {
        return false;
    }
    if axes.bare && !(has_h || has_w) {
        return false;
    }
    true
}

fn required_min_hint(axes: &OverflowAxes) -> &'static str {
    match (axes.y, axes.x, axes.bare) {
        (true, false, false) => "minH(0)",
        (false, true, false) => "minW(0)",
        (true, true, _) => "minH(0) and minW(0)",
        _ => "minH(0) or minW(0)",
    }
}

fn has_zero_min_h(sx: &str) -> bool {
    style_call(sx, "minH", "0")
        || style_call(sx, "minh", "0")
        || style_call(sx, "minHeight", "0")
        || size_slot(sx, "min-h", "0")
        || token_atom(sx, "min-h-0")
        || token_atom(sx, "minh0")
        || sx.contains("s-minh0")
        || contains_ci(sx, "minH(bind")
        || contains_ci(sx, "minh(bind")
        || size_slot(sx, "min-h", "bind")
}

fn has_zero_min_w(sx: &str) -> bool {
    style_call(sx, "minW", "0")
        || style_call(sx, "minw", "0")
        || style_call(sx, "minWidth", "0")
        || size_slot(sx, "min-w", "0")
        || token_atom(sx, "min-w-0")
        || token_atom(sx, "minw0")
        || sx.contains("s-minw0")
        || contains_ci(sx, "minW(bind")
        || contains_ci(sx, "minw(bind")
        || size_slot(sx, "min-w", "bind")
}

/// `name(arg)` Style-API call (case-insensitive name/arg).
fn style_call(sx: &str, name: &str, arg: &str) -> bool {
    let pat = format!("{name}({arg})");
    contains_ci(sx, &pat)
}

fn has_style_arg(sx: &str, name: &str, arg: &str) -> bool {
    style_call(sx, name, arg)
}

/// Product `overflow({ y: auto })` / `overflow({ x: hidden })` / `overflow({ all: clip })`.
fn overflow_slot(sx: &str, slot: &str, mode: &str) -> bool {
    for body in overflow_object_bodies(sx) {
        if object_slot_eq(body, slot, mode) {
            return true;
        }
    }
    false
}

/// Product `size({ min-h: 0 })` / `size({ h: screen })` slot match.
/// `min-h: screen` must not count as `h: screen`.
fn size_slot(sx: &str, slot: &str, val: &str) -> bool {
    object_slot_eq(sx, slot, val)
}

fn overflow_object_bodies(sx: &str) -> Vec<&str> {
    let lower = sx.to_ascii_lowercase();
    let bytes = sx.as_bytes();
    let mut out = Vec::new();
    let mut search = 0usize;
    while let Some(rel) = lower[search..].find("overflow({") {
        let open = search + rel + "overflow(".len();
        if open >= bytes.len() || bytes[open] != b'{' {
            search += rel + 1;
            continue;
        }
        let mut depth = 0i32;
        let mut i = open;
        let mut end = None;
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if let Some(close) = end {
            out.push(&sx[open..=close]);
            search = close + 1;
        } else {
            break;
        }
    }
    out
}

/// `{ slot: val }` / `slot: val` with a key boundary so `min-h` ≠ `h`.
fn object_slot_eq(hay: &str, slot: &str, val: &str) -> bool {
    let hay_l = hay.to_ascii_lowercase();
    let slot_l = slot.to_ascii_lowercase();
    let val_l = val.to_ascii_lowercase();
    let spaced = format!("{slot_l}: {val_l}");
    let tight = format!("{slot_l}:{val_l}");
    for pat in [spaced.as_str(), tight.as_str()] {
        let mut from = 0usize;
        while let Some(rel) = hay_l[from..].find(pat) {
            let i = from + rel;
            let before_ok = i == 0 || {
                let b = hay_l.as_bytes()[i - 1];
                b == b'{' || b == b',' || b.is_ascii_whitespace()
            };
            let after = i + pat.len();
            let after_ok = after >= hay_l.len() || {
                let b = hay_l.as_bytes()[after];
                b == b'}' || b == b',' || b == b'(' || b.is_ascii_whitespace()
            };
            if before_ok && after_ok {
                return true;
            }
            from = i + 1;
        }
    }
    false
}

/// Token atom match: `atom` must not be a substring of a longer identifier-like run.
/// So `overflow-hidden` matches, but `overflow-hidden-sm` does not.
fn token_atom(sx: &str, atom: &str) -> bool {
    let hay = sx.as_bytes();
    let needle = atom.as_bytes();
    if needle.is_empty() || hay.len() < needle.len() {
        return false;
    }
    let hay_l: Vec<u8> = hay.iter().map(|b| b.to_ascii_lowercase()).collect();
    let needle_l: Vec<u8> = needle.iter().map(|b| b.to_ascii_lowercase()).collect();
    let n = needle_l.len();
    for i in 0..=hay_l.len() - n {
        if hay_l[i..i + n] != needle_l[..] {
            continue;
        }
        let before_ok = i == 0 || !is_atom_char(hay_l[i - 1]);
        let after_ok = i + n >= hay_l.len() || !is_atom_char(hay_l[i + n]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn is_atom_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    fn lint(src: &str) -> Vec<Finding> {
        // Synthetic path: no sail.config on walk → Warn.
        check(Path::new("/tmp/no-config/box.sail"), src)
    }

    #[test]
    fn overflow_auto_without_min_warns() {
        let src = r#"<column style({ overflow(auto).grow(1) })></column>"#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
    }

    #[test]
    fn overflow_with_minh0_passes() {
        let src =
            r#"<column style({ overflow({ y: auto }).size({ min-h: 0 }).grow(1) })></column>"#;
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    #[test]
    fn no_overflow_passes() {
        let src = r#"<column style({ grow(1).bg(surface) })></column>"#;
        assert!(lint(src).is_empty());
    }

    #[test]
    fn overflow_y_requires_min_h_not_only_min_w() {
        let src = r#"<column style({ overflow({ y: auto }).size({ min-w: 0 }) })></column>"#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "minW alone must not satisfy overflowY: {f:?}");
        assert!(f[0].message.contains("minH(0)"), "{}", f[0].message);
    }

    #[test]
    fn overflow_x_requires_min_w_not_only_min_h() {
        let src = r#"<row style({ overflow({ x: auto }).size({ min-h: 0 }) })></row>"#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "minH alone must not satisfy overflowX: {f:?}");
        assert!(f[0].message.contains("minW(0)"), "{}", f[0].message);
    }

    #[test]
    fn bare_overflow_accepts_either_min() {
        let a = r#"<div style({ overflow(auto).size({ min-w: 0 }) })></div>"#;
        let b = r#"<div style({ overflow(hidden).size({ min-h: 0 }) })></div>"#;
        assert!(lint(a).is_empty(), "{:?}", lint(a));
        assert!(lint(b).is_empty(), "{:?}", lint(b));
    }

    #[test]
    fn viewport_clip_root_with_screen_height_skipped() {
        // Studio chrome shell: definite viewport size + outer clip only.
        let src = r#"<row style({ size({ min-h: screen, h: screen, w: full }).overflow(hidden) })></row>"#;
        assert!(
            lint(src).is_empty(),
            "viewport clip root must not fire: {:?}",
            lint(src)
        );
    }

    #[test]
    fn viewport_sized_but_scroll_still_flagged() {
        // Scroll owners always need min-size even if size({ h: screen }).
        let src = r#"<column style({ size({ h: screen }).overflow({ y: auto }) })></column>"#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "scroll owner must still flag: {f:?}");
    }

    #[test]
    fn token_overflow_hidden_sm_does_not_over_match() {
        // Word-boundary: longer atom must not count as overflow-hidden.
        let src = r#"<div style="overflow-hidden-sm grow"></div>"#;
        assert!(
            lint(src).is_empty(),
            "must not treat overflow-hidden-sm as overflow-hidden: {:?}",
            lint(src)
        );
    }

    #[test]
    fn token_overflow_hidden_matches() {
        let src = r#"<div style="overflow-hidden grow"></div>"#;
        assert_eq!(lint(src).len(), 1, "{:?}", lint(src));
    }

    #[test]
    fn layout_error_profile_promotes_to_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("sail.config.json"),
            r#"{ "layout": "error" }"#,
        )
        .expect("config");
        let sail = dir.path().join("page.sail");
        {
            let mut f = std::fs::File::create(&sail).expect("create");
            write!(
                f,
                r#"<column style({{ overflow(auto).grow(1) }})></column>"#
            )
            .expect("write");
        }
        let src = std::fs::read_to_string(&sail).expect("read");
        let f = check(&sail, &src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(
            f[0].severity,
            Severity::Error,
            "layout:error must promote height-chain to Error"
        );
    }
}
