<!-- FILE_CAP_OVERRIDE: gallery covers 8 arcade components × 2-3 variant cards + composed example; declarative markup, no logic. -->
<column gap="8" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <header sx="border-b border-b-border pb-4">
    <h1 sx="text-2xl font-bold text-foreground mb-2">ship/arcade — Game Engine Primitives</h1>
    <p sx="text-muted-foreground text-sm">
      Pack <strong>ship/arcade</strong>. Each card below instantiates the real
      custom element tag with representative prop combinations. The arcade
      elements are canvas-backed — <code>sail-stage</code> mounts a Pixi
      renderer; child elements (<code>sail-shape</code>, <code>sail-sprite</code>,
      <code>sail-collider</code>, <code>sail-emitter</code>, <code>sail-hud</code>,
      <code>sail-input</code>, <code>sail-sfx</code>) are read once by the stage
      at init and carry no visible DOM output themselves.
    </p>
  </header>

  <!-- ─── sail-stage ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-stage</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Root canvas host. Props: <code>width</code>, <code>height</code>,
      <code>tick-rate</code>, <code>fps</code>, <code>pixel-ratio</code>,
      <code>paused</code>, <code>background</code>,
      <code>renderer: auto|webgl|canvas</code>,
      <code>bounds: clamp|wrap|open</code>.
      Slot: child arcade elements + HUD.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">renderer=auto bounds=open (default)</span>
        <sail-stage width="320" height="200" renderer="auto" bounds="open" background="surface">
          <sail-shape kind="rect" x="60" y="80" width="40" height="40" fill="accent"></sail-shape>
          <sail-shape kind="circle" x="200" y="80" radius="28" fill="tone-success-bg"></sail-shape>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">renderer=canvas bounds=clamp paused</span>
        <sail-stage width="320" height="200" renderer="canvas" bounds="clamp" background="surface" paused>
          <sail-shape kind="polygon" x="160" y="100" points="0,-40 38,30 -38,30" fill="accent" stroke="foreground" stroke-width="2"></sail-shape>
        </sail-stage>
      </column>

    </row>
  </column>

  <!-- ─── sail-shape ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-shape</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Declarative geometric entity. Props: <code>kind: rect|circle|polygon|line</code>,
      <code>width</code>, <code>height</code>, <code>radius</code>,
      <code>points</code>, <code>x</code>, <code>y</code>, <code>z</code>,
      <code>vx</code>, <code>vy</code>, <code>scale</code>, <code>rotation</code>,
      <code>alpha</code>, <code>fill</code>, <code>stroke</code>,
      <code>stroke-width</code>, <code>group</code>, <code>tag</code>.
      Host: <code>display:none</code> — must live inside a <code>sail-stage</code>.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=rect + kind=circle</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-shape kind="rect" x="40" y="60" width="80" height="60" fill="accent" rotation="15"></sail-shape>
          <sail-shape kind="circle" x="220" y="100" radius="45" fill="tone-success-bg" alpha="0.85"></sail-shape>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=polygon (triangle) + kind=line</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-shape kind="polygon" x="90" y="110" points="0,-55 50,40 -50,40" fill="accent" stroke="foreground" stroke-width="2"></sail-shape>
          <sail-shape kind="line" x="180" y="50" points="0,0 100,120" stroke="tone-warning-bg" stroke-width="4"></sail-shape>
        </sail-stage>
      </column>

    </row>
  </column>

  <!-- ─── sail-sprite ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-sprite</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Declarative image entity. Props: <code>src</code>, <code>x</code>,
      <code>y</code>, <code>z</code>, <code>vx</code>, <code>vy</code>,
      <code>scale</code>, <code>rotation</code>,
      <code>anchor: center|top-left|…</code>, <code>tint</code>, <code>alpha</code>,
      <code>frames</code>, <code>frame-rate</code>, <code>frame</code>,
      <code>group</code>, <code>tag</code>.
      Host: <code>display:none</code> — must live inside a <code>sail-stage</code>.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">anchor=center (default) + anchor=top-left</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <!-- Sprite with no src falls back to a placeholder; the entity still spawns. -->
          <sail-sprite src="" x="80" y="100" scale="2" anchor="center" tag="player"></sail-sprite>
          <sail-sprite src="" x="220" y="60" scale="1.5" anchor="top-left" tag="item"></sail-sprite>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">tint + alpha + rotation</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-sprite src="" x="80" y="100" tint="accent" alpha="0.8" rotation="30" scale="2" tag="tinted-a"></sail-sprite>
          <sail-sprite src="" x="220" y="100" tint="tone-success-bg" alpha="0.5" rotation="-20" scale="1.8" tag="tinted-b"></sail-sprite>
        </sail-stage>
      </column>

    </row>
  </column>

  <!-- ─── sail-collider ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-collider</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Declares a hitbox on a parent <code>sail-shape</code> or
      <code>sail-sprite</code>. Props: <code>kind: aabb|circle</code>,
      <code>width</code>, <code>height</code>, <code>radius</code>,
      <code>offset-x</code>, <code>offset-y</code>, <code>group</code>,
      <code>collides-with</code>, <code>trigger</code>.
      No slots, no visible DOM output.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">aabb collider on a rect (solid, physical)</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-shape kind="rect" x="80" y="100" width="60" height="60" fill="accent" group="player">
            <sail-collider kind="aabb" width="60" height="60" group="player" collides-with="enemies"></sail-collider>
          </sail-shape>
          <sail-shape kind="rect" x="220" y="100" width="50" height="50" fill="tone-danger-bg" group="enemies">
            <sail-collider kind="aabb" width="50" height="50" group="enemies" collides-with="player"></sail-collider>
          </sail-shape>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">circle collider on a circle (trigger only)</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-shape kind="circle" x="100" y="100" radius="40" fill="accent" group="pickup">
            <sail-collider kind="circle" radius="40" group="pickup" collides-with="player" trigger></sail-collider>
          </sail-shape>
          <sail-shape kind="circle" x="230" y="100" radius="28" fill="tone-success-bg" group="player">
            <sail-collider kind="circle" radius="28" group="player" collides-with="pickup" trigger></sail-collider>
          </sail-shape>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">aabb with offset (hitbox shifted from center)</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-shape kind="rect" x="160" y="100" width="80" height="80" fill="accent" group="shifted">
            <sail-collider kind="aabb" width="40" height="40" offset-x="20" offset-y="-20" group="shifted" collides-with="wall"></sail-collider>
          </sail-shape>
        </sail-stage>
      </column>
    </row>
  </column>

  <!-- ─── sail-emitter ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-emitter</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Declarative particle emitter. Props: <code>kind: radial|cone|line|explosion</code>,
      <code>x</code>, <code>y</code>, <code>rate</code>, <code>burst</code>,
      <code>lifetime</code>, <code>spread</code>, <code>direction</code>,
      <code>speed</code>, <code>speed-jitter</code>, <code>gravity</code>,
      <code>color</code>, <code>size</code>, <code>max</code>, <code>on</code>.
      No slots, no visible DOM output.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=radial rate=20 continuous</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-emitter kind="radial" x="160" y="100" rate="20" burst="8" lifetime="800" spread="360" speed="80" speed-jitter="0.4" color="accent" size="4..0" max="80"></sail-emitter>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=cone direction=-90 (upward fountain)</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-emitter kind="cone" x="160" y="170" rate="15" burst="12" lifetime="1000" spread="40" direction="-90" speed="140" speed-jitter="0.3" gravity="60" color="tone-success-bg" size="5..0" max="64"></sail-emitter>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=explosion on=fire (burst on action)</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-emitter kind="explosion" x="160" y="100" rate="0" burst="32" lifetime="500" spread="360" speed="160" speed-jitter="0.5" gravity="80" color="tone-warning-bg" size="6..0" max="64" on="fire"></sail-emitter>
          <input map="Space" action="fire">
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=line horizontal sweep</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-emitter kind="line" x="160" y="100" rate="25" burst="10" lifetime="600" spread="20" direction="0" speed="100" speed-jitter="0.2" gravity="0" color="accent" size="3..0" max="96"></sail-emitter>
        </sail-stage>
      </column>
    </row>
  </column>

  <!-- ─── sail-hud ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-hud</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      DOM overlay above the canvas. Props:
      <code>align: top|bottom|left|right|center|corner-tl|corner-tr|corner-bl|corner-br</code>,
      <code>z-layer</code>,
      <code>pointer-events: auto|none</code>.
      Slot: default (any DOM content, bound to stage signals via
      <code>data-bind</code>).
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">align=corner-tl + align=corner-br</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-hud align="corner-tl" z-layer="2">
            <span>Score: 0</span>
          </sail-hud>
          <sail-hud align="corner-br" z-layer="2">
            <span>Lives: 3</span>
          </sail-hud>
          <sail-shape kind="rect" x="160" y="100" width="50" height="50" fill="accent"></sail-shape>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">align=top + align=bottom</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-hud align="top">
            <span>LEVEL 1</span>
          </sail-hud>
          <sail-hud align="bottom" pointer-events="auto">
            <button>Pause</button>
          </sail-hud>
          <sail-shape kind="circle" x="160" y="100" radius="30" fill="accent"></sail-shape>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">align=center (game-over overlay)</span>
        <sail-stage width="320" height="200" background="surface" paused>
          <sail-hud align="center" pointer-events="auto">
            <column gap="2" align="center">
              <strong>GAME OVER</strong>
              <button>Restart</button>
            </column>
          </sail-hud>
        </sail-stage>
      </column>
    </row>
  </column>

  <!-- ─── sail-input ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-input</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Declarative key-to-action binding for the enclosing stage. Props:
      <code>map</code> (comma-separated key names, e.g. <code>"ArrowLeft,A"</code>),
      <code>action</code> (logical name the game listens on),
      <code>repeat</code> (whether to emit while held).
      No slots, no visible DOM output.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">WASD + arrow keys → move actions</span>
        <sail-stage width="320" height="200" background="surface">
          <input map="ArrowLeft,A" action="move-left">
          <input map="ArrowRight,D" action="move-right">
          <input map="ArrowUp,W" action="move-up">
          <input map="ArrowDown,S" action="move-down">
          <sail-hud align="center">
            <span>WASD / Arrows → move</span>
          </sail-hud>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">Space → fire, Escape → pause (repeat=false)</span>
        <sail-stage width="320" height="200" background="surface">
          <input map="Space" action="fire" repeat="false">
          <input map="Escape" action="pause" repeat="false">
          <sail-hud align="center">
            <span>Space=fire  Esc=pause</span>
          </sail-hud>
        </sail-stage>
      </column>

    </row>
  </column>

  <!-- ─── sail-sfx ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground"><code sx="font-mono">sail-sfx</code></h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Declarative Web Audio one-shot binding. Props: <code>src</code> (audio URL),
      <code>on</code> (action name to trigger playback),
      <code>volume</code>, <code>pitch-range</code> (<code>"lo..hi"</code>),
      <code>bus: sfx|music|ui</code>.
      No slots, no visible DOM output. Preloads on stage init; plays on action.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">bus=sfx pitch-range=0.9..1.1 (randomised)</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-sfx src="/sounds/laser.wav" on="fire" volume="0.8" pitch-range="0.9..1.1" bus="sfx"></sail-sfx>
          <input map="Space" action="fire" repeat="false">
          <sail-hud align="center">
            <span>Space → laser (bus=sfx)</span>
          </sail-hud>
        </sail-stage>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">bus=ui volume=1 (button click feedback)</span>
        <sail-stage width="320" height="200" background="surface">
          <sail-sfx src="/sounds/click.wav" on="ui-click" volume="1" pitch-range="0.95..1.05" bus="ui"></sail-sfx>
          <sail-hud align="center" pointer-events="auto">
            <button on:click={() => void 0}>Click (bus=ui)</button>
          </sail-hud>
        </sail-stage>
      </column>
    </row>
  </column>

  <!-- ─── Composed example: a stage with all primitives ─── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground">Composed — all primitives in one stage</h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      A single <code>sail-stage</code> composing
      <code>sail-shape</code>, <code>sail-collider</code>,
      <code>sail-emitter</code>, <code>sail-hud</code>, <code>sail-input</code>,
      and <code>sail-sfx</code> together, mirroring a minimal game scene.
    </p>

    <row gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2 w-full">
        <sail-stage width="640" height="280" renderer="auto" bounds="wrap" background="surface" tick-rate="60" fps="60">
          <!-- player ship -->
          <sail-shape kind="polygon" x="80" y="140" points="0,-24 20,20 -20,20" fill="accent" tag="player" group="player" vx="0" vy="0">
            <sail-collider kind="aabb" width="40" height="44" group="player" collides-with="enemies"></sail-collider>
          </sail-shape>

          <!-- enemy rect -->
          <sail-shape kind="rect" x="500" y="80" width="48" height="48" fill="tone-danger-bg" tag="enemy-a" group="enemies" vx="-60" vy="40">
            <sail-collider kind="aabb" width="48" height="48" group="enemies" collides-with="player,bullets"></sail-collider>
          </sail-shape>

          <!-- pickup circle -->
          <sail-shape kind="circle" x="340" y="140" radius="18" fill="tone-success-bg" tag="pickup" group="pickups">
            <sail-collider kind="circle" radius="18" group="pickups" collides-with="player" trigger></sail-collider>
          </sail-shape>

          <!-- thruster particle trail -->
          <sail-emitter kind="cone" x="80" y="152" rate="30" burst="6" lifetime="400" spread="30" direction="90" speed="90" speed-jitter="0.4" gravity="0" color="accent" size="3..0" max="48"></sail-emitter>

          <!-- explosion on hit -->
          <sail-emitter kind="explosion" x="500" y="80" rate="0" burst="24" lifetime="400" spread="360" speed="140" speed-jitter="0.5" gravity="60" color="tone-danger-bg" size="5..0" max="64" on="enemy-hit"></sail-emitter>

          <!-- inputs -->
          <input map="ArrowLeft,A" action="move-left">
          <input map="ArrowRight,D" action="move-right">
          <input map="ArrowUp,W" action="move-up">
          <input map="ArrowDown,S" action="move-down">
          <input map="Space" action="fire" repeat="false">

          <!-- sfx -->
          <sail-sfx src="/sounds/laser.wav" on="fire" volume="0.7" pitch-range="0.9..1.1" bus="sfx"></sail-sfx>
          <sail-sfx src="/sounds/boom.wav" on="enemy-hit" volume="1" pitch-range="0.85..1.15" bus="sfx"></sail-sfx>

          <!-- HUD -->
          <sail-hud align="corner-tl" z-layer="2">
            <span data-bind="score">Score: 0</span>
          </sail-hud>
          <sail-hud align="corner-tr" z-layer="2">
            <span data-bind="state">idle</span>
          </sail-hud>
          <sail-hud align="bottom">
            <span>WASD=move  Space=fire</span>
          </sail-hud>
        </sail-stage>
      </column>
    </row>
  </column>
</column>
