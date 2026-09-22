//! Capabilities + compiler text-host law.
//!
//! Textish setters (`type`) are legal only on hosts that publish text.
//! Host identity comes from HTML tags + the L1 registry, not a Channel enum.

use sail_compiler::{is_text_host, text_host_error};
use spinnaker::{is_geometry_setter, parse, setter_caps, style_is_textish};

#[test]
fn button_is_text_host_via_registry() {
    // Build-time derived table was folded away — the roots are inline now.
    let roots = [("chip", "span"), ("badge", "span")];
    assert!(is_text_host("button"), "L1 button publishes text");
    assert!(is_text_host("p"), "HTML <p> publishes text");
    assert!(
        sail_compiler::text_host::is_text_host_with("chip", &roots),
        "L1 chip roots on <span>"
    );
}

#[test]
fn layout_and_structure_are_not_text_hosts() {
    for tag in ["div", "row", "column", "grid", "svg", "scroll-area"] {
        assert!(!is_text_host(tag), "{tag}");
    }
}

#[test]
fn textish_style_is_type_not_pad() {
    let (ty, _) = parse("style({ type({ size: sm }) })");
    assert!(style_is_textish(&ty));
    let (boxy, _) = parse("style({ pad(4).bg(primary) })");
    assert!(!style_is_textish(&boxy));
}

#[test]
fn geometry_flag_not_display() {
    assert!(is_geometry_setter("pad"));
    assert!(is_geometry_setter("size"));
    assert!(!is_geometry_setter("display"));
    assert!(!is_geometry_setter("type"));
    assert!(setter_caps("play").unwrap().motion);
    assert!(setter_caps("type").unwrap().textish);
}

#[test]
fn text_host_error_names_the_tag() {
    let msg = text_host_error("div");
    assert!(msg.contains("<div>"), "{msg}");
    assert!(msg.contains("text-bearing"), "{msg}");
}
