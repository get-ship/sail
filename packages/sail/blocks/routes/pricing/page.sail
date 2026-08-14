<!--
  pricing — marketing section at the Tailwind Plus bar.
  Contained measure. Cards are equal columns of a max-width page, not
  viewport slabs. Host props only on L1.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).pad({ x: 6, y: 10 }) }
  style intro = { size({ max-w: panel-max-w-lg }).margin({ x: auto }) }
  style faq = { size({ max-w: panel-max-w-lg }).margin({ x: auto }) }
</scope>

<column gap="8" style(shell)>
  <navbar bordered>
    <span slot="brand">Sail</span>
    <row slot="center" gap="2">
      <a href="/dashboard">Product</a>
      <a href="/pricing">Pricing</a>
      <a href="/settings">Docs</a>
    </row>
    <row slot="actions" gap="2">
      <button variant="ghost" size="sm">Sign in</button>
      <button size="sm">Start free</button>
    </row>
  </navbar>

  <column gap="8" style(page)>
    <column gap="3" align="center" style(intro)>
      <badge>Pricing</badge>
      <h1>Pricing that scales with you</h1>
      <p>Start free. Upgrade when the team does. Every plan includes the catalog themes, L1 components, and the compiler.</p>
      <toggle-group type="single" value="monthly" variant="segmented">
        <toggle value="monthly">Monthly</toggle>
        <toggle value="annual">Annually</toggle>
      </toggle-group>
    </column>

    <row gap="4" align="stretch">
      <card>
        <column gap="4">
          <column gap="1">
            <h2>Hobby</h2>
            <p>For trying Sail on a side project.</p>
          </column>
          <row gap="1" align="end">
            <h2>$0</h2>
            <small>/month</small>
          </row>
          <button variant="outline">Start for free</button>
          <separator />
          <column gap="2">
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>1 project</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Community themes</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>L1 components</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Forum support</span></row>
          </column>
        </column>
      </card>

      <card variant="elevated">
        <column gap="4">
          <row justify="between" align="center">
            <h2>Studio</h2>
            <badge tone="primary">Most popular</badge>
          </row>
          <p>For teams shipping a real product theme.</p>
          <row gap="1" align="end">
            <h2>$24</h2>
            <small>/month</small>
          </row>
          <button>Choose Studio</button>
          <separator />
          <column gap="2">
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Unlimited projects</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Theme Studio overrides</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Blocks library</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>SSO-ready seats</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Email support</span></row>
          </column>
        </column>
      </card>

      <card variant="outline">
        <column gap="4">
          <column gap="1">
            <h2>Enterprise</h2>
            <p>For orgs that need audit, SLA, and a private catalog.</p>
          </column>
          <row gap="1" align="end">
            <h2>Talk</h2>
          </row>
          <button variant="ghost">Contact sales</button>
          <separator />
          <column gap="2">
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Everything in Studio</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Private theme catalog</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Audit log + SSO</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>Dedicated slack</span></row>
            <row gap="2" align="center"><icon name="check" size="sm"></icon><span>On-prem compiler option</span></row>
          </column>
        </column>
      </card>
    </row>

    <column gap="4" style(faq)>
      <h2>Questions</h2>
      <accordion name="pricing-faq">
        <strong slot="trigger">Can I switch plans later?</strong>
        <p>Yes. Upgrade or downgrade from settings. Proration is automatic.</p>
      </accordion>
      <accordion name="pricing-faq">
        <strong slot="trigger">What counts as a project?</strong>
        <p>One Sail app with its own theme overlay. Hobby is capped at one.</p>
      </accordion>
      <accordion name="pricing-faq">
        <strong slot="trigger">Do you take a cut of usage?</strong>
        <p>No. The compiler and L1 catalog are licensed per seat, not metered.</p>
      </accordion>
    </column>
  </column>
</column>
