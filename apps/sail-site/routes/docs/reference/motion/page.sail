<!--
  /docs/reference/motion — Style animate channel: triggers, properties, presets, value grammar.
-->
<scope>
  style root = { box: size({ min-w: 0 }) }
  style intro = { box: size({ max-w: breakpoint-tablet }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style kvRow = { box: space({ pad-x: 3, pad-y: 2 }), paint: fill({ color: surface-1 }).radius(md) }
  style kvKey = { box: size({ min-w: 9 }) }
  style kvVal = { box: size({ max-w: panel-max-w-sm }), paint: ink({ color: muted-foreground }) }
  style cardMin = { box: size({ min-w: 0 }) }
  style cardDoc = { box: size({ max-w: panel-max-w-sm }), paint: ink({ color: muted-foreground }) }
  style muted = { paint: ink({ color: muted-foreground }) }
  style sectionLead = { box: size({ max-w: breakpoint-tablet }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { MOTION, MOTION_VALUE_FORMS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  motionTriggers: () => MOTION.triggers.join(' · '),
  motionProperties: () => MOTION.properties.join(' · '),
  motionStagger: () => MOTION.staggerKey,
  motionPresets: () => MOTION.presets,
  valueForms: () => MOTION_VALUE_FORMS,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>Motion — one system</h1>
    <p style(lead)>
      Animation is declared on the element via the Style <strong>animate
      channel</strong>.
      <code>style&#40;&#123; animate: enter(&#123;…&#125;) &#125;&#41;</code> is the tip form;
      retired <code>transition:</code> / <code>in:</code> / <code>out:</code>
      attributes are not legal.
    </p>
  </column>

  <row gap="3" style(kvRow)><code style(kvKey)>triggers</code><small style(kvVal)>{motionTriggers()}</small></row>
  <row gap="3" style(kvRow)><code style(kvKey)>properties</code><small style(kvVal)>{motionProperties()} — plus {motionStagger()}: a duration step between siblings</small></row>

  <column gap="3">
    <h3>Presets</h3>
    <grid cols="3" gap="2">
      <for {motionPresets() as p} key={p.name}>
        <card padding="sm" style(cardMin)>
          <column gap="1">
            <code>{p.name}</code>
            <small style(cardDoc)>({p.kind}) {p.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Config value grammar</h3>
    <p style(sectionLead)>
      Inside an <code>animate: &lt;trigger&gt;(&#123;…&#125;)</code> config object, every value
      takes one of these forms.
    </p>
    <grid cols="1" gap="2">
      <for {valueForms() as f} key={f.form}>
        <card padding="sm" style(cardMin)>
          <column gap="1">
            <code>{f.form}</code>
            <small style(muted)>{f.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/docs/reference/sx">← Style</a>
    <a variant="solid" href="/docs/reference/actions">Actions →</a>
  </row>
</column>
