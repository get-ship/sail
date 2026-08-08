<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style pageHead = { box: padB(4), paint: borderBWidth(1).borderB(border) }
  style title = { box: mb(2), paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style lede = { paint: fg(muted-foreground), text: fontSize(sm) }
  style card = { box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style cardTitle = { paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) }
  style badge = {
    box: padX(2).padY(1),
    paint: fg(warning-foreground).bg(warning).radius(sm),
    text: fontSize(xs).fontWeight(medium).fontFamily(body).tracking(wide),
  }
  style body = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style stage = { box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) }
  style propList = { paint: fg(muted-foreground) }
  style propRows = { box: display(contents), text: fontSize(sm) }
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
