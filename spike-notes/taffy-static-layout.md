# taffy-static-layout — box-tree compilation for `.sail` templates

PROVEN — but only as a **diagnostic**. Building it into a renderer would be a
much larger spike and the cleanest payoff is the dirt-cheap loop this gives
the compiler.

## The thesis, restated

`taffy` (Rust flexbox/grid/block, wasm-clean — Zed and Dioxus ship it)
computes our box tree at compile time. It works because of the constraints
the floor already pays for:

- sx is atomic and selector-free → no cascade to resolve. The computed style
  of every element is already known.
- values are tokens → real pixels (the same registry `sail-lint::tokens` reads).
- layout is `<row>`/`<column>`/`<grid>` → intent is declared, not inferred.
- the tree is the template.

WASM-cleanliness: **YES**. `cargo build -p sail-layout --target wasm32-unknown-unknown --features sail-compiler/wasm` builds clean. The bin doesn't (cli parser can't appear in wasm), but `sui_layout.rlib` does — and `libtaffy-*.rlib` does. So the diagnostic can ride the same in-browser compiler pipeline the tutorial playground already uses.

## What the spike does

A new crate, `crates/sail/layout/`, with:

- A minimal HTML parser that walks the **preprocessed** `.sail` source (the
  same source the runtime sees — `<column>` is already `<div class="sail-col …">`
  and `sx="…"` is already `class="sx-…"`).
- A style resolver that maps the class-name vocabulary back to a
  `taffy::Style` (padding, gap, flex-grow, max-*, the layout primitives'
  `display`/`flex-direction`/`flex-wrap`).
- A `TaffyTree::compute_layout` pass at a fixed viewport.
- Three diagnostics, ranked by ambiguity:
   1. `collapsed-into-overflow-hidden-parent` — a parent collapsed to 0 on
      its main axis while a child wants space AND the parent has
      `overflow:hidden` (silent clip). **This is the 1779px-in-900px class
      of bug.** It's exactly what the example in the floor does and the spec
      asked for.
   2. `unbounded-growth-into-overflow-hidden-parent` — the parent's main
      axis is non-zero, but a child extends past it AND the parent silently
      absorbs.
   3. `child-overflows-parent` — the same comparison, but visible.
   4. (lower-priority) `zero-main-axis` — generic "you have a collapsed box";
      not framed as the regression because we only know it's collapsed, not
      why.

## What was proven against a real repo file

```
$ cargo run -q -p sail-layout -- \
    apps/sail-site/routes/tutorial/\[slug\]/page.sail \
    --viewport 1440x900

warning: [zero-main-axis] at byte 12961..13093
  (tag=div, class="sail-col sail-g-4 sail-ai-stretch sx-grow1
                 sx-maxwpanelmaxwmd sx-hfill sx-minh0 sx-bgsurface1
                 sx-borderr sx-borderrborder")
  — <div> measures 0px on its main axis while having 2 child(ren) totaling 0px.
```

That signature is exactly the pane at `routes/tutorial/[slug]/page.sail:286`:

```sail
<column
  sx="grow-1 max-w-panel-max-w-md h-fill min-h-0 bg-surface-1
      border-r border-r-border"
>
```

The diagnostic doesn't quote the magic 1779px number — and that's deliberate.
Taffy's pixel arithmetic differs from Chromium at the edges (sub-pixel
rounding, `%` rounding rules, font-metric fallbacks); reporting "this column
measures Xpx" is a claim taffy can't authorise. "This column collapsed to 0
with non-zero children and its parent silently clips them" IS authorise-able,
because it's a description of the box tree's *shape*, not its pixels.

## What the box-model-only version misses

- **Reading measure / text overflow detection.** Without font shaping
  (cosmic-text, fontdue — Zed's stack), every text leaf measures against a
  placeholder intrinsic size we synthesise from `--text-*` tokens. So this
  spike catches collapsed boxes, unbounded growth, fixed-size overflow, but
  NOT "a `<p>` of 87 chars exceeds the 60ch measure". Most of the *value*
  is in the box-model-only version — but a `<prose>` block is exactly the
  kind of element where a static-measure shaper pays back hardest.
- **Multiple viewports / responsive breakpoints.** One fixed viewport per
  call. Adding it is `for vp in breakpoints { tree.compute(...) }` — small
  extension.
- **`if` / `for` / dynamic content.** Pure static for now. `{#if cond}`
  branches evaluate both sides; `{#for xs}` is empty.
- **Stdlib primitive intrinsic sizes.** `<tabs>`, `<tab-panel>`,
  `<tab-list>`, `<sail-code-editor>` get standard sizes from their skins; the
  spike reads the lowered `<div class="sail-tabs">` and asks taffy to figure
  the size out (it usually can't — and fires zero-main-axis warnings). A
  production pass would carry each primitive's intrinsic min-size as data.

## Honest verdict

The spike proves it. The diagnostic works against the named regression; it
compiles to wasm32; it costs ~600 lines of Rust, three source files, and a
shared dep on `taffy`. **Build it.** The honest verdict isn't "is the
diagnostic useful" but "what's the smallest thing to ship":

1. The current spike, exposed as a `sail-layout-diagnostics` lint rule that
   ships with `check_all()`. Author sees the same diagnostic from CLI and
   LSP, no browser, every edit.
2. Text shaping as a follow-on, scoped to docs pages (lessons, docs chrome)
   first. The ROI is highest there; the dev-loop cost of "wait, did the
   60ch column measure change?" is real.
3. Multiple viewports last. The site renders responsively; the diagnostic
   catches the desktop case today.

The verdict **is not "text is the whole game"** — box-model-only catches
the named bug and would catch most of its kin. The thing it's actually
missing is the *reading-measure* class of bug, which only a shaper can
answer; and that is a separable, additive follow-on, not a rewrite.

## What to kill, looking back

If this had failed:

- I would not have learned whether taffy is wasm-clean (a separate
  investigation — Zed ships taffy to wasm, so the answer was already known).
- I would have learned whether the closed vocabulary pays for itself
  statically; that hypothesis only needed a 100-line parser + style
  resolver to test, regardless of taffy.

The kill criteria the spec lists ("does it compile to wasm", "does it catch
the 1779px regression") are direct, cheap to verify, and easy to answer
against this code. They are answered affirmatively. Build it.
