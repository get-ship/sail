<!--
  /docs/reference/config — every sail.config.json key from lint CONFIG_KEY_DOCS.
-->
<scope>
  style root = { box: size({ min-w: 0 }) }
  style intro = { box: size({ max-w: breakpoint-tablet }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style cardMin = { box: size({ min-w: 0 }) }
  style keyType = { paint: ink({ color: muted-foreground }), text: type({ family: mono }) }
  style cardDoc = { box: size({ max-w: breakpoint-tablet }), paint: ink({ color: muted-foreground }) }
  style defaultVal = { paint: ink({ color: muted-foreground }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { CONFIG_KEYS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  keys: () => CONFIG_KEYS,
  keyCount: () => CONFIG_KEYS.length,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>sail.config.json — {keyCount()} keys</h1>
    <p style(lead)>
      Resolved by one walk-up from a source file to the nearest
      <code>sail.config.json</code>, one parse. Any other top-level key is a
      typo or a made-up name — the compiler refuses it loudly rather than
      ignoring it.
    </p>
  </column>

  <column gap="2">
    <for {keys() as k} key={k.key}>
      <card padding="sm" style(cardMin)>
        <column gap="1">
          <row gap="3" align="center">
            <code>{k.key}</code>
            <small style(keyType)>{k.type}</small>
          </row>
          <small style(cardDoc)>{k.doc}</small>
          <small style(defaultVal)>default: {k.default}</small>
        </column>
      </card>
    </for>
  </column>

  <row>
    <a variant="default" href="/docs/reference/runtime">← Runtime</a>
    <a variant="solid" href="/docs/reference">Reference index →</a>
  </row>
</column>
