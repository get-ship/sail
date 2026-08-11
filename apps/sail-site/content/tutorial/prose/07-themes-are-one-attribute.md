---
title: "Themes are one attribute"
description: "Components read tokens, never colors — so data-theme repaints everything, and previews pin their own table."
order: 7
slug: "07-themes-are-one-attribute"
prev: "06-motion-is-a-directive"
next: "08-the-compiler-talks-back"
chapter: "system"
---

Nothing in the last six lessons named a color. Badges asked for a
*tone*, cards asked for a *surface*, sx asked for *tokens*. That
discipline pays off here: because every component reads tokens and only
tokens, a theme is **one attribute on the root element** —
`data-theme` — and swapping it repaints the entire page.

```
import { applyTheme, persistTheme, readStoredTheme } from '@ship/tokens/themes'

applyTheme('console')     // sets data-theme + data-mode
persistTheme('console')   // survives reload
applyTheme(readStoredTheme())  // on boot
```

`applyTheme` is the **only writer** of theme state. No component is
consulted, none opts in — a theme is a `[data-theme]` block over the
token ladder, and everything downstream just reads the new values.

## The tone contract is what makes it hold

`tone="destructive"` never meant "red"; it meant *the destructive slot
of the current theme's table*. That is why the tone row at the bottom
of the solved example survives every theme change with its meaning
intact — Console renders it in phosphor greens and ambers, Paper in
warm print inks, and "destructive" stays legible in both.

## Previews pin their own table

Sometimes one page needs two themes — a picker showing you Console
while the page stays Paper. That is `theme`: the same
token table, scoped to a subtree, without touching the reader's choice.
The two panels in the example are byte-identical markup; only the
attribute differs.

```
<column theme="paper">…</column>
<column theme="console">…</column>
```

## Try it

1. **Fix pick()** (TODO 1). Import `applyTheme` and `persistTheme`, call
   both, and click through all the themes. Reload — your pick held.
2. **Pin the second panel** (TODO 2). Add
   `theme="console"` and watch the panels disagree while
   the page follows the picker.
3. **Audit the panels.** Change the page theme with both panels pinned:
   nothing inside them moves. Remove a pin: that panel follows the page
   again. One attribute, scoped by where you put it.
4. **Find the seam.** Try to make the pinned card's badge red *without*
   a tone. You cannot — there is no color surface to grab. That missing
   escape hatch is the entire theming guarantee.
