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
    /* Round (circle) is the default — uses --shape-pill (9999px) for consistency
       with pill shapes elsewhere. (50% was prior residue; square overrides.) */
    border-radius: var(--shape-pill);
    font-family: var(--font-body);
    font-weight: var(--font-weight-bold);
    letter-spacing: 0.02em;
    text-transform: uppercase;
    line-height: var(--leading-none);
    color: var(--foreground);
    /* Default (no branded tone) fill: the brand gradient. Branded tones below
       override it with a solid fill; default/muted keep the gradient. */
    background: linear-gradient(135deg, var(--primary), var(--accent));
    box-shadow: inset 0 0 0 1px var(--border), inset 0 1px 0 var(--background);
    /* Size channels (local): md base; size attrs reassign channels only.
       Channels cover w/h/fs so no duplicates on base. */
    --avatar-size: var(--space-7);
    --avatar-fs: var(--text-base);
    width: var(--avatar-size);
    height: var(--avatar-size);
    font-size: var(--avatar-fs);
  }
  .sail-avatar[hidden] { display: none; }
  .sail-avatar[shape='square'] { border-radius: var(--radius-sm); }

  .sail-avatar[size='xs'] { --avatar-size: var(--space-5); --avatar-fs: var(--text-xs); }
  .sail-avatar[size='sm'] { --avatar-size: var(--control-h-sm); --avatar-fs: var(--text-xs); }
  .sail-avatar[size='lg'] { --avatar-size: var(--control-h-lg); --avatar-fs: var(--text-lg); }
  .sail-avatar[size='xl'] { --avatar-size: var(--control-h-xl); --avatar-fs: var(--text-2xl); }

  .sail-avatar[tone='primary']     { background: var(--tone);           color: var(--tone-contrast); }
  .sail-avatar[tone='accent']      { background: var(--tone);            color: var(--tone-contrast); }
  .sail-avatar[tone='info']        { background: var(--tone-bg);      color: var(--tone-fg); }
  .sail-avatar[tone='success']     { background: var(--tone-success-bg);   color: var(--success); }
  .sail-avatar[tone='warning']     { background: var(--tone-warning-bg);   color: var(--warning); }
  .sail-avatar[tone='destructive'] { background: var(--tone-bg); color: var(--tone-fg); }

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
