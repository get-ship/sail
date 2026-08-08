# RESULT — idiomatic fine-comb packages-sail-1

**Date:** 2026-08-08  
**Branch:** `job/idiomatic-packages-sail-1`  
**Rundoc:** `RUNDOC-IDIOMATIC-CORPUS-FINECOMB.md`  
**Job:** `IDIOMATIC-BATCH-PACKAGES-SAIL-1`  
**Push:** none  

## Files (12 product `.sail` + supporting)

| # | File | Notes |
|---|------|-------|
| 1 | `packages/sail/runtime/example/sail-app/src/sail-app.sail` | Tip Style + layout keywords; scrub `data-*`; drop unused `props:`; named Style scope |
| 2 | `packages/sail/tutorial/.../02-styling-is-sx/solved/tut-02-usage-card.sail` | Comment: Style product (not sx); folder name kept |
| 3 | `packages/sail/tutorial/.../02-styling-is-sx/starter/tut-02-usage-card-starter.sail` | Already tip Style TODOs — compile gate only |
| 4 | `packages/sail/tutorial/.../04-templates-react/solved/tut-04-release-console.sail` | Readable match arms; tip `style({ animate: enter })` |
| 5 | `packages/sail/tutorial/.../04-templates-react/starter/tut-04-release-console-starter.sail` | Fixed broken match TODO comment; tip Style TODO text |
| 6 | `packages/sail/tutorial/.../05-data-and-forms/solved/tut-05-team-directory.sail` | Format scrub; import → compositions |
| 7 | `packages/sail/tutorial/.../05-data-and-forms/starter/tut-05-team-directory-starter.sail` | Import → compositions; TODOs preserved |
| 8 | `packages/sail/tutorial/.../06-motion-is-a-directive/solved/tut-06-notification-tray.sail` | **Was broken** body Style stmts → tag `style(name, { animate })` |
| 9 | `packages/sail/tutorial/.../06-motion-is-a-directive/starter/tut-06-notification-tray-starter.sail` | Tip animate TODOs; remove stray `})`; compile green |
| 10 | `packages/sail/tutorial/.../07-themes-are-one-attribute/solved/tut-07-theme-lab.sail` | **P** `<partial pinned-preview>` for dual panels |
| 11 | `packages/sail/tutorial/.../09-review-like-an-owner/solved/tut-09-inbox.sail` | **Was broken** `messageRow.enter` body stmt → tag Style; format |
| 12 | `packages/sail/tutorial/.../09-review-like-an-owner/starter/tut-09-inbox-starter.sail` | Style attach fixed; five flaws still plantable + compile |

### Supporting (not in ≤12 product count)

| Path | Why |
|------|-----|
| `packages/sail/tutorial/src/compositions/tut-05-directory-client.ts` | X1 — legal `src/compositions/` home for query mock |
| `packages/sail/tutorial/src/compositions/tut-09-inbox-client.ts` | same |
| `packages/sail/tutorial/src/prose/05-data-and-forms.md` | import path match |
| `packages/sail/tutorial/src/prose/06-motion-is-a-directive.md` | tip Style animate; scrub fx/sx leftovers |
| `packages/sail/tutorial/src/prose/09-review-like-an-owner.md` | tip animate wording |

**Not renamed:** lesson folder `02-styling-is-sx` (slug/prose links). Style product taught inside files + prose title already “Styling is Style”.

## Smell table (all 12 — required)

