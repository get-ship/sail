# Sail patch of takumi-core 0.6.2 (RENDER-0)

Vendored from crates.io `takumi-core 0.6.2` with a minimal fix for a parley
panic when measuring non-empty text at tiny/zero content widths.

## Changes (`src/layout/inline.rs`)

1. **`create_inline_constraint`**: do not treat the full viewport height as an
   implicit text `max_height`. That forced the float-aware `BreakLines` path for
   every text measure; combined with min-content width 0 and real glyph runs it
   tripped parley 0.11's `layout_max_advance` / `line_max_advance` assert
   (`line_break.rs:453`). Only apply max height when `clamp_lines` is set.
   Also clamp finite width constraints to `>= 0`.

   **Removed optimization:** upstream always set
   `MaxHeight::Absolute(viewport_height)` when the viewport height was known
   (“reduce unnecessary calculation”). That arm is gone. Unclamped multi-line
   text may fully break/layout beyond the viewport during measure (more work;
   theoretically different content-box heights for extremely tall unclamped
   text). Paint-time `resolve_inline_max_height` still owns ellipsis and
   clamp-lines via the content-box height — only the measure-time viewport
   shortcut was removed.

   **Do not reintroduce viewport-as-max_height** for ordinary measures — that
   is the panic path. Prefer an upstream takumi/parley fix of the float-aware
   `BreakLines` assert if the optimization is restored later.

2. **`FloatLayoutState::update_breaker_line`**: clamp `layout_max` to `>= 0` and
   ensure `line_max <= layout_max` so the parley assert cannot fire if the
   float path is used (still active when `clamp_lines` / floats apply).

## Residual measure vs paint (clamp_lines)

When `clamp_lines` is set **and** the viewport height is known, measure still
uses `MaxHeight::HeightAndLines(viewport_height, lines)`. Paint-time
`resolve_inline_max_height` uses the **content-box** height instead. Sail does
not lower CSS `line-clamp` onto hosts yet (raw takumi regression covers panic
only). If/when line-clamp lands in the host, align measure max height with the
content box used at paint (or document intentional difference). Until then the
mismatch is low practical impact.

## Residual: unclamped tall text height

Unclamped multi-line text no longer stops measure at viewport height. A
regression fixture (`parley-unclamped-tall-text.sail` + golden test) pins that
measure completes without panic and yields a non-zero text height; it does not
assert pixel parity with upstream’s viewport-capped measure.

## Remove when

Upstream takumi (or parley) ships an equivalent fix; then drop the
`[patch.crates-io] takumi-core` entry in the workspace `Cargo.toml` and this
directory.
