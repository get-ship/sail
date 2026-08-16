<!--
  pricing — Tailwind Plus marketing bar: hero, frequency, three tiers with
  an emphasized middle, logos, comparison table, quotes, FAQ, CTA, footer.
  Measure is contained. L1 hosts take props only.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).pad({ x: 6, y: 10 }) }
  style intro = { size({ max-w: panel-max-w-lg }).margin({ x: auto }) }
  style eyebrow = { fg(primary).type({ size: sm, weight: semibold, align: center }) }
  style display = { fg(foreground).type({ family: display, size: display, weight: semibold, leading: tight, tracking: tighter, align: center }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed, align: center }) }
  style planName = { type({ size: sm, weight: semibold }) }
  style planLead = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style currency = { type({ size: lg, weight: semibold }) }
  style price = { type({ size: 4xl, weight: semibold, leading: none, nums: tabular }) }
  style period = { fg(muted-foreground).type({ size: sm }) }
  style feature = { fg(muted-foreground).type({ size: sm }) }
  style check = { fg(success) }
  style sectionKicker = { fg(muted-foreground).type({ size: sm, weight: semibold, align: center }) }
  style sectionTitle = { type({ size: 3xl, weight: semibold, align: center, leading: tight }) }
  style logo = { fg(muted-foreground).type({ size: sm, weight: semibold }) }
  style quote = { type({ size: md, leading: relaxed }) }
  style cite = { fg(muted-foreground).type({ size: sm }) }
  style faq = { size({ max-w: panel-max-w-lg }).margin({ x: auto }) }
  style footCol = { fg(muted-foreground).type({ size: sm }) }
  style footHead = { fg(foreground).type({ size: sm, weight: semibold }) }
  style ctaTitle = { type({ size: 2xl, weight: semibold, align: center }) }
</scope>

<controller>
  export default controller(() => {
    const period = signal('monthly')
    const annual = derived(() => period() === 'annual')
    const studioPrice = derived(() => (annual() ? '19' : '24'))
    const periodLabel = derived(() =>
      annual() ? '/month, billed yearly' : '/month',
    )
    return {
      period,
      annual,
      studioPrice,
      periodLabel,
      onPeriod: (detail: { value?: string }): void => {
        const next = detail?.value
        if (next === 'monthly' || next === 'annual') period.set(next)
      },
    }
  })
</controller>

