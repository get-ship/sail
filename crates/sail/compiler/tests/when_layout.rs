//! Layout enumeration: both `<when>` branches are in the static tree
//! so taffy can report overflow that exists only in the wide branch.

use sail_compiler::check::layout;

#[test]
fn layout_enumerates_when_wide_branch_overflow() {
    // Wide branch: fixed-width child larger than a narrow viewport parent
    // would still be checked because layout walks the source tree, not the
    // runtime-selected branch. Mobile else is tiny; wide has overflow.
    let src = r#"
<div data-viewport="400x200" style({ size({ w: 20, h: 24 }).overflow({ all: hidden }) })>
  <when {tablet}>
    <div style({ size({ w: 40, h: 10 }) })>wide-only-overflow</div>
    {else =>
      <div style({ size({ w: 16, h: 4 }) })>narrow-ok</div>
    }
  </when>
</div>
"#;
    // Must not panic / hard-fail on <when> trees.
    let _diags = layout::collect(src);

    // Preprocess + taffy HTML walker sees BOTH branches (enumeration contract).
    let pre = sail_compiler::parse::preprocess::preprocess(src).expect("preprocess");
    assert!(
        pre.contains("wide-only-overflow") && pre.contains("narrow-ok"),
        "both when branches remain in preprocessed source for layout"
    );
    let elems = sail_layout::html::parse_source(&pre);
    let texts: Vec<_> = elems.iter().map(|e| e.text.as_str()).collect();
    assert!(
        texts.iter().any(|t| t.contains("wide-only-overflow")),
        "layout HTML walker must enumerate the when-then branch: {texts:?}"
    );
    assert!(
        texts.iter().any(|t| t.contains("narrow-ok")),
        "layout HTML walker must enumerate the when-else branch: {texts:?}"
    );
    assert!(
        elems.iter().any(|e| e.tag == "when"),
        "when control tag must appear in the layout element forest"
    );
}
