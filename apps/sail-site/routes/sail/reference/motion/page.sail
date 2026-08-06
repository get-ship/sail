<!--
  /sail/reference/motion — the one animation system: triggers, properties,
  presets, and the config-value grammar. Rendered from
  src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { MOTION, MOTION_VALUE_FORMS } from '../../../../src/language-surface.gen'

export default controller((_ctx: ControllerScope<Scope>) => ({
  motionTriggers: () => MOTION.triggers.join(' · '),
  motionProperties: () => MOTION.properties.join(' · '),
  motionStagger: () => MOTION.staggerKey,
  motionPresets: () => MOTION.presets,
  valueForms: () => MOTION_VALUE_FORMS,
}))
</controller>

<column gap="6">
  style({ box: minW(0) })

  <column gap="5">
    style({ box: maxW(breakpoint-tablet) })

    <p token="doc-eyebrow">Reference</p>
    <h1>Motion — one system</h1>
    <p token="doc-lead">
      <code>style&#46;&lt;trigger&gt;&#40;&#123;…&#125;&#41;</code> is the only animation directive;
      <code>transition:</code> / <code>in:</code> / <code>out:</code> are
      retired.
    </p>
  </column>

  <row gap="3">
    style({ box: padX(3).padY(2), paint: bg(surface-1).radius(md) })
<code>
    style({ box: minW(9) })
triggers</code><small>
    style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{motionTriggers()}</small></row>
  <row gap="3">
    style({ box: padX(3).padY(2), paint: bg(surface-1).radius(md) })
<code>
    style({ box: minW(9) })
properties</code><small>
    style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{motionProperties()} — plus {motionStagger()}: a duration step between siblings</small></row>

  <column gap="3">
    <h3>Presets</h3>
    <grid cols="3" gap="2">
      <for {motionPresets()} as="p" key={p.name}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>{p.name}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
({p.kind}) {p.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Config value grammar</h3>
    <p>
      style({ box: maxW(breakpoint-tablet) })

      Inside a <code>style&#46;&lt;trigger&gt;&#40;&#123;…&#125;&#41;</code> config object, every value
      takes one of these forms.
    </p>
    <grid cols="1" gap="2">
      <for {valueForms()} as="f" key={f.form}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>{f.form}</code>
            <small>
              style({ paint: fg(muted-foreground) })
{f.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/sail/reference/sx">← sx</a>
    <a variant="solid" href="/sail/reference/actions">Actions →</a>
  </row>
</column>
