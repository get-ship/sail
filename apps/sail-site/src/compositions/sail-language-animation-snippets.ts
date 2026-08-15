// Snippets for /docs/concepts/animation. Kept OUT of the controller so the
// compiler's scope scan isn't fooled. Every SAIL sample is compiled by the
// build; each one is the code of the live demo beside it.
import { signal, timeline } from '@ship/sail-runtime'
export const code = {
  stagger: `<if {dealt()}>
  <row gap="3">
    <div style({ size(9).radius(md).bg(primary), animate: { enter: play(slide-up) } })>
    </div>
    <!-- ×5 — each waits 0.07s after its previous sibling -->
  </row>
</if>`,

  lifecycle: `<if {open()}>
  <card style({ animate: { enter: play(fade-scale-in), exit: play(fade) } })>
    I animate in, and I animate out.
  </card>
</if>`,

  express: `<div style({ animate: { loop: play(fade) } })>
  </div>
<div style({ animate: { loop: play(fade) } })>
</div>`,

  reveal: `<card style({ animate: { reveal: play(slide-up) } })>
  This one fades up as it enters the viewport.
</card>`,

  loop: `<div style({ animate: { loop: play(slide-up) } })>
  </div>

<div style({ animate: { loop: play(fade) } })>
</div>`,

  gestures: `<button style({ animate: { press: play(fade) } })>
    press me</button>
<button style({ animate: { focus-visible: play(fade) } })>
  tab to me</button>`,

  choreography: `<controller>
export default controller(() => {
  const scrubText = signal('0')
  const p = () => Number(scrubText()) / 100   // slider value → progress 0…1

  const { tileX, tileRotate, tileScale } = timeline(p, {
    // move right, then HOLD (the same value at a later offset)…
    tileX:      [[0, '0px'], [0.34, '120px'], [0.64, '120px'], [1, '0px']],
    tileRotate: [[0, '0deg'], [0.34, '15deg'], [0.64, '15deg'], [1, '0deg']],
    // …while scale pops on its own, between .54 and .64
    tileScale:  [[0.44, 1], [0.54, 1.75], [0.64, 1]],
  }, { easing: [0.6, 0, 0.2, 1] })

  return { scrubText, tileX, tileRotate, tileScale }
})
</controller>

<slider min="0" max="100" bind:value={scrubText} />
<card style({ translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale)) })>
</card>`,

  drivers: `const p = () => Number(scrubText()) / 100  // the slider above
const p = clock(3600, { loop: true })       // a wall clock: plays itself
const p = scrollProgress(section)           // this element's travel through the viewport
const p = springValue(target)               // physics chasing another signal`,
}
