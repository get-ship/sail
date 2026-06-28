---
title: "class: and style: directives"
description: "Toggle classes and CSS variables from signals."
group: "Tutorial"
order: 14
slug: "14-class-style"
prev: "13-refs"
next: "15-onmount-oncleanup"
chapter: "composition"
---

Two attribute directives that take the boilerplate out of conditional
styling:

- `class:foo={expr}` — adds the `foo` class when `expr` is truthy,
  removes it otherwise.
- `style:--my-var={expr}` — sets the CSS variable `--my-var` on the
  element to `expr`'s value.

```sui
<li
  class:done={item.done}
  class:overdue={item.dueDate < Date.now()}
  style:--row-height={'32px'}
>
  {item.label}
</li>
```

Both are reactive — re-evaluate whenever any signal they read
changes — and both compose with regular `class=` and `style=`
attributes (they merge, they don't replace).

Why two directives instead of just templating into `class={…}`:

- **`class:` is fine-grained.** Other classes set by the runtime
  (state classes, theme classes) aren't clobbered. The reconciler
  only adds/removes the named class.
- **`style:` for CSS custom properties** keeps the design-token API
  reachable from data. Components reading `var(--row-height)` pick
  up the value without you re-writing their CSS.

## Try it

The example renders a list of todos with overdue + done states. Try:

1. **Add a `class:selected={item.id === selectedId()}`** that
   highlights the currently-clicked row. Track `selectedId` in a
   signal.
2. **Style a progress bar**: add a `<div class="bar">` whose width
   you control via `style:--pct={pct() + '%'}`, with the CSS reading
   `width: var(--pct, 0%)`.
3. **Mix with `class=`**: the static `class="todo-row"` should
   persist next to whatever `class:` toggles. Verify by inspecting
   the rendered element.
