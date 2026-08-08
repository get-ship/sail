// Snippets for /sail/language/animation. Kept OUT of the controller so the
// compiler's scope scan isn't fooled. Every SAIL sample is compiled by the
// build; each one is the code of the live demo beside it.
export const code = {
  stagger: `<if {dealt()}>
  <row gap="3">
    <div style({ box: w(9).h(9), paint: radius(md).bg(primary), animate: enter({ translate-y: 'spring:snappy', stagger: '0.07s' }) })>
    </div>
    <!-- ×5 — each waits 0.07s after its previous sibling -->
  </row>
</if>`,

  lifecycle: `<if {open()}>
  <card style({ animate: enter({ opacity: 'fade', scale: 'spring:snappy' }).exit({ opacity: 'fade-out' }) })>
    I animate in, and I animate out.
  </card>
</if>`,

  express: `<div style({ animate: loop({ translate-x: { from: 0, to: '80px' }, duration: 500 }) })>
  </div>
<div style({ animate: loop({ translate-x: { from: 0, to: '80px' }, duration: 2500 }) })>
</div>`,

  reveal: `<card style({ animate: reveal({ opacity: 'fade', translate-y: 'spring:gentle' }) })>
  This one fades up as it enters the viewport.
</card>`,

  loop: `<div style({ animate: loop({ translate-y: 'spring:gentle', stagger: '0.15s' }) })>
  </div>

<div style({ animate: loop({ rotate: 'spin' }) })>
</div>`,

  gestures: `<button style({ animate: press({ scale: 'press' }) })>
    press me</button>
<button style({ animate: focus-visible({ scale: 'spring:snappy' }) })>
  tab to me</button>`,

  choreography: `<controller>
import { timeline } from '@ship/sail-runtime'
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

<input type="range" min="0" max="100" bind:value={scrubText} />
<card style({ paint: translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale)) })>
</card>`,

  drivers: `const p = () => Number(scrubText()) / 100  // the slider above
const p = clock(3600, { loop: true })       // a wall clock: plays itself
const p = scrollProgress(section)           // this element's travel through the viewport
const p = springValue(target)               // physics chasing another signal`,
}
