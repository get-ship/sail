<!--
  pricing — the document. Layout + which pieces. Period lives here.
-->

<controller>
  export default controller(() => {
    const period = signal('monthly')
    const annual = derived(() => period() === 'annual')
    return {
      period,
      annual,
      onPeriod: (detail: { value?: string }): void => {
        const next = detail?.value
        if (next === 'monthly' || next === 'annual') period.set(next)
      },
    }
  })
</controller>

<column gap="0">
  <navbar bordered sticky>
    <strong slot="brand">Sail</strong>
    <row slot="center" gap="2">
      <a href="/dashboard">Product</a>
      <a href="/pricing">Pricing</a>
      <a href="/settings">Docs</a>
    </row>
    <row slot="actions" gap="2">
      <theme-picker></theme-picker>
      <button variant="ghost" size="sm">Sign in</button>
      <button size="sm">Start free</button>
    </row>
  </navbar>

  <page>
    <pricing-hero>
      <toggle-group type="single" value={period()} variant="segmented" on-toggle-change={onPeriod}>
        <toggle value="monthly">Monthly</toggle>
        <toggle value="annual">Annually</toggle>
      </toggle-group>
    </pricing-hero>
    <pricing-plans period={period()}></pricing-plans>
    <pricing-logos></pricing-logos>
    <pricing-compare></pricing-compare>
    <pricing-quotes></pricing-quotes>
    <pricing-faq></pricing-faq>
    <pricing-cta></pricing-cta>
    <pricing-footer></pricing-footer>
  </page>
</column>
