//! §2.6 class names + sheet-size accounting.

use std::collections::HashMap;
use std::collections::HashSet;

use spinnaker::parse::parse;

use super::{emit_style, lower_style, lower_style_source};

fn classes_of(src: &str) -> String {
    let (style, diags) = parse(src);
    assert!(
        diags.is_empty(),
        "{src}: {:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
    lower_style(&style, &HashMap::new()).0
}

#[test]
fn bg_surface_is_vocab_class() {
    let classes = classes_of("style({ bg(surface) })");
    assert!(
        classes.split_whitespace().any(|c| c == "s-bg-surface"),
        "bg(surface) → .s-bg-surface, got {classes}"
    );
    assert!(
        !classes.contains("s-bgsurface"),
        "chain class must die: {classes}"
    );
}

#[test]
fn pad_xy_is_vocab_class() {
    let classes = classes_of("style({ pad({ x: 4, y: 16 }) })");
    assert!(
        classes.split_whitespace().any(|c| c == "s-pad-x4-y16"),
        "pad({{ x: 4, y: 16 }}) → .s-pad-x4-y16, got {classes}"
    );
    assert!(
        !classes.contains("s-padx4y16"),
        "chain class must die: {classes}"
    );
}

#[test]
fn size_w_pct_is_vocab_class() {
    let classes = classes_of("style({ size({ w: 100% }) })");
    assert!(
        classes.split_whitespace().any(|c| c == "s-size-w-100pct"),
        "size({{ w: 100% }}) → .s-size-w-100pct, got {classes}"
    );
}

#[test]
fn hover_bg_muted_is_vocab_class() {
    let classes = classes_of("style({ when: { hover: bg(muted) } })");
    assert!(
        classes.split_whitespace().any(|c| c == "s-hover-bg-muted"),
        "hover bg(muted) → .s-hover-bg-muted, got {classes}"
    );
    assert!(
        !classes.contains("s-hover-bgmuted"),
        "chain class must die: {classes}"
    );
}

#[test]
fn inspector_reads_like_the_style_attr() {
    let (style, diags) = parse("style({ bg(surface).pad({ x: 4, y: 16 }) })");
    assert!(diags.is_empty(), "{:?}", diags);
    let (classes, _) = lower_style(&style, &HashMap::new());
    let set: HashSet<&str> = classes.split_whitespace().collect();
    assert!(set.contains("s-bg-surface"), "{classes}");
    assert!(set.contains("s-pad-x4-y16"), "{classes}");
}

#[test]
fn shared_concern_emits_one_rule() {
    let mut css = String::new();
    let mut seen = HashSet::new();
    let env = HashMap::new();
    let a = lower_style_source("style({ bg(surface).pad(4) })", &env, &mut css, &mut seen)
        .expect("face A");
    let b = lower_style_source("style({ bg(surface).pad(8) })", &env, &mut css, &mut seen)
        .expect("face B");
    assert!(a.split_whitespace().any(|c| c == "s-bg-surface"), "{a}");
    assert!(b.split_whitespace().any(|c| c == "s-bg-surface"), "{b}");
    assert_eq!(
        css.matches(".s-bg-surface{").count(),
        1,
        "two bg(surface) spends share one rule:\n{css}"
    );
    assert!(
        a.split_whitespace().any(|c| c == "s-pad-4"),
        "pad(4) → .s-pad-4, got {a}"
    );
    assert!(
        b.split_whitespace().any(|c| c == "s-pad-8"),
        "pad(8) → .s-pad-8, got {b}"
    );
}

#[test]
fn emit_style_dedupes_same_concern_inside_one_program() {
    let (style, diags) = parse("style({ bg(surface) })");
    assert!(diags.is_empty());
    let rules = emit_style(&style, &HashMap::new());
    let n = rules.iter().filter(|r| r.class == "s-bg-surface").count();
    assert_eq!(n, 1, "{rules:?}");
}

#[test]
fn chain_class_placeinset_dies() {
    let classes = classes_of("style({ position({ scheme: relative }) })");
    assert!(
        classes
            .split_whitespace()
            .any(|c| c == "s-position-scheme-relative"),
        "{classes}"
    );
    assert!(
        !classes.contains("s-positionschemerelative"),
        "concatenated chain class must die: {classes}"
    );
}
