<column gap="8">
  style({ box: padY(6).padX(8).maxW(breakpoint-tablet) })

  <header>
    <h1>
      style({ box: mb(2), paint: fg(foreground), text: fontSize(xl).fontWeight(bold) })
ship/pipelines — Gallery</h1>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Pack <strong>ship/pipelines</strong>. Real component instantiations —
      one card per primitive, representative prop combinations across every
      enum variant. Static attribute values; no live data.
    </p>
  </header>

  <!-- ── sail-pipeline-step ──────────────────────────────────────────────── -->
  <column gap="4">
    <h2>
      style({ paint: fg(foreground), text: fontSize(base).fontWeight(semibold) })
<code>sail-pipeline-step</code></h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Props: <code>type: agent|gate|merge|human</code>,
      <code>state: ok|running|awaiting|err|none</code>,
      <code>compact: bool</code>.
      Slots: default (id/title), <code>sub</code>, <code>arts</code>.
    </p>

    <column gap="3">
      <!-- type variants, state=none -->
      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
type=agent · state=none</span>
      <sail-pipeline-step type="agent" state="none">
        lint-check
        <span slot="sub">Runs ESLint + Prettier over changed files</span>
      </sail-pipeline-step>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
type=gate · state=awaiting</span>
      <sail-pipeline-step type="gate" state="awaiting">
        human-review
        <span slot="sub">Waiting for approval before deploy</span>
        <span slot="arts">artifact: review-notes.md</span>
      </sail-pipeline-step>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
type=merge · state=ok</span>
      <sail-pipeline-step type="merge" state="ok">
        merge-results
        <span slot="sub">Aggregated outputs from parallel lanes</span>
        <span slot="arts">artifact: summary.json</span>
      </sail-pipeline-step>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
type=human · state=running</span>
      <sail-pipeline-step type="human" state="running">
        manual-deploy
        <span slot="sub">Operator confirms production cut-over</span>
      </sail-pipeline-step>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
type=agent · state=err</span>
      <sail-pipeline-step type="agent" state="err">
        build-release
        <span slot="sub">cargo build --release exited 1</span>
        <span slot="arts">artifact: build.log</span>
      </sail-pipeline-step>

      <!-- compact variants -->
      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
compact=true — all four types</span>
      <row gap="3">
        <sail-pipeline-step type="agent" state="ok" compact>ci-check</sail-pipeline-step>
        <sail-pipeline-step type="gate"  state="awaiting" compact>review-gate</sail-pipeline-step>
        <sail-pipeline-step type="merge" state="running" compact>fan-in</sail-pipeline-step>
        <sail-pipeline-step type="human" state="none" compact>sign-off</sail-pipeline-step>
      </row>
    </column>
  </column>

  <!-- ── sail-pipeline-card ─────────────────────────────────────────────── -->
  <column gap="4">
    <h2>
      style({ paint: fg(foreground), text: fontSize(base).fontWeight(semibold) })
<code>sail-pipeline-card</code></h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Props: <code>state: ok|running|awaiting|err|none</code>,
      <code>interactive: bool</code>.
      Slots: default (name), <code>desc</code>, <code>steps</code>,
      <code>recent</code>.
    </p>

    <column gap="3">
      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=none (default)</span>
      <sail-pipeline-card state="none">
        nightly-build
        <span slot="desc">Full build + test suite run every night at 02:00 UTC.</span>
        <span slot="steps">agent · gate · agent</span>
      </sail-pipeline-card>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=running · interactive</span>
      <sail-pipeline-card state="running" interactive>
        deploy-staging
        <span slot="desc">Builds the release image and promotes to staging.</span>
        <span slot="steps">agent · agent · gate · agent</span>
        <span slot="recent">run #42 · run #41 · run #40</span>
      </sail-pipeline-card>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=awaiting · interactive</span>
      <sail-pipeline-card state="awaiting" interactive>
        release-prod
        <span slot="desc">Promotes staging to production after human sign-off.</span>
        <span slot="steps">human · gate · agent</span>
        <span slot="recent">run #18 · run #17</span>
      </sail-pipeline-card>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=ok</span>
      <sail-pipeline-card state="ok">
        smoke-test
        <span slot="desc">Runs the smoke suite against the current deploy.</span>
        <span slot="steps">agent · merge</span>
        <span slot="recent">run #55 · run #54 · run #53</span>
      </sail-pipeline-card>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=err</span>
      <sail-pipeline-card state="err">
        integration-suite
        <span slot="desc">Full integration test pass across all services.</span>
        <span slot="steps">agent · agent · gate · merge · agent</span>
        <span slot="recent">run #9</span>
      </sail-pipeline-card>
    </column>
  </column>

  <!-- ── sail-pipeline-run ──────────────────────────────────────────────── -->
  <column gap="4">
    <h2>
      style({ paint: fg(foreground), text: fontSize(base).fontWeight(semibold) })
