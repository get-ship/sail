// Code snippets for the motion page. Kept out of the controller so the compiler's
// scope scan isn't fooled by `return {…}` / `signal(…)` patterns. Every SUI sample
// here is compiled by the build (crates/sail/compiler/tests/site_snippets_compile.rs).
//
// The vocabulary is the compiler's: MOTION_TRIGGERS and MOTION_PROPERTIES in
// crates/sail/lint, presets in crates/sail/lint/animation-presets.json. Read those
// when adding to this page.
export const code = {
  loop: `<!-- AMBIENT — alternates forever; stagger is phase; off under
     prefers-reduced-motion. -->
<div sx="w-4 h-4 rounded-full bg-primary"
     motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>`,

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

  timeline: `<!-- TIMELINE — driven by scroll POSITION, not by time. These attach a
     native ViewTimeline, which the browser drives on the compositor: no
     JavaScript runs while you scroll, only once at mount to set it up. -->
<section motion:view={{ opacity: 'fade', translate-y: 'spring:gentle' }}>
  Fades up as it scrolls into view.
</section>

<div motion:scroll={{ scale: 'spring:deliberate' }}>
  Tracks the scroll progress of its container.
</div>`,

  stagger: `<!-- stagger is RESERVED, not a property: it describes the relationship
     BETWEEN siblings (the delay step), not anything about any one of them. -->
{#for n of items() key={n}}
  <card motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy', stagger: '0.05s' }}>
    {n}
  </card>
{/for}`,
}
