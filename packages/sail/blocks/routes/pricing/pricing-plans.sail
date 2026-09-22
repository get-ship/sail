<!-- Three plan cards. Features are a loop. -->
<scope>
  props: {
    period = 'monthly'
  }
</scope>

<controller>
  export default controller((ctx: { period: () => string }) => {
    const annual = derived(() => ctx.period() === 'annual')
    const plans = [
      {
        id: 'hobby',
        name: 'Hobby',
        blurb: 'For trying Sail on a side project.',
        variant: 'outline',
        featured: false,
        cta: 'Start for free',
        ctaVariant: 'outline',
        price: 'free',
        monthly: '0',
        yearly: '0',
        features: ['1 project', 'Community themes', 'L1 component catalog', 'Forum support'],
      },
      {
        id: 'studio',
        name: 'Studio',
        blurb: 'For teams shipping a real product theme.',
        variant: 'elevated',
        featured: true,
        cta: 'Choose Studio',
        ctaVariant: 'solid',
        price: 'seat',
        monthly: '24',
        yearly: '19',
        features: [
          'Unlimited projects',
          'Theme Studio overrides',
          'Blocks library',
          '10 SSO-ready seats',
          'Email support',
        ],
      },
      {
        id: 'enterprise',
        name: 'Enterprise',
        blurb: 'For orgs that need audit, SLA, and a private catalog.',
        variant: 'soft',
        featured: false,
        cta: 'Contact sales',
        ctaVariant: 'outline',
        price: 'talk',
        monthly: '',
        yearly: '',
        features: [
          'Everything in Studio',
          'Private theme catalog',
          'Audit log + SSO',
          'Dedicated Slack',
          'On-prem compiler option',
        ],
      },
    ]
    return { plans, annual }
  })
</controller>

<grid min="lg" gap="6">
  <for {plans as plan} key={plan.id}>
    <card variant={plan.variant} spotlight={plan.featured} padding="lg">
      <column gap="5">
        <row justify="between" align="center">
          <h2>{plan.name}</h2>
          <if {plan.featured}>
            <badge tone="primary" pill>Most popular</badge>
          </if>
        </row>
        <p>{plan.blurb}</p>
        <row gap="1" align="end">
          <if {plan.price === 'talk'}>
            <strong>Talk</strong>
          {else =>
            <strong>$</strong>
            <if {annual()}>
              <strong>{plan.yearly}</strong>
            {else =>
              <strong>{plan.monthly}</strong>
            }
            </if>
            <column gap="0">
              <small>/month</small>
              <small>{annual() ? 'billed yearly' : '\u00a0'}</small>
            </column>
          }
          </if>
        </row>
        <if {plan.ctaVariant === 'outline'}>
          <button variant="outline">{plan.cta}</button>
        {else =>
          <button>{plan.cta}</button>
        }
        </if>
        <separator />
        <column gap="3">
          <for {plan.features as feat} key={feat}>
            <row gap="2" align="center">
              <icon name="check" size="sm"></icon>
              <small>{feat}</small>
            </row>
          </for>
        </column>
      </column>
    </card>
  </for>
</grid>
