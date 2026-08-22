<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style pageHead = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lede = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style card = { pad(5).border({ width: 1, color: border }).radius(lg).bg(surface) }
  style cardTitle = { fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style badge = { pad({ x: 2, y: half }).margin({ s: 3 }).fg(warning-foreground).bg(warning).border({ width: 1, color: warning }).radius(sm).type({ size: xs, weight: medium, family: body, tracking: wide, wrap: nowrap }) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style stage = { pad(3).border({ width: 1, color: divider }).radius(md).bg(surface) }
  style caption = { fg(muted-foreground).type({ size: xs, weight: medium, case: lowercase, tracking: wide, family: mono }) }
</scope>

<column gap="8" style(page)>
  <header style(pageHead)>
    <h1 style(title)>ship/code — Domain Stubs</h1>
    <p style(body)>
      Pack <strong>ship/code</strong>. Each card renders the real custom-element
      tag with static mock attribute values so the shell registers and paints.
      A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-code-editor</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>value: string</code> (form-value), <code>language: string</code>,
      <code>placeholder: string</code>, <code>readonly: bool</code>,
      <code>line-numbers: bool</code>, <code>size: sm|md|lg</code>,
      <code>tone: default|info|success|warning|destructive</code>,
      <code>theme: light|dark|auto</code>.
      Wraps CodeMirror 6; language extensions are lazy-imported. No slots.
    </p>

    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>size=sm · language=typescript · line-numbers</span>
        <ship-code-editor
          value="const greet = (name: string) =&gt; name.toUpperCase();"
          language="typescript"
          size="sm"
          line-numbers>
        </ship-code-editor>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>size=md · language=rust · tone=default (default)</span>
        <ship-code-editor
          value="fn main() { println!(&quot;Hello, world!&quot;); }"
          language="rust"
          size="md">
        </ship-code-editor>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>size=lg · language=json · tone=info</span>
        <ship-code-editor
          value="name: ship, version: 1.0.0"
          language="json"
          size="lg"
          tone="info">
        </ship-code-editor>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>readonly · tone=success · theme=dark</span>
        <ship-code-editor
          value="SELECT id, name FROM agents WHERE active = true;"
          language="sql"
          readonly
          tone="success"
          theme="dark">
        </ship-code-editor>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>tone=warning · placeholder shown</span>
        <ship-code-editor
          language="typescript"
          placeholder="Paste code here…"
          tone="warning">
        </ship-code-editor>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>tone=destructive · size=sm</span>
        <ship-code-editor
          value="import bad from &quot;danger&quot;;"
          language="typescript"
          size="sm"
          tone="destructive">
        </ship-code-editor>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-git-diff</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>files: number</code>, <code>additions: number</code>,
      <code>deletions: number</code>, <code>empty: bool</code>.
      Slot: default (accepts <code>ship-git-diff-file</code> children). Renders
      a toolbar with file count + add/remove totals and a collapse-all toggle.
    </p>

    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>2 files · +18 / -7 · expanded</span>
        <ship-git-diff files="2" additions="18" deletions="7">
          <ship-git-diff-file path="src/main.rs" additions="12" deletions="4">
            <ship-git-diff-hunk header="@@ -1,6 +1,8 @@ fn main()"></ship-git-diff-hunk>
            <ship-git-diff-line type="context" old="1" new="1">fn main()</ship-git-diff-line>
            <ship-git-diff-line type="del" old="2" new="">    println!("old message");</ship-git-diff-line>
            <ship-git-diff-line type="add" old="" new="2">    println!("new message");</ship-git-diff-line>
            <ship-git-diff-line type="add" old="" new="3">    println!("another line");</ship-git-diff-line>
            <ship-git-diff-line type="context" old="3" new="4">}</ship-git-diff-line>
          </ship-git-diff-file>
          <ship-git-diff-file path="Cargo.toml" additions="6" deletions="3">
            <ship-git-diff-hunk header="@@ -8,4 +8,7 @@ [dependencies]"></ship-git-diff-hunk>
            <ship-git-diff-line type="context" old="8" new="8" style(dependencies)></ship-git-diff-line>
            <ship-git-diff-line type="del" old="9" new="">serde = "1.0"</ship-git-diff-line>
            <ship-git-diff-line type="add" old="" new="9">serde = "2.0"</ship-git-diff-line>
            <ship-git-diff-line type="add" old="" new="10">tokio = "1"</ship-git-diff-line>
          </ship-git-diff-file>
        </ship-git-diff>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>1 file · collapsed · line with comment</span>
        <ship-git-diff files="1" additions="3" deletions="1">
          <ship-git-diff-file path="controller.ts" additions="3" deletions="1" collapsed>
            <ship-git-diff-hunk header="@@ -10,5 +10,7 @@ export default controller"></ship-git-diff-hunk>
            <ship-git-diff-line type="del" old="10" new="">  const count = signal(0);</ship-git-diff-line>
            <ship-git-diff-line type="add" old="" new="10" comment-count="2">  const count = signal(42);</ship-git-diff-line>
            <ship-git-diff-line type="context" old="11" new="11">  return count;</ship-git-diff-line>
          </ship-git-diff-file>
        </ship-git-diff>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>empty state</span>
        <ship-git-diff empty files="0" additions="0" deletions="0"></ship-git-diff>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-git-diff-file</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>path: string</code>, <code>additions: number</code>,
      <code>deletions: number</code>, <code>collapsed: bool</code>.
      Slot: default (accepts hunks + lines). Collapsible per-file header
      with path + stat counts. Normally slotted inside <code>ship-git-diff</code>.
    </p>

    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>expanded · additions + deletions</span>
        <ship-git-diff-file path="packages/runtime/src/index.ts" additions="8" deletions="2">
          <ship-git-diff-hunk header="@@ -42,6 +42,12 @@ export"></ship-git-diff-hunk>
          <ship-git-diff-line type="context" old="42" new="42">export signal from reactive;</ship-git-diff-line>
          <ship-git-diff-line type="add" old="" new="43">export batch from reactive;</ship-git-diff-line>
          <ship-git-diff-line type="del" old="43" new="">// TODO: export batch</ship-git-diff-line>
        </ship-git-diff-file>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>collapsed · additions only</span>
        <ship-git-diff-file path="README.md" additions="40" deletions="0" collapsed>
          <ship-git-diff-line type="add" old="" new="1">Ship Framework</ship-git-diff-line>
        </ship-git-diff-file>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-git-diff-hunk</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>header: string</code>. No slots. Purely presentational —
      renders a mono hunk-header line (e.g. <code>@@ -10,7 +10,8 @@</code>).
      The daemon emits the header text; the template renders it inside a
      toned pill.
    </p>

    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>typical hunk header</span>
        <ship-git-diff-hunk header="@@ -10,7 +10,8 @@ fn main()"></ship-git-diff-hunk>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>file-start hunk</span>
        <ship-git-diff-hunk header="@@ -0,0 +1,24 @@"></ship-git-diff-hunk>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>large range hunk</span>
        <ship-git-diff-hunk header="@@ -128,42 +128,58 @@ impl Router"></ship-git-diff-hunk>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-git-diff-line</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>type: add|del|context</code>, <code>old: string</code>,
      <code>new: string</code>, <code>comment-count: number</code>.
      Slot: default (line content text). Single diff line with old/new gutters,
      +/-/space signature, and a hover-revealed comment button.
    </p>

    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>type=context</span>
        <ship-git-diff-line type="context" old="10" new="10">  let x = signal(0);</ship-git-diff-line>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>type=add</span>
        <ship-git-diff-line type="add" old="" new="11">  let x = signal(42);</ship-git-diff-line>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>type=del</span>
        <ship-git-diff-line type="del" old="11" new="">  let x = 0;</ship-git-diff-line>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>type=add · comment-count=3</span>
        <ship-git-diff-line type="add" old="" new="12" comment-count="3">  return count;</ship-git-diff-line>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-terminal</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>rows: number</code>, <code>cols: number</code>,
      <code>font-size: number</code>, <code>readonly: bool</code>,
      <code>theme: light|dark|auto</code>, <code>fit: bool</code>.
      No slots. Wraps xterm.js; exposes <code>write()</code>,
      <code>writeln()</code>, <code>clear()</code>, <code>focus()</code>
      imperative methods on the host.
    </p>

    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>rows=8 · cols=80 · theme=auto (default)</span>
        <ship-terminal rows="8" cols="80"></ship-terminal>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>rows=12 · readonly · theme=dark</span>
        <ship-terminal rows="12" cols="80" readonly theme="dark"></ship-terminal>
      </column>

      <column gap="2" style(stage)>
        <span style(caption)>rows=6 · font-size=11 · theme=light · fit</span>
        <ship-terminal rows="6" cols="80" font-size="11" theme="light" fit></ship-terminal>
      </column>
    </column>
  </column>
</column>