| File | Smells | Action taken | Needs human? |
|------|--------|--------------|--------------|
| `sail-app` | `data-*` stamps not matching index.css classes; unused `props:`; raw CSS-var tones; fat kanban controller; strategy header | Style object scope + layout keywords; remove data-*/props; keep DnD smoke intent | low — external index.html CSS now largely unused (light-DOM demo residual); module-level `nextId`/DB for demo ok |
| `tut-02` solved | comment said “Style on sx” | Scrub to Style product | no |
| `tut-02` starter | bare structure (intentional) | none | no |
| `tut-04` solved | match arms jammed on one line | format | no |
| `tut-04` starter | broken HTML comment inside match; TODO 5 vs key contradiction | fixed comment; keep key (lint requires); TODO teaches remove/restore | no |
| `tut-05` solved | import convention on co-located `.ts` client; messy whitespace | move client to compositions; format | no |
| `tut-05` starter | same import; intentional incomplete match | import fix; TODOs kept | no |
| `tut-06` solved | **non-compiling** fluent `name.enter/.loop` as body statements (migration residue) | tip `style(name, { animate: … })` | no |
| `tut-06` starter | stray `})`; mangled TODO 2 press comment | scrub + tip TODOs | no |
| `tut-07` solved | duplicated theme preview panels | same-file `<partial>` | no |
| `tut-09` solved | **non-compiling** `messageRow.enter` body stmt; import convention | tag Style; compositions import | no |
| `tut-09` starter | same body Style; free-floating error arm no longer parses; import | Style attach; empty `{loading =>}` blank hole preserves checklist; other 4 flaws kept | no |

### Needs-human roll-up

1. **Lesson slug `02-styling-is-sx`** still says sx in path/slug (prose title already Style). Full rename needs site routes + gen-tutorial + prev/next links — out of this batch’s “no invent renames unless easy.”
2. **`sail-app` index.html** still carries hand CSS for classes the component no longer emits — demo host residual; optional follow-up to delete dead CSS or re-theme via tokens only.
3. **Prose 06 residual** may still mention older motion vocabulary in later sections not fully re-edited; opener + animate examples tip.

## Checklist hits

| | |
|--|--|
| **S** | Controllers already signal/derived/query; sail-app retains demo state graph |
| **Y1** | Tip object Style + named `style name =` throughout; motion on `animate` channel |
| **Y2** | Named Style for reused pills/rows/cards; sail-app scope names |
| **Y3 bind** | sail-app swatch `bg(bind(swatchColor))` retained |
| **P** | tut-07 `pinned-preview` partial |
| **C** | tut-09 solved keeps `<empty-state>`; starter still plants improvised div |
| **X1** | tutorial mock clients → `src/compositions/` |
| **X2** | sail-app scrubbed `data-*` |
| **X3** | comment scrubs (sx→Style, migration essays, strategy headers) |
| **X9** | no new mega-controllers; clients stay out of controllers |

## Gaps reported

| Gap | Notes |
|-----|--------|
| Lesson dir slug still `02-styling-is-sx` | Product teach Style inside; rename needs coordinated site/gen |
| Free-floating match arms no longer parse | Lesson 09 starter now uses empty loading arm for blank-hole flaw (still “no loading UI”) |
| `<for>` without `key=` is hard error | Starter 04 cannot ship keyless for pedagogy; TODO teaches remove/restore |
| sail-app light-DOM CSS host | precompile patches `shadow:false`; host CSS not Style |

## Commands run

```text
PATH=release-integrate/target/debug:$PATH
sail-compiler build --component-name <tag> <path>   # ×12 — all compiled 1 file(s)
sail render <file> --theme ship --mode light --viewport 800x600 \
  --out /tmp/web-qa/packages-sail-1/screenshots/<name>-after-light.png --json
# tut-02, tut-01 (ref), tut-07 afters
```

## Visual regression (takumi)

| Surface | Path | Notes |
|---------|------|-------|
| tut-02 after | `/tmp/web-qa/packages-sail-1/screenshots/tut-02-after-light.png` | static Style card |
| tut-07 after | `/tmp/web-qa/packages-sail-1/screenshots/tut-07-after-light.png` | partial expand may be limited under static render |
| Controllers (04–06, 09, sail-app) | skipped honest before/after | need runtime query/DnD/motion; compile green is the gate |

Before shots omitted: several sources **did not compile** pre-fix (06/09 body Style), so pixel-before would be empty/misleading.

## Non-goals respected

- No lesson-directory renames  
- No feat/next / no push  
- Starter pedagogical TODOs / planted flaws kept  
- No inventing language APIs
## Commit

Local on `job/idiomatic-packages-sail-1` — **DO NOT push.**

**Feature commit (fine-comb):** `5000dc83` (`5000dc83bbd04c277cf500c81ab8b708afb1f8c6`)

**Branch tip:** filled by tip-SHA chore.

