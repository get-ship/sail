//! SPEC-LAYOUT-DIAG-1 / S4.1 — red gate for experimental layout kinds.
//!
//! Kinds (Diagnostic.kind):
//! - `unintended-overflow` — child extents exceed container with no author
//!   opt-in (no scroll-area, no overflow setter, no truncate)
//! - `clipped-text` — measured text run cut without a truncate marker
//!
//! LAYOUT-5: `unintended-overflow` and `clipped-text` are default Error.
//! `sub-min-tap-target` stays experimental. Product entry:
//! [`sail_layout::check_preprocessed_with`] + [`sail_layout::CheckOptions`].
//!
//! Gate cases (job-spec-spike-s4-1-layout-kinds):
//! 1. Overflow without opt-in → finding with kind id
//! 2. Overflow with scroll-area / overflow setter → no finding
//! 3. Clipped text without truncate → finding (experimental/text_metrics path)
//! 4. Truncate present → no clipped-text
//! 5. Default check emits promoted kinds; tap stays experimental-off
//!
//! Implementer: do not weaken these assertions to green-wash a partial land.
//! Adversarial "must not fire" cases are part of the promotion law.

use sail_layout::{CheckOptions, Diagnostic, check_preprocessed, check_preprocessed_with};

const VP: (f32, f32) = (400.0, 300.0);

const KIND_UNINTENDED_OVERFLOW: &str = "unintended-overflow";
const KIND_CLIPPED_TEXT: &str = "clipped-text";

/// Fixed-height column, child taller + shrink-0, **no** overflow setter.
/// Solved geometry overflows on the main axis without author opt-in.
const OVERFLOW_NO_OPTIN: &str = r#"
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Same overflow geometry, author opted in via overflow-y:auto.
const OVERFLOW_WITH_OVERFLOW_Y_AUTO: &str = r#"
<div class="sail-col s-h20 s-wfull s-overflowyauto">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Same geometry, author opted in via overflow:scroll.
const OVERFLOW_WITH_OVERFLOW_SCROLL: &str = r#"
<div class="sail-col s-h20 s-wfull s-overflowscroll">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Scroll-area vocabulary opt-in (stdlib host tag + overflow axis).
const OVERFLOW_WITH_SCROLL_AREA: &str = r#"
<scroll-area class="s-h20 s-wfull s-overflowyauto">
  <div class="s-h40 s-shrink0">x</div>
</scroll-area>
"#;

/// Narrow box + long unbreakable text, overflow-x hidden, **no** truncate.
/// Text metrics path must report clipped-text under experimental+text_metrics.
const CLIPPED_TEXT_NO_TRUNCATE: &str = r#"
<div class="s-w16 s-overflowxhidden">Supercalifragilisticexpialidocious</div>
"#;

/// Same shape with truncate opt-in — must not fire clipped-text.
const CLIPPED_TEXT_WITH_TRUNCATE: &str = r#"
<div class="s-w16 s-truncate">Supercalifragilisticexpialidocious</div>
"#;

fn experimental() -> CheckOptions {
    CheckOptions::experimental()
}

fn kinds(diags: &[Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.kind).collect()
}

fn has_kind(diags: &[Diagnostic], kind: &str) -> bool {
    diags.iter().any(|d| d.kind == kind)
}

// ── 1. Overflow without opt-in → finding ────────────────────────────────────

#[test]
fn unintended_overflow_fires_without_opt_in_under_experimental() {
    let result =
        check_preprocessed_with(OVERFLOW_NO_OPTIN, VP, experimental()).expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflow without scroll-area / overflow setter / truncate must emit \
         `{KIND_UNINTENDED_OVERFLOW}` under experimental: kinds={:?} diags={:?}",
        kinds(&result.diagnostics),
        result.diagnostics
    );
    let d = result
        .diagnostics
        .iter()
        .find(|d| d.kind == KIND_UNINTENDED_OVERFLOW)
        .unwrap();
    assert!(
        d.span.0 < d.span.1 || d.span.1 > 0,
        "finding must carry a source span via layout mapping: {d:?}"
    );
}

