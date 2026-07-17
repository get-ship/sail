// sail-table — light-DOM CSS for the slot/native variants.
//
// The slot/native variants project a consumer-authored `<table>` into the host
// LIGHT DOM (shadow CSS can't reach it), so the table styling is adopted onto
// the host document once per document (WeakSet-guarded). The data variant is
// shadow-rendered and styled in `sail-table.sui`'s `<style>` instead.
//
// Tokens only: density via the platform `--density-*` channel, selection via
// the prebuilt `--selected-bg`/`--tone-primary-bg` ramps, surfaces via
// `--surface-2`, dividers via `--divider`, mono via `--mono-font`, motion via
// the role-token easings — no `color-mix`/`oklch`/hex/`var(--x, fallback)`/
// hardcoded-font literals (design-system anti-smells, review-enforced).

const lightDomInstalled = new WeakSet<Document>()

export function installLightDomStyles(doc: Document): void {
  if (lightDomInstalled.has(doc)) return
  lightDomInstalled.add(doc)
  try {
    if (typeof CSSStyleSheet !== 'undefined' && 'replaceSync' in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet()
      sheet.replaceSync(SUI_TABLE_LIGHT_DOM_CSS)
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet]
      return
    }
  } catch {
    /* fall through to a <style> element */
  }
  const style = doc.createElement('style')
  style.setAttribute('data-sail-table', '')
  style.textContent = SUI_TABLE_LIGHT_DOM_CSS
  doc.head.appendChild(style)
}

export const SUI_TABLE_LIGHT_DOM_CSS = `
  sail-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  sail-table th,
  sail-table td {
    text-align: start;
    padding: var(--density-row-py) var(--density-row-px);
    vertical-align: middle;
    border-bottom: 1px solid var(--divider);
    font-size: var(--density-text);
    color: var(--foreground);
    box-sizing: border-box;
  }
  sail-table th {
    font-weight: var(--font-weight-semibold);
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
    user-select: none;
    background: var(--surface-2);
  }
  sail-table thead {
    background: var(--surface-2);
  }
  sail-table > tr:last-of-type > *,
  sail-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  sail-table th[align='right'], sail-table td[align='right'] { text-align: right; }
  sail-table th[align='center'], sail-table td[align='center'] { text-align: center; }

  sail-table .mono {
    font-family: var(--mono-font);
    font-size: var(--text-sm);
    letter-spacing: -0.01em;
  }
  sail-table .muted { color: var(--muted-foreground); }
  sail-table .null { color: var(--muted-foreground); font-style: italic; }

  sail-table > tr:not([slot='head']),
  sail-table tbody tr {
    transition: background-color var(--duration-fast) var(--ease-out);
  }
  sail-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sail-table:not([hoverable='false']) tbody tr:hover > * {
    background: var(--hover-bg);
  }

  sail-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  sail-table[striped] tbody tr:nth-child(even) > * {
    background: var(--tone-muted-bg);
  }
  sail-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sail-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: var(--hover-bg);
  }

  sail-table tr[data-selected] > *,
  sail-table tr[aria-selected='true'] > * {
    background: var(--selected-bg);
  }
  sail-table tr[data-selected] > *:first-child,
  sail-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  sail-table[sticky-header] thead th,
  sail-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky);
    background: var(--surface-2);
    box-shadow: 0 1px 0 0 var(--border);
  }

  sail-table th[data-sort-key],
  sail-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast) var(--ease-out);
  }
  sail-table th[data-sort-key]:hover,
  sail-table th.sortable:hover {
    color: var(--foreground);
  }
  sail-table th[aria-sort='ascending'],
  sail-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  sail-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width) var(--ring-color);
  }

  sail-table[data-empty] > table {
    display: none;
  }

  sail-table[sticky-first-col] th:first-child,
  sail-table[sticky-first-col] td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--card);
    border-right: 1px solid var(--border);
  }
  sail-table[sticky-first-col] thead th:first-child {
    z-index: 3;
    background: var(--surface-2);
  }
  sail-table[sticky-first-col] tbody tr:hover td:first-child {
    background: var(--hover-bg);
  }

  sail-table[gutter] th.sail-table-gutter-cell,
  sail-table[gutter] td.sail-table-gutter-cell {
    width: 40px;
    min-width: 40px;
    max-width: 40px;
    padding: 0;
    text-align: center;
    color: var(--muted-foreground);
    background: var(--surface-2);
    border-right: 1px solid var(--border);
    position: sticky;
    left: 0;
    z-index: 2;
  }
  sail-table[gutter] thead th.sail-table-gutter-cell { z-index: 4; }

  sail-table tr[data-row-id] > td:not(.sail-table-gutter-cell) {
    cursor: pointer;
  }

  sail-table .st {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }
  sail-table .st .dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-full);
    background: var(--muted-foreground);
  }
  sail-table .st .lbl { color: var(--foreground); }
  sail-table .st.ok .dot { background: var(--success); box-shadow: 0 0 0 3px var(--tone-success-bg); }
  sail-table .st.warn .dot { background: var(--warning); box-shadow: 0 0 0 3px var(--tone-warning-bg); }
  sail-table .st.err .dot { background: var(--destructive); box-shadow: 0 0 0 3px var(--tone-destructive-bg); }

  @media (forced-colors: active) {
    sail-table th, sail-table td { border-bottom-color: CanvasText; }
  }
`
