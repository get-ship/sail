//! `<client>` directive — the per-view, host-injected data-source declaration
//! that replaces the hardcoded ambient `ship` (multiple `<client>`s = multiple
//! data sources). The emitted setup must destructure each declared client from
//! `ctx.clients` and seed `$scope` so a `<data source="X.op()">` (and inline
//! handlers) resolve bare `X` at runtime.

use sui_compiler::{compile, CompileOpts};

fn js(source: &str) -> String {
    compile(source, CompileOpts::default()).expect("compile").js
}

fn dts(source: &str) -> String {
    compile(source, CompileOpts::default()).expect("compile").dts.expect("dts emitted")
}

#[test]
fn data_source_typed_from_declared_client() {
    let out = dts(
        "<client name=\"feed\" />\n\
         <data name=\"t\" source=\"feed.clock()\" live />\n\
         <data name=\"o\" source=\"feed.once()\" />\n\
         <div>{t().kind}{o()}</div>",
    );
    assert!(
        out.contains("t: LiveSignal<LiveValueOf<ReturnType<Clients['feed']['clock']>>>"),
        "live data must type from the client return; got:\n{out}"
    );
    assert!(
        out.contains("o: Signal<Awaited<ReturnType<Clients['feed']['once']>>>"),
        "one-shot data must type from the client return; got:\n{out}"
    );
}

#[test]
fn data_type_falls_back_to_unknown_for_ship_or_chained() {
    // Grandfathered `ship` (not a declared client) and a chained call both keep
    // the conservative `unknown` — no false inference.
    let out = dts(
        "<client name=\"feed\" />\n\
         <data name=\"s\" source=\"ship.git.overview()\" live />\n\
         <data name=\"c\" source=\"feed.a().b()\" live />\n\
         <div>{s().kind}{c().kind}</div>",
    );
    assert!(out.contains("s: Signal<LiveState<unknown>>"), "ship source stays unknown");
    assert!(out.contains("c: Signal<LiveState<unknown>>"), "chained call stays unknown");
}

#[test]
fn declared_client_is_destructured_from_ctx_clients() {
    let out = js(
        "<client name=\"feed\" />\n\
         <data name=\"t\" source=\"feed.clock()\" live />\n\
         <div>{t().kind}</div>",
    );
    assert!(
        out.contains("const { feed } = ctx.clients"),
        "setup must destructure the declared client from ctx.clients; got:\n{out}"
    );
    assert!(out.contains("$scope.feed = feed"), "setup must seed $scope.<client>");
    assert!(out.contains("feed.clock()"), "the data-source thunk calls the client verbatim");
}

#[test]
fn multiple_clients_each_destructured() {
    let out = js(
        "<client name=\"crm\" />\n\
         <client name=\"analytics\" />\n\
         <data name=\"a\" source=\"crm.leads()\" live />\n\
         <data name=\"b\" source=\"analytics.traffic()\" live />\n\
         <div>{a().kind}{b().kind}</div>",
    );
    assert!(out.contains("const { crm } = ctx.clients"), "first client destructured");
    assert!(
        out.contains("const { analytics } = ctx.clients"),
        "second client destructured — multiple data sources in one view"
    );
}

#[test]
fn client_name_must_be_a_js_identifier() {
    // A kebab name would emit `const { my-api } = ctx.clients` — a JS syntax
    // error in every consumer. Reject at parse with a directive-shaped span.
    let err = compile("<client name=\"my-api\" />\n<div>x</div>", CompileOpts::default())
        .expect_err("kebab client name must be rejected");
    assert!(
        err.message.contains("must be a JS identifier"),
        "got: {}",
        err.message
    );
}

#[test]
fn reserved_setup_context_names_rejected() {
    for reserved in ["route", "navigate", "ctx", "scope", "props", "derived"] {
        let src = format!("<client name=\"{reserved}\" />\n<div>x</div>");
        let err = compile(&src, CompileOpts::default())
            .expect_err("reserved client name must be rejected");
        assert!(
            err.message.contains("reserved"),
            "`{reserved}` should be reserved; got: {}",
            err.message
        );
    }
}

#[test]
fn duplicate_client_declaration_rejected() {
    let err = compile(
        "<client name=\"feed\" />\n<client name=\"feed\" />\n<div>x</div>",
        CompileOpts::default(),
    )
    .expect_err("duplicate client must be rejected");
    assert!(err.message.contains("more than once"), "got: {}", err.message);
}

#[test]
fn default_export_carries_declared_client_list() {
    // The mount contract: hosts/runtime introspect which client keys a view
    // consumes from the default export, beside _compilerVersion.
    let out = js(
        "<client name=\"crm\" />\n\
         <client name=\"analytics\" />\n\
         <data name=\"a\" source=\"crm.leads()\" live />\n\
         <div>{a().kind}</div>",
    );
    assert!(
        out.contains("clients: [\"crm\", \"analytics\"]"),
        "default export must carry the declared client list; got:\n{out}"
    );
}

#[test]
fn undeclared_view_exports_empty_client_list() {
    let out = js("<div>hello</div>");
    assert!(
        out.contains("clients: []"),
        "no-declaration views export an empty list (ambient-ship compatibility default); got:\n{out}"
    );
}

#[test]
fn meta_sidecar_carries_declared_client_list() {
    let out = compile(
        "<client name=\"feed\" />\n<data name=\"t\" source=\"feed.clock()\" live />\n<div>{t().kind}</div>",
        CompileOpts::default(),
    )
    .expect("compile");
    let meta = out.meta.expect("meta emitted");
    assert_eq!(
        meta["clients"],
        serde_json::json!(["feed"]),
        "meta sidecar must list declared clients; got: {meta}"
    );
}

#[test]
fn declared_ship_routes_through_ctx_clients_with_legacy_fallback() {
    // `<client name="ship" />` is the migration affordance: the same key now
    // flows through the keyed map, while `ctx.ship` stays a fallback for
    // hosts that haven't switched their mount call yet.
    let out = js(
        "<client name=\"ship\" />\n\
         <data name=\"t\" source=\"ship.git.overview()\" live />\n\
         <div>{t().kind}</div>",
    );
    assert!(
        out.contains("const ship = (ctx.clients ?? {}).ship ?? ctx.ship;"),
        "declared ship resolves clients-first with ctx.ship fallback; got:\n{out}"
    );
    assert!(
        !out.contains("const { ship"),
        "declared ship must not also be destructured from ctx; got:\n{out}"
    );
    assert!(out.contains("clients: [\"ship\"]"), "export list includes ship");
}

#[test]
fn ship_grandfathered_not_routed_through_ctx_clients() {
    // A `ship.*` view with no `<client>` still emits the hardcoded destructure
    // (existing views are untouched through the migration); it is NOT routed
    // through ctx.clients.
    let out = js(
        "<data name=\"t\" source=\"ship.git.overview()\" live />\n<div>{t().kind}</div>",
    );
    assert!(out.contains("ship.git.overview()"), "ship source still resolves");
    assert!(
        !out.contains("const { ship } = ctx.clients"),
        "grandfathered ship must NOT be routed through ctx.clients"
    );
}