// ── 2. Opt-in vocabulary → no unintended-overflow ───────────────────────────

#[test]
fn unintended_overflow_suppressed_when_overflow_y_auto() {
    let result = check_preprocessed_with(OVERFLOW_WITH_OVERFLOW_Y_AUTO, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflowY(auto) is an opt-in overflow setter — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_when_overflow_scroll() {
    let result = check_preprocessed_with(OVERFLOW_WITH_OVERFLOW_SCROLL, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflow(scroll) is an opt-in overflow setter — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_scroll_area() {
    let result = check_preprocessed_with(OVERFLOW_WITH_SCROLL_AREA, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "scroll-area is explicit scroll vocabulary — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

// ── 3–4. Clipped text ───────────────────────────────────────────────────────

#[test]
fn clipped_text_fires_without_truncate_under_experimental_text_metrics() {
    let result = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "measured text cut without truncate must emit `{KIND_CLIPPED_TEXT}` \
         under experimental+text_metrics: kinds={:?} diags={:?}",
        kinds(&result.diagnostics),
        result.diagnostics
    );
}

#[test]
fn clipped_text_suppressed_when_truncate_present() {
    let result = check_preprocessed_with(CLIPPED_TEXT_WITH_TRUNCATE, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "truncate() is the opt-in for ellipsis clipping — must NOT emit \
         `{KIND_CLIPPED_TEXT}`: {:?}",
        result.diagnostics
    );
}

// ── 5. Promoted kinds emit on default; tap stays experimental ────────────────

#[test]
fn default_check_emits_promoted_overflow() {
    let defaulted = check_preprocessed(OVERFLOW_NO_OPTIN, VP).expect("default check");
    let explicit = check_preprocessed_with(OVERFLOW_NO_OPTIN, VP, CheckOptions::default())
        .expect("explicit default");
    for (label, result) in [("default", &defaulted), ("explicit-default", &explicit)] {
        let hit = result
            .diagnostics
            .iter()
            .find(|d| d.kind == KIND_UNINTENDED_OVERFLOW)
            .unwrap_or_else(|| {
                panic!(
                    "{label}: promoted `{KIND_UNINTENDED_OVERFLOW}` must emit: {:?}",
                    result.diagnostics
                )
            });
        assert_eq!(
            hit.severity,
            sail_layout::Severity::Error,
            "{label}: promoted overflow is Error: {hit:?}"
        );
    }
}

#[test]
fn default_check_emits_promoted_clipped_text() {
    let result = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, CheckOptions::default())
        .expect("layout check");
    let hit = result
        .diagnostics
        .iter()
        .find(|d| d.kind == KIND_CLIPPED_TEXT)
        .unwrap_or_else(|| {
            panic!(
                "promoted `{KIND_CLIPPED_TEXT}` must emit on default: {:?}",
                result.diagnostics
            )
        });
    assert_eq!(
        hit.severity,
        sail_layout::Severity::Error,
        "promoted clipped-text is Error: {hit:?}"
    );
}

#[test]
fn text_metrics_false_suppresses_clipped_text_even_when_experimental() {
    // SPEC: if text measurement is noisy, gate clipped-text behind textMetrics.
    let opts = CheckOptions {
        experimental: true,
        text_metrics: Some(false),
    };
    let result = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, opts).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "text_metrics: false must suppress `{KIND_CLIPPED_TEXT}` even under experimental: {:?}",
        result.diagnostics
    );
}

// ── Additive unintended-overflow opt-in / containment ───────────────────────

/// Stdlib splice: overflow lives in `s-orientation*-overflow*`, not `s-overflow*`.
const OVERFLOW_ORIENTATION_PREFIXED: &str = r#"
<div class="sail-col s-h20 s-wfull s-orientationvertical-overflowYauto">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Inner column overflows; ancestor (not the parent) is the scroller.
const OVERFLOW_ANCESTOR_SCROLLER: &str = r#"
<div class="sail-col s-h20 s-wfull s-overflowyauto">
  <div class="sail-col s-h10">
    <div class="s-h40 s-shrink0">x</div>
  </div>
</div>
"#;

/// Auto-height text: document flow grows. Must not fire even if taffy
/// reports a 2× line-height parent/child split.
const AUTO_HEIGHT_TEXT: &str = r#"
<div class="sail-col s-wfull">
  <span class="s-fontSizexs">No results</span>
</div>
"#;

/// 1px taller than a definite parent — rounding, not a missing scroller.
const OVERFLOW_ONE_PX: &str = r#"
<div class="sail-col" style:h="80px">
  <div class="s-shrink0" style:h="81px">x</div>
</div>
"#;

/// Just above the kind epsilon — still a real overflow vehicle.
const OVERFLOW_THREE_PX: &str = r#"
<div class="sail-col" style:h="80px">
  <div class="s-shrink0" style:h="83px">x</div>
</div>
"#;

/// Out-of-flow child: author cannot overflow this box on screen.
const OVERFLOW_POSITION_FIXED: &str = r#"
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0 s-positionfixed">x</div>
</div>
"#;

const OVERFLOW_DISPLAY_CONTENTS: &str = r#"
<div class="sail-col s-h20 s-wfull s-displaycontents">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

const OVERFLOW_HIDDEN_ATTR: &str = r#"
<div class="sail-col s-h20 s-wfull" hidden="true">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// `<when {tablet}>` body is not on screen at the phone viewport.
const OVERFLOW_WHEN_TABLET: &str = r#"
<when {tablet}>
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0">x</div>
</div>
</when>
"#;

#[test]
fn unintended_overflow_suppressed_for_orientation_prefixed_overflow_class() {
    let result = check_preprocessed_with(OVERFLOW_ORIENTATION_PREFIXED, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "s-orientationvertical-overflowYauto is a scroll-area splice opt-in — \
         must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_when_ancestor_is_scroller() {
    let result = check_preprocessed_with(OVERFLOW_ANCESTOR_SCROLLER, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "child overflowing a pad wrapper inside an overflowY(auto) ancestor \
         is intended — must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_does_not_fire_on_auto_height_text() {
    let result =
        check_preprocessed_with(AUTO_HEIGHT_TEXT, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "auto-height text (2× line-height measurement lie) is document flow — \
         must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_ignores_one_px_rounding() {
    let result =
        check_preprocessed_with(OVERFLOW_ONE_PX, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "1px over a definite parent is rounding noise — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_just_above_kind_epsilon() {
    let result =
        check_preprocessed_with(OVERFLOW_THREE_PX, VP, experimental()).expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "3px over a definite parent with no opt-in must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_position_fixed_child() {
    let result =
        check_preprocessed_with(OVERFLOW_POSITION_FIXED, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "position:fixed is out of flow — must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_display_contents_host() {
    let result = check_preprocessed_with(OVERFLOW_DISPLAY_CONTENTS, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "display:contents is not an on-screen box — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_hidden_true_host() {
    let result =
        check_preprocessed_with(OVERFLOW_HIDDEN_ATTR, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "hidden=\"true\" is never shown — must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_skips_when_tablet_at_phone_viewport() {
    let phone = check_preprocessed_with(OVERFLOW_WHEN_TABLET, (390.0, 844.0), experimental())
        .expect("phone layout");
    assert!(
        !has_kind(&phone.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "<when {{tablet}}> at 390 is off-screen — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        phone.diagnostics
    );
}

/// Distant overflow(hidden) is clip, not a scroller.
const OVERFLOW_CLIP_SHELL_ANCESTOR: &str = r#"
<div class="sail-col s-wfull s-overflowhidden">
  <div class="sail-row sail-nowrap s-w20">
    <div class="sail-row sail-nowrap s-shrink0">
      <div class="s-w40 s-shrink0">Open</div>
      <div class="s-w40 s-shrink0">Thread</div>
      <div class="s-w40 s-shrink0">Copy</div>
    </div>
  </div>
</div>
"#;

/// nowrap shrink-0 cell inside a ticker host that already clips.
const OVERFLOW_TICKER_HOST: &str = r#"
<ticker class="s-w20 s-overflowhidden">
  <span class="s-minWmax s-whitespacenowrap s-shrink0">tokens · signals · a11y · hot reload</span>
</ticker>
"#;

/// In-flow nowrap row inflated by a fixed descendant (dropdown/dialog IR).
const OVERFLOW_OUT_OF_FLOW_DESCENDANT: &str = r#"
<div class="sail-row sail-nowrap s-w20">
  <div class="sail-row sail-nowrap s-shrink0">
    <div class="s-w10 s-shrink0">Undo</div>
    <div class="s-positionfixed s-w40 s-h20">dialog</div>
  </div>
</div>
"#;

/// Sibling scroller must not suppress a neighbor with no opt-in.
const OVERFLOW_SIBLING_SCROLLER: &str = r#"
<div class="sail-col s-wfull">
  <div class="sail-col s-h20 s-overflowyauto">
    <div class="s-h40 s-shrink0">ok</div>
  </div>
  <div class="sail-col s-h20">
    <div class="s-h40 s-shrink0">x</div>
  </div>
</div>
"#;

const OVERFLOW_HIDDEN_FALSE: &str = r#"
<div class="sail-col s-h20 s-wfull" hidden="false">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

const OVERFLOW_POSITION_RELATIVE: &str = r#"
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0 s-positionrelative">x</div>
</div>
"#;

/// Vertical scroller ancestor must not hide a horizontal leftover.
const OVERFLOW_VERTICAL_SCROLLER_HORIZONTAL_LEFTOVER: &str = r#"
<div class="sail-col s-h40 s-wfull s-overflowyauto">
  <div class="sail-row sail-nowrap s-w20">
    <div class="sail-row sail-nowrap s-shrink0">
      <div class="s-w40 s-shrink0">Open</div>
      <div class="s-w40 s-shrink0">Thread</div>
      <div class="s-w40 s-shrink0">Copy</div>
    </div>
  </div>
</div>
"#;

/// Leftover nowrap cluster plus a small absolute badge must stay red.
const OVERFLOW_LEFTOVER_WITH_ABSOLUTE_BADGE: &str = r#"
<div class="sail-row sail-nowrap s-w20">
  <div class="sail-row sail-nowrap s-shrink0">
    <div class="s-w40 s-shrink0">Open</div>
    <div class="s-w40 s-shrink0">Thread</div>
    <div class="s-w40 s-shrink0">Copy</div>
    <div class="s-positionabsolute s-w4 s-h4">·</div>
  </div>
</div>
"#;

/// Overflow inside a positioned shell is still on-screen.
const OVERFLOW_INSIDE_FIXED_SHELL: &str = r#"
<div class="s-positionfixed s-wfull">
  <div class="sail-col s-h20 s-wfull">
    <div class="s-h40 s-shrink0">x</div>
  </div>
</div>
"#;

#[test]
fn unintended_overflow_still_fires_when_hidden_is_false() {
    let result =
        check_preprocessed_with(OVERFLOW_HIDDEN_FALSE, VP, experimental()).expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "hidden=\"false\" is shown — no-opt-in overflow must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_for_position_relative() {
    let result = check_preprocessed_with(OVERFLOW_POSITION_RELATIVE, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "position:relative is in-flow — no-opt-in overflow must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_inside_overflow_hidden_clip_shell() {
    let result = check_preprocessed_with(OVERFLOW_CLIP_SHELL_ANCESTOR, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflow(hidden) on a distant clip shell is not a scroller opt-in — \
         leftover nowrap actions must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_inside_ticker_host() {
    let result =
        check_preprocessed_with(OVERFLOW_TICKER_HOST, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "nowrap shrink-0 cells inside a ticker host are the marquee — \
         must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_when_child_contains_out_of_flow() {
    let result = check_preprocessed_with(OVERFLOW_OUT_OF_FLOW_DESCENDANT, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "in-flow width inflated by a position:fixed descendant is overlay IR — \
         must NOT emit `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_when_only_a_sibling_is_a_scroller() {
    let result = check_preprocessed_with(OVERFLOW_SIBLING_SCROLLER, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "a sibling scroll-area is not an ancestor opt-in — no-opt-in neighbor \
         must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_for_horizontal_leftover_inside_vertical_scroller() {
    let result = check_preprocessed_with(
        OVERFLOW_VERTICAL_SCROLLER_HORIZONTAL_LEFTOVER,
        VP,
        experimental(),
    )
    .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflowY(auto) ancestor must not hide a horizontal leftover: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_when_leftover_has_absolute_badge() {
    let result = check_preprocessed_with(OVERFLOW_LEFTOVER_WITH_ABSOLUTE_BADGE, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "in-flow leftover that still overflows after excluding an absolute badge \
         must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_inside_position_fixed_shell() {
    let result = check_preprocessed_with(OVERFLOW_INSIDE_FIXED_SHELL, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "no-opt-in overflow inside a position:fixed shell must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_fires_when_tablet_at_desktop_viewport() {
    let desktop = check_preprocessed_with(OVERFLOW_WHEN_TABLET, (1440.0, 900.0), experimental())
        .expect("desktop layout");
    assert!(
        has_kind(&desktop.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "<when {{tablet}}> at 1440 is on-screen; no-opt-in overflow must stay red: {:?}",
        desktop.diagnostics
    );
}

const KIND_TAP: &str = "sub-min-tap-target";

const TAP_AT_FLOOR: &str = r#"<button style:w="24px" style:h="24px">ok</button>"#;
const TAP_UNDER_WIDTH: &str = r#"<button style:w="23px" style:h="40px">x</button>"#;
const TAP_UNDER_HEIGHT: &str = r#"<button style:w="40px" style:h="23px">x</button>"#;
const TAP_TEXT_ONLY: &str = r#"<div style:w="16px" style:h="16px">not a control</div>"#;

#[test]
fn tap_target_at_floor_does_not_fire() {
    let result = check_preprocessed_with(TAP_AT_FLOOR, VP, experimental()).expect("layout");
    assert!(
        !has_kind(&result.diagnostics, KIND_TAP),
        "24px floor must not fire: {:?}",
        result.diagnostics
    );
}

#[test]
fn tap_target_under_floor_fires() {
    for (label, src) in [("width", TAP_UNDER_WIDTH), ("height", TAP_UNDER_HEIGHT)] {
        let result = check_preprocessed_with(src, VP, experimental()).expect("layout");
        assert!(
            has_kind(&result.diagnostics, KIND_TAP),
            "{label} under 24px must fire: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn tap_target_ignores_non_interactive() {
    let result = check_preprocessed_with(TAP_TEXT_ONLY, VP, experimental()).expect("layout");
    assert!(
        !has_kind(&result.diagnostics, KIND_TAP),
        "plain text box is not a tap target: {:?}",
        result.diagnostics
    );
}

#[test]
fn tap_target_experimental_off() {
    let result = check_preprocessed(TAP_UNDER_WIDTH, VP).expect("layout");
    assert!(
        !has_kind(&result.diagnostics, KIND_TAP),
        "tap target stays experimental: {:?}",
        result.diagnostics
    );
}

const TAP_ROLE_BUTTON_UNDER: &str = r#"<div role="button" style:w="23px" style:h="40px">x</div>"#;
const TAP_ON_CLICK_UNDER: &str = r#"<div on:click="{go}" style:w="40px" style:h="23px">x</div>"#;
const TAP_HREF_UNDER: &str = r##"<a href="#" style:w="23px" style:h="40px">x</a>"##;
const TAP_LABEL_UNDER: &str = r#"<label style:w="23px" style:h="40px">x</label>"#;
const TAP_UNDER_BESIDE_FLOOR: &str = r#"
<div class="sail-row">
  <button style:w="24px" style:h="24px">ok</button>
  <button style:w="23px" style:h="40px">x</button>
</div>
"#;

#[test]
fn tap_target_role_on_handler_href_and_label_under_floor_stay_red() {
    for (label, src) in [
        ("role=button", TAP_ROLE_BUTTON_UNDER),
        ("on:click", TAP_ON_CLICK_UNDER),
        ("a[href]", TAP_HREF_UNDER),
        ("label", TAP_LABEL_UNDER),
    ] {
        let result = check_preprocessed_with(src, VP, experimental()).expect("layout");
        assert!(
            has_kind(&result.diagnostics, KIND_TAP),
            "{label} under 24px must stay red: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn tap_target_floor_sibling_does_not_exempt_under_floor_neighbor() {
    let result =
        check_preprocessed_with(TAP_UNDER_BESIDE_FLOOR, VP, experimental()).expect("layout");
    assert!(
        has_kind(&result.diagnostics, KIND_TAP),
        "a 24px sibling must not swallow a 23px neighbor: {:?}",
        result.diagnostics
    );
}

const TAP_UNDER_INSIDE_CUSTOM_HOST: &str = r#"
<studio-list-row>
  <button style:w="23px" style:h="40px">x</button>
</studio-list-row>
"#;

const TAP_UNKNOWN_HOST_UNDER: &str =
    r#"<studio-list-row role="button" style:w="23px" style:h="40px">x</studio-list-row>"#;

const TAP_KNOWN_HOST_UNKNOWN_CHILD: &str = r#"
<button style:w="23px" style:h="40px">
  <sail-code-editor></sail-code-editor>
</button>
"#;

#[test]
fn tap_target_unknown_host_does_not_swallow_known_under_floor_child() {
    let result =
        check_preprocessed_with(TAP_UNDER_INSIDE_CUSTOM_HOST, VP, experimental()).expect("layout");
    assert!(
        has_kind(&result.diagnostics, KIND_TAP),
        "unknown custom host must not hide a measurable 23px button: {:?}",
        result.diagnostics
    );
}

#[test]
fn tap_target_unknown_interactive_host_under_floor_stays_quiet() {
    let result =
        check_preprocessed_with(TAP_UNKNOWN_HOST_UNDER, VP, experimental()).expect("layout");
    assert!(
        !has_kind(&result.diagnostics, KIND_TAP),
        "hyphenated host box is a measurement lie — must stay quiet: {:?}",
        result.diagnostics
    );
}

#[test]
fn tap_target_known_host_with_unknown_child_stays_red() {
    let result =
        check_preprocessed_with(TAP_KNOWN_HOST_UNKNOWN_CHILD, VP, experimental()).expect("layout");
    assert!(
        has_kind(&result.diagnostics, KIND_TAP),
        "known 23px button must stay red even when a descendant is unknown: {:?}",
        result.diagnostics
    );
}

// ── LAYOUT-5 overflow FP classes ────────────────────────────────────────────

/// Unbound `{expr}` source is not a solved width.
const OVERFLOW_UNBOUND_EXPR: &str = r#"
<div class="sail-row sail-nowrap s-w8">
  <p class="s-shrink0 s-fontSizexs">{session.paintMode()}</p>
</div>
"#;

/// Same leftover with a known static string — must stay red.
const OVERFLOW_STATIC_TEXT: &str = r#"
<div class="sail-row sail-nowrap s-w20">
  <p class="s-shrink0 s-whitespacenowrap">unbound-expr-is-not-this-static-run</p>
</div>
"#;

/// Overlay IR: popover host measured in-flow must not inflate the row.
const OVERFLOW_POPOVER_OVERLAY: &str = r#"
<div class="sail-row sail-nowrap s-w20">
  <div class="s-w10 s-shrink0">Undo</div>
  <div popover class="s-w40 s-h20">dialog</div>
</div>
"#;

/// Host splice: stdlib button class vs a leftover narrower than the token.
const OVERFLOW_HOST_SPLICE: &str = r#"
<div class="sail-col s-w14">
  <button class="sail-button s-sizemd-minHcontrolhmd s-w20 s-h28">x</button>
</div>
"#;

#[test]
fn unintended_overflow_suppressed_for_unbound_expr_width() {
    let result =
        check_preprocessed_with(OVERFLOW_UNBOUND_EXPR, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "unbound {{expr}} width is measured-as-unknown — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_still_fires_for_static_text_leftover() {
    let result =
        check_preprocessed_with(OVERFLOW_STATIC_TEXT, VP, experimental()).expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "known static text wider than a nowrap leftover must stay red: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_popover_overlay() {
    let result = check_preprocessed_with(OVERFLOW_POPOVER_OVERLAY, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "[popover] is out-of-flow overlay IR — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_spliced_l1_host() {
    let result =
        check_preprocessed_with(OVERFLOW_HOST_SPLICE, VP, experimental()).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "sail-button splice vs leftover is host-splice geometry — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

// ── LAYOUT-5 tap-target scope ───────────────────────────────────────────────

const TAP_PHRASING_LINK: &str = r#"<p>See the <a href="/docs">docs</a> for more.</p>"#;
const TAP_CHECKBOX_FACE: &str = r#"<input type="checkbox" style:w="16px" style:h="16px">"#;
const TAP_RADIO_FACE: &str = r#"<input type="radio" style:w="16px" style:h="16px">"#;
const TAP_RANGE_TRACK: &str = r#"<input type="range" style:w="120px" style:h="20px">"#;
const TAP_CAPTION_LABEL: &str = r#"<label>Name</label>"#;
const TAP_LABEL_WRAPS_CHECKBOX: &str = r#"
<label>
  <input type="checkbox" style:w="16px" style:h="16px">
  Enable
</label>
"#;

#[test]
fn tap_target_skips_phrasing_anchor() {
    let result = check_preprocessed_with(TAP_PHRASING_LINK, VP, experimental()).expect("layout");
    assert!(
        !has_kind(&result.diagnostics, KIND_TAP),
        "inline docs <a> is phrasing, not a standalone target: {:?}",
        result.diagnostics
    );
}

#[test]
fn tap_target_skips_control_internals() {
    for (label, src) in [
        ("checkbox", TAP_CHECKBOX_FACE),
        ("radio", TAP_RADIO_FACE),
        ("range", TAP_RANGE_TRACK),
    ] {
        let result = check_preprocessed_with(src, VP, experimental()).expect("layout");
        assert!(
            !has_kind(&result.diagnostics, KIND_TAP),
            "{label} face/track is a control internal — label owns the hit area: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn tap_target_skips_caption_label_and_wrapped_face() {
    for (label, src) in [
        ("caption", TAP_CAPTION_LABEL),
        ("wraps-checkbox", TAP_LABEL_WRAPS_CHECKBOX),
    ] {
        let result = check_preprocessed_with(src, VP, experimental()).expect("layout");
        assert!(
            !has_kind(&result.diagnostics, KIND_TAP),
            "{label}: caption / wrapped face is not a standalone target: {:?}",
            result.diagnostics
        );
    }
}
