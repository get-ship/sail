<!--
  <empty-state> — L1 language element (element mode). A centered placeholder for
  empty lists / missing content: an icon tile, a title, description copy, and an
  action row. Named slots (icon / title / action + the default description slot)
  project at compile time; bare tag lowers to <div class="sail-empty-state">.

  Faithful to the feat/next sail-empty-state baseline. Shadow-isms dissolved at L1:
  - host/.box split → collapsed onto the single root (the root IS the box).
  - reactive align={variant==='inline'?...} on the copy column → CSS variant rule.
  - reflect-empty="data-no-icon" (a JS slot-emptiness reflect) → `.icon:empty`
    (pure CSS — an empty icon slot leaves the tile empty, so it hides itself).
  - reactive role/aria-live (only the `connecting` variant is a live region) →
    the use:emptyState action, off a static role=note default.
  Icon tile tints per tone from the global [tone] channel; default/muted stay
  neutral (empty states read quiet by default).
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }
</scope>

<style>
  .sail-empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--es-gap);
    padding: var(--es-pad-y) var(--es-pad-x);
    text-align: center;
    font-family: var(--font-body);
    color: var(--muted-foreground);
    background: var(--es-bg);
    border: var(--es-border);
    border-radius: var(--es-radius);
    box-shadow: var(--es-shadow);
    /* size: md base; sm/lg override below */
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md);
    --es-desc-fs: var(--text-sm);
    --es-radius: var(--radius-lg);
    --es-border: 1px solid var(--border);
    --es-bg: var(--tone-muted-bg);
    --es-shadow: var(--shadow-xs);
  }

  .sail-empty-state[size='sm'] {
    --es-pad-y: var(--space-7); --es-pad-x: var(--space-4); --es-gap: var(--space-2);
    --es-icon: 16px; --es-tile: 36px; --es-title-fs: var(--text-base);
  }
  .sail-empty-state[size='lg'] {
    --es-pad-y: 96px; --es-pad-x: var(--space-6); --es-gap: var(--space-4);
    --es-icon: 28px; --es-tile: 64px; --es-title-fs: var(--text-lg); --es-desc-fs: var(--text-base);
  }

  .sail-empty-state[variant='placeholder'],
  .sail-empty-state[variant='connecting'] {
    --es-border: 0; --es-bg: transparent; --es-radius: 0; --es-shadow: none; --es-pad-y: var(--space-6);
  }
  .sail-empty-state[variant='inline'] {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: start;
    --es-border: 1px dashed var(--border);
    --es-bg: var(--tone-muted-border);
    --es-radius: var(--radius-md);
    --es-pad-y: var(--space-3);
    --es-pad-x: var(--space-4);
    --es-gap: var(--space-3);
  }

  .sail-empty-state .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-lg);
    background: var(--tone-bg);
    color: var(--tone-fg);
    line-height: 1;
    box-shadow: inset 0 0 0 1px var(--tone-ring);
  }
  .sail-empty-state:not([tone]) .icon,
  .sail-empty-state[tone='default'] .icon,
  .sail-empty-state[tone='muted'] .icon {
    background: var(--muted);
    color: var(--muted-foreground);
    box-shadow: none;
  }
  .sail-empty-state .icon:empty { display: none; }
  .sail-empty-state[variant='inline'] .icon {
    width: var(--es-icon); height: var(--es-icon);
    background: transparent; box-shadow: none;
  }
  .sail-empty-state .icon svg { width: var(--es-icon); height: var(--es-icon); }

  .sail-empty-state .copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    align-items: center;
    min-width: 0;
  }
  .sail-empty-state[variant='inline'] .copy { align-items: start; }

  .sail-empty-state .title {
    font-family: var(--heading-font);
    font-size: var(--es-title-fs);
    font-weight: var(--heading-weight);
    color: var(--foreground);
    line-height: var(--leading-tight);
  }
  .sail-empty-state[variant='connecting'] .title,
  .sail-empty-state[variant='placeholder'] .title {
    font-weight: var(--font-weight-medium);
    color: var(--muted-foreground);
  }
  .sail-empty-state .title:empty { display: none; }

  .sail-empty-state .desc {
    font-size: var(--es-desc-fs);
    max-width: 38ch;
    line-height: 1.55;
    color: var(--muted-foreground);
  }
  .sail-empty-state[variant='inline'] .desc { max-width: 60ch; }
  .sail-empty-state .desc:empty { display: none; }

  .sail-empty-state .actions {
    display: flex;
    gap: var(--space-2);
    justify-content: center;
    margin-top: var(--space-1);
    flex-wrap: wrap;
  }
  .sail-empty-state[variant='inline'] .actions {
    margin-top: 0;
    margin-inline-start: auto;
    justify-content: flex-end;
  }
  .sail-empty-state .actions:empty { display: none; }

  @media (forced-colors: active) {
    .sail-empty-state { border-color: CanvasText; }
    .sail-empty-state .icon { background: Canvas; color: CanvasText; }
  }
</style>

<div use:emptyState role="note" aria-live="off">
  <div class="icon" aria-hidden="true"><slot name="icon"></slot></div>
  <div class="copy">
    <div class="title"><slot name="title"></slot></div>
    <div class="desc"><slot></slot></div>
  </div>
  <div class="actions"><slot name="action"></slot></div>
</div>
