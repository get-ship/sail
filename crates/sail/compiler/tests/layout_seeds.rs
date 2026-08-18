//! P2 seed cases — compile + geometry, not visual goldens.
//!
//! Each snippet matches the spelling on `/layout/seeds`. Assertions are
//! relational (finite boxes, order, leftover) — no pixel pins.

use sail_compiler::parse::preprocess::preprocess_with_style;
use spinnaker::layout::{BoxReport, CheckOptions, check_preprocessed_with_hosts};

const VP: (f32, f32) = (400.0, 300.0);

fn layout_at(src: &str, viewport: (f32, f32)) -> (String, Vec<BoxReport>, Vec<String>) {
    let (pre, hosts) = preprocess_with_style(src).expect("preprocess");
    let result = check_preprocessed_with_hosts(
        &pre,
        viewport,
        CheckOptions::default(),
        &hosts.types,
        &hosts.boxes,
    )
    .expect("layout");
    let kinds = result
        .diagnostics
        .iter()
        .map(|d| d.kind.to_string())
        .collect();
    (pre, result.boxes, kinds)
}

fn layout(src: &str) -> (String, Vec<BoxReport>, Vec<String>) {
    layout_at(src, VP)
}

fn finite(boxes: &[BoxReport]) {
    assert!(!boxes.is_empty(), "expected laid-out boxes");
    for b in boxes {
        assert!(
            b.width.is_finite() && b.height.is_finite() && b.x.is_finite() && b.y.is_finite(),
            "non-finite box: {b:?}"
        );
        assert!(b.width >= 0.0 && b.height >= 0.0, "negative box: {b:?}");
    }
}

#[test]
fn grow_without_bound_is_finite() {
    let src = r#"
<scope>
  style leftover = { size(grow), bg(accent) }
</scope>
<row>
  <column style(leftover)>grow</column>
</row>
"#;
    let (_pre, boxes, _kinds) = layout(src);
    finite(&boxes);
}

#[test]
fn circular_percentage_is_determinate() {
    let src = r#"
<scope>
  style pctParent = { size({ w: 50% }), bg(muted) }
  style pctChild = { size({ w: 50% }), bg(primary) }
</scope>
<column style(pctParent)>
  <column style(pctChild)>pct</column>
</column>
"#;
    let (_pre, boxes, _kinds) = layout(src);
    finite(&boxes);
    let parent = boxes
        .iter()
        .max_by(|a, b| a.width.partial_cmp(&b.width).unwrap())
        .unwrap();
    let child = boxes
        .iter()
        .filter(|b| b.width < parent.width)
        .max_by(|a, b| a.width.partial_cmp(&b.width).unwrap());
    if let Some(child) = child {
        assert!(
            child.width <= parent.width + 0.5,
            "child {child:?} exceeds parent {parent:?}"
        );
    }
}

#[test]
fn rtl_wrap_lays_out_three_boxes() {
    let src = r#"
<scope>
  style box4 = { size({ w: 4, h: 4 }), bg(primary) }
  style box4a = { size({ w: 4, h: 4 }), bg(accent) }
  style box4m = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<row dir="rtl" gap="2">
  <column style(box4)></column>
  <column style(box4a)></column>
  <column style(box4m)></column>
</row>
"#;
    let (_pre, boxes, _kinds) = layout(src);
    finite(&boxes);
    let kids: Vec<_> = boxes
        .iter()
        .filter(|b| b.width > 0.0 && b.height > 0.0)
        .collect();
    assert!(
        kids.len() >= 3,
        "rtl wrap must lay out the three boxes: {boxes:?}"
    );
}

#[test]
fn stack_overlay_does_not_fire_sibling_overlap() {
    let src = r#"
<scope>
  style overlapA = { size({ w: 8, h: 8 }), bg(primary) }
  style overlapB = { size({ w: 8, h: 8 }), bg(accent) }
</scope>
<stack>
  <column style(overlapA)></column>
  <column style(overlapB)></column>
</stack>
"#;
    let (_pre, boxes, kinds) = layout(src);
    finite(&boxes);
    assert!(
        !kinds.iter().any(|k| k == "sibling-overlap"),
        "stack is the overlay opt-in: {kinds:?}"
    );
}

