//! Integration tests for the SSR render walker.
//!
//! D-1 nailed marker shapes; D-2 evaluates the renderable subset
//! against the page's initial scope (just `<data>` envelopes plus
//! `route`), inlines the rendered values alongside markers, and
//! emits the `__set_init__` JSON island after the body. Reactive
//! component state is declared in the controller's `scope:` factory
//! and only materializes during hydrate, so state-bound expressions
//! fall through to marker-only output here.

use sui_compiler::bindings;
use sui_compiler::parse::parse;
use sui_compiler::render::{RenderOpts, RenderOutput, render_default as render};

fn render_source(src: &str) -> String {
    render_full(src).html
}

fn render_full(src: &str) -> RenderOutput {
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "test".into() };
    render(&program, &opts).expect("render")
}

const ISLAND_OPEN: &str = "<script type=\"application/json\" id=\"__set_init__\">";

#[test]
fn static_html_renders_verbatim() {
    let html = render_source(r#"<div class="x">hello</div>"#);
    assert!(
        html.starts_with(r#"<div class="x">hello</div>"#),
        "static body must come first; got: {html:?}"
    );
    // No data decls -> island is the empty object.
    assert!(html.ends_with("{}</script>"), "expected empty island; got: {html:?}");
}

#[test]
fn dynamic_text_emits_comment_marker() {
    // Unbound identifier -> evaluator fails -> marker-only fallback.
    let html = render_source("<p>{count}</p>");
    assert!(
        html.contains("<p><!--sui:text:b0--><!--/sui--></p>"),
        "expected marker-only dynamic-text shape; got: {html:?}"
    );
}

#[test]
fn event_handler_emits_on_attr() {
    let src = "<button on:click={() => x()}></button>";
    let html = render_source(src);
    assert!(
        html.contains(r#"<button on:click="b0">"#),
        "expected on:click marker; got: {html:?}"
    );
    assert!(html.contains("</button>"));
}

#[test]
fn bind_value_emits_bind_attr() {
    // <input> is a void element — no closing tag.
    let src = "<input bind:value={q} />";
    let html = render_source(src);
    assert!(
        html.contains(r#"bind:value="b0""#),
        "expected bind:value marker; got: {html:?}"
    );
    assert!(
        !html.contains("</input>"),
        "void element must not emit a closing tag; got: {html:?}"
    );
}

#[test]
fn bind_checked_emits_bind_attr() {
    let src = "<input type=\"checkbox\" bind:checked={on} />";
    let html = render_source(src);
    assert!(
        html.contains(r#"bind:checked="b0""#),
        "expected bind:checked marker; got: {html:?}"
    );
}

#[test]
fn class_directive_emits_class_attr() {
    let src = "<p class:active={on}>x</p>";
    let html = render_source(src);
    assert!(
        html.contains(r#"class:active="b0""#),
        "expected class marker; got: {html:?}"
    );
}

#[test]
fn expr_attribute_emits_attr_marker() {
    let src = "<a href={u}>x</a>";
    let html = render_source(src);
    assert!(
        html.contains(r#"attr:href="b0""#),
        "expected attr:href marker; got: {html:?}"
    );
}

#[test]
fn attr_prefixed_literal_name_emits_attr_marker() {
    // `attr:<ident>={expr}` shorthand should compile to the same shape
    // as `<ident>={expr}` — a static-name attribute binding.
    let src = "<label attr:for={target}>x</label>";
    let html = render_source(src);
    assert!(
        html.contains(r#"attr:for="b0""#),
        "expected attr:for marker; got: {html:?}"
    );
}

#[test]
fn attr_prefixed_dynamic_name_emits_attr_dynamic_marker() {
    // Regression guard: `attr:{nameExpr}={valueExpr}` continues to compile
    // to the dynamic-name attribute binding.
    let src = "<a attr:{name}={value}>x</a>";
    let html = render_source(src);
    assert!(
        html.contains(r#"attr-dynamic="b0""#),
        "expected attr-dynamic marker; got: {html:?}"
    );
}

#[test]
fn if_emits_placeholder() {
    // Unbound condition → evaluator errors → placeholder fallback.
    let src = "{#if missing}<p>x</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<sui-if key="b0" hidden></sui-if>"#),
        "expected if placeholder; got: {html:?}"
    );
    assert!(
        !html.contains("<p>x</p>"),
        "body must not be recursed when condition is unevaluable; got: {html:?}"
    );
}

#[test]
fn for_emits_placeholder() {
    let src = "{#for x of xs}<li>x</li>{/for}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<sui-for key="b0" hidden></sui-for>"#),
        "expected for placeholder; got: {html:?}"
    );
}

#[test]
fn match_emits_placeholder() {
    // Unbound discriminant → evaluator errors → placeholder fallback.
    let src = "{#match missing}{:case 'a'}A{:default}D{/match}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<sui-match key="b0" hidden></sui-match>"#),
        "expected match placeholder; got: {html:?}"
    );
    assert!(
        !html.contains(">A<") && !html.contains(">D<"),
        "no body must render when discriminant is unevaluable; got: {html:?}"
    );
}

#[test]
fn static_attrs_are_html_escaped() {
    // The `&` in a static attribute value must become `&amp;`.
    let html = render_source(r#"<a title="a & b">x</a>"#);
    assert!(
        html.contains(r#"<a title="a &amp; b">"#),
        "expected escaped attr; got: {html:?}"
    );
}

#[test]
fn data_decl_skips_in_html() {
    let src = "<data name=\"agents\" source=\"ship.agents.summaries()\" live />\n<p>hi</p>";
    let html = render_source(src);
    // <data> declarations are lifted by the parser, never appear in
    // the template tree, so they cannot be rendered. Confirm.
    assert!(!html.contains("<data"), "data must not render: {html:?}");
    assert!(html.contains("<p>hi</p>"));
}

#[test]
fn keys_increment_in_source_order() {
    // Two dynamic interpolations should get b0 and b1 in order.
    let src = "<p>{a}-{b}</p>";
    let html = render_source(src);
    let i0 = html.find("sui:text:b0").expect("first marker");
    let i1 = html.find("sui:text:b1").expect("second marker");
    assert!(i0 < i1, "keys must increment in source order: {html:?}");
}

#[test]
fn real_view_renders_without_panic() {
    // A real repo view through the full render pipeline. (Was `views/agents`,
    // which the studio reorg moved to apps/shell/src/studio/; `views/git` is a
    // real top-level view that still lives here.)
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("views/git/page.sui"),
    )
    .expect("read git view");
    let program = parse(&src).expect("git view parses");
    let opts = RenderOpts { view_name: "git".into() };
    let out = render(&program, &opts).expect("git view renders");
    assert!(!out.html.is_empty(), "git view must produce non-empty HTML");
    // Sanity: the static container should be present.
    assert!(out.html.contains("ship-view"), "container class missing: {}", &out.html[..200.min(out.html.len())]);
}

// ---------- Phase D-2: scope evaluation + JSON island ----------

#[test]
fn data_initial_envelope_is_loading() {
    let src = "<data name=\"agents\" source=\"ship.agents.summaries()\" live />";
    let html = render_source(src);
    assert!(
        html.contains(r#""agents":{"kind":"loading"}"#),
        "expected data envelope in island; got: {html:?}"
    );
}

#[test]
fn unevaluable_expr_emits_marker_only() {
    // `ship` not in scope -> Unbound -> marker-only fallback.
    let html = render_source("<p>{ship.foo()}</p>");
    assert!(
        html.contains("<p><!--sui:text:b0--><!--/sui--></p>"),
        "expected marker-only fallback; got: {html:?}"
    );
}

#[test]
fn class_directive_renders_initial() {
    let src = "<div class:active={true}>x</div>";
    let html = render_source(src);
    assert!(
        html.contains(r#"<div class="active" class:active="b0">"#),
        "expected merged class plus marker; got: {html:?}"
    );
}

#[test]
fn json_island_present_after_body() {
    let src = "<data name=\"agents\" source=\"ship.agents.summaries()\" live />\n<p>hi</p>";
    let html = render_source(src);
    let count = html.matches(ISLAND_OPEN).count();
    assert_eq!(count, 1, "expected exactly one island; got {count}: {html:?}");
    let island_pos = html.find(ISLAND_OPEN).unwrap();
    let body_marker = html.find("<p>hi</p>").expect("static body present");
    assert!(
        body_marker < island_pos,
        "island must follow body content; got: {html:?}"
    );
}

#[test]
fn agents_view_island_carries_loading_envelope() {
    // Inline template rather than reading a view file — exercises the same
    // live-`<data>` → loading-envelope + init-map behavior without depending on
    // a specific view's on-disk location (agents moved in the studio reorg).
    let src = "<data name=\"agents\" source=\"ship.agents.summaries()\" live />\n<p>hi</p>";
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("render");
    assert!(out.html.contains(ISLAND_OPEN), "island missing");
    assert!(
        out.html.contains(r#""agents":{"kind":"loading"}"#),
        "expected agents:loading in island"
    );
    let agents = out.init.get("agents").expect("init has agents key");
    assert_eq!(
        agents,
        &serde_json::json!({"kind": "loading"}),
        "init.agents shape mismatch"
    );
}

// ---------- Phase D-3: control-flow rendering ----------

#[test]
fn if_unevaluable_falls_back_to_placeholder() {
    // State signals reach scope only at hydrate, so a `{#if x}` over a
    // controller-scope identifier collapses to the placeholder shape
    // here at SSR time.
    let src = "{#if missing.id === '42'}<p>x</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<sui-if key="b0" hidden></sui-if>"#),
        "expected placeholder; got: {html:?}"
    );
    assert!(!html.contains("<p>x</p>"));
}

#[test]
fn for_unevaluable_iterable_falls_back() {
    let src = "{#for n of missing}<li>{n}</li>{/for}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<sui-for key="b0" hidden></sui-for>"#),
        "expected placeholder for unevaluable iterable; got: {html:?}"
    );
    assert!(!html.contains("<li>"));
}

#[test]
fn match_unevaluable_falls_back() {
    let src = "{#match missing}{:case 'a'}<p>A</p>{:default}<p>D</p>{/match}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<sui-match key="b0" hidden></sui-match>"#),
        "expected placeholder for unevaluable discriminant; got: {html:?}"
    );
    assert!(!html.contains("<p>A</p>"));
    assert!(!html.contains("<p>D</p>"));
}

#[test]
fn agents_view_round_trip_with_d3() {
    // Inline template mirroring the agents view's live-data + guarded-for shape
    // (the view file moved to apps/shell/src/studio/ in the reorg). The outer
    // `{#if agents().kind === 'ok'}` is falsy against the loading envelope at
    // SSR, so the inner `{#for}` never reaches the output.
    let src = "<data name=\"agents\" source=\"ship.agents.summaries()\" live />\n{#if agents().kind === 'ok'}{#for a of agents().value key={a.id}}<p>{a.id}</p>{/for}{/if}";
    let program = parse(src).expect("parse agents template");
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("render agents template");
    // Outer `{#if agents().kind === 'ok'}` evaluates against the loading
    // envelope at SSR time, so it falls into the placeholder branch.
    assert!(
        out.html.contains("<sui-if key="),
        "expected at least one sui-if boundary: {}",
        &out.html[..200.min(out.html.len())]
    );
    // The agents view contains a `{#for}` somewhere; its outer guard
    // (the `{#if agents().kind === 'ok'}`) is falsy, so the for marker
    // never reaches the output. We only require that the walker did
    // not panic and produced a reachable shape — assert the `__set_init__`
    // island is still emitted.
    assert!(out.html.contains("__set_init__"), "init island missing");
    assert_eq!(
        out.init.get("agents"),
        Some(&serde_json::json!({"kind": "loading"})),
    );
}

// ---------- Phase E-prep: walker / registry parity ----------

/// For every marker the walker emits — `<!--sui:text:bN-->` comments,
/// `<sui-{kind} key="bN">` boundary elements, and author-syntax binding
/// attributes like `on:click="bN"` — look up `bindings[N]` and confirm
/// the registry entry's kind matches.
#[test]
fn walker_keys_match_registry() {
    // Parity is view-agnostic; exercise it against a real complex view.
    // (Was `views/agents`, moved in the reorg → `views/git` is a real
    // top-level view with many binding kinds.)
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("views/git/page.sui"),
    )
    .expect("read git view");
    let program = parse(&src).expect("parse");
    let registry = bindings::build(&program);
    let opts = RenderOpts { view_name: "git".into() };
    let out = render(&program, &opts).expect("render");

    // Pull `(kind_str, key)` pairs out of the rendered HTML in order.
    let pairs = extract_marker_pairs(&out.html);
    assert!(
        !pairs.is_empty(),
        "agents view should emit at least one marker; got: {}",
        &out.html[..400.min(out.html.len())]
    );

    for (marker_kind, key) in pairs {
        let n: usize = key
            .strip_prefix('b')
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("malformed key {key}"));
        let entry = registry
            .get(n)
            .unwrap_or_else(|| panic!("registry missing index {n} (key {key})"));
        assert_eq!(
            entry.key, key,
            "registry[{n}].key={} but marker key={key}",
            entry.key
        );
        assert_eq!(
            entry.kind.as_marker_str(),
            marker_kind,
            "registry[{n}].kind={} but marker kind={marker_kind}",
            entry.kind.as_marker_str()
        );
    }
}

/// Find every marker the SSR walker emits, returning `(kind_str, key)` pairs.
///
/// New marker shapes (post SSR re-cutover):
///   - `<!--sui:text:bN-->` → `("text", "bN")`
///   - `<sui-{kind} key="bN"` → `("{kind}", "bN")`
///   - `on:{event}="bN"`, `bind:value="bN"`, `class:{name}="bN"`,
///     `style:{name}="bN"`, `attr:{name}="bN"`, `attr-dynamic="bN"`
///     → `("{kind}", "bN")`
fn extract_marker_pairs(html: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();

    // <!--sui:text:bN-->
    let text_prefix = "<!--sui:text:";
    let mut j = 0;
    while let Some(start) = html[j..].find(text_prefix) {
        let abs = j + start + text_prefix.len();
        if let Some(end_off) = html[abs..].find("-->") {
            let key = &html[abs..abs + end_off];
            if key.starts_with('b') {
                out.push(("text".to_string(), key.to_string()));
            }
            j = abs + end_off + 3;
        } else {
            break;
        }
    }

    // <sui-{kind} key="bN"
    for kind in ["if", "for", "match"] {
        let open_pat = format!("<sui-{kind} key=\"");
        let mut i = 0;
        while let Some(pos) = html[i..].find(&open_pat) {
            let val_start = i + pos + open_pat.len();
            if let Some(qe) = html[val_start..].find('"') {
                let key = &html[val_start..val_start + qe];
                if key.starts_with('b') {
                    out.push((kind.to_string(), key.to_string()));
                }
                i = val_start + qe + 1;
            } else {
                break;
            }
        }
    }

    // Author-syntax binding attribute markers: scan for ATTR_NAME="bN"
    // where ATTR_NAME matches a known binding-attribute pattern.
    // Strategy: find `="b` followed by digits followed by `"`, then
    // look back for the attribute name after the preceding space.
    let mut i = 0;
    while i < html.len() {
        if let Some(pos) = html[i..].find("=\"b") {
            let eq_pos = i + pos;
            let val_start = eq_pos + 2; // after ="
            let val_rest = &html[val_start..];
            // Collect the key: 'b' followed by ASCII digits
            let digit_count = val_rest[1..].chars().take_while(|c| c.is_ascii_digit()).count();
            let key_end = 1 + digit_count; // total length of bN
            if digit_count > 0 && val_rest.as_bytes().get(key_end) == Some(&b'"') {
                let key = &val_rest[..key_end];
                // Find the attribute name by scanning backward from eq_pos
                let prefix = &html[..eq_pos];
                let name_start = prefix
                    .rfind(|c: char| c.is_ascii_whitespace() || c == '<')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                let attr_name = &prefix[name_start..];
                if is_binding_attr(attr_name) {
                    out.push((attr_name.to_string(), key.to_string()));
                }
                i = val_start + key_end + 1;
                continue;
            }
            i = eq_pos + 3;
        } else {
            break;
        }
    }

    out
}

fn is_binding_attr(name: &str) -> bool {
    name.starts_with("on:")
        || name.starts_with("bind:")
        || name.starts_with("class:")
        || name.starts_with("style:")
        || name.starts_with("attr:")
        || name == "attr-dynamic"
}

#[test]
fn walker_keys_match_registry_for_counter_fixture() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/counter.sui"),
    )
    .expect("read counter fixture");
    let program = parse(&src).expect("parse");
    let registry = bindings::build(&program);
    let opts = RenderOpts { view_name: "counter".into() };
    let out = render(&program, &opts).expect("render");

    for (marker_kind, key) in extract_marker_pairs(&out.html) {
        let n: usize = key.strip_prefix('b').unwrap().parse().unwrap();
        let entry = registry.get(n).expect("registry has entry");
        assert_eq!(entry.kind.as_marker_str(), marker_kind);
    }
}
