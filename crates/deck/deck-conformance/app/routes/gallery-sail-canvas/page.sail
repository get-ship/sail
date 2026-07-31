<column gap="8" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <header sx={style.borderBWidth(1).borderB(border).padB(4)}>
    <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground).mb(2)}>sail/canvas</h1>
    <p sx={style.fg(muted-foreground).fontSize(sm)}>
      Pack <strong>sail/canvas</strong> — pan/zoom substrate, SVG diagram board,
      freehand whiteboard, and graph layout surface.
      Cards marked <em>domain-stub</em> await a backend spec before live data
      is wired; all props below are static mock values only.
    </p>
  </header>

  <grid min="sm">

    <!-- sail-canvas ─────────────────────────────────────────────── -->
    <column gap="4" sx={style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)}>
      <row gap="3">
        <span sx={style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)}>sail-canvas</span>
        <span sx={style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).radius(sm).padX(2).padY(1).tracking(wide)}>domain-stub — awaits backend spec</span>
      </row>
      <p sx={style.fg(muted-foreground).fontSize(sm).leading(relaxed)}>
        Bare pan/zoom/infinite-surface substrate. Hosts arbitrary world-space
        content via the default slot; pan and zoom with pointer/wheel.
      </p>
      <column gap="2" sx={style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)}>
        <sail-canvas theme="light" grid="dots" label="Demo canvas">
          <div>Node A</div>
          <div>Node B</div>
        </sail-canvas>
      </column>
      <dl sx={style.fontSize(sm).fg(muted-foreground)}>
        <dt>theme</dt><dd>"light"</dd>
        <dt>grid</dt><dd>"dots"</dd>
        <dt>label</dt><dd>"Demo canvas"</dd>
        <dt>readonly</dt><dd>false</dd>
      </dl>
    </column>

    <!-- sail-board ──────────────────────────────────────────────── -->
    <column gap="4" sx={style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)}>
      <row gap="3">
        <span sx={style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)}>sail-board</span>
        <span sx={style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).radius(sm).padX(2).padY(1).tracking(wide)}>domain-stub — awaits backend spec</span>
      </row>
      <p sx={style.fg(muted-foreground).fontSize(sm).leading(relaxed)}>
        SVG-native node/edge/port diagram. Nodes are DOM divs; edges are SVG
        paths. Geometry is owned by the board; slot content owns visuals.
      </p>
      <column gap="2" sx={style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)}>
        <sail-board theme="light" grid="dots" layout="none" label="Demo board">
          <div slot="node-a">Agent A</div>
          <div slot="node-b">Agent B</div>
        </sail-board>
      </column>
      <dl sx={style.fontSize(sm).fg(muted-foreground)}>
        <dt>theme</dt><dd>"light"</dd>
        <dt>grid</dt><dd>"dots"</dd>
        <dt>layout</dt><dd>"none"</dd>
        <dt>label</dt><dd>"Demo board"</dd>
      </dl>
    </column>

    <!-- sail-draw ───────────────────────────────────────────────── -->
    <column gap="4" sx={style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)}>
      <row gap="3">
        <span sx={style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)}>sail-draw</span>
        <span sx={style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).radius(sm).padX(2).padY(1).tracking(wide)}>domain-stub — awaits backend spec</span>
      </row>
      <p sx={style.fg(muted-foreground).fontSize(sm).leading(relaxed)}>
        Excalidraw-class freehand whiteboard. SVG-native strokes + shapes +
        selection. No canvas engine — token-themeable paths throughout.
      </p>
      <column gap="2" sx={style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)}>
        <sail-draw theme="light" grid="lines" tool="select" label="Demo whiteboard">
          <span slot="toolbar">toolbar slot</span>
        </sail-draw>
      </column>
      <dl sx={style.fontSize(sm).fg(muted-foreground)}>
        <dt>theme</dt><dd>"light"</dd>
        <dt>grid</dt><dd>"lines"</dd>
        <dt>tool</dt><dd>"select"</dd>
        <dt>sketchy</dt><dd>false</dd>
        <dt>label</dt><dd>"Demo whiteboard"</dd>
      </dl>
    </column>

    <!-- sail-graph-canvas ───────────────────────────────────────── -->
    <column gap="4" sx={style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)}>
      <row gap="3">
        <span sx={style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)}>sail-graph-canvas</span>
        <span sx={style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).radius(sm).padX(2).padY(1).tracking(wide)}>domain-stub — awaits backend spec</span>
      </row>
      <p sx={style.fg(muted-foreground).fontSize(sm).leading(relaxed)}>
        Positioned-graph primitive. Renders bezier SVG edges + absolutely-
        positioned node cards. Per-kind node templates via the hidden "node"
        slot.
      </p>
      <column gap="2" sx={style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)}>
        <sail-graph-canvas
          nodes='[{"id":"a","label":"Commander","kind":"agent","x":120,"y":80},{"id":"b","label":"Coder","kind":"agent","x":340,"y":180}]'
          edges='[{"id":"e1","source":"a","target":"b","kind":"agent"}]'
          width="480"
          height="240"
          layout="positioned"
        ></sail-graph-canvas>
      </column>
      <dl sx={style.fontSize(sm).fg(muted-foreground)}>
        <dt>nodes</dt><dd>2 mock nodes (JSON)</dd>
        <dt>edges</dt><dd>1 mock edge (JSON)</dd>
        <dt>width</dt><dd>480</dd>
        <dt>height</dt><dd>240</dd>
        <dt>layout</dt><dd>"positioned"</dd>
      </dl>
    </column>

  </grid>
</column>
