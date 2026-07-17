<!--
  <avatar> — L1 language element (element mode). A circular/square identity
  frame that shows an image, an icon (simpleicons slug), or initials. The frame
  is the SKIN (this file); the image/initials render + the broken-image → initials
  fallback are the BEHAVIOR, delivered by the toolchain action `use:avatar`
  (packages/sail/runtime/src/actions/avatar.ts) — no per-instance controller.

  This is the first fully-behavioral compiles-away L1 element: the tag lowers to
  a bare <span class="sail-avatar">, `use:avatar` lowers to a runAction() at the
  use-site (action auto-imported, tree-shaken, shared across instances). The
  props (src/icon-slug/name/initials) merge onto the root as attributes, which
  the action reads — so `<avatar src="…" name="Ada Lovelace">` keeps its API,
  compiles away, and keeps its auto-fallback.

  Faithful to the feat/next sail-avatar baseline for shape/size/tone/gradient;
  the host/.root split collapses onto the single root span (light DOM).
-->

<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'md'
    shape: 'round' | 'square' = 'round'
  }
</scope>

<style>
  .sail-avatar {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;
    flex: none;
    overflow: hidden;
    user-select: none;
    /* Round is the default — a perfect circle (base, since a defaulted shape
       isn't reflected; square overrides below). sx has no 50% utility. */
    border-radius: 50%;
    font-family: var(--font-body);
    font-weight: var(--font-weight-bold);
    letter-spacing: 0.02em;
    text-transform: uppercase;
    line-height: 1;
    color: var(--foreground);
    /* Default (no branded tone) fill: the brand gradient. Branded tones below
       override it with a solid fill; default/muted keep the gradient. */
    background: linear-gradient(135deg, var(--primary), var(--accent));
    box-shadow: inset 0 0 0 1px var(--border), inset 0 1px 0 var(--background);
    /* Size: md base; others override. */
    width: var(--space-7);
    height: var(--space-7);
    font-size: var(--text-base);
  }
  .sail-avatar[hidden] { display: none; }
  .sail-avatar[shape='square'] { border-radius: var(--radius-sm); }

  .sail-avatar[size='xs'] { width: 1.25rem;             height: 1.25rem;             font-size: var(--text-xs); }
  .sail-avatar[size='sm'] { width: var(--control-h-sm); height: var(--control-h-sm); font-size: var(--text-xs); }
  .sail-avatar[size='md'] { width: var(--space-7);      height: var(--space-7);      font-size: var(--text-base); }
  .sail-avatar[size='lg'] { width: var(--control-h-lg); height: var(--control-h-lg); font-size: var(--text-lg); }
  .sail-avatar[size='xl'] { width: var(--control-h-xl); height: var(--control-h-xl); font-size: var(--text-2xl); }

  .sail-avatar[tone='primary']     { background: var(--primary);           color: var(--primary-foreground); }
  .sail-avatar[tone='accent']      { background: var(--accent);            color: var(--accent-foreground); }
  .sail-avatar[tone='info']        { background: var(--tone-info-bg);      color: var(--info); }
  .sail-avatar[tone='success']     { background: var(--tone-success-bg);   color: var(--success); }
  .sail-avatar[tone='warning']     { background: var(--tone-warning-bg);   color: var(--warning); }
  .sail-avatar[tone='destructive'] { background: var(--tone-destructive-bg); color: var(--destructive); }

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    padding: 10%;
    box-sizing: border-box;
  }
</style>

<span use:avatar></span>
