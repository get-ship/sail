<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style pageHead = { box: padB(4), paint: borderBWidth(1).borderB(border) }
  style title = { box: mb(2), paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style lede = { paint: fg(muted-foreground), text: fontSize(sm) }
  style card = { box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style cardTitle = { paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) }
  style codeTitle = { text: fontFamily(mono) }
  style badge = { box: padX(2).padY(1).ms(3), paint: fg(warning-foreground).bg(warning).radius(full), text: fontSize(xs).fontWeight(medium) }
  style body = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style stage = { box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) }
  style caption = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) }
  style dialogFrame = { box: position(relative).overflow(hidden), paint: borderWidth(1).border(dashed).border(border).radius(md) }
</scope>

<column gap="8" style(page)>
  <header style(pageHead)>
    <h1 style(title)>ship/authority — Domain Stubs</h1>
    <p style(lede)>
      Pack <strong>ship/authority</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code style(codeTitle)>ship-approval-card</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>request: string</code> (JSON-encoded AuthorityRequest). No slots.
      In production the host owns <code>ship.authority.pending()</code> via
      <code>live()</code> and passes each item via <code>request</code>;
      this stub passes a static JSON string. Emits <code>ship-decide</code> and
      <code>ship-inspect</code> custom events.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>kind=capability · risk=high</span>
        <ship-approval-card request={'{"id":"req-001","agent":"claude-opus","space":"work","kind":"capability","scope":"read_file","path":"/etc/hosts","why":"Need to read the hosts file to resolve local DNS entries","risk":"high"}'}></ship-approval-card>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>kind=command · risk=medium</span>
        <ship-approval-card request={'{"id":"req-002","agent":"codex-agent","space":"personal","kind":"command","command":"npm install --global some-cli","why":"Installing a global CLI tool needed for the build pipeline","risk":"medium"}'}></ship-approval-card>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>kind=capability · risk=low</span>
        <ship-approval-card request={'{"id":"req-003","agent":"cursor-bot","space":"work","kind":"capability","scope":"read_file","path":"./src/config.ts","why":"Checking project config for context","risk":"low"}'}></ship-approval-card>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code style(codeTitle)>ship-capability-request</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>request: string</code>, <code>duration: once|session|always</code>,
      <code>open: bool</code>. Wraps <code>sail-dialog</code> as an alertdialog.
      In production the host opens it when the operator clicks the agent name on
      <code>ship-approval-card</code>. Emits <code>ship-decide</code> and
      <code>ship-dismiss</code>. Shown open here so the dialog chrome is visible.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>open=true · duration=session</span>
        <column style(dialogFrame)>
          <ship-capability-request
            open
            duration="session"
            request={'{"id":"req-004","agent":"claude-opus","space":"work","kind":"capability","scope":"write_file","path":"./output/report.md","why":"Writing the final analysis report to the output directory","risk":"medium"}'}
          ></ship-capability-request>
        </column>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>open=true · duration=once</span>
        <column style(dialogFrame)>
          <ship-capability-request
            open
            duration="once"
            request={'{"id":"req-005","agent":"gemini-agent","space":"personal","kind":"capability","scope":"read_file","path":"~/.ssh/config","why":"Reading SSH config to list available hosts","risk":"high"}'}
          ></ship-capability-request>
        </column>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code style(codeTitle)>sail-auth-dialog</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>open: bool</code>, <code>mode: sign-in|sign-up</code>,
      <code>loading: bool</code>, <code>backdrop: default|blur|solid</code>.
      Slots: <code>header</code>, <code>harnesses</code>, <code>terms</code>.
      Emits <code>sail-auth-submit</code>, <code>sail-mode-change</code>,
      <code>sail-close</code>. Shown open here so the dialog chrome is visible.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>mode=sign-in · backdrop=blur · open=true</span>
        <column style(dialogFrame)>
          <sail-auth-dialog open mode="sign-in" backdrop="blur">
            <span slot="header">Sign in to Ship</span>
            <div slot="harnesses">
              <sail-button variant="outline" size="md">Continue with Claude</sail-button>
            </div>
            <span slot="terms">By signing in you agree to the Terms of Service.</span>
          </sail-auth-dialog>
        </column>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>mode=sign-up · backdrop=solid · open=true</span>
        <column style(dialogFrame)>
          <sail-auth-dialog open mode="sign-up" backdrop="solid">
            <span slot="header">Create your account</span>
            <span slot="terms">Free during beta. No credit card required.</span>
          </sail-auth-dialog>
        </column>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>loading=true · mode=sign-in · open=true</span>
        <column style(dialogFrame)>
          <sail-auth-dialog open mode="sign-in" loading backdrop="default">
            <span slot="header">Sign in to Ship</span>
            <span slot="terms">Hang tight…</span>
          </sail-auth-dialog>
        </column>
      </column>
    </column>
  </column>
</column>
