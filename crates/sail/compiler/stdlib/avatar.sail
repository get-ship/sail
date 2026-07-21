<!--
  <avatar> — identity frame (image/icon/initials). use:avatar owns render/fallback.

  Host paint: typeface/overflow/select/align on sx. Residue: size channels,
  gradient default fill, tone matrix, inset ring shadow, shape, img anatomy,
  letter-spacing 0.02em, flex center geometry.
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
    /* residue: text-transform governed by channel; keep literal here, not in sx */
    align-items: center;
    justify-content: center;
    flex: none;
    /* Round (circle) is the default — uses --shape-pill (9999px). */
    border-radius: var(--shape-pill);
    letter-spacing: 0.02em; /* residue */
    /* Default fill: brand gradient. Branded tones override below. */
    background: linear-gradient(135deg, var(--primary), var(--accent));
    box-shadow: inset 0 0 0 1px var(--border), inset 0 1px 0 var(--background);
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

<span use:avatar sx="font-body font-bold leading-none text-foreground overflow-hidden select-none align-middle"></span>
