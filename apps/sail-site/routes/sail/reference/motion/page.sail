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

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>Motion — one system</h1>
    <p role="doc-lead">
      <code>style.&lt;trigger&gt;({…})</code> is the only animation directive;
      <code>transition:</code> / <code>in:</code> / <code>out:</code> are
      retired.
    </p>
  </column>

  <row gap="3" sx={style.padX(3).padY(2).bg(surface-1).radius(md)}><code sx={style.minW(9)}>triggers</code><small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{motionTriggers()}</small></row>
  <row gap="3" sx={style.padX(3).padY(2).bg(surface-1).radius(md)}><code sx={style.minW(9)}>properties</code><small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{motionProperties()} — plus {motionStagger()}: a duration step between siblings</small></row>

  <column gap="3">
    <h3>Presets</h3>
    <grid cols="3" gap="2">
      <for as="p" of={motionPresets()} key={p.name}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code>{p.name}</code>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>({p.kind}) {p.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Config value grammar</h3>
    <p sx={style.maxW(breakpoint-tablet)}>
      Inside a <code>style.&lt;trigger&gt;({…})</code> config object, every value
      takes one of these forms.
    </p>
    <grid cols="1" gap="2">
      <for as="f" of={valueForms()} key={f.form}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code>{f.form}</code>
            <small sx={style.fg(muted-foreground)}>{f.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row role="navigation">
    <a variant="default" href="/sail/reference/sx">← sx</a>
    <a variant="solid" href="/sail/reference/actions">Actions →</a>
  </row>
</column>
