<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style pageHead = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lede = { fg(muted-foreground).type({ size: sm }) }
  style card = { pad(5).border({ width: 1, color: border }).radius(lg).bg(surface) }
  style cardTitle = { fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style badge = { pad({ x: 2, y: 1 }).fg(warning-foreground).bg(warning).radius(sm).type({ size: xs, weight: medium, family: body, tracking: wide }) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style stage = { pad(3).border({ width: 1, color: divider }).radius(md).bg(surface) }
  style propList = { fg(muted-foreground) }
  style propRows = { display(contents).type({ size: sm }) }
</scope>

<column gap="8" style(page)>
  <header style(pageHead)>
    <h1 style(title)>sail/canvas</h1>
    <p style(lede)>
      Pack <strong>sail/canvas</strong> — pan/zoom substrate, SVG diagram board,
      freehand whiteboard, and graph layout surface.
      Cards marked <em>domain-stub</em> await a backend spec before live data
      is wired; all props below are static mock values only.
    </p>
  </header>

  <grid min="sm">

    <column gap="4" style(card)>
      <row gap="3">
        <span style(cardTitle)>sail-canvas</span>
        <span style(badge)>domain-stub — awaits backend spec</span>
      </row>
      <p style(body)>
        Bare pan/zoom/infinite-surface substrate. Hosts arbitrary world-space
        content via the default slot; pan and zoom with pointer/wheel.
      </p>
      <column gap="2" style(stage)>
        <sail-canvas theme="light" grid="dots" label="Demo canvas">
          <div>Node A</div>
          <div>Node B</div>
        </sail-canvas>
      </column>
      <dl style(propList)>
        <span style(propRows)>
          <dt>theme</dt><dd>"light"</dd>
          <dt>grid</dt><dd>"dots"</dd>
          <dt>label</dt><dd>"Demo canvas"</dd>
          <dt>readonly</dt><dd>false</dd>
        </span>
      </dl>
    </column>

    <column gap="4" style(card)>
      <row gap="3">
        <span style(cardTitle)>sail-board</span>
        <span style(badge)>domain-stub — awaits backend spec</span>
      </row>
      <p style(body)>
        SVG-native node/edge/port diagram. Nodes are DOM divs; edges are SVG
        paths. Geometry is owned by the board; slot content owns visuals.
      </p>
      <column gap="2" style(stage)>
        <sail-board theme="light" grid="dots" layout="none" label="Demo board">
          <div slot="node-a">Agent A</div>
          <div slot="node-b">Agent B</div>
        </sail-board>
      </column>
      <dl style(propList)>
        <span style(propRows)>
          <dt>theme</dt><dd>"light"</dd>
          <dt>grid</dt><dd>"dots"</dd>
          <dt>layout</dt><dd>"none"</dd>
          <dt>label</dt><dd>"Demo board"</dd>
        </span>
      </dl>
    </column>

    <column gap="4" style(card)>
      <row gap="3">
        <span style(cardTitle)>sail-draw</span>
        <span style(badge)>domain-stub — awaits backend spec</span>
      </row>
      <p style(body)>
        Excalidraw-class freehand whiteboard. SVG-native strokes + shapes +
        selection. No canvas engine — token-themeable paths throughout.
      </p>
      <column gap="2" style(stage)>
        <sail-draw theme="light" grid="lines" tool="select" label="Demo whiteboard">
          <span slot="toolbar">toolbar slot</span>
        </sail-draw>
      </column>
      <dl style(propList)>
        <span style(propRows)>
          <dt>theme</dt><dd>"light"</dd>
          <dt>grid</dt><dd>"lines"</dd>
          <dt>tool</dt><dd>"select"</dd>
          <dt>sketchy</dt><dd>false</dd>
          <dt>label</dt><dd>"Demo whiteboard"</dd>
        </span>
      </dl>
    </column>

    <column gap="4" style(card)>
      <row gap="3">
        <span style(cardTitle)>sail-graph-canvas</span>
        <span style(badge)>domain-stub — awaits backend spec</span>
      </row>
      <p style(body)>
        Positioned-graph primitive. Renders bezier SVG edges + absolutely-
        positioned node cards. Per-kind node templates via the hidden "node"
        slot.
      </p>
      <column gap="2" style(stage)>
        <sail-graph-canvas
          nodes='[{"id":"a","label":"Commander","kind":"agent","x":120,"y":80},{"id":"b","label":"Coder","kind":"agent","x":340,"y":180}]'
          edges='[{"id":"e1","source":"a","target":"b","kind":"agent"}]'
          width="480"
          height="240"
          layout="positioned"
        ></sail-graph-canvas>
      </column>
      <dl style(propList)>
        <span style(propRows)>
          <dt>nodes</dt><dd>2 mock nodes (JSON)</dd>
          <dt>edges</dt><dd>1 mock edge (JSON)</dd>
          <dt>width</dt><dd>480</dd>
          <dt>height</dt><dd>240</dd>
          <dt>layout</dt><dd>"positioned"</dd>
        </span>
      </dl>
    </column>

  </grid>
</column>
