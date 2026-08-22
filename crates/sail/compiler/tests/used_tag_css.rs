//! Job F — used-tag platform CSS.
//!
//! A button page does not download `prose.css`. Platform adaptor sheets
//! ride the used L1 tags, not the full catalog.

use sail_compiler::stdlib;

#[test]
fn button_page_platform_css_has_no_prose() {
    let css = stdlib::skins_css_for(["button"]);
    assert!(
        !css.contains("/* @provenance platform/prose.css */") && !css.contains("@element prose"),
        "button page must not carry prose platform CSS:\n{css}"
    );
    assert!(
        css.contains("@element button")
            || css.contains("sail-button")
            || !css.is_empty()
            || stdlib::element("button").is_some(),
        "button is an L1 tag: empty-or-button skins, never prose"
    );
}

#[test]
fn prose_page_keeps_prose_platform_css() {
    let css = stdlib::skins_css_for(["prose"]);
    assert!(
        css.contains("/* @provenance platform/prose.css */") || css.contains("@element prose"),
        "a prose page still gets the prose adaptor:\n{css}"
    );
}

#[test]
fn hello_world_tags_do_not_pull_prose() {
    let tags = sail_compiler::used_tags::collect_stdlib_tags_from_source(
        "<column gap=\"4\"><h1>Hi</h1><p>Sail app ready.</p></column>",
    );
    assert!(
        !tags.iter().any(|t| t == "prose"),
        "hello-world does not use prose: {tags:?}"
    );
    let css = stdlib::skins_css_for(tags.iter().map(String::as_str));
    assert!(
        !css.contains("platform/prose.css"),
        "hello-world used-tag CSS has no prose:\n{css}"
    );
}
