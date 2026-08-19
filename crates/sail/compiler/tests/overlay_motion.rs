//! Job D — motion on existing overlays.
//!
//! `transition: play(colors|surface|control)` is CSS. `enter` / `exit` /
//! `hover` / `loop` attach via `__sail_motion_attach` (`/motion`). Stamp
//! enter/exit `play()` on dialog, drawer, popover, dropdown, hover-card,
//! tooltip. Only those pages download motion.

use sail_compiler::{CompileMode, CompileOpts, compile, compile_element};

const OVERLAYS: &[&str] = &[
    "dialog",
    "drawer",
    "popover",
    "dropdown",
    "hover-card",
    "tooltip",
];

fn page_opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn stdlib_src(tag: &str) -> &'static str {
    sail_compiler::stdlib::sources()
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("{tag} registered"))
}

#[test]
fn element_mode_accepts_enter_exit_play() {
    let src = r#"
<scope>
  style host = {bg(overlay), animate: { enter: play(fade), exit: play(fade) }}
</scope>
<div style(host)>x</div>
"#;
    let el = compile_element(src, "probe")
        .expect("element mode: enter/exit play() rides the static splice, like use:");
    assert!(
        el.structure_html.contains("_sail_m_enter") && el.structure_html.contains("_sail_m_exit"),
        "enter/exit become compiler motion stamps, not a dynamic-attr reject: {}",
        el.structure_html
    );
}

#[test]
fn overlay_sources_stamp_enter_exit_play() {
    for tag in OVERLAYS {
        let src = stdlib_src(tag);
        assert!(
            src.contains("enter:") && src.contains("play(") && src.contains("exit:"),
            "{tag}: overlay Style stamps enter/exit play():\n{src}"
        );
        let el = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag}: {e:?}"));
        let stamped = el.structure_html.contains("_sail_m_enter")
            && el.structure_html.contains("_sail_m_exit");
        assert!(
            stamped,
            "{tag} structure carries enter/exit motion stamps: {}",
            el.structure_html
        );
    }
}

#[test]
fn overlay_page_downloads_motion() {
    for (tag, src) in [
        ("dialog", "<dialog>Hi</dialog>"),
        ("drawer", "<drawer>Hi</drawer>"),
        ("popover", "<popover>Hi</popover>"),
        ("dropdown", "<dropdown>Hi</dropdown>"),
        ("hover-card", "<hover-card>Hi</hover-card>"),
        (
            "tooltip",
            "<tooltip content=\"tip\"><button>i</button></tooltip>",
        ),
    ] {
        let js = compile(src, page_opts())
            .unwrap_or_else(|e| panic!("{tag} page: {e:?}"))
            .js;
        assert!(
            js.contains("__sail_motion_attach")
                && js.contains("from \"@ship/sail-runtime/motion\""),
            "{tag} page downloads /motion for enter/exit:\n{js}"
        );
        assert!(
            js.contains("\"enter\"") && js.contains("\"exit\""),
            "{tag} page wires enter and exit triggers:\n{js}"
        );
    }
}

#[test]
fn button_page_does_not_download_motion() {
    let js = compile("<button>Go</button>", page_opts())
        .expect("button page")
        .js;
    assert!(
        !js.contains("@ship/sail-runtime/motion") && !js.contains("__sail_motion_attach"),
        "only overlay pages download motion:\n{js}"
    );
}
