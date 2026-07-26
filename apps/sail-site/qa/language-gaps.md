# Language gaps — not expressible (or not only) in Sail author surface

**Date:** 2026-07-26 · **Tip stamp:** `@ship/sail-*@0.3.0` (Verdaccio)  
**Branch:** `job/style-synthesis`

Things product/apps still need that **are not** pure closed language (Style / layout / blocks / query / Host). Tracked so polyrepo upgrades don't pretend the language covers everything — especially **marketing visual parity**.

## Host / platform (outside `.sail`)

| Gap | Why language can't own it | Marketing impact |
|-----|---------------------------|------------------|
| Deck/Astro shell, HTML shell, asset hashing | Site framework | **High** — shell CSS/JS outside Sail |
| Importmap / framework JS bake (`deck build-assets`) | Host tooling | Bundle swap can change paint if tokens/skins drift |
| Theme persistence / `data-theme` boot | Runtime theme hatch + host | Wrong theme = visual regression |
| Fonts woff2 delivery | sail-base `@font-face` + bundler rewrite | Body font / FOUT |
| SSR/prerender HTML snapshots | Host | Hydration mismatch risk |

## Web substrate (l1-web, intentional)

| Gap | Notes |
|-----|--------|
| Native `popover` / `<dialog>` / top-layer | L1 wraps platform; not a Sail Dialog IR |
| Pointer/keyboard/focus-trap/scroll-lock | Actions + scroll-lock module |
| Spatial actions (rect, pin, zoom-pan, port-edge, world-transform) | Layout metrics / matrix — not Style |
| Virtualization (`use:virtualRow`) | TanStack core, not language |
| WAAPI / Motion attach implementation | Style `.enter`/`.exit` is author surface; engine is web |
| CSS PE catalog (`spin` / shimmer / pulse) | Platform chrome loops, not multi-step `timeline` |
| Form control binding (`bind:value`/`checked`) | DOM form model |

## Incomplete / dual surfaces (migrate carefully)

| Gap | Risk |
|-----|------|
| Package rename `@ship/sui-*` → `@ship/sail-*` | Marketing still on **sui** names — **do not force until visual gate** |
| Extension `.sui` → `.sail` | Same |
| Host class stamps for skins vs `data-sail-host` for actions | CSS still class-keyed |
| `style.animate(spin)` naming | PE disguised as animate |
| Control-flow codemod + template-literal `key={\`…\`}` | Broken one-shot migrate on tree-grid (fixed); re-check complex keys |
| Mutation state shape | Still evolving; don't freeze apps on uncommitted details |
| Public `use:` 12 vs internal stdlib stamps | Apps must not invent L1-private actions |

## Visual regression policy (marketing)

1. **Keep marketing on published `@ship/sui-*` until a dedicated cutover PR** with screenshot/playwright gate. Publishing `@ship/sail-*@0.3.0` does **not** swap marketing deps by itself.
2. When cutting marketing: pin exact `0.3.0`, rebuild bundles from this tip, compare landing + docs + arcade screenshots.
3. Tokens/themes/fonts are the highest shared paint risk — compare `tokens@0.3.0` vs prior `sui` token set before flipping marketing.

## Language *does* cover (use for upgrades)

- Style product + `when` / `match` / motion methods  
- Element control-flow (`<if>` / `<for>` / …)  
- Layout elements + participation attrs  
- `query` / `mutation` (no `data()`)  
- Flat `Host` inject  
- Closed tokens (no raw CSS authoring under strict policy)
