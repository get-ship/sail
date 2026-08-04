<!--
  /sail/reference/config — every sail.config.json key, rendered from
  src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`,
  sourced from sail_lint::config::CONFIG_KEY_DOCS).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { CONFIG_KEYS } from '../../../../src/language-surface.gen'

export default controller((_ctx: ControllerScope<Scope>) => ({
  keys: () => CONFIG_KEYS,
  keyCount: () => CONFIG_KEYS.length,
}))
</controller>

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
    <p token="doc-eyebrow">Reference</p>
    <h1>sail.config.json — {keyCount()} keys</h1>
    <p token="doc-lead">
      Resolved by one walk-up from a source file to the nearest
      <code>sail.config.json</code>, one parse. Any other top-level key is a
      typo or a made-up name — the compiler refuses it loudly rather than
      ignoring it.
    </p>
  </column>

  <column gap="2">
    <for {keys()} as="k" key={k.key}>
      <card padding="sm" sx={style.minW(0)}>
        <column gap="1">
          <row gap="3" align="center">
            <code>{k.key}</code>
            <small sx={style.fg(muted-foreground).fontFamily(mono)}>{k.type}</small>
          </row>
          <small sx={style.fg(muted-foreground).maxW(breakpoint-tablet)}>{k.doc}</small>
          <small sx={style.fg(muted-foreground)}>default: {k.default}</small>
        </column>
      </card>
    </for>
  </column>

  <row>
    <a variant="default" href="/sail/reference/runtime">← Runtime</a>
    <a variant="solid" href="/sail/reference">Reference index →</a>
  </row>
</column>
