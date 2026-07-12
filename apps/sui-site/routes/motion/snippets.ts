// Code snippets for the motion page. Kept OUT of the controller so the compiler's
// scope scan isn't fooled by `return {…}` / `signal(…)` patterns.
//
// Every SUI sample here is COMPILED BY THE BUILD (see
// crates/sui/compiler/tests/site_snippets_compile.rs). This page replaced
// /transitions, which taught `transition:` / `in:` / `out:` — three directives and
// a second animation system — for months after the compiler stopped accepting any
// of them. The guard exists so that cannot happen twice.
//
// The vocabulary below is the compiler's: 8 triggers (crates/sui/lint —
// MOTION_TRIGGERS), 5 properties (MOTION_PROPERTIES), the reserved `stagger` key,
// and 9 presets (crates/sui/lint/animation-presets.json). If you are adding to this
// page, read those, not this comment.
export const code = {
  // The whole idea, in one directive.
  intro: `<!-- one directive, one system. The trigger says WHEN; the body says WHAT. -->
<card motion:hover={{ scale: 'spring:gentle' }}>
  Lift on hover.
</card>`,

  interaction: `<!-- INTERACTION — the browser already knows when these happen. -->
<button
  motion:hover={{ scale: 'spring:gentle' }}
  motion:press={{ scale: 'press' }}
  motion:focus-visible={{ scale: 'spring:snappy' }}
>
  Press me
</button>`,

  lifecycle: `<!-- LIFECYCLE — enter when it mounts, exit when it leaves. The exit
     actually delays removal until the animation finishes; that is the whole
     reason a framework has to be involved at all. -->
{#if open()}
  <card
    motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}
    motion:exit={{ opacity: 'fade-out' }}
  >
    A panel that arrives and leaves.
  </card>
{/if}`,

  timeline: `<!-- TIMELINE — driven by scroll position, not by time. These compile to
     native CSS animation-timeline: the main thread is not involved, and there is
     no JavaScript in the frame loop at all. -->
<section motion:view={{ opacity: 'fade', translate-y: 'spring:gentle' }}>
  Fades up as it scrolls into view.
</section>

<div motion:scroll={{ scale: 'spring:deliberate' }}>
  Tracks the scroll progress of its container.
</div>`,

  stagger: `<!-- stagger is RESERVED, not a property: it describes the relationship
     BETWEEN siblings (the delay step), not anything about any one of them. -->
<ul>
  {#for item of items() key={item.id}}
    <li motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy', stagger: '0.04s' }}>
      {item.label}
    </li>
  {/for}
</ul>`,
}
