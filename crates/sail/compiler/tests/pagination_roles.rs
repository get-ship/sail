//! SPEC-PAGINATION-1 — the nav brain is the <pagination>/<page-item>
//! controllers (mount-form), not a use: action. Host keeps the navigation
//! landmark (host anatomy) + label; the item stamps aria-current="page" via
//! the controller return; clicks resolve through the parent `select`.

use sail_compiler::{CompileOpts, compile, compile_element};

const PAGE_ITEM_SRC: &str = include_str!("../stdlib/pagination/page-item.sail");
const PAGINATION_SRC: &str = include_str!("../stdlib/pagination/pagination.sail");

#[test]
fn pagination_compiles_as_mount_form_without_action() {
    let out = compile(
        r#"<pagination value="2"><page-item value="1">1</page-item><page-item value="2">2</page-item></pagination>"#,
        CompileOpts::default(),
    )
    .expect("pagination compiles");
    assert!(
        !out.js.contains("use:pagination"),
        "no use:pagination stamp survives (the brain is the controller):\n{}",
        out.js
    );
    assert!(
        !out.js.contains("@ship/sail-runtime/actions"),
        "no runtime action import (pagination action retired):\n{}",
        out.js
    );
    assert!(
        out.js.contains("mountComponent"),
        "mount-form via mountComponent:\n{}",
        out.js
    );
    assert!(
        out.js.contains("stdlib/pagination/pagination.sail")
            && out.js.contains("stdlib/pagination/page-item.sail"),
        "both widget modules are imported:\n{}",
        out.js
    );
    assert!(
        out.js.contains("instance.page") && out.js.contains("instance.select"),
        "page-item takes page + select as props from the host instance (signal cell):\n{}",
        out.js
    );
}

#[test]
fn pagination_host_keeps_navigation_landmark_and_label() {
    let out = compile_element(PAGINATION_SRC, "pagination").expect("pagination compiles");
    assert!(
        out.structure_html.contains(r#"role="navigation""#),
        "host anatomy role navigation survives the brain:\n{}",
        out.structure_html
    );
    assert!(
        out.structure_html.contains(r#"aria-label="Pagination""#),
        "the static label replaces the retired action's runtime aria-label:\n{}",
        out.structure_html
    );
}

#[test]
fn page_item_binds_current_and_click_to_controller() {
    let out = compile_element(PAGE_ITEM_SRC, "page-item").expect("page-item compiles");
    let binds = out
        .controller_attaches
        .iter()
        .filter(|a| {
            matches!(
                a,
                sail_compiler::check::controller_attach::ControllerAttach::Bind { .. }
            )
        })
        .collect::<Vec<_>>();
    assert!(
        binds.iter().any(|a| format!("{a:?}").contains("current")),
        "item stamps `current` from the controller (lowers to aria-current):\n{binds:?}"
    );
    assert!(
        out.controller_attaches.iter().any(|a| {
            matches!(
                a,
                sail_compiler::check::controller_attach::ControllerAttach::On {
                    event,
                    handler,
                    ..
                } if event == "click" && handler == "onSelect"
            )
        }),
        "click resolves through the controller select (parent signal):\n{:?}",
        out.controller_attaches
    );
    assert!(
        out.skin_css.contains("s-current"),
        "when {{ current }} paint classes stay on the item:\n{}",
        out.skin_css
    );
}

#[test]
fn pagination_widget_has_no_action_manifest_row() {
    let src = PAGINATION_SRC;
    assert!(
        !src.contains("use:pagination"),
        "host template carries no use:pagination:\n{src}"
    );
    let manifests = sail_compiler::stdlib::sources();
    let (_, pag) = manifests
        .iter()
        .find(|(t, _)| *t == "pagination")
        .expect("pagination stdlib entry");
    assert!(
        pag.contains("<controller>"),
        "pagination.sail owns a controller:\n{pag}"
    );
    let (_, item) = manifests
        .iter()
        .find(|(t, _)| *t == "page-item")
        .expect("page-item stdlib entry");
    assert!(
        item.contains("<controller>"),
        "page-item.sail owns a controller:\n{item}"
    );
}
