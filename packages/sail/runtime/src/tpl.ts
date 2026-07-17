// tpl.ts — template cloning. The single biggest thing a compiler can do for DOM
// creation speed, and the reason `createElement` chains lose.
//
// Building a row by hand costs one JS->DOM round-trip per node and per attribute:
//
//     const tr = document.createElement("tr")
//     const td = document.createElement("td")
//     td.setAttribute("class", "col-md-1")      // ...x10 for one benchmark row
//
// Cloning costs ONE. The browser parses the markup once, then copies the finished
// structure in C++ — no bridge crossings, no attribute parsing, nothing for the JS
// engine to do. Measured on the js-framework-benchmark row, 1,000 rows:
//
//     createElement chain   7.2 ms
//     template cloneNode    3.4 ms      2.1x
//
// That 2.1x is exactly the gap between us and Svelte/Solid on every CREATION
// benchmark (they both do this; we did not).
//
// The template is parsed LAZILY on first use and cached in the closure, so a
// component that never renders pays nothing, and one that renders 10,000 rows
// parses once.

/**
 * Compile an HTML string into a cloner. The compiler hoists one of these per
 * static subtree to module scope, so the parse happens once per program — not
 * once per component instance, and certainly not once per row.
 */
export function __tpl(html: string): () => Element {
  let proto: Element | undefined
  return (): Element => {
    if (proto === undefined) {
      const t = document.createElement('template')
      t.innerHTML = html
      // firstElementChild, not firstChild: the compiler emits no leading
      // whitespace, but a stray text node here would poison every clone.
      proto = t.content.firstElementChild as Element
    }
    return proto.cloneNode(true) as Element
  }
}