<code>sail-pipeline-run</code></h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Props: <code>state: ok|running|awaiting|err|none</code>.
      Slots: default (name), <code>meta</code>, <code>rail</code>.
    </p>

    <column gap="3">
      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=running</span>
      <sail-pipeline-run state="running">
        deploy-staging / run #42
        <span slot="meta">started 3m ago</span>
        <row slot="rail" gap="1">
          <sail-pipeline-step type="agent" state="ok" compact>build</sail-pipeline-step>
          <sail-pipeline-step type="gate"  state="running" compact>smoke</sail-pipeline-step>
          <sail-pipeline-step type="agent" state="none" compact>promote</sail-pipeline-step>
        </row>
      </sail-pipeline-run>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=awaiting</span>
      <sail-pipeline-run state="awaiting">
        release-prod / run #18
        <span slot="meta">waiting 12m</span>
        <row slot="rail" gap="1">
          <sail-pipeline-step type="agent" state="ok" compact>build</sail-pipeline-step>
          <sail-pipeline-step type="human" state="awaiting" compact>sign-off</sail-pipeline-step>
          <sail-pipeline-step type="agent" state="none" compact>deploy</sail-pipeline-step>
        </row>
      </sail-pipeline-run>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=ok</span>
      <sail-pipeline-run state="ok">
        nightly-build / run #101
        <span slot="meta">finished 2h ago · 4m 12s</span>
        <row slot="rail" gap="1">
          <sail-pipeline-step type="agent" state="ok" compact>lint</sail-pipeline-step>
          <sail-pipeline-step type="agent" state="ok" compact>test</sail-pipeline-step>
          <sail-pipeline-step type="merge" state="ok" compact>report</sail-pipeline-step>
        </row>
      </sail-pipeline-run>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
state=err</span>
      <sail-pipeline-run state="err">
        integration-suite / run #9
        <span slot="meta">failed 5h ago · 1m 03s</span>
        <row slot="rail" gap="1">
          <sail-pipeline-step type="agent" state="ok" compact>setup</sail-pipeline-step>
          <sail-pipeline-step type="agent" state="err" compact>e2e-tests</sail-pipeline-step>
          <sail-pipeline-step type="merge" state="none" compact>summary</sail-pipeline-step>
        </row>
      </sail-pipeline-run>
    </column>
  </column>

  <!-- ── sail-pipeline-dag ──────────────────────────────────────────────── -->
  <column gap="4">
    <h2>
      style({ paint: fg(foreground), text: fontSize(base).fontWeight(semibold) })
<code>sail-pipeline-dag</code></h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Props: <code>steps: string</code> (JSON array of
      <code>&#123; id, type, state?, after?: string[] &#125;</code>).
      The controller lays out a dependency DAG and renders it as SVG.
    </p>

    <column gap="3">
      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
linear chain — all states</span>
      <sail-pipeline-dag steps='[{"id":"lint","type":"agent","state":"ok"},{"id":"test","type":"agent","state":"ok","after":["lint"]},{"id":"review","type":"gate","state":"awaiting","after":["test"]},{"id":"deploy","type":"agent","state":"none","after":["review"]}]'></sail-pipeline-dag>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
fan-out / fan-in — running</span>
      <sail-pipeline-dag steps='[{"id":"build","type":"agent","state":"ok"},{"id":"unit","type":"agent","state":"ok","after":["build"]},{"id":"e2e","type":"agent","state":"running","after":["build"]},{"id":"merge","type":"merge","state":"none","after":["unit","e2e"]}]'></sail-pipeline-dag>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
human step mid-pipeline — err branch</span>
      <sail-pipeline-dag steps='[{"id":"prepare","type":"agent","state":"ok"},{"id":"sign-off","type":"human","state":"awaiting","after":["prepare"]},{"id":"promote","type":"agent","state":"err","after":["sign-off"]}]'></sail-pipeline-dag>
    </column>
  </column>

  <!-- ── sail-log-stream ────────────────────────────────────────────────── -->
  <column gap="4">
    <h2>
      style({ paint: fg(foreground), text: fontSize(base).fontWeight(semibold) })
<code>sail-log-stream</code></h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Props: <code>url: string</code>, <code>event: string</code> (default
      <code>log</code>), <code>autoscroll: bool</code>,
      <code>active: bool</code>, <code>max-bytes: number</code> (default 1 MB).
      Append-only SSE log pane. When <code>active=false</code> (default) no
      connection is opened — safe for gallery use. The pre shows the empty
      placeholder.
    </p>

    <column gap="3">
      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
inactive (default) — placeholder state</span>
      <sail-log-stream url="" event="log"></sail-log-stream>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
inactive · autoscroll prop set — no connection opened</span>
      <sail-log-stream url="/_deck/streams/demo-run" event="log" autoscroll></sail-log-stream>

      <span>
        style({ box: padT(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
inactive · max-bytes=4096 — same placeholder</span>
      <sail-log-stream url="/_deck/streams/demo-run" event="log" max-bytes="4096"></sail-log-stream>
    </column>
  </column>
</column>
