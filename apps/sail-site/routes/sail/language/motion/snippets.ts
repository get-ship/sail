// Code snippets for the motion page. Kept out of the controller so the compiler's
// scope scan isn't fooled by `return {…}` / `signal(…)` patterns. Every Sail sample
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

  spin: `<!-- continuous rotate — one direction, never ping-pongs. -->
<div sx="w-8 h-8 rounded-md bg-primary" motion:loop={{ rotate: 'spin' }}></div>`,

  intro: `<!-- one directive, one system. The trigger says WHEN; the body says WHAT. -->
<button motion:press={{ scale: 'press' }}>
  Tactile on press.
</button>`,

  interaction: `<!-- INTERACTION — press and focus are motion's; the browser already
     knows when they happen. Hover is NOT: hover feedback is a hover: atom
     in sx — CSS, composited, interruptible — see the styling chapter. -->
<card sx="transition-all duration-fast ease-out hover:-translate-y-0_75 hover:shadow-hover">
  Lifts on hover — styling, not motion.
</card>

<button
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

  choreography: `<!-- CHOREOGRAPHY — the other lane. Something else owns the clock
     (here a slider), keyframes are DATA, and every track comes back as a
     signal the style layer already knows how to read. No effect, no
     el.style, no offset arithmetic. -->
<controller>
import { controller, signal, timeline } from '@ship/sail-runtime'
export default controller(() => {
  const scrubText = signal('0')
  const p = () => Number(scrubText()) / 100
  const { tileX, tileRotate, tileScale } = timeline(p, {
    // translate settles at .34 and HOLDS — a hold is the same VALUE
    // repeated at a later offset.
    tileX:     [[0, '0px'], [0.12, '0px'], [0.34, '120px'], [0.44, '120px'],
                [0.54, '120px'], [0.64, '120px'], [1, '0px']],
    tileRotate:[[0, '0deg'], [0.12, '0deg'], [0.34, '15deg'], [0.44, '15deg'],
                [0.54, '15deg'], [0.64, '15deg'], [1, '0deg']],
    // …while the scale pops on its own, between .54 and .64.
    tileScale: [[0.44, 1], [0.54, 1.75], [0.64, 1]],
  }, { easing: [0.6, 0, 0.2, 1] })
  return { scrubText, tileX, tileRotate, tileScale }
})
</controller>

<input type="range" min="0" max="100" step="1" bind:value={scrubText} />
<card sx="translate-x-(--tile-x) rotate-(--tile-rotate) scale-(--tile-scale)">
  scrub me
</card>`,

  drivers: `<!-- A driver is just a signal. Swap the line, keep everything else. -->
const p = clock(3600, { loop: true })   // wall-clock, with .play()/.pause()
const p = scrollProgress(section)        // this element's travel through the viewport
const p = springValue(target)            // real physics chasing another signal
const p = signal(0)                      // …or a plain signal you write yourself

// Reduced motion is decided once, in the driver — not at every call site.
// clock() jumps to the end and never schedules a frame.`,

  stagger: `<!-- stagger is RESERVED, not a property: it describes the relationship
     BETWEEN siblings (the delay step), not anything about any one of them. -->
{#for n of items() key={n}}
  <card motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy', stagger: '0.05s' }}>
    {n}
  </card>
{/for}`,
}