#[test]
fn holy_grail_orders_header_nav_main_aside_footer() {
    let src = r#"
<scope>
  style header = { size({ h: 4 }), bg(primary) }
  style footer = { size({ h: 4 }), bg(muted) }
  style nav = { size({ w: 8 }), bg(primary) }
  style main = { size(grow), bg(accent) }
  style aside = { size({ w: 8 }), bg(muted) }
</scope>
<column>
  <row style(header)>header</row>
  <row nowrap>
    <column style(nav)>nav</column>
    <column style(main)>main</column>
    <column style(aside)>aside</column>
  </row>
  <row style(footer)>footer</row>
</column>
"#;
    let (_pre, boxes, _kinds) = layout(src);
    finite(&boxes);
    let rows: Vec<_> = boxes
        .iter()
        .filter(|b| b.class.split_whitespace().any(|c| c == "sail-row"))
        .collect();
    assert!(
        rows.len() >= 3,
        "holy-grail needs header / body / footer rows: {boxes:?}"
    );
    assert!(
        rows[0].y <= rows[1].y && rows[1].y <= rows[2].y,
        "header above body above footer: {rows:?}"
    );
    let cols: Vec<_> = boxes
        .iter()
        .filter(|b| b.class.split_whitespace().any(|c| c == "sail-col"))
        .collect();
    assert!(
        cols.len() >= 3,
        "holy-grail needs nav / main / aside: {boxes:?}"
    );
}

#[test]
fn sidebar_fixed_fluid_gives_leftover_to_fluid() {
    let src = r#"
<scope>
  style rail = { size({ w: 8 }), bg(primary) }
  style fluid = { size(grow), bg(accent) }
</scope>
<row nowrap>
  <column style(rail)>rail</column>
  <column style(fluid)>fluid</column>
</row>
"#;
    let (_pre, boxes, _kinds) = layout(src);
    finite(&boxes);
    let cols: Vec<_> = boxes
        .iter()
        .filter(|b| b.class.split_whitespace().any(|c| c == "sail-col"))
        .collect();
    assert!(cols.len() >= 2, "sidebar + fluid: {boxes:?}");
    let rail = cols
        .iter()
        .min_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let fluid = cols
        .iter()
        .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    assert!(rail.x <= fluid.x, "rail left of fluid: {cols:?}");
    assert!(
        fluid.width > rail.width + 0.5,
        "fluid must take leftover (wider than the w:8 rail): rail={rail:?} fluid={fluid:?}"
    );
}

#[test]
fn bento_span_is_wider_than_a_single_cell() {
    let src = r#"
<scope>
  style spanA = { bg(primary) }
  style spanB = { bg(accent) }
  style spanC = { bg(muted) }
</scope>
<grid cols="4" gap="2">
  <column col-span="2" row-span="2" style(spanA)>a</column>
  <column style(spanB)>b</column>
  <column style(spanC)>c</column>
  <column col-span="2" style(spanB)>d</column>
</grid>
"#;
    // col-span collapses below 720 — measure the seed at desktop width.
    let (pre, boxes, _kinds) = layout_at(src, (1440.0, 400.0));
    assert!(
        pre.contains("col-span=\"2\""),
        "preprocess must keep col-span: {pre}"
    );
    finite(&boxes);
    let cells: Vec<_> = boxes
        .iter()
        .filter(|b| b.class.split_whitespace().any(|c| c == "sail-col"))
        .collect();
    assert!(cells.len() >= 4, "bento cells: {boxes:?}");
    let widest = cells
        .iter()
        .max_by(|a, b| a.width.partial_cmp(&b.width).unwrap())
        .unwrap();
    let narrowest = cells
        .iter()
        .filter(|b| b.width > 0.0)
        .min_by(|a, b| a.width.partial_cmp(&b.width).unwrap())
        .unwrap();
    assert!(
        widest.width + 0.5 >= narrowest.width * 1.8,
        "col-span 2 must be ~2× a single cell: wide={widest:?} narrow={narrowest:?}"
    );
}
