<!--
  /sail/guide/embedding-third-party-code — the config-grant story told as a
  task. Two real grants, two real components, two verbatim config files:
  code-editor needs `unconfined` (dynamic import), canvas needs
  `network: ["self"]` (fetch). Different grants, different reasons — the page
  keeps them apart on purpose.
-->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Guide</p>
  <h1>Embedding third-party code</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    Sooner or later a component needs something the language does not hand
    it: a library loaded on demand, or a call to a server. Both are
    available, and both are refused until a file next to your component says
    they are allowed. This guide is that file, and the two grants that
    matter.
  </p>

  <h2>The default is refusal</h2>
  <p>
    Controllers are <strong>confined</strong> by default. Confinement is not
    a restriction on JavaScript — you can write whatever logic you like, and
    ordinary imports are untouched. It is a short, enumerated list of ways
    code reaches outside the page. This is the entire list, from the rule
    that enforces it:
  </p>
  <code-block code={code.banned} lang="rust"></code-block>
  <p>
    Alongside those: dynamic <code>import()</code>,
    <code>document.cookie</code>, <code>navigator.sendBeacon</code>, and
    <code>new XMLHttpRequest()</code>. Use one in a confined controller and
    the build stops with a message naming what you used and what to do about
    it.
  </p>
  <p>
    A confined package declares no grant at all. This is the primitives
    pack's whole config — styling capabilities, nothing else:
  </p>
  <code-block code={code.confined} lang="json"></code-block>
  <p>
    The file is <code>sail.config.json</code>, and it governs its own
    directory and everything below it. The compiler walks up from a source
    file and uses the first one it finds, so the grant lives with the package
    that needs it rather than in a project-wide setting. There are exactly
    seven recognized keys, and an unrecognized one is rejected outright —
    with the whole file's grants voided rather than partially honored:
  </p>
  <code-block code={code.keys} lang="rust"></code-block>

  <h2>Grant one: <code>network</code> — talking to a server</h2>
  <p>
    The canvas pack has an annotation layer that posts a note back to the
    application it is embedded in:
  </p>
  <code-block code={code.canvasNeed} lang="ts"></code-block>
  <p>Which is why its config carries a network allowlist:</p>
  <code-block code={code.canvasConfig} lang="json"></code-block>
  <p>
    <code>network</code> is an array of origins, and the values are checked.
    <code>"self"</code> permits same-origin and relative URLs —
    <code>/api/feedback</code> is relative, so <code>"self"</code> is exactly
    the grant this needs and nothing more. Naming a host permits that host:
  </p>
  <code-block code={code.namedOrigin} lang="json"></code-block>
  <p>
    The same list is enforced three times, which is worth knowing because it
    means the config is not advisory. The compiler checks call sites against
    it. The runtime hands the controller a <code>fetch</code> wrapper only
    when the list is non-empty — with an empty list, <code>fetch</code> is
    simply not defined in the sandbox. And the server derives its
    <code>connect-src</code> Content-Security-Policy header from the same
    list, so the browser refuses the connection too.
  </p>
  <p sx="text-sm text-muted-foreground">
    A URL built at runtime rather than written as a literal is allowed only
    when the allowlist is non-empty; the compiler cannot check a value it
    cannot see, so it defers to the runtime and the CSP.
  </p>

  <h2>Grant two: <code>unconfined</code> — loading a library on demand</h2>
  <p>
    The code editor embeds CodeMirror. That part needs no grant at all:
    CodeMirror is a plain import, and plain imports are never confined. What
    needs the grant is how the editor loads its language server — lazily,
    only once someone actually types:
  </p>
  <code-block code={code.editorNeed} lang="ts"></code-block>
  <p>
    Dynamic <code>import()</code> is on the always-banned list. It is not
    network-gated, because it is not a network question: it evaluates code
    chosen at runtime, which no origin allowlist can describe. The only key
    that clears it is <code>unconfined</code>:
  </p>
  <code-block code={code.editorConfig} lang="json"></code-block>
  <p>
    <code>unconfined: true</code> skips the gate entirely — every item on the
    banned list, including <code>eval</code> and unrestricted
    <code>fetch</code>. It is the widest grant in the system and the one to
    reach for last.
  </p>

  <h2>Choosing between them</h2>
  <p>
    The two grants are independent; neither implies the other. If your
    component talks to a server, you want <code>network</code>, with the
    narrowest list that works — usually <code>["self"]</code>. If it must
    evaluate code it chooses at runtime — a lazily loaded module, a plugin
    system — <code>unconfined</code> is the only option, and the honest way
    to use it is to keep that code in its own package so the grant covers as
    little as possible. That is why these two live in separate packs rather
    than one permissive one.
  </p>
  <p>
    Anything the SDK already offers should go through the SDK instead. That
    is what the diagnostic suggests when it fires, and a call routed through
    it needs no grant.
  </p>

  <h2>When the build stops</h2>
  <p>
    The diagnostic is <code>sail-controller-no-ambient-authority</code>, and
    it comes in two shapes. If you used something always banned, it names
    what you used. If you used <code>fetch</code> to an origin that is not on
    your list, it says so and tells you where the list lives. In both cases
    the fix is one of three things: route it through the SDK, add the origin
    to <code>network</code>, or — knowing what it costs — grant
    <code>unconfined</code>.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" justify="between" gap="2">
    <a variant="default" href="/sail/guide/making-a-theme">← Making a theme</a>
    <a variant="solid" tone="primary" href="/sail/reference">Reference →</a>
  </row>
</column>
