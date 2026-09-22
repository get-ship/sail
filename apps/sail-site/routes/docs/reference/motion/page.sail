<!--
  /docs/reference/motion — Style animate channel: triggers, properties, presets, value grammar.
-->

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

<column gap="6">
  <column gap="5">
    <p>Reference</p>
    <h1>Motion — one system</h1>
    <p>
      Animation is declared on the element via the Style <strong>animate
      channel</strong>.
      <code>style&#40;&#123; animate: &#123; enter: play(fade) &#125; &#125;&#41;</code> is the tip form;
      retired <code>transition:</code> / <code>in:</code> / <code>out:</code>
      attributes are not legal.
    </p>
  </column>

  <card padding="sm">
    <row gap="3">
      <code>triggers</code>
      <small>{motionTriggers()}</small>
    </row>
  </card>
  <card padding="sm">
    <row gap="3">
      <code>properties</code>
      <small>{motionProperties()} — plus {motionStagger()}: a duration step between siblings</small>
    </row>
  </card>

  <column gap="3">
    <h3>Presets</h3>
    <grid cols="3" gap="2">
      <for {motionPresets() as p} key={p.name}>
        <card padding="sm">
          <column gap="1">
            <code>{p.name}</code>
            <small>({p.kind}) {p.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Config value grammar</h3>
    <p>
      Inside <code>animate: &#123; &lt;trigger&gt;: play(…) &#125;</code>, the play name is the vocabulary; every value
      takes one of these forms.
    </p>
    <grid cols="1" gap="2">
      <for {valueForms() as f} key={f.form}>
        <card padding="sm">
          <column gap="1">
            <code>{f.form}</code>
            <small>{f.summary}</small>
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