<column gap="0" style(shell)>
  <alert variant="banner" tone="info">
    <span slot="title">Annual billing is live</span>
    Two months free on Studio when you pay yearly.
  </alert>

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

  <column gap="8" style(page)>
    <column gap="4" align="center" style(intro)>
      <p style(eyebrow)>Pricing</p>
      <h1 style(display)>Pricing that grows with you</h1>
      <p style(lead)>Start free. Upgrade when the team does. Every plan includes the catalog themes, L1 components, and the compiler.</p>
      <row gap="3" align="center">
        <toggle-group type="single" value={period()} variant="segmented" on-toggle-change={onPeriod}>
          <toggle value="monthly">Monthly</toggle>
          <toggle value="annual">Annually</toggle>
        </toggle-group>
        <if {annual()}>
          <badge tone="success" pill>2 months free</badge>
        </if>
      </row>
    </column>

    <grid min="lg" gap="6">
      <card variant="outline" padding="lg">
        <column gap="5">
          <column gap="1">
            <h2 style(planName)>Hobby</h2>
            <p style(planLead)>For trying Sail on a side project.</p>
          </column>
          <row gap="1" align="end">
            <span style(currency)>$</span>
            <span style(price)>0</span>
            <span style(period)>{periodLabel()}</span>
          </row>
          <button variant="outline">Start for free</button>
          <separator />
          <column gap="3">
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>1 project</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Community themes</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>L1 component catalog</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Forum support</span></row>
          </column>
        </column>
      </card>

      <card variant="elevated" spotlight padding="lg">
        <column gap="5">
          <row justify="between" align="center">
            <h2 style(planName)>Studio</h2>
            <badge tone="primary" pill>Most popular</badge>
          </row>
          <p style(planLead)>For teams shipping a real product theme.</p>
          <row gap="1" align="end">
            <span style(currency)>$</span>
            <span style(price)>{studioPrice()}</span>
            <span style(period)>{periodLabel()}</span>
          </row>
          <button>Choose Studio</button>
          <separator />
          <column gap="3">
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Unlimited projects</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Theme Studio overrides</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Blocks library</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>10 SSO-ready seats</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Email support</span></row>
          </column>
        </column>
      </card>

      <card variant="soft" padding="lg">
        <column gap="5">
          <column gap="1">
            <h2 style(planName)>Enterprise</h2>
            <p style(planLead)>For orgs that need audit, SLA, and a private catalog.</p>
          </column>
          <row gap="1" align="end">
            <span style(price)>Talk</span>
          </row>
          <button variant="outline">Contact sales</button>
          <separator />
          <column gap="3">
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Everything in Studio</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Private theme catalog</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Audit log + SSO</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>Dedicated Slack</span></row>
            <row gap="2" align="center"><span style(check)><icon name="check" size="sm"></icon></span><span style(feature)>On-prem compiler option</span></row>
          </column>
        </column>
      </card>
    </grid>

    <column gap="4" align="center">
      <p style(sectionKicker)>Trusted by teams who already shipped</p>
      <row gap="8" align="center" justify="center">
        <span style(logo)>Harbor</span>
        <span style(logo)>North Pier</span>
        <span style(logo)>Ada Labs</span>
        <span style(logo)>Meridian</span>
        <span style(logo)>Field &amp; Co</span>
        <span style(logo)>Lumen</span>
      </row>
    </column>

    <column gap="5">
      <column gap="2" align="center">
        <h2 style(sectionTitle)>Compare plans</h2>
        <p style(lead)>Same compiler. The seats and the catalog change.</p>
      </column>
      <scroll-area orientation="horizontal">
        <table sticky-header bordered>
          <thead>
            <tr>
              <th>Feature</th>
              <th>Hobby</th>
              <th>Studio</th>
              <th>Enterprise</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>Projects</td>
              <td>1</td>
              <td>Unlimited</td>
              <td>Unlimited</td>
            </tr>
            <tr>
              <td>Seats</td>
              <td>1</td>
              <td>10</td>
              <td>Unlimited</td>
            </tr>
            <tr>
              <td>Theme Studio</td>
              <td>—</td>
              <td><icon name="check" size="sm"></icon></td>
              <td><icon name="check" size="sm"></icon></td>
            </tr>
            <tr>
              <td>Blocks library</td>
              <td>—</td>
              <td><icon name="check" size="sm"></icon></td>
              <td><icon name="check" size="sm"></icon></td>
            </tr>
            <tr>
              <td>SSO</td>
              <td>—</td>
              <td>—</td>
              <td><icon name="check" size="sm"></icon></td>
            </tr>
            <tr>
              <td>Audit log</td>
              <td>—</td>
              <td>—</td>
              <td><icon name="check" size="sm"></icon></td>
            </tr>
            <tr>
              <td>Support</td>
              <td>Forum</td>
              <td>Email</td>
              <td>Dedicated Slack</td>
            </tr>
            <tr>
              <td>On-prem compiler</td>
              <td>—</td>
              <td>—</td>
              <td><icon name="check" size="sm"></icon></td>
            </tr>
          </tbody>
        </table>
      </scroll-area>
    </column>

    <column gap="5">
      <h2 style(sectionTitle)>Loved by the people who theme it</h2>
      <grid min="sm" gap="4">
        <card variant="soft">
          <column gap="4">
            <p style(quote)>“We dropped a custom design system and kept the tokens. Studio is the first theme tool the designers actually open.”</p>
            <row gap="3" align="center">
              <avatar initials="AL" tone="primary"></avatar>
              <column gap="0">
                <strong>Ada L.</strong>
                <span style(cite)>Design lead, Ada Labs</span>
              </column>
            </row>
          </column>
        </card>
        <card variant="soft">
          <column gap="4">
            <p style(quote)>“The pricing page used to be a full-bleed accident. Defaults now hold a measure. That is the whole product.”</p>
            <row gap="3" align="center">
              <avatar initials="HC" presence="online"></avatar>
              <column gap="0">
                <strong>Harbor Co</strong>
                <span style(cite)>Founder</span>
              </column>
            </row>
          </column>
        </card>
        <card variant="soft">
          <column gap="4">
            <p style(quote)>“Enterprise bought us a private catalog and an audit log. We did not rewrite a component to get there.”</p>
            <row gap="3" align="center">
              <avatar initials="NP" ring></avatar>
              <column gap="0">
                <strong>North Pier</strong>
                <span style(cite)>Platform</span>
              </column>
            </row>
          </column>
        </card>
      </grid>
    </column>

    <column gap="4" style(faq)>
      <h2 style(sectionTitle)>Questions</h2>
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
      <accordion name="pricing-faq">
        <strong slot="trigger">Is annual billed up front?</strong>
        <p>Yes. Studio annual is two months free versus month-to-month.</p>
      </accordion>
      <accordion name="pricing-faq">
        <strong slot="trigger">Can Hobby stay free forever?</strong>
        <p>Yes, for one project. Move to Studio when you need Theme Studio or more seats.</p>
      </accordion>
    </column>

    <card variant="soft" padding="lg">
      <column gap="4" align="center">
        <h2 style(ctaTitle)>Ready when the theme is</h2>
        <p style(lead)>Start on Hobby. Promote the same app to Studio without a rewrite.</p>
        <row gap="2">
          <button variant="outline">Talk to sales</button>
          <button>Start for free</button>
        </row>
      </column>
    </card>

    <separator />

    <grid min="xs" gap="6">
      <column gap="2">
        <strong>Sail</strong>
        <span style(footCol)>Themes, L1, and a compiler that stay one language.</span>
      </column>
      <column gap="2">
        <span style(footHead)>Product</span>
        <a href="/dashboard">Dashboard</a>
        <a href="/pricing">Pricing</a>
        <a href="/settings">Settings</a>
      </column>
      <column gap="2">
        <span style(footHead)>Company</span>
        <a href="/pricing">Customers</a>
        <a href="/pricing">Changelog</a>
        <a href="/pricing">Contact</a>
      </column>
      <column gap="2">
        <span style(footHead)>Legal</span>
        <a href="/pricing">Terms</a>
        <a href="/pricing">Privacy</a>
        <a href="/pricing">Security</a>
      </column>
    </grid>
  </column>
</column>
