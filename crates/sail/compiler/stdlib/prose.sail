<!--
  <prose> — the prose typography surface. Wraps RENDERED content (markdown
  output, doc blobs) and scopes the rhythm-between-elements sheet: heading
  cadence, paragraph/list spacing, links, inline and block code, blockquote,
  tables. Authored templates don't need it — bare h1/p take the type scale;
  this element exists because a runtime-rendered subtree has no other way to
  declare "this is prose".
-->

<scope>
</scope>

<style>
  .sail-prose {
    display: block;
    font-family: var(--font-body);
    color: var(--foreground);
    line-height: var(--leading-normal-plus);
  }

  .sail-prose h1, .sail-prose h2, .sail-prose h3,
  .sail-prose h4, .sail-prose h5, .sail-prose h6 {
    font-family: var(--heading-font);
    font-weight: var(--heading-weight);
    letter-spacing: var(--heading-tracking);
    line-height: var(--leading-tight);
    margin: var(--space-5) 0 var(--space-2);
  }
  .sail-prose h1 { font-size: var(--text-2xl); margin-top: var(--space-6); margin-bottom: var(--space-3); }
  .sail-prose h2 { font-size: var(--text-xl); margin-top: var(--space-6); margin-bottom: var(--space-3); }
  .sail-prose h3 { font-size: var(--text-lg); }
  .sail-prose h4, .sail-prose h5, .sail-prose h6 { font-size: var(--text-md); }

  .sail-prose p { margin: var(--space-3) 0; }

  .sail-prose a {
    color: var(--primary);
    text-decoration: underline;
    text-decoration-color: var(--selected-border);
    text-underline-offset: var(--ring-offset);
  }
  .sail-prose a:hover { text-decoration-color: var(--primary); }

  .sail-prose code {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--surface-1);
    border: 1px solid var(--border-subtle);
  }
  .sail-prose pre {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    background: var(--surface-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    line-height: var(--leading-normal-plus);
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }
  .sail-prose pre code { background: transparent; border: none; padding: 0; }

  .sail-prose blockquote {
    margin: var(--space-4) 0;
    padding: var(--space-2) var(--space-4);
    border-inline-start: 3px solid var(--primary);
    color: var(--muted-foreground);
    background: var(--tone-primary-bg);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  }

  .sail-prose ul, .sail-prose ol { margin: var(--space-3) 0; padding-inline-start: var(--space-5); }
  .sail-prose li { margin: var(--space-1) 0; }
  .sail-prose li::marker { color: var(--muted-foreground); }
  .sail-prose strong { font-weight: var(--font-weight-semibold); }

  .sail-prose table {
    border-collapse: collapse;
    width: 100%;
    margin: var(--space-3) 0;
    font-size: var(--text-sm);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .sail-prose thead th {
    background: var(--muted);
    color: var(--foreground);
    font-weight: var(--font-weight-semibold);
  }
  .sail-prose th, .sail-prose td {
    border-bottom: 1px solid var(--border-subtle);
    padding: var(--space-2) var(--space-3);
    text-align: left;
    font-variant-numeric: tabular-nums;
  }
  .sail-prose tbody tr:last-child td { border-bottom: 0; }
</style>

<div><slot></slot></div>
