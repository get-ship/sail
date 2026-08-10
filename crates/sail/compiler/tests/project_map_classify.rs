//! Gate tests for SPEC-PROJECT-STRUCTURE-1 §2 — file-kind classification.
//!
//! Spike S1.1 red gate. Calls the product `project_map` API only.
//! Implementer must not modify, skip, or weaken these tests.

use std::path::Path;

use sail_compiler::project_map::{classify, misplacement_message, Classified, FileKind};

fn kind_of(path: &str, source: Option<&str>) -> Classified {
    classify(Path::new(path), source)
}

#[test]
fn route_page_classifies_as_route() {
    let c = kind_of("apps/x/routes/foo/page.sail", None);
    assert_eq!(c.kind, FileKind::Route, "page.sail under routes/ is a route");
    assert!(
        c.home.contains("routes"),
        "route home must name routes/, got {:?}",
        c.home
    );
    assert!(c.ok, "route under routes/ must be ok");
}

#[test]
fn component_sail_classifies_as_component() {
    let c = kind_of("apps/x/src/widget/widget.sail", None);
    assert_eq!(
        c.kind,
        FileKind::Component,
        ".sail outside routes/ is a component"
    );
}

#[test]
fn action_path_classifies_as_action() {
    let c = kind_of(
        "apps/x/src/actions/focus.ts",
        Some("export function focus(node) { return {} }\n"),
    );
    assert_eq!(
        c.kind,
        FileKind::Action,
        "path under src/actions/ is an action"
    );
}

#[test]
fn store_under_src_stores_with_define_store_is_ok() {
    let src = "export const tally = defineStore('tally', () => ({}))\n";
    let c = kind_of("apps/x/src/stores/tally.ts", Some(src));
    assert_eq!(c.kind, FileKind::Store);
    assert!(c.ok, "defineStore under src/stores/ must be ok");
    assert!(
        c.home.contains("src/stores") || c.home.contains("stores"),
        "store home must name src/stores/, got {:?}",
        c.home
    );
}

#[test]
fn define_store_under_compositions_is_misplaced_store() {
    let src = "export const counter = defineStore('docs.tally', () => ({}))\n";
    let path = Path::new("apps/x/src/compositions/counter-store.ts");
    let c = classify(path, Some(src));
    assert_eq!(
        c.kind,
        FileKind::Store,
        "defineStore shape wins even under compositions/"
    );
    assert!(!c.ok, "store under compositions/ must not be ok");

    let msg = misplacement_message(c.kind, path);
    assert!(
        msg.contains("src/stores/"),
        "misplacement message must name src/stores/, got: {msg}"
    );
}

#[test]
fn composition_without_define_store() {
    let src = "export function useFoo() {\n  return signal(0)\n}\n";
    let c = kind_of("apps/x/src/compositions/use-foo.ts", Some(src));
    assert_eq!(
        c.kind,
        FileKind::Composition,
        "pure helper under compositions/ without defineStore is Composition"
    );
    assert!(c.ok, "composition in its home must be ok");
}

#[test]
fn theme_path_classifies_as_theme() {
    let c = kind_of("apps/x/themes/ship.json", None);
    assert_eq!(c.kind, FileKind::Theme, "path under themes/ is Theme");
}
