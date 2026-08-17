<!--
  /docs/reference/config — every sail.config.json key from lint CONFIG_KEY_DOCS.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { CONFIG_KEYS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  keys: () => CONFIG_KEYS,
  keyCount: () => CONFIG_KEYS.length,
}))
</controller>

<column gap="6">
  <column gap="5">
    <p>Reference</p>
    <h1>sail.config.json — {keyCount()} keys</h1>
    <p>
      Resolved by one walk-up from a source file to the nearest
      <code>sail.config.json</code>, one parse. Any other top-level key is a
      typo or a made-up name — the compiler refuses it loudly rather than
      ignoring it.
    </p>
  </column>

  <column gap="2">
    <for {keys() as k} key={k.key}>
      <card padding="sm">
        <column gap="1">
          <row gap="3" align="center">
            <code>{k.key}</code>
            <small>{k.type}</small>
          </row>
          <small>{k.doc}</small>
          <small>default: {k.default}</small>
        </column>
      </card>
    </for>
  </column>

  <row>
    <a variant="default" href="/docs/reference/runtime">← Runtime</a>
    <a variant="solid" href="/docs/reference">Reference index →</a>
  </row>
</column>
