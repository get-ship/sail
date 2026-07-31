(() => {
  var __defProp = Object.defineProperty;
  var __returnValue = (v) => v;
  function __exportSetter(name, newValue) {
    this[name] = __returnValue.bind(null, newValue);
  }
  var __export = (target, all) => {
    for (var name in all)
      __defProp(target, name, {
        get: all[name],
        enumerable: true,
        configurable: true,
        set: __exportSetter.bind(all, name)
      });
  };
  var __require = /* @__PURE__ */ ((x) => typeof require !== "undefined" ? require : typeof Proxy !== "undefined" ? new Proxy(x, {
    get: (a, b) => (typeof require !== "undefined" ? require : a)[b]
  }) : x)(function(x) {
    if (typeof require !== "undefined")
      return require.apply(this, arguments);
    throw Error('Dynamic require of "' + x + '" is not supported');
  });

  // ../runtime/src/actions/index.ts
  var exports_actions = {};
  __export(exports_actions, {
    zoomPan: () => zoomPan,
    worldTransform: () => worldTransform,
    virtualSpacer: () => virtualSpacer,
    virtualRow: () => virtualRow,
    variantLive: () => variantLive,
    toggleGroup: () => toggleGroup,
    tipDescribe: () => tipDescribe,
    themeSelect: () => themeSelect,
    targetCursor: () => targetCursor,
    tabs: () => tabs,
    syntaxHighlight: () => syntaxHighlight,
    swatch: () => swatch,
    spotlight: () => spotlight,
    rect: () => rect,
    recoverySurface: () => recoverySurface,
    rangeFill: () => rangeFill,
    radioGroup: () => radioGroup,
    primeIconCache: () => primeIconCache,
    pressed: () => pressed,
    pressable: () => pressable,
    portEdge: () => portEdge,
    pin: () => pin,
    pagination: () => pagination,
    menu: () => menu,
    mediaFallback: () => mediaFallback,
    maxWidth: () => maxWidth,
    marquee: () => marquee,
    liveRegion: () => liveRegion,
    icon: () => icon,
    hoverSurface: () => hoverSurface,
    gridTracks: () => gridTracks,
    eventSpan: () => eventSpan,
    dropzone: () => dropzone,
    dismissible: () => dismissible,
    dismissControl: () => dismissControl,
    detailsToggle: () => detailsToggle,
    detailsDisabled: () => detailsDisabled,
    describedControl: () => describedControl,
    contextMenu: () => contextMenu,
    combobox: () => combobox,
    clickOutside: () => clickOutside,
    chromeOffset: () => chromeOffset,
    cellPad: () => cellPad,
    catchRender: () => catchRender,
    autosize: () => autosize,
    autoDismiss: () => autoDismiss,
    anchor: () => anchor
  });

  // ../runtime/src/actions/autosize.ts
  var autosize = (node) => {
    const el = node;
    el.dataset.autosize = "";
    const reflow = () => {
      el.style.setProperty("--autosize-h", "auto");
      el.style.setProperty("--autosize-h", `${el.scrollHeight}px`);
    };
    reflow();
    el.addEventListener("input", reflow);
    return {
      destroy: () => {
        el.removeEventListener("input", reflow);
        el.style.removeProperty("--autosize-h");
        delete el.dataset.autosize;
      }
    };
  };
  // ../runtime/src/actions/click-outside.ts
  var clickOutside = (node, onOutside) => {
    const handler = (e) => {
      const target = e.target;
      if (target && !node.contains(target))
        onOutside?.();
    };
    let armed = false;
    const arm = () => {
      armed = true;
      document.addEventListener("pointerdown", handler, true);
    };
    const raf = requestAnimationFrame(arm);
    return {
      update: (fn) => {
        onOutside = fn;
      },
      destroy: () => {
        cancelAnimationFrame(raf);
        if (armed)
          document.removeEventListener("pointerdown", handler, true);
      }
    };
  };
  // ../runtime/src/actions/anchor.ts
  var GAP = 4;
  var clamp = (v, lo, hi) => Math.max(lo, Math.min(v, hi));
  var anchor = (node, placement) => {
    const el = node;
    let place = placement ?? (el.getAttribute("placement") || "bottom");
    const trigger = () => {
      if (!el.id)
        return null;
      const sel = `[popovertarget="${el.id}"]`;
      const root = el.getRootNode();
      const inRoot = root.querySelector?.(sel);
      if (inRoot)
        return inRoot;
      return root.host?.querySelector(sel) ?? null;
    };
    let openMirror = false;
    const isOpen = () => {
      try {
        return el.matches(":popover-open");
      } catch {
        return openMirror;
      }
    };
    const reposition = () => {
      if (!isOpen())
        return;
      const t = trigger();
      if (!t)
        return;
      const r = t.getBoundingClientRect();
      const w = el.offsetWidth;
      const h = el.offsetHeight;
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      const side = place.split("-")[0];
      const end = place.endsWith("-end");
      let top;
      let left;
      if (side === "top" || side === "bottom") {
        top = side === "bottom" ? r.bottom + GAP : r.top - h - GAP;
        if (side === "bottom" && top + h > vh && r.top - h - GAP >= 0) {
          top = r.top - h - GAP;
        } else if (side === "top" && top < 0 && r.bottom + h + GAP <= vh) {
          top = r.bottom + GAP;
        }
        left = end ? r.right - w : r.left;
      } else {
        left = side === "right" ? r.right + GAP : r.left - w - GAP;
        if (side === "right" && left + w > vw && r.left - w - GAP >= 0) {
          left = r.left - w - GAP;
        } else if (side === "left" && left < 0 && r.right + w + GAP <= vw) {
          left = r.right + GAP;
        }
        top = end ? r.bottom - h : r.top;
      }
      el.style.setProperty("--anchor-left", `${clamp(left, GAP, Math.max(GAP, vw - w - GAP))}px`);
      el.style.setProperty("--anchor-top", `${clamp(top, GAP, Math.max(GAP, vh - h - GAP))}px`);
      el.style.setProperty("--anchor-width", `${Math.max(r.width, 0)}px`);
    };
    const onToggle = (e) => {
      const opening = e.newState === "open";
      openMirror = opening;
      if (opening)
        requestAnimationFrame(reposition);
    };
    el.addEventListener("toggle", onToggle);
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, true);
    return {
      update: (p) => {
        place = p ?? "bottom";
        reposition();
      },
      destroy: () => {
        el.removeEventListener("toggle", onToggle);
        window.removeEventListener("resize", reposition);
        window.removeEventListener("scroll", reposition, true);
        el.style.removeProperty("--anchor-left");
        el.style.removeProperty("--anchor-top");
        el.style.removeProperty("--anchor-width");
      }
    };
  };
  // ../runtime/src/actions/dropzone.ts
  var dropzone = (node) => {
    const el = node;
    const getInput = () => el.querySelector('input[type="file"]');
    const getSummary = () => el.querySelector("[data-dz-files]");
    let summaryDefault = null;
    const reflect = () => {
      const input = getInput();
      const summary = getSummary();
      if (!input || !summary)
        return;
      if (summaryDefault === null)
        summaryDefault = summary.textContent ?? "";
      const n = input.files?.length ?? 0;
      summary.textContent = n === 0 ? summaryDefault : n === 1 ? input.files?.[0]?.name ?? "1 file" : `${n} files selected`;
    };
    const onClick = (e) => {
      const input = getInput();
      if (input && e.target !== input && !input.contains(e.target)) {
        input.click();
      }
    };
    const onOver = (e) => {
      e.preventDefault();
      el.setAttribute("data-drag", "");
    };
    const onLeave = (e) => {
      if (!el.contains(e.relatedTarget))
        el.removeAttribute("data-drag");
    };
    const onDrop = (e) => {
      e.preventDefault();
      el.removeAttribute("data-drag");
      const input = getInput();
      if (input && e.dataTransfer?.files?.length) {
        input.files = e.dataTransfer.files;
        input.dispatchEvent(new Event("change", { bubbles: true }));
      }
    };
    el.addEventListener("click", onClick);
    el.addEventListener("dragover", onOver);
    el.addEventListener("dragleave", onLeave);
    el.addEventListener("drop", onDrop);
    el.addEventListener("change", reflect);
    return {
      destroy: () => {
        el.removeEventListener("click", onClick);
        el.removeEventListener("dragover", onOver);
        el.removeEventListener("dragleave", onLeave);
        el.removeEventListener("drop", onDrop);
        el.removeEventListener("change", reflect);
      }
    };
  };
  // ../runtime/src/actions/media-fallback.ts
  var mediaFallback = (node) => {
    const el = node;
    const doc = el.ownerDocument;
    const initialsOf = () => {
      const explicit = el.getAttribute("initials")?.trim();
      if (explicit)
        return explicit.slice(0, 2).toUpperCase();
      const name = el.getAttribute("name")?.trim() ?? "";
      return name.slice(0, 1).toUpperCase() || "?";
    };
    const showInitials = () => {
      el.textContent = "";
      const span = doc.createElement("span");
      span.setAttribute("part", "initials");
      span.textContent = initialsOf();
      el.appendChild(span);
    };
    const render = () => {
      const slug = el.getAttribute("icon-slug")?.trim();
      const src = slug ? `https://cdn.simpleicons.org/${slug}` : el.getAttribute("src")?.trim() ?? "";
      if (!src) {
        showInitials();
        return;
      }
      el.textContent = "";
      const img = doc.createElement("img");
      img.setAttribute("part", "img");
      img.alt = el.getAttribute("name") ?? "";
      img.addEventListener("error", showInitials, { once: true });
      img.src = src;
      el.appendChild(img);
    };
    render();
    const obs = new MutationObserver(render);
    obs.observe(el, {
      attributes: true,
      attributeFilter: ["src", "icon-slug", "name", "initials"]
    });
    return { destroy: () => obs.disconnect() };
  };
  // ../runtime/src/actions/tip-describe.ts
  var _seq = 0;
  var applyPlacement = (el, raw) => {
    const p = raw === "bottom" || raw === "left" || raw === "right" ? raw : "top";
    const gap = "calc(100% + var(--space-2))";
    const arrow = "calc(-1 * var(--space-1))";
    const shift = "-3.5px";
    for (const side of ["top", "bottom", "left", "right"]) {
      el.style.setProperty(`--tt-${side}`, "auto");
      el.style.setProperty(`--tt-arrow-${side}`, "auto");
    }
    el.style.setProperty("--tt-tx", "0");
    el.style.setProperty("--sx-translate-y", "0");
    el.style.setProperty("--tt-arrow-ml", "0px");
    el.style.setProperty("--tt-arrow-mt", "0px");
    if (p === "bottom") {
      el.style.setProperty("--tt-top", gap);
      el.style.setProperty("--tt-left", "50%");
      el.style.setProperty("--tt-tx", "-50%");
      el.style.setProperty("--tt-arrow-top", arrow);
      el.style.setProperty("--tt-arrow-left", "50%");
      el.style.setProperty("--tt-arrow-ml", shift);
    } else if (p === "left") {
      el.style.setProperty("--tt-right", gap);
      el.style.setProperty("--tt-top", "50%");
      el.style.setProperty("--sx-translate-y", "-50%");
      el.style.setProperty("--tt-arrow-right", arrow);
      el.style.setProperty("--tt-arrow-top", "50%");
      el.style.setProperty("--tt-arrow-mt", shift);
    } else if (p === "right") {
      el.style.setProperty("--tt-left", gap);
      el.style.setProperty("--tt-top", "50%");
      el.style.setProperty("--sx-translate-y", "-50%");
      el.style.setProperty("--tt-arrow-left", arrow);
      el.style.setProperty("--tt-arrow-top", "50%");
      el.style.setProperty("--tt-arrow-mt", shift);
    } else {
      el.style.setProperty("--tt-bottom", gap);
      el.style.setProperty("--tt-left", "50%");
      el.style.setProperty("--tt-tx", "-50%");
      el.style.setProperty("--tt-arrow-bottom", arrow);
      el.style.setProperty("--tt-arrow-left", "50%");
      el.style.setProperty("--tt-arrow-ml", shift);
    }
  };
  var tipDescribe = (node) => {
    const el = node;
    const n = ++_seq;
    const bubbleId = `sail-tooltip-${n}`;
    const bubble = el.querySelector('[role="tooltip"]');
    const sync = () => {
      applyPlacement(el, el.getAttribute("placement"));
      if (bubble) {
        bubble.id = bubbleId;
        bubble.textContent = el.getAttribute("content") ?? "";
      }
      const trigger = Array.from(el.children).find((c) => c !== bubble);
      if (trigger) {
        const existing = trigger.getAttribute("aria-describedby") ?? "";
        if (!existing.split(/\s+/).filter(Boolean).includes(bubbleId)) {
          trigger.setAttribute("aria-describedby", existing ? `${existing} ${bubbleId}` : bubbleId);
        }
      }
    };
    sync();
    const obs = new MutationObserver(sync);
    obs.observe(el, { attributes: true, attributeFilter: ["content", "placement"] });
    const onKeydown = (e) => {
      if (e.key === "Escape")
        el.blur();
    };
    el.addEventListener("keydown", onKeydown);
    return {
      destroy: () => {
        obs.disconnect();
        el.removeEventListener("keydown", onKeydown);
        for (const prop of [
          "--tt-top",
          "--tt-bottom",
          "--tt-left",
          "--tt-right",
          "--tt-tx",
          "--sx-translate-y",
          "--tt-arrow-top",
          "--tt-arrow-bottom",
          "--tt-arrow-left",
          "--tt-arrow-right",
          "--tt-arrow-ml",
          "--tt-arrow-mt"
        ]) {
          el.style.removeProperty(prop);
        }
      }
    };
  };
  // ../runtime/src/actions/variant-live.ts
  var variantLive = (node) => {
    const el = node;
    const sync = () => {
      const connecting = el.getAttribute("variant") === "connecting";
      el.setAttribute("role", connecting ? "status" : "note");
      el.setAttribute("aria-live", connecting ? "polite" : "off");
    };
    sync();
    const obs = new MutationObserver(sync);
    obs.observe(el, { attributes: true, attributeFilter: ["variant"] });
    return { destroy: () => obs.disconnect() };
  };
  // ../runtime/src/transition-runner.ts
  var _runEnter = () => {};
  var _runLeave = () => null;
  var _cancelTransitions = () => {};
  var runLeave = (nodes) => _runLeave(nodes);
  function setTransitionRunner(impl) {
    _runEnter = impl.runEnter;
    _runLeave = impl.runLeave;
    _cancelTransitions = impl.cancelTransitions;
  }

  // ../runtime/src/error-boundary.ts
  var exports_error_boundary = {};
  __export(exports_error_boundary, {
    runProtected: () => runProtected,
    findNearestBoundary: () => findNearestBoundary,
    detachBoundary: () => detachBoundary,
    attachBoundary: () => attachBoundary
  });
  var BOUNDARY_KEY = Symbol.for("sail:error-boundary");
  function attachBoundary(host, handle) {
    host[BOUNDARY_KEY] = handle;
  }
  function detachBoundary(host) {
    delete host[BOUNDARY_KEY];
  }
  function findNearestBoundary(node) {
    let cur = node;
    while (cur) {
      if (cur instanceof Element) {
        const handle = cur[BOUNDARY_KEY];
        if (handle)
          return handle;
      }
      const parent = cur.parentNode;
      if (parent) {
        cur = parent;
        continue;
      }
      const root = cur;
      if (root.host) {
        cur = root.host;
        continue;
      }
      cur = null;
    }
    return null;
  }
  function runProtected(node, source, fn) {
    try {
      return fn();
    } catch (err) {
      const handle = findNearestBoundary(node);
      if (handle) {
        handle.catch(err, source);
        return;
      }
      console.error("[sail] uncaught throw (no error boundary):", err);
      return;
    }
  }

  // ../runtime/src/error-overlay.ts
  function dispatchSailError(host, error, phase) {
    const detail = { error, phase };
    let target = typeof document !== "undefined" ? document : null;
    if (host != null && typeof host.tagName === "string") {
      detail.host = host;
      target = host;
      const tag = host.dataset?.component;
      if (tag)
        detail.tag = tag;
    }
    if (typeof target?.dispatchEvent !== "function")
      return;
    if (typeof CustomEvent !== "function")
      return;
    try {
      target.dispatchEvent(new CustomEvent("sail:error", { detail, bubbles: true, composed: true }));
    } catch {}
  }

  // ../runtime/src/effect-scope.ts
  var CURRENT_SCOPE = null;
  function withScope(scope, body) {
    const prev = CURRENT_SCOPE;
    CURRENT_SCOPE = scope;
    try {
      return body();
    } finally {
      CURRENT_SCOPE = prev;
    }
  }
  function onCleanup(fn) {
    const scope = CURRENT_SCOPE;
    if (!scope)
      return;
    scope.cleanup(fn);
  }
  function currentScope() {
    return CURRENT_SCOPE;
  }

  // ../runtime/src/signal.ts
  var CLEAN = 0;
  var DIRTY = 2;
  function makeOwnerScope(node) {
    const ctx = new Map;
    return {
      cleanup(fn) {
        node.cleanups.push(fn);
      },
      registerMount(fn) {
        node.owner?.registerMount(fn);
      },
      runMounts() {},
      dispose() {},
      disposed: false,
      parent: node.owner ?? null,
      contexts: ctx
    };
  }
  var CURRENT_OBSERVER = null;
  var HOST_STACK = [];
  function currentHost() {
    return HOST_STACK.length > 0 ? HOST_STACK[HOST_STACK.length - 1] : null;
  }
  function runCleanups(node) {
    const cs = node.cleanups;
    if (cs.length === 0)
      return;
    node.cleanups = [];
    for (let i = 0;i < cs.length; i++) {
      try {
        cs[i]();
      } catch (e) {
        console.error("[sail] cleanup threw:", e);
      }
    }
  }
  function unlinkSources(node) {
    const srcs = node.sources;
    for (let i = 0;i < srcs.length; i++)
      srcs[i].observers.delete(node);
    node.sources = [];
  }
  function runEffect(node) {
    runCleanups(node);
    unlinkSources(node);
    node.state = CLEAN;
    const scope = node.scope ??= makeOwnerScope(node);
    const prev = CURRENT_OBSERVER;
    CURRENT_OBSERVER = node;
    try {
      const c = withScope(scope, () => node.fn());
      if (typeof c === "function")
        node.cleanups.push(c);
    } catch (e) {
      const boundary = node.host ? findNearestBoundary(node.host) : null;
      if (boundary) {
        boundary.catch(e, "effect");
      } else {
        console.error("[sail] effect threw:", e);
        dispatchSailError(node.host ?? null, e, "effect");
      }
    } finally {
      CURRENT_OBSERVER = prev;
    }
  }
  function effect(fn) {
    const node = {
      fn,
      state: DIRTY,
      sources: [],
      observers: new Set,
      cleanups: [],
      isEffect: true,
      active: true,
      computation: true,
      host: currentHost(),
      owner: currentScope()
    };
    runEffect(node);
    const dispose = () => {
      if (!node.active)
        return;
      node.active = false;
      runCleanups(node);
      unlinkSources(node);
      node.state = CLEAN;
    };
    if (CURRENT_OBSERVER !== null)
      onCleanup(dispose);
    return dispose;
  }
  function untrack(fn) {
    const prev = CURRENT_OBSERVER;
    CURRENT_OBSERVER = null;
    try {
      return fn();
    } finally {
      CURRENT_OBSERVER = prev;
    }
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/GroupAnimation.mjs
  class GroupAnimation {
    constructor(animations) {
      this.stop = () => this.runAll("stop");
      this.animations = animations.filter(Boolean);
    }
    get finished() {
      return Promise.all(this.animations.map((animation) => animation.finished));
    }
    getAll(propName) {
      return this.animations[0][propName];
    }
    setAll(propName, newValue) {
      for (let i = 0;i < this.animations.length; i++) {
        this.animations[i][propName] = newValue;
      }
    }
    attachTimeline(timeline) {
      const subscriptions = this.animations.map((animation) => animation.attachTimeline(timeline));
      return () => {
        subscriptions.forEach((cancel, i) => {
          cancel && cancel();
          this.animations[i].stop();
        });
      };
    }
    get time() {
      return this.getAll("time");
    }
    set time(time) {
      this.setAll("time", time);
    }
    get speed() {
      return this.getAll("speed");
    }
    set speed(speed) {
      this.setAll("speed", speed);
    }
    get state() {
      return this.getAll("state");
    }
    get startTime() {
      return this.getAll("startTime");
    }
    get duration() {
      return getMax(this.animations, "duration");
    }
    get iterationDuration() {
      return getMax(this.animations, "iterationDuration");
    }
    runAll(methodName) {
      this.animations.forEach((controls) => controls[methodName]());
    }
    play() {
      this.runAll("play");
    }
    pause() {
      this.runAll("pause");
    }
    cancel() {
      this.runAll("cancel");
    }
    complete() {
      this.runAll("complete");
    }
  }
  function getMax(animations, propName) {
    let max = 0;
    for (let i = 0;i < animations.length; i++) {
      const value = animations[i][propName];
      if (value !== null && value > max) {
        max = value;
      }
    }
    return max;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/GroupAnimationWithThen.mjs
  class GroupAnimationWithThen extends GroupAnimation {
    then(onResolve, _onReject) {
      return this.finished.finally(onResolve).then(() => {});
    }
  }
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/clamp.mjs
  var clamp2 = (min, max, v) => {
    if (v > max)
      return max;
    if (v < min)
      return min;
    return v;
  };
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/format-error-message.mjs
  function formatErrorMessage(message, errorCode) {
    return errorCode ? `${message}. For more information and steps for solving, visit https://motion.dev/troubleshooting/${errorCode}` : message;
  }

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/errors.mjs
  var warning = () => {};
  var invariant = () => {};
  if (typeof process !== "undefined" && process.env?.NODE_ENV !== "production") {
    warning = (check, message, errorCode) => {
      if (!check && typeof console !== "undefined") {
        console.warn(formatErrorMessage(message, errorCode));
      }
    };
    invariant = (check, message, errorCode) => {
      if (!check) {
        throw new Error(formatErrorMessage(message, errorCode));
      }
    };
  }
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/memo.mjs
  function memo(callback) {
    let result;
    return () => {
      if (result === undefined)
        result = callback();
      return result;
    };
  }
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/noop.mjs
  var noop = (any) => any;
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/time-conversion.mjs
  var secondsToMilliseconds = (seconds) => seconds * 1000;
  var millisecondsToSeconds = (milliseconds) => milliseconds / 1000;
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/modifiers/reverse.mjs
  var reverseEasing = (easing) => (p) => 1 - easing(1 - p);
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/utils/is-bezier-definition.mjs
  var isBezierDefinition = (easing) => Array.isArray(easing) && typeof easing[0] === "number";
  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/cubic-bezier.mjs
  var calcBezier = (t, a1, a2) => (((1 - 3 * a2 + 3 * a1) * t + (3 * a2 - 6 * a1)) * t + 3 * a1) * t;
  var subdivisionPrecision = 0.0000001;
  var subdivisionMaxIterations = 12;
  function binarySubdivide(x, lowerBound, upperBound, mX1, mX2) {
    let currentX;
    let currentT;
    let i = 0;
    do {
      currentT = lowerBound + (upperBound - lowerBound) / 2;
      currentX = calcBezier(currentT, mX1, mX2) - x;
      if (currentX > 0) {
        upperBound = currentT;
      } else {
        lowerBound = currentT;
      }
    } while (Math.abs(currentX) > subdivisionPrecision && ++i < subdivisionMaxIterations);
    return currentT;
  }
  function cubicBezier(mX1, mY1, mX2, mY2) {
    if (mX1 === mY1 && mX2 === mY2)
      return noop;
    const getTForX = (aX) => binarySubdivide(aX, 0, 1, mX1, mX2);
    return (t) => t === 0 || t === 1 ? t : calcBezier(getTForX(t), mY1, mY2);
  }

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/modifiers/mirror.mjs
  var mirrorEasing = (easing) => (p) => p <= 0.5 ? easing(2 * p) / 2 : (2 - easing(2 * (1 - p))) / 2;

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/back.mjs
  var backOut = /* @__PURE__ */ cubicBezier(0.33, 1.53, 0.69, 0.99);
  var backIn = /* @__PURE__ */ reverseEasing(backOut);
  var backInOut = /* @__PURE__ */ mirrorEasing(backIn);

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/anticipate.mjs
  var anticipate = (p) => p >= 1 ? 1 : (p *= 2) < 1 ? 0.5 * backIn(p) : 0.5 * (2 - Math.pow(2, -10 * (p - 1)));

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/circ.mjs
  var circIn = (p) => 1 - Math.sin(Math.acos(p));
  var circOut = reverseEasing(circIn);
  var circInOut = mirrorEasing(circIn);

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/ease.mjs
  var easeIn = /* @__PURE__ */ cubicBezier(0.42, 0, 1, 1);
  var easeOut = /* @__PURE__ */ cubicBezier(0, 0, 0.58, 1);
  var easeInOut = /* @__PURE__ */ cubicBezier(0.42, 0, 0.58, 1);

  // ../../../node_modules/.bun/motion-utils@12.39.0/node_modules/motion-utils/dist/es/easing/utils/map.mjs
  var easingLookup = {
    linear: noop,
    easeIn,
    easeInOut,
    easeOut,
    circIn,
    circInOut,
    circOut,
    backIn,
    backInOut,
    backOut,
    anticipate
  };
  var isValidEasing = (easing) => {
    return typeof easing === "string";
  };
  var easingDefinitionToFunction = (definition) => {
    if (isBezierDefinition(definition)) {
      invariant(definition.length === 4, `Cubic bezier arrays must contain four numerical values.`, "cubic-bezier-length");
      const [x1, y1, x2, y2] = definition;
      return cubicBezier(x1, y1, x2, y2);
    } else if (isValidEasing(definition)) {
      invariant(easingLookup[definition] !== undefined, `Invalid easing type '${definition}'`, "invalid-easing-type");
      return easingLookup[definition];
    }
    return definition;
  };
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/render/dom/is-css-var.mjs
  var isCSSVar = (name) => name.startsWith("--");

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/render/dom/style-set.mjs
  function setStyle(element, name, value) {
    isCSSVar(name) ? element.style.setProperty(name, value) : element.style[name] = value;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/supports/flags.mjs
  var supportsFlags = {};

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/supports/memo.mjs
  function memoSupports(callback, supportsFlag) {
    const memoized = memo(callback);
    return () => supportsFlags[supportsFlag] ?? memoized();
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/supports/scroll-timeline.mjs
  var supportsScrollTimeline = /* @__PURE__ */ memoSupports(() => window.ScrollTimeline !== undefined, "scrollTimeline");

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/keyframes/get-final.mjs
  var isNotNull = (value) => value !== null;
  function getFinalKeyframe(keyframes, { repeat, repeatType = "loop" }, finalKeyframe, speed = 1) {
    const resolvedKeyframes = keyframes.filter(isNotNull);
    const useFirstKeyframe = speed < 0 || repeat && repeatType !== "loop" && repeat % 2 === 1;
    const index = useFirstKeyframe ? 0 : resolvedKeyframes.length - 1;
    return !index || finalKeyframe === undefined ? resolvedKeyframes[index] : finalKeyframe;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/utils/WithPromise.mjs
  class WithPromise {
    constructor() {
      this.updateFinished();
    }
    get finished() {
      return this._finished;
    }
    updateFinished() {
      this._finished = new Promise((resolve) => {
        this.resolve = resolve;
      });
    }
    notifyFinished() {
      this.resolve();
    }
    then(onResolve, onReject) {
      return this.finished.then(onResolve, onReject);
    }
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/supports/linear-easing.mjs
  var supportsLinearEasing = /* @__PURE__ */ memoSupports(() => {
    try {
      document.createElement("div").animate({ opacity: 0 }, { easing: "linear(0, 1)" });
    } catch (e) {
      return false;
    }
    return true;
  }, "linearEasing");

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/utils/linear.mjs
  var generateLinearEasing = (easing, duration, resolution = 10) => {
    let points = "";
    const numPoints = Math.max(Math.round(duration / resolution), 2);
    for (let i = 0;i < numPoints; i++) {
      points += Math.round(easing(i / (numPoints - 1)) * 1e4) / 1e4 + ", ";
    }
    return `linear(${points.substring(0, points.length - 2)})`;
  };

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/easing/cubic-bezier.mjs
  var cubicBezierAsString = ([a, b, c, d]) => `cubic-bezier(${a}, ${b}, ${c}, ${d})`;

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/easing/supported.mjs
  var supportedWaapiEasing = {
    linear: "linear",
    ease: "ease",
    easeIn: "ease-in",
    easeOut: "ease-out",
    easeInOut: "ease-in-out",
    circIn: /* @__PURE__ */ cubicBezierAsString([0, 0.65, 0.55, 1]),
    circOut: /* @__PURE__ */ cubicBezierAsString([0.55, 0, 1, 0.45]),
    backIn: /* @__PURE__ */ cubicBezierAsString([0.31, 0.01, 0.66, -0.59]),
    backOut: /* @__PURE__ */ cubicBezierAsString([0.33, 1.53, 0.69, 0.99])
  };

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/easing/map-easing.mjs
  function mapEasingToNativeEasing(easing, duration) {
    if (!easing) {
      return;
    } else if (typeof easing === "function") {
      return supportsLinearEasing() ? generateLinearEasing(easing, duration) : "ease-out";
    } else if (isBezierDefinition(easing)) {
      return cubicBezierAsString(easing);
    } else if (Array.isArray(easing)) {
      return easing.map((segmentEasing) => mapEasingToNativeEasing(segmentEasing, duration) || supportedWaapiEasing.easeOut);
    } else {
      return supportedWaapiEasing[easing];
    }
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/start-waapi-animation.mjs
  function startWaapiAnimation(element, valueName, keyframes, { delay = 0, duration = 300, repeat = 0, repeatType = "loop", ease = "easeOut", times } = {}, pseudoElement = undefined) {
    const keyframeOptions = {
      [valueName]: keyframes
    };
    if (times)
      keyframeOptions.offset = times;
    const easing = mapEasingToNativeEasing(ease, duration);
    if (Array.isArray(easing))
      keyframeOptions.easing = easing;
    const options = {
      delay,
      duration,
      easing: !Array.isArray(easing) ? easing : "linear",
      fill: "both",
      iterations: repeat + 1,
      direction: repeatType === "reverse" ? "alternate" : "normal"
    };
    if (pseudoElement)
      options.pseudoElement = pseudoElement;
    return element.animate(keyframeOptions, options);
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/generators/utils/is-generator.mjs
  function isGenerator(type) {
    return typeof type === "function" && "applyToOptions" in type;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/utils/apply-generator.mjs
  function applyGeneratorOptions({ type, ...options }) {
    if (isGenerator(type) && supportsLinearEasing()) {
      return type.applyToOptions(options);
    } else {
      options.duration ?? (options.duration = 300);
      options.ease ?? (options.ease = "easeOut");
    }
    return options;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/NativeAnimation.mjs
  class NativeAnimation extends WithPromise {
    constructor(options) {
      super();
      this.finishedTime = null;
      this.isStopped = false;
      this.manualStartTime = null;
      if (!options)
        return;
      const { element, name, keyframes, pseudoElement, allowFlatten = false, finalKeyframe, onComplete } = options;
      this.isPseudoElement = Boolean(pseudoElement);
      this.allowFlatten = allowFlatten;
      this.options = options;
      invariant(typeof options.type !== "string", `Mini animate() doesn't support "type" as a string.`, "mini-spring");
      const transition = applyGeneratorOptions(options);
      this.animation = startWaapiAnimation(element, name, keyframes, transition, pseudoElement);
      if (transition.autoplay === false) {
        this.animation.pause();
      }
      this.animation.onfinish = () => {
        this.finishedTime = this.time;
        if (!pseudoElement) {
          const keyframe = getFinalKeyframe(keyframes, this.options, finalKeyframe, this.speed);
          if (this.updateMotionValue) {
            this.updateMotionValue(keyframe);
          }
          setStyle(element, name, keyframe);
          this.animation.cancel();
        }
        onComplete?.();
        this.notifyFinished();
      };
    }
    play() {
      if (this.isStopped)
        return;
      this.manualStartTime = null;
      this.animation.play();
      if (this.state === "finished") {
        this.updateFinished();
      }
    }
    pause() {
      this.animation.pause();
    }
    complete() {
      this.animation.finish?.();
    }
    cancel() {
      try {
        this.animation.cancel();
      } catch (e) {}
    }
    stop() {
      if (this.isStopped)
        return;
      this.isStopped = true;
      const { state } = this;
      if (state === "idle" || state === "finished") {
        return;
      }
      if (this.updateMotionValue) {
        this.updateMotionValue();
      } else {
        this.commitStyles();
      }
      if (!this.isPseudoElement)
        this.cancel();
    }
    commitStyles() {
      const element = this.options?.element;
      if (!this.isPseudoElement && element?.isConnected) {
        this.animation.commitStyles?.();
      }
    }
    get duration() {
      const duration = this.animation.effect?.getComputedTiming?.().duration || 0;
      return millisecondsToSeconds(Number(duration));
    }
    get iterationDuration() {
      const { delay = 0 } = this.options || {};
      return this.duration + millisecondsToSeconds(delay);
    }
    get time() {
      return millisecondsToSeconds(Number(this.animation.currentTime) || 0);
    }
    set time(newTime) {
      const wasFinished = this.finishedTime !== null;
      this.manualStartTime = null;
      this.finishedTime = null;
      this.animation.currentTime = secondsToMilliseconds(newTime);
      if (wasFinished) {
        this.animation.pause();
      }
    }
    get speed() {
      return this.animation.playbackRate;
    }
    set speed(newSpeed) {
      if (newSpeed < 0)
        this.finishedTime = null;
      this.animation.playbackRate = newSpeed;
    }
    get state() {
      return this.finishedTime !== null ? "finished" : this.animation.playState;
    }
    get startTime() {
      return this.manualStartTime ?? Number(this.animation.startTime);
    }
    set startTime(newStartTime) {
      this.manualStartTime = this.animation.startTime = newStartTime;
    }
    attachTimeline({ timeline, rangeStart, rangeEnd, observe }) {
      if (this.allowFlatten) {
        this.animation.effect?.updateTiming({ easing: "linear" });
      }
      this.animation.onfinish = null;
      if (timeline && supportsScrollTimeline()) {
        this.animation.timeline = timeline;
        if (rangeStart)
          this.animation.rangeStart = rangeStart;
        if (rangeEnd)
          this.animation.rangeEnd = rangeEnd;
        return noop;
      } else {
        return observe(this);
      }
    }
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/utils/active-animations.mjs
  var animationMaps = new WeakMap;
  var animationMapKey = (name, pseudoElement = "") => `${name}:${pseudoElement}`;
  function getAnimationMap(element) {
    let map = animationMaps.get(element);
    if (!map) {
      map = new Map;
      animationMaps.set(element, map);
    }
    return map;
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/utils/resolve-transition.mjs
  function resolveTransition(transition, parentTransition) {
    if (transition?.inherit && parentTransition) {
      const { inherit: _, ...rest } = transition;
      return { ...parentTransition, ...rest };
    }
    return transition;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/utils/get-value-transition.mjs
  function getValueTransition(transition, key) {
    const valueTransition = transition?.[key] ?? transition?.["default"] ?? transition;
    if (valueTransition !== transition) {
      return resolveTransition(valueTransition, transition);
    }
    return valueTransition;
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/generators/utils/calc-duration.mjs
  var maxGeneratorDuration = 20000;
  function calcGeneratorDuration(generator) {
    let duration = 0;
    const timeStep = 50;
    let state = generator.next(duration);
    while (!state.done && duration < maxGeneratorDuration) {
      duration += timeStep;
      state = generator.next(duration);
    }
    return duration >= maxGeneratorDuration ? Infinity : duration;
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/generators/utils/create-generator-easing.mjs
  function createGeneratorEasing(options, scale = 100, createGenerator) {
    const generator = createGenerator({ ...options, keyframes: [0, scale] });
    const duration = Math.min(calcGeneratorDuration(generator), maxGeneratorDuration);
    return {
      type: "keyframes",
      ease: (progress) => {
        return generator.next(duration * progress).value / scale;
      },
      duration: millisecondsToSeconds(duration)
    };
  }

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/generators/spring.mjs
  var springDefaults = {
    stiffness: 100,
    damping: 10,
    mass: 1,
    velocity: 0,
    duration: 800,
    bounce: 0.3,
    visualDuration: 0.3,
    restSpeed: {
      granular: 0.01,
      default: 2
    },
    restDelta: {
      granular: 0.005,
      default: 0.5
    },
    minDuration: 0.01,
    maxDuration: 10,
    minDamping: 0.05,
    maxDamping: 1
  };
  function calcAngularFreq(undampedFreq, dampingRatio) {
    return undampedFreq * Math.sqrt(1 - dampingRatio * dampingRatio);
  }
  var rootIterations = 12;
  function approximateRoot(envelope, derivative, initialGuess) {
    let result = initialGuess;
    for (let i = 1;i < rootIterations; i++) {
      result = result - envelope(result) / derivative(result);
    }
    return result;
  }
  var safeMin = 0.001;
  function findSpring({ duration = springDefaults.duration, bounce = springDefaults.bounce, velocity = springDefaults.velocity, mass = springDefaults.mass }) {
    let envelope;
    let derivative;
    warning(duration <= secondsToMilliseconds(springDefaults.maxDuration), "Spring duration must be 10 seconds or less", "spring-duration-limit");
    let dampingRatio = 1 - bounce;
    dampingRatio = clamp2(springDefaults.minDamping, springDefaults.maxDamping, dampingRatio);
    duration = clamp2(springDefaults.minDuration, springDefaults.maxDuration, millisecondsToSeconds(duration));
    if (dampingRatio < 1) {
      envelope = (undampedFreq2) => {
        const exponentialDecay = undampedFreq2 * dampingRatio;
        const delta = exponentialDecay * duration;
        const a = exponentialDecay - velocity;
        const b = calcAngularFreq(undampedFreq2, dampingRatio);
        const c = Math.exp(-delta);
        return safeMin - a / b * c;
      };
      derivative = (undampedFreq2) => {
        const exponentialDecay = undampedFreq2 * dampingRatio;
        const delta = exponentialDecay * duration;
        const d = delta * velocity + velocity;
        const e = Math.pow(dampingRatio, 2) * Math.pow(undampedFreq2, 2) * duration;
        const f = Math.exp(-delta);
        const g = calcAngularFreq(Math.pow(undampedFreq2, 2), dampingRatio);
        const factor = -envelope(undampedFreq2) + safeMin > 0 ? -1 : 1;
        return factor * ((d - e) * f) / g;
      };
    } else {
      envelope = (undampedFreq2) => {
        const a = Math.exp(-undampedFreq2 * duration);
        const b = (undampedFreq2 - velocity) * duration + 1;
        return -safeMin + a * b;
      };
      derivative = (undampedFreq2) => {
        const a = Math.exp(-undampedFreq2 * duration);
        const b = (velocity - undampedFreq2) * (duration * duration);
        return a * b;
      };
    }
    const initialGuess = 5 / duration;
    const undampedFreq = approximateRoot(envelope, derivative, initialGuess);
    duration = secondsToMilliseconds(duration);
    if (isNaN(undampedFreq)) {
      return {
        stiffness: springDefaults.stiffness,
        damping: springDefaults.damping,
        duration
      };
    } else {
      const stiffness = Math.pow(undampedFreq, 2) * mass;
      return {
        stiffness,
        damping: dampingRatio * 2 * Math.sqrt(mass * stiffness),
        duration
      };
    }
  }
  var durationKeys = ["duration", "bounce"];
  var physicsKeys = ["stiffness", "damping", "mass"];
  function isSpringType(options, keys) {
    return keys.some((key) => options[key] !== undefined);
  }
  function getSpringOptions(options) {
    let springOptions = {
      velocity: springDefaults.velocity,
      stiffness: springDefaults.stiffness,
      damping: springDefaults.damping,
      mass: springDefaults.mass,
      isResolvedFromDuration: false,
      ...options
    };
    if (!isSpringType(options, physicsKeys) && isSpringType(options, durationKeys)) {
      springOptions.velocity = 0;
      if (options.visualDuration) {
        const visualDuration = options.visualDuration;
        const root = 2 * Math.PI / (visualDuration * 1.2);
        const stiffness = root * root;
        const damping = 2 * clamp2(0.05, 1, 1 - (options.bounce || 0)) * Math.sqrt(stiffness);
        springOptions = {
          ...springOptions,
          mass: springDefaults.mass,
          stiffness,
          damping
        };
      } else {
        const derived = findSpring({ ...options, velocity: 0 });
        springOptions = {
          ...springOptions,
          ...derived,
          mass: springDefaults.mass
        };
        springOptions.isResolvedFromDuration = true;
      }
    }
    return springOptions;
  }
  function spring(optionsOrVisualDuration = springDefaults.visualDuration, bounce = springDefaults.bounce) {
    const options = typeof optionsOrVisualDuration !== "object" ? {
      visualDuration: optionsOrVisualDuration,
      keyframes: [0, 1],
      bounce
    } : optionsOrVisualDuration;
    let { restSpeed, restDelta } = options;
    const origin = options.keyframes[0];
    const target = options.keyframes[options.keyframes.length - 1];
    const state = { done: false, value: origin };
    const { stiffness, damping, mass, duration, velocity, isResolvedFromDuration } = getSpringOptions({
      ...options,
      velocity: -millisecondsToSeconds(options.velocity || 0)
    });
    const initialVelocity = velocity || 0;
    const dampingRatio = damping / (2 * Math.sqrt(stiffness * mass));
    const initialDelta = target - origin;
    const undampedAngularFreq = millisecondsToSeconds(Math.sqrt(stiffness / mass));
    const isGranularScale = Math.abs(initialDelta) < 5;
    restSpeed || (restSpeed = isGranularScale ? springDefaults.restSpeed.granular : springDefaults.restSpeed.default);
    restDelta || (restDelta = isGranularScale ? springDefaults.restDelta.granular : springDefaults.restDelta.default);
    let resolveSpring;
    let resolveVelocity;
    let angularFreq;
    let A;
    let sinCoeff;
    let cosCoeff;
    if (dampingRatio < 1) {
      angularFreq = calcAngularFreq(undampedAngularFreq, dampingRatio);
      A = (initialVelocity + dampingRatio * undampedAngularFreq * initialDelta) / angularFreq;
      resolveSpring = (t) => {
        const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
        return target - envelope * (A * Math.sin(angularFreq * t) + initialDelta * Math.cos(angularFreq * t));
      };
      sinCoeff = dampingRatio * undampedAngularFreq * A + initialDelta * angularFreq;
      cosCoeff = dampingRatio * undampedAngularFreq * initialDelta - A * angularFreq;
      resolveVelocity = (t) => {
        const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
        return envelope * (sinCoeff * Math.sin(angularFreq * t) + cosCoeff * Math.cos(angularFreq * t));
      };
    } else if (dampingRatio === 1) {
      resolveSpring = (t) => target - Math.exp(-undampedAngularFreq * t) * (initialDelta + (initialVelocity + undampedAngularFreq * initialDelta) * t);
      const C = initialVelocity + undampedAngularFreq * initialDelta;
      resolveVelocity = (t) => Math.exp(-undampedAngularFreq * t) * (undampedAngularFreq * C * t - initialVelocity);
    } else {
      const dampedAngularFreq = undampedAngularFreq * Math.sqrt(dampingRatio * dampingRatio - 1);
      resolveSpring = (t) => {
        const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
        const freqForT = Math.min(dampedAngularFreq * t, 300);
        return target - envelope * ((initialVelocity + dampingRatio * undampedAngularFreq * initialDelta) * Math.sinh(freqForT) + dampedAngularFreq * initialDelta * Math.cosh(freqForT)) / dampedAngularFreq;
      };
      const P = (initialVelocity + dampingRatio * undampedAngularFreq * initialDelta) / dampedAngularFreq;
      const sinhCoeff = dampingRatio * undampedAngularFreq * P - initialDelta * dampedAngularFreq;
      const coshCoeff = dampingRatio * undampedAngularFreq * initialDelta - P * dampedAngularFreq;
      resolveVelocity = (t) => {
        const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
        const freqForT = Math.min(dampedAngularFreq * t, 300);
        return envelope * (sinhCoeff * Math.sinh(freqForT) + coshCoeff * Math.cosh(freqForT));
      };
    }
    const generator = {
      calculatedDuration: isResolvedFromDuration ? duration || null : null,
      velocity: (t) => secondsToMilliseconds(resolveVelocity(t)),
      next: (t) => {
        if (!isResolvedFromDuration && dampingRatio < 1) {
          const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
          const sin = Math.sin(angularFreq * t);
          const cos = Math.cos(angularFreq * t);
          const current2 = target - envelope * (A * sin + initialDelta * cos);
          const currentVelocity = secondsToMilliseconds(envelope * (sinCoeff * sin + cosCoeff * cos));
          state.done = Math.abs(currentVelocity) <= restSpeed && Math.abs(target - current2) <= restDelta;
          state.value = state.done ? target : current2;
          return state;
        }
        const current = resolveSpring(t);
        if (!isResolvedFromDuration) {
          const currentVelocity = secondsToMilliseconds(resolveVelocity(t));
          state.done = Math.abs(currentVelocity) <= restSpeed && Math.abs(target - current) <= restDelta;
        } else {
          state.done = t >= duration;
        }
        state.value = state.done ? target : current;
        return state;
      },
      toString: () => {
        const calculatedDuration = Math.min(calcGeneratorDuration(generator), maxGeneratorDuration);
        const easing = generateLinearEasing((progress) => generator.next(calculatedDuration * progress).value, calculatedDuration, 30);
        return calculatedDuration + "ms " + easing;
      },
      toTransition: () => {}
    };
    return generator;
  }
  spring.applyToOptions = (options) => {
    const generatorOptions = createGeneratorEasing(options, 100, spring);
    options.ease = generatorOptions.ease;
    options.duration = secondsToMilliseconds(generatorOptions.duration);
    options.type = "keyframes";
    return options;
  };
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/border-radius.mjs
  var cornerRadiusProps = [
    "borderTopLeftRadius",
    "borderTopRightRadius",
    "borderBottomRightRadius",
    "borderBottomLeftRadius"
  ];

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/waapi/utils/px-values.mjs
  var pxValues = new Set([
    "borderWidth",
    "borderTopWidth",
    "borderRightWidth",
    "borderBottomWidth",
    "borderLeftWidth",
    "borderRadius",
    ...cornerRadiusProps,
    "width",
    "maxWidth",
    "height",
    "maxHeight",
    "top",
    "right",
    "bottom",
    "left",
    "inset",
    "insetBlock",
    "insetBlockStart",
    "insetBlockEnd",
    "insetInline",
    "insetInlineStart",
    "insetInlineEnd",
    "padding",
    "paddingTop",
    "paddingRight",
    "paddingBottom",
    "paddingLeft",
    "paddingBlock",
    "paddingBlockStart",
    "paddingBlockEnd",
    "paddingInline",
    "paddingInlineStart",
    "paddingInlineEnd",
    "margin",
    "marginTop",
    "marginRight",
    "marginBottom",
    "marginLeft",
    "marginBlock",
    "marginBlockStart",
    "marginBlockEnd",
    "marginInline",
    "marginInlineStart",
    "marginInlineEnd",
    "fontSize",
    "backgroundPositionX",
    "backgroundPositionY"
  ]);

  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/keyframes/utils/apply-px-defaults.mjs
  function applyPxDefaults(keyframes, name) {
    for (let i = 0;i < keyframes.length; i++) {
      if (typeof keyframes[i] === "number" && pxValues.has(name)) {
        keyframes[i] = keyframes[i] + "px";
      }
    }
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/animation/keyframes/utils/fill-wildcards.mjs
  function fillWildcards(keyframes) {
    for (let i = 1;i < keyframes.length; i++) {
      keyframes[i] ?? (keyframes[i] = keyframes[i - 1]);
    }
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/render/dom/style-computed.mjs
  function getComputedStyle(element, name) {
    const computedStyle = window.getComputedStyle(element);
    return isCSSVar(name) ? computedStyle.getPropertyValue(name) : computedStyle[name];
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/resolve-elements.mjs
  function resolveElements(elementOrSelector, scope, selectorCache) {
    if (elementOrSelector == null) {
      return [];
    }
    if (elementOrSelector instanceof EventTarget) {
      return [elementOrSelector];
    } else if (typeof elementOrSelector === "string") {
      let root = document;
      if (scope) {
        root = scope.current;
      }
      const elements = selectorCache?.[elementOrSelector] ?? root.querySelectorAll(elementOrSelector);
      return elements ? Array.from(elements) : [];
    }
    return Array.from(elementOrSelector).filter((element) => element != null);
  }
  // ../../../node_modules/.bun/motion-dom@12.42.2/node_modules/motion-dom/dist/es/utils/stagger.mjs
  function getOriginIndex(from, total) {
    if (from === "first") {
      return 0;
    } else {
      const lastIndex = total - 1;
      return from === "last" ? lastIndex : lastIndex / 2;
    }
  }
  function stagger(duration = 0.1, { startDelay = 0, from = 0, ease } = {}) {
    return (i, total) => {
      const fromIndex = typeof from === "number" ? from : getOriginIndex(from, total);
      const distance = Math.abs(fromIndex - i);
      let delay = duration * distance;
      if (ease) {
        const maxDelay = total * duration;
        const easingFunction = easingDefinitionToFunction(ease);
        delay = easingFunction(delay / maxDelay) * maxDelay;
      }
      return startDelay + delay;
    };
  }
  // ../../../node_modules/.bun/framer-motion@12.42.2/node_modules/framer-motion/dist/es/animation/animators/waapi/animate-elements.mjs
  function animateElements(elementOrSelector, keyframes2, options, scope) {
    if (elementOrSelector == null) {
      return [];
    }
    const elements = resolveElements(elementOrSelector, scope);
    const numElements = elements.length;
    invariant(Boolean(numElements), "No valid elements provided.", "no-valid-elements");
    const animationDefinitions = [];
    for (let i = 0;i < numElements; i++) {
      const element = elements[i];
      const elementTransition = { ...options };
      if (typeof elementTransition.delay === "function") {
        elementTransition.delay = elementTransition.delay(i, numElements);
      }
      for (const valueName in keyframes2) {
        let valueKeyframes = keyframes2[valueName];
        if (!Array.isArray(valueKeyframes)) {
          valueKeyframes = [valueKeyframes];
        }
        const valueOptions = {
          ...getValueTransition(elementTransition, valueName)
        };
        valueOptions.duration && (valueOptions.duration = secondsToMilliseconds(valueOptions.duration));
        valueOptions.delay && (valueOptions.delay = secondsToMilliseconds(valueOptions.delay));
        const map = getAnimationMap(element);
        const key = animationMapKey(valueName, valueOptions.pseudoElement || "");
        const currentAnimation = map.get(key);
        currentAnimation && currentAnimation.stop();
        animationDefinitions.push({
          map,
          key,
          unresolvedKeyframes: valueKeyframes,
          options: {
            ...valueOptions,
            element,
            name: valueName,
            allowFlatten: !elementTransition.type && !elementTransition.ease
          }
        });
      }
    }
    for (let i = 0;i < animationDefinitions.length; i++) {
      const { unresolvedKeyframes, options: animationOptions } = animationDefinitions[i];
      const { element, name, pseudoElement } = animationOptions;
      if (!pseudoElement && unresolvedKeyframes[0] === null) {
        unresolvedKeyframes[0] = getComputedStyle(element, name);
      }
      fillWildcards(unresolvedKeyframes);
      applyPxDefaults(unresolvedKeyframes, name);
      if (!pseudoElement && unresolvedKeyframes.length < 2) {
        unresolvedKeyframes.unshift(getComputedStyle(element, name));
      }
      animationOptions.keyframes = unresolvedKeyframes;
    }
    const animations = [];
    for (let i = 0;i < animationDefinitions.length; i++) {
      const { map, key, options: animationOptions } = animationDefinitions[i];
      const animation = new NativeAnimation(animationOptions);
      map.set(key, animation);
      animation.finished.finally(() => map.delete(key));
      animations.push(animation);
    }
    return animations;
  }

  // ../../../node_modules/.bun/framer-motion@12.42.2/node_modules/framer-motion/dist/es/animation/animators/waapi/animate-style.mjs
  var createScopedWaapiAnimate = (scope) => {
    function scopedAnimate(elementOrSelector, keyframes2, options) {
      return new GroupAnimationWithThen(animateElements(elementOrSelector, keyframes2, options, scope));
    }
    return scopedAnimate;
  };
  var animateMini = /* @__PURE__ */ createScopedWaapiAnimate();
  // ../runtime/src/motion.ts
  var reducedMedia = null;
  function prefersReducedMotion2() {
    if (typeof matchMedia === "undefined")
      return false;
    if (!reducedMedia) {
      const mql = matchMedia("(prefers-reduced-motion: reduce)");
      reducedMedia = mql;
      if (typeof mql.addEventListener === "function") {
        mql.addEventListener("change", () => {});
      }
    }
    return reducedMedia.matches;
  }
  var REST_VALUE = {
    opacity: 0,
    scale: 0.96,
    "scale-x": 0,
    "scale-y": 0,
    "translate-y": 16,
    "translate-x": 16,
    rotate: -6
  };
  var FINAL_VALUE = {
    opacity: 1,
    scale: 1,
    "scale-x": 1,
    "scale-y": 1,
    "translate-y": 0,
    "translate-x": 0,
    rotate: 0
  };
  function hasUnit(v) {
    return typeof v === "string" && v.trim() !== "" && Number.isNaN(Number(v));
  }
  function withUnit(v, unit, fallback) {
    if (hasUnit(v))
      return v.trim();
    const n = v === undefined || v === "" ? fallback : Number(v);
    return `${Number.isNaN(n) ? fallback : n}${unit}`;
  }
  function withoutUnit(v, fallback) {
    if (v === undefined || v === "")
      return fallback;
    const n = typeof v === "number" ? v : Number.parseFloat(v);
    return Number.isNaN(n) ? fallback : n;
  }
  function isHTMLElement2(el) {
    return "style" in el;
  }
  function applyRestStyle(el, cfg) {
    if (!isHTMLElement2(el))
      return;
    const value = REST_VALUE[cfg.property];
    switch (cfg.property) {
      case "opacity":
        el.style.opacity = String(value);
        return;
      case "scale":
        el.style.scale = String(value);
        return;
      case "scale-x":
        el.style.scale = `${value} 1`;
        return;
      case "scale-y":
        el.style.scale = `1 ${value}`;
        return;
      case "translate-y":
        el.style.translate = `0 ${value}px`;
        return;
      case "translate-x":
        el.style.translate = `${value}px 0`;
        return;
      case "rotate":
        el.style.rotate = `${value}deg`;
        return;
    }
  }
  function applyFinalStyle(el, cfg) {
    if (!isHTMLElement2(el))
      return;
    const value = FINAL_VALUE[cfg.property];
    switch (cfg.property) {
      case "opacity":
        el.style.opacity = String(value);
        return;
      case "scale":
        el.style.scale = String(value);
        return;
      case "scale-x":
        el.style.scale = `${value} 1`;
        return;
      case "scale-y":
        el.style.scale = `1 ${value}`;
        return;
      case "translate-y":
        el.style.translate = "0 0";
        return;
      case "translate-x":
        el.style.translate = "0 0";
        return;
      case "rotate":
        el.style.rotate = "0deg";
        return;
    }
  }
  function applyReducedMotionSettle(el, motions) {
    if (!isHTMLElement2(el))
      return;
    for (const m of motions)
      applyFinalStyle(el, m);
  }
  function axisRange(cfg, el) {
    if (cfg.preset === "bounce") {
      if (cfg.property === "translate-y") {
        const h = el.getBoundingClientRect().height || el.offsetHeight || 8;
        return ["0px", `${-h}px`];
      }
      if (cfg.property === "translate-x") {
        const w = el.getBoundingClientRect().width || el.offsetWidth || 8;
        return ["0px", `${-w}px`];
      }
    }
    return [
      withUnit(cfg.from, "px", REST_VALUE[cfg.property]),
      withUnit(cfg.to, "px", FINAL_VALUE[cfg.property])
    ];
  }
  function buildKeyframes(cfg, el) {
    if (cfg.property === "translate-y" || cfg.property === "translate-x") {
      const [from2, to2] = axisRange(cfg, el);
      return cfg.property === "translate-y" ? { translate: [`0px ${from2}`, `0px ${to2}`] } : { translate: [`${from2} 0px`, `${to2} 0px`] };
    }
    const from = withoutUnit(cfg.from, REST_VALUE[cfg.property]);
    const to = withoutUnit(cfg.to, FINAL_VALUE[cfg.property]);
    switch (cfg.property) {
      case "opacity":
        return { opacity: [from, to] };
      case "scale":
        return { scale: [from, to] };
      case "scale-x":
        return { scale: [`${from} 1`, `${to} 1`] };
      case "scale-y":
        return { scale: [`1 ${from}`, `1 ${to}`] };
      case "rotate":
        return {
          rotate: [
            withUnit(cfg.from, "deg", REST_VALUE.rotate),
            withUnit(cfg.to, "deg", FINAL_VALUE.rotate)
          ]
        };
    }
  }
  function buildMergedTranslateKeyframes(xCfg, yCfg, el) {
    const [xFrom, xTo] = xCfg ? axisRange(xCfg, el) : ["0px", "0px"];
    const [yFrom, yTo] = yCfg ? axisRange(yCfg, el) : ["0px", "0px"];
    return { translate: [`${xFrom} ${yFrom}`, `${xTo} ${yTo}`] };
  }
  var ANIMATABLE_KEYFRAME_PROPERTIES = new Set(["opacity", "scale", "translate", "rotate"]);
  function warnIfNotAnimatable(el, keyframes2) {
    const supports = typeof CSS !== "undefined" && typeof CSS.supports === "function" ? CSS.supports : null;
    for (const prop of Object.keys(keyframes2)) {
      if (!ANIMATABLE_KEYFRAME_PROPERTIES.has(prop)) {
        console.warn(`[sail/motion] keyframe property "${prop}" is not one of this runtime's known-animatable ` + `properties (${[...ANIMATABLE_KEYFRAME_PROPERTIES].join(", ")}) — WAAPI accepts unknown ` + `keyframe properties silently, so this animation will run forever and move nothing.`, el);
        continue;
      }
      if (!supports)
        continue;
      for (const value of keyframes2[prop]) {
        if (!supports(prop, String(value))) {
          console.warn(`[sail/motion] keyframe value ${JSON.stringify(value)} is not valid for CSS property ` + `"${prop}" — the animation will silently no-op.`, el);
        }
      }
    }
  }
  function warnIfOutOfVisibleRange(el, cfg, keyframes2) {
    if (cfg.property !== "opacity")
      return;
    for (const value of keyframes2.opacity ?? []) {
      const n = typeof value === "number" ? value : Number(value);
      if (Number.isFinite(n) && (n < 0 || n > 1)) {
        console.warn(`[sail/motion] opacity keyframe value ${JSON.stringify(value)} (from preset "${cfg.preset}") ` + `is outside the visible 0..1 range — the computed style clamps it, so this animation runs ` + `and moves nothing. "${cfg.preset}" looks like a preset authored for a different property ` + `(e.g. scale) — check the property this preset is attached to.`, el);
        return;
      }
    }
  }
  function buildTransition(cfg, delaySec) {
    const t = {};
    if (cfg.kind === "spring") {
      const springArgs = {
        keyframes: [0, 1]
      };
      if (cfg.stiffness != null)
        springArgs.stiffness = cfg.stiffness;
      if (cfg.damping != null)
        springArgs.damping = cfg.damping;
      if (cfg.mass != null)
        springArgs.mass = cfg.mass;
      if (cfg.restSpeed != null)
        springArgs.restSpeed = cfg.restSpeed;
      if (cfg.restDelta != null)
        springArgs.restDelta = cfg.restDelta;
      const generator = spring(springArgs);
      const durationMs = calcGeneratorDuration(generator);
      t.duration = durationMs / 1000;
      t.ease = generateLinearEasing((s) => generator.next(s * durationMs).value, durationMs);
    } else {
      if (cfg.duration != null)
        t.duration = cfg.duration;
      if (cfg.easing)
        t.ease = cfg.easing;
    }
    if (cfg.iterations != null)
      t.repeat = cfg.iterations;
    if (cfg.direction != null) {
      t.repeatType = cfg.direction === "normal" ? "loop" : cfg.direction === "alternate" ? "reverse" : cfg.direction;
    }
    if (delaySec != null || cfg.delay != null) {
      t.delay = (delaySec ?? 0) + (cfg.delay ?? 0);
    }
    return t;
  }
  var staggerIndices = new WeakMap;
  function nextStaggerIndex(parent, trigger) {
    let byTrigger = staggerIndices.get(parent);
    if (!byTrigger) {
      byTrigger = new Map;
      staggerIndices.set(parent, byTrigger);
    }
    const index = byTrigger.get(trigger) ?? 0;
    byTrigger.set(trigger, index + 1);
    return index;
  }
  function computeStaggerDelay(config, el) {
    if (config.stagger == null)
      return;
    const parent = el.parentElement;
    if (!parent)
      return 0;
    const index = nextStaggerIndex(parent, config.trigger);
    return stagger(config.stagger)(index, index + 1);
  }
  function seekTo(el, timeMs) {
    if (timeMs == null)
      return;
    try {
      for (const a of el.getAnimations?.() ?? []) {
        const total = a.effect?.getTiming?.().duration ?? 0;
        a.currentTime = total > 0 ? timeMs % total : timeMs;
      }
    } catch {}
  }
  function createAnimations(el, config, delaySec) {
    const controls = [];
    const total = config.motions.length;
    const xIndex = config.motions.findIndex((m) => m.property === "translate-x");
    const yIndex = config.motions.findIndex((m) => m.property === "translate-y");
    const merged = new Set;
    if (xIndex !== -1 && yIndex !== -1) {
      const xCfg = config.motions[xIndex];
      const yCfg = config.motions[yIndex];
      const keyframes2 = buildMergedTranslateKeyframes(xCfg, yCfg, el);
      warnIfNotAnimatable(el, keyframes2);
      const leadIndex = Math.min(xIndex, yIndex);
      const transition = buildTransition(config.motions[leadIndex], delaySec);
      try {
        const c = animateMini(el, keyframes2, transition);
        controls.push(c);
      } catch {
        applyFinalStyle(el, xCfg);
        applyFinalStyle(el, yCfg);
      }
      merged.add(xIndex);
      merged.add(yIndex);
    }
    for (let i = 0;i < total; i++) {
      if (merged.has(i))
        continue;
      const cfg = config.motions[i];
      const keyframes2 = buildKeyframes(cfg, el);
      warnIfNotAnimatable(el, keyframes2);
      warnIfOutOfVisibleRange(el, cfg, keyframes2);
      const transition = buildTransition(cfg, delaySec);
      try {
        const c = animateMini(el, keyframes2, transition);
        controls.push(c);
      } catch {
        applyFinalStyle(el, cfg);
      }
    }
    return controls;
  }
  function createAnimationsWhenConnected(el, config, delaySec, carriedTimeMs = null, afterCreate) {
    if (el.isConnected || typeof requestAnimationFrame === "undefined") {
      const controls2 = createAnimations(el, config, delaySec);
      seekTo(el, carriedTimeMs);
      const extra2 = afterCreate?.(controls2);
      return () => {
        extra2?.();
        for (const c of controls2)
          c.cancel();
      };
    }
    for (const m of config.motions)
      applyRestStyle(el, m);
    let controls = [];
    let extra;
    let disposed = false;
    const start = () => {
      if (disposed)
        return;
      if (!el.isConnected) {
        requestAnimationFrame(start);
        return;
      }
      controls = createAnimations(el, config, delaySec);
      seekTo(el, carriedTimeMs);
      extra = afterCreate?.(controls);
    };
    requestAnimationFrame(start);
    return () => {
      disposed = true;
      extra?.();
      for (const c of controls)
        c.cancel();
    };
  }
  function makeProgressTimeline(kind, el) {
    const g = globalThis;
    try {
      if (kind === "reveal") {
        return g.ViewTimeline ? new g.ViewTimeline({ subject: el }) : undefined;
      }
      if (typeof document === "undefined")
        return;
      return g.ScrollTimeline ? new g.ScrollTimeline({ source: document.scrollingElement }) : undefined;
    } catch {
      return;
    }
  }
  function observeProgressFallback(el, kind, c) {
    if (kind === "reveal" && typeof IntersectionObserver === "function") {
      c.pause?.();
      const io = new IntersectionObserver((entries) => {
        if (entries.some((e) => e.isIntersecting)) {
          io.disconnect();
          c.play?.();
        }
      }, { threshold: 0.15 });
      io.observe(el);
      return () => io.disconnect();
    }
    console.warn(`[sail/motion] fx:${kind} needs ${kind === "reveal" ? "ViewTimeline or IntersectionObserver" : "ScrollTimeline"}, ` + `and this environment has neither — settling at the final pose instead of animating. ` + `The trigger is degraded, not silently dropped.`, el);
    c.complete?.();
    return () => {};
  }
  function attachProgressTimeline(el, kind, controls) {
    const timeline = makeProgressTimeline(kind, el);
    const detachers = [];
    for (const c of controls) {
      if (typeof c.attachTimeline === "function") {
        detachers.push(c.attachTimeline({
          timeline,
          ...kind === "reveal" && timeline ? { rangeStart: "entry 0%", rangeEnd: "entry 100%" } : {},
          observe: () => observeProgressFallback(el, kind, c)
        }));
      } else {
        detachers.push(observeProgressFallback(el, kind, c));
      }
    }
    return () => {
      for (const d of detachers)
        d();
    };
  }
  var TRIGGER_EVENTS = {
    hover: ["mouseenter", "mouseleave"],
    press: ["pointerdown", "pointerup", "pointercancel"],
    focus: ["focus", "blur"],
    "focus-visible": ["focusin", "focusout"]
  };
  var focusSource = "unknown";
  var focusTrackerInstalled = false;
  function trackFocusSource() {
    if (focusTrackerInstalled || typeof window === "undefined")
      return;
    focusTrackerInstalled = true;
    window.addEventListener("keydown", () => {
      focusSource = "keyboard";
    }, { capture: true });
    window.addEventListener("mousedown", () => {
      focusSource = "mouse";
    }, { capture: true });
  }
  function isFocusVisible(ev) {
    const target = ev.target;
    if (target && typeof target.matches === "function") {
      try {
        return target.matches(":focus-visible");
      } catch {}
    }
    return focusSource === "keyboard";
  }
  function classifyTrigger(trigger) {
    if (trigger === "loop")
      return "loop";
    if (trigger === "enter" || trigger === "exit")
      return trigger;
    if (trigger === "reveal" || trigger === "scroll")
      return trigger;
    if (TRIGGER_EVENTS[trigger])
      return "event";
    return "noop";
  }
  function __sail_motion_attach(el, config) {
    if (typeof config !== "function")
      return attachOnce(el, config);
    let inner = null;
    let carried = null;
    const stop = effect(() => {
      const next = config();
      untrack(() => {
        if (inner) {
          carried = readPlaybackTime(el, next);
          inner();
          inner = null;
        }
        inner = attachOnce(el, next, carried);
      });
    });
    return () => {
      stop();
      if (inner)
        inner();
      inner = null;
    };
  }
  function readPlaybackTime(el, cfg) {
    const infinite = cfg.motions.some((m) => m.iterations === Infinity) || cfg.trigger === "loop";
    if (!infinite)
      return null;
    try {
      const anims = el.getAnimations?.() ?? [];
      for (const a of anims) {
        const t = a.currentTime;
        if (typeof t === "number" && Number.isFinite(t))
          return t;
      }
    } catch {}
    return null;
  }
  function attachOnce(el, config, carriedTimeMs = null) {
    if (!isHTMLElement2(el))
      return () => {};
    const reduced = prefersReducedMotion2();
    const branch = classifyTrigger(config.trigger);
    if (branch === "noop")
      return () => {};
    const delaySec = computeStaggerDelay(config, el);
    if (branch === "loop") {
      if (reduced) {
        applyReducedMotionSettle(el, config.motions);
        return () => {};
      }
      const looped = {
        ...config,
        motions: config.motions.map((m) => ({
          ...m,
          iterations: m.iterations ?? Infinity,
          direction: m.direction ?? "alternate"
        }))
      };
      return createAnimationsWhenConnected(el, looped, delaySec, carriedTimeMs);
    }
    if (branch === "enter" || branch === "exit") {
      installLifecycleRunner();
      if (branch === "enter") {
        if (reduced) {
          applyReducedMotionSettle(el, config.motions);
          return () => {};
        }
        return createAnimationsWhenConnected(el, config, delaySec);
      }
      if (reduced)
        return () => {};
      EXITS.set(el, { config, controls: [], delaySec });
      return () => {
        const entry = EXITS.get(el);
        EXITS.delete(el);
        if (entry)
          for (const c of entry.controls)
            c.cancel();
      };
    }
    if (branch === "reveal" || branch === "scroll") {
      if (reduced) {
        applyReducedMotionSettle(el, config.motions);
        return () => {};
      }
      return createAnimationsWhenConnected(el, config, delaySec, null, (controls) => attachProgressTimeline(el, branch, controls));
    }
    const events = TRIGGER_EVENTS[config.trigger];
    if (config.trigger === "focus-visible")
      trackFocusSource();
    const [enterEvent, leaveEvent, cancelEvent] = events;
    let live = [];
    const playForward = () => {
      if (prefersReducedMotion2())
        return;
      for (const c of live)
        c.cancel();
      live = createAnimations(el, config, delaySec);
    };
    const onEnter = (ev) => {
      if (config.trigger === "focus-visible" && !isFocusVisible(ev))
        return;
      playForward();
    };
    const onLeave = () => {
      playForward();
    };
    el.addEventListener(enterEvent, onEnter);
    el.addEventListener(leaveEvent, onLeave);
    if (cancelEvent)
      el.addEventListener(cancelEvent, onLeave);
    return () => {
      el.removeEventListener(enterEvent, onEnter);
      el.removeEventListener(leaveEvent, onLeave);
      if (cancelEvent)
        el.removeEventListener(cancelEvent, onLeave);
      for (const c of live) {
        try {
          c.cancel();
        } catch {}
      }
    };
  }
  var EXITS = new WeakMap;
  var runnerInstalled = false;
  var EXIT_TIMEOUT_MS = 4000;
  function installLifecycleRunner() {
    if (runnerInstalled)
      return;
    runnerInstalled = true;
    setTransitionRunner({
      runEnter: () => null,
      runLeave: (nodes) => {
        const matched = [];
        for (const n of nodes) {
          if (!(n instanceof Element))
            continue;
          const targets = EXITS.has(n) ? [n] : [...n.querySelectorAll("*")].filter((e) => EXITS.has(e));
          for (const t of targets) {
            const entry = EXITS.get(t);
            if (entry)
              matched.push({ el: t, entry });
          }
        }
        if (matched.length === 0)
          return null;
        if (prefersReducedMotion2()) {
          for (const { el, entry } of matched) {
            for (const m of entry.config.motions)
              applyFinalStyle(el, m);
          }
          return null;
        }
        const playing = [];
        for (const { el, entry } of matched) {
          const controls = createAnimations(el, entry.config, entry.delaySec);
          entry.controls = controls;
          for (const c of controls)
            playing.push(c.finished.catch(() => {}));
        }
        let timeoutId;
        const settled = Promise.race([
          Promise.all(playing).then(() => {}),
          new Promise((resolve) => {
            timeoutId = setTimeout(resolve, EXIT_TIMEOUT_MS);
          })
        ]);
        return settled.then(() => {
          clearTimeout(timeoutId);
          for (const { el, entry } of matched) {
            for (const c of entry.controls)
              c.cancel();
            for (const m of entry.config.motions)
              applyFinalStyle(el, m);
            entry.controls = [];
          }
        });
      },
      cancelTransitions: (node) => {
        if (!(node instanceof Element))
          return;
        const entry = EXITS.get(node);
        if (entry) {
          for (const c of entry.controls)
            c.cancel();
          entry.controls = [];
        }
      }
    });
  }

  // ../runtime/src/actions/icon.ts
  var ENDPOINT = "/_deck/icons/";
  var cache = new Map;
  var inflight = new Map;
  function primeIconCache(glyphs) {
    for (const [name, svg] of Object.entries(glyphs))
      cache.set(name, svg);
  }
  function fetchKey(url, key) {
    const pending = inflight.get(key);
    if (pending)
      return pending;
    const p = fetch(url).then((r) => r.ok ? r.text() : null).catch(() => null).then((svg) => {
      cache.set(key, svg);
      inflight.delete(key);
      return svg;
    });
    inflight.set(key, p);
    return p;
  }
  function parseSvg(doc, markup) {
    const parsed = new DOMParser().parseFromString(markup, "image/svg+xml");
    const root = parsed.documentElement;
    if (root.nodeName.toLowerCase() !== "svg")
      return null;
    return doc.adoptNode(root);
  }
  var icon = (node) => {
    let svgEl = null;
    let lastKey = null;
    let seq = 0;
    let spinDispose = null;
    const syncAria = () => {
      const label = node.getAttribute("label") ?? "";
      if (label) {
        node.setAttribute("role", "img");
        node.setAttribute("aria-label", label);
        node.removeAttribute("aria-hidden");
      } else {
        node.setAttribute("aria-hidden", "true");
        node.removeAttribute("role");
        node.removeAttribute("aria-label");
      }
      if (svgEl) {
        if (label)
          svgEl.removeAttribute("aria-hidden");
        else
          svgEl.setAttribute("aria-hidden", "true");
      }
    };
    const syncSpin = () => {
      spinDispose?.();
      spinDispose = null;
      if (!node.hasAttribute("spin"))
        return;
      spinDispose = __sail_motion_attach(node, {
        trigger: "loop",
        motions: [
          {
            property: "rotate",
            preset: "spin",
            kind: "tween",
            from: 0,
            to: 360,
            duration: 0.7,
            easing: "linear",
            iterations: Infinity,
            direction: "normal"
          }
        ]
      });
    };
    const drop = () => {
      if (svgEl) {
        svgEl.remove();
        svgEl = null;
      }
      lastKey = null;
    };
    const inject = (markup, key) => {
      if (key === lastKey)
        return;
      const parsed = parseSvg(node.ownerDocument, markup);
      if (!parsed)
        return;
      if (svgEl)
        svgEl.replaceWith(parsed);
      else
        node.insertBefore(parsed, node.firstChild);
      svgEl = parsed;
      lastKey = key;
      syncAria();
    };
    const render = () => {
      syncAria();
      syncSpin();
      const src = node.getAttribute("src") ?? "";
      const name = node.getAttribute("name") ?? "";
      const key = src ? `src:${src}` : name ? `name:${name}` : "";
      if (!key) {
        drop();
        return;
      }
      let cached = cache.get(key);
      if (cached === undefined && !src)
        cached = cache.get(name);
      if (cached !== undefined) {
        if (cached)
          inject(cached, key);
        else
          drop();
        return;
      }
      const ticket = ++seq;
      const url = src || `${ENDPOINT}${encodeURIComponent(name)}.svg`;
      fetchKey(url, key).then((svg) => {
        if (ticket !== seq)
          return;
        if (svg)
          inject(svg, key);
        else
          drop();
      });
    };
    const mo = new MutationObserver(render);
    mo.observe(node, { attributes: true, attributeFilter: ["name", "src", "label", "spin"] });
    render();
    return {
      destroy: () => {
        mo.disconnect();
        spinDispose?.();
        spinDispose = null;
      }
    };
  };
  // ../runtime/src/actions/marquee.ts
  var DURATION_S = {
    fast: 16,
    normal: 24,
    slow: 32
  };
  function hostOf(node) {
    return node.closest('[data-sail-host="ticker"]') ?? node.parentElement;
  }
  function configFor(host) {
    const key = host?.getAttribute("duration") || "normal";
    const duration = DURATION_S[key] ?? 24;
    const reverse = host?.hasAttribute("reverse") ?? false;
    return {
      trigger: "loop",
      motions: [
        {
          property: "translate-x",
          preset: "inline",
          kind: "tween",
          from: 0,
          to: reverse ? "50%" : "-50%",
          duration,
          easing: "linear",
          iterations: Infinity,
          direction: "normal"
        }
      ]
    };
  }
  var marquee = (node) => {
    const host = hostOf(node);
    let dispose = null;
    const attach = () => {
      dispose?.();
      dispose = __sail_motion_attach(node, configFor(host));
    };
    const mo = host && new MutationObserver(attach);
    mo && host && mo.observe(host, { attributes: true, attributeFilter: ["duration", "reverse"] });
    attach();
    return {
      destroy: () => {
        mo && mo.disconnect();
        dispose?.();
        dispose = null;
      }
    };
  };
  // ../runtime/src/actions/recovery-surface.ts
  var recoverySurface = (node) => {
    const el = node;
    const copy = el.children[1];
    const code = copy?.children[2] ?? null;
    const toggle = copy?.querySelector('button[type="button"]') ?? null;
    const details = toggle?.nextElementSibling ?? null;
    const actions = el.children[2];
    const syncCode = () => {
      if (code)
        code.textContent = el.getAttribute("code") ?? "";
    };
    const hasDetails = () => !!details && (details.textContent ?? "").trim().length > 0;
    const isOpen = () => {
      const s = el.getAttribute("data-state") ?? "";
      return s.split(/\s+/).includes("open");
    };
    const setState = (has, open) => {
      if (!has) {
        el.removeAttribute("data-state");
        return;
      }
      el.setAttribute("data-state", open ? "has-details open" : "has-details");
    };
    const syncDetails = () => {
      if (hasDetails()) {
        setState(true, isOpen());
      } else {
        setState(false, false);
        if (toggle) {
          toggle.textContent = "Show details";
          toggle.setAttribute("aria-expanded", "false");
        }
      }
    };
    const onToggle = () => {
      const open = !isOpen();
      setState(hasDetails(), open);
      if (toggle) {
        toggle.textContent = open ? "Hide details" : "Show details";
        toggle.setAttribute("aria-expanded", open ? "true" : "false");
      }
    };
    toggle?.addEventListener("click", onToggle);
    syncCode();
    syncDetails();
    const obs = new MutationObserver(() => {
      syncCode();
      syncDetails();
    });
    obs.observe(el, { attributes: true, attributeFilter: ["code"], childList: true, subtree: true, characterData: true });
    const firstAction = actions?.querySelector("button, [tabindex], a");
    if (firstAction)
      requestAnimationFrame(() => firstAction.focus?.());
    return {
      destroy: () => {
        obs.disconnect();
        toggle?.removeEventListener("click", onToggle);
      }
    };
  };
  // ../runtime/src/actions/catch-render.ts
  var catchRender = (node) => {
    const el = node;
    const alert = el.querySelector('[role="alert"]');
    const retry = alert?.querySelector('button[type="button"]');
    const message = retry?.previousElementSibling ?? null;
    const reset = () => {
      if (!el.hasAttribute("data-caught"))
        return;
      el.removeAttribute("data-caught");
      el.removeAttribute("data-error-message");
      el.dispatchEvent(new CustomEvent("sail-error-reset", { bubbles: true, composed: true }));
    };
    attachBoundary(el, {
      catch(err, source) {
        if (el.hasAttribute("data-caught"))
          return;
        const error = err instanceof Error ? err : new Error(String(err));
        el.setAttribute("data-caught", "");
        el.setAttribute("data-error-message", error.message);
        if (message)
          message.textContent = el.getAttribute("message") || "Something went wrong";
        el.dispatchEvent(new CustomEvent("sail-error-caught", {
          bubbles: true,
          composed: true,
          detail: { error, source }
        }));
      },
      reset
    });
    retry?.addEventListener("click", reset);
    el.reset = reset;
    return {
      destroy: () => {
        detachBoundary(el);
        retry?.removeEventListener("click", reset);
        delete el.reset;
      }
    };
  };
  // ../runtime/src/actions/highlight.ts
  var KEYWORDS = new Set([
    "import",
    "export",
    "from",
    "const",
    "let",
    "var",
    "return",
    "function",
    "default",
    "if",
    "else",
    "for",
    "of",
    "in",
    "new",
    "await",
    "async",
    "type",
    "interface",
    "class",
    "extends",
    "as"
  ]);
  var VERBS = new Set([
    "signal",
    "derived",
    "effect",
    "controller",
    "scope",
    "mount",
    "untrack",
    "prop",
    "data",
    "client",
    "npm",
    "install"
  ]);
  var CODE_TAGS = new Set(["controller", "scope", "script"]);
  function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }
  function span(cls, text) {
    return `<span class="tok-${cls}">${esc(text)}</span>`;
  }
  function findMatchingBrace(source, openIdx) {
    let depth = 1;
    let i = openIdx + 1;
    let quote = null;
    while (i < source.length) {
      const ch = source[i];
      if (quote) {
        if (ch === "\\") {
          i += 2;
          continue;
        }
        if (ch === quote)
          quote = null;
        i++;
        continue;
      }
      if (ch === '"' || ch === "'" || ch === "`") {
        quote = ch;
        i++;
        continue;
      }
      if (ch === "{") {
        depth++;
        i++;
        continue;
      }
      if (ch === "}") {
        depth--;
        if (depth === 0)
          return i;
        i++;
        continue;
      }
      i++;
    }
    return -1;
  }
  var IDENT_START = /[A-Za-z_$]/;
  var IDENT = /[\w$]/;
  function atLineStart(src, idx) {
    for (let k = idx - 1;k >= 0; k--) {
      const c = src[k];
      if (c === `
`)
        return true;
      if (!/\s/.test(c))
        return false;
    }
    return true;
  }
  function highlightCode(src, allowMarkupHandoff = false) {
    let out = "";
    let plain = "";
    const flush = () => {
      out += esc(plain);
      plain = "";
    };
    const len = src.length;
    let i = 0;
    while (i < len) {
      const ch = src[i];
      if (allowMarkupHandoff && ch === "<" && /[A-Za-z/]/.test(src[i + 1] ?? "") && atLineStart(src, i)) {
        flush();
        out += highlightMarkup(src.slice(i));
        return out;
      }
      if (ch === "/" && src[i + 1] === "/") {
        const end = src.indexOf(`
`, i);
        const stop = end < 0 ? len : end;
        flush();
        out += span("comment", src.slice(i, stop));
        i = stop;
        continue;
      }
      if (ch === "/" && src[i + 1] === "*") {
        const end = src.indexOf("*/", i + 2);
        const stop = end < 0 ? len : end + 2;
        flush();
        out += span("comment", src.slice(i, stop));
        i = stop;
        continue;
      }
      if (ch === "#" && (i === 0 || /\s/.test(src[i - 1]))) {
        const end = src.indexOf(`
`, i);
        const stop = end < 0 ? len : end;
        flush();
        out += span("comment", src.slice(i, stop));
        i = stop;
        continue;
      }
      if (ch === '"' || ch === "'" || ch === "`") {
        let j = i + 1;
        while (j < len) {
          if (src[j] === "\\") {
            j += 2;
            continue;
          }
          if (src[j] === ch) {
            j++;
            break;
          }
          j++;
        }
        flush();
        out += span("string", src.slice(i, j));
        i = j;
        continue;
      }
      if (/\d/.test(ch)) {
        let j = i + 1;
        while (j < len && /\d/.test(src[j]))
          j++;
        if (src[j] === "." && /\d/.test(src[j + 1] ?? "")) {
          j += 2;
          while (j < len && /\d/.test(src[j]))
            j++;
        }
        flush();
        out += span("number", src.slice(i, j));
        i = j;
        continue;
      }
      if (IDENT_START.test(ch)) {
        let j = i + 1;
        while (j < len && IDENT.test(src[j]))
          j++;
        const word = src.slice(i, j);
        flush();
        if (KEYWORDS.has(word))
          out += span("keyword", word);
        else if (VERBS.has(word))
          out += span("fn", word);
        else
          out += esc(word);
        i = j;
        continue;
      }
      plain += ch;
      i++;
    }
    flush();
    return out;
  }
  function highlightMarkup(src) {
    let out = "";
    const len = src.length;
    let i = 0;
    while (i < len) {
      const ch = src[i];
      if (src.startsWith("<!--", i)) {
        const end = src.indexOf("-->", i + 4);
        const stop = end < 0 ? len : end + 3;
        out += span("comment", src.slice(i, stop));
        i = stop;
        continue;
      }
      if (ch === "<" && /[A-Za-z/]/.test(src[i + 1] ?? "")) {
        const closing = src[i + 1] === "/";
        const nameStart = closing ? i + 2 : i + 1;
        let j2 = nameStart;
        while (j2 < len && /[\w-]/.test(src[j2]))
          j2++;
        if (j2 === nameStart) {
          out += esc(ch);
          i++;
          continue;
        }
        const tagName = src.slice(nameStart, j2);
        out += span("tag", src.slice(i, j2));
        i = j2;
        let selfClosing = false;
        while (i < len) {
          const c = src[i];
          if (c === ">") {
            out += span("tag", ">");
            i++;
            break;
          }
          if (c === "/" && src[i + 1] === ">") {
            out += span("tag", "/>");
            i += 2;
            selfClosing = true;
            break;
          }
          if (c === '"' || c === "'") {
            let k2 = i + 1;
            while (k2 < len) {
              if (src[k2] === "\\") {
                k2 += 2;
                continue;
              }
              if (src[k2] === c) {
                k2++;
                break;
              }
              k2++;
            }
            out += span("string", src.slice(i, k2));
            i = k2;
            continue;
          }
          if (c === "{") {
            const close = findMatchingBrace(src, i);
            if (close >= 0) {
              out += span("bind", src.slice(i, close + 1));
              i = close + 1;
              continue;
            }
          }
          if (/[\s=]/.test(c)) {
            out += esc(c);
            i++;
            continue;
          }
          let k = i + 1;
          while (k < len && !/[\s=/>'"{}]/.test(src[k]))
            k++;
          out += span("attr", src.slice(i, k));
          i = k;
        }
        if (!closing && !selfClosing && CODE_TAGS.has(tagName.toLowerCase())) {
          const closeIdx = src.indexOf(`</${tagName}`, i);
          const stop = closeIdx < 0 ? len : closeIdx;
          out += highlightCode(src.slice(i, stop));
          i = stop;
        }
        continue;
      }
      if (ch === "{") {
        const close = findMatchingBrace(src, i);
        if (close >= 0) {
          out += span("bind", src.slice(i, close + 1));
          i = close + 1;
          continue;
        }
      }
      let j = i + 1;
      while (j < len && src[j] !== "<" && src[j] !== "{")
        j++;
      out += esc(src.slice(i, j));
      i = j;
    }
    return out;
  }
  function highlight(code) {
    const trimmed = code.trimStart();
    const isMarkup = trimmed.startsWith("<") || trimmed.startsWith("{#");
    return isMarkup ? highlightMarkup(code) : highlightCode(code, true);
  }

  // ../runtime/src/actions/syntax-highlight.ts
  var syntaxHighlight = (node) => {
    const el = node;
    const pre = el.querySelector("pre") ?? el;
    const render = () => {
      const src = el.getAttribute("code") ?? el.textContent ?? "";
      pre.innerHTML = highlight(src);
    };
    render();
    const mo = new MutationObserver(render);
    mo.observe(el, { attributes: true, attributeFilter: ["code"] });
    return { destroy: () => mo.disconnect() };
  };
  // ../runtime/src/actions/pressable.ts
  var pressable = (node) => {
    const el = node;
    if (!el.hasAttribute("role"))
      el.setAttribute("role", "button");
    const sync2 = () => {
      if (el.hasAttribute("disabled")) {
        el.setAttribute("aria-disabled", "true");
        el.setAttribute("tabindex", "-1");
      } else {
        el.removeAttribute("aria-disabled");
        if (el.getAttribute("tabindex") == null || el.getAttribute("tabindex") === "-1") {
          el.setAttribute("tabindex", "0");
        }
      }
    };
    sync2();
    const obs = new MutationObserver(sync2);
    obs.observe(el, { attributes: true, attributeFilter: ["disabled"] });
    const onKeydown = (e) => {
      if (el.hasAttribute("disabled"))
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        el.click();
      }
    };
    el.addEventListener("keydown", onKeydown);
    return {
      destroy: () => {
        obs.disconnect();
        el.removeEventListener("keydown", onKeydown);
      }
    };
  };
  // ../runtime/src/actions/pressed.ts
  var pressed = (node) => {
    const el = node;
    const sync2 = () => {
      el.setAttribute("aria-pressed", el.hasAttribute("selected") ? "true" : "false");
    };
    sync2();
    const obs = new MutationObserver(sync2);
    obs.observe(el, { attributes: true, attributeFilter: ["selected"] });
    return {
      destroy: () => {
        obs.disconnect();
      }
    };
  };
  // ../runtime/src/actions/dismissible.ts
  var dismissible = (node) => {
    const el = node;
    const emitRemove = () => {
      el.dispatchEvent(new CustomEvent("sail-remove", { detail: {}, bubbles: true, composed: true }));
    };
    const onKeydown = (e) => {
      if (el.hasAttribute("disabled"))
        return;
      if (!el.hasAttribute("removable"))
        return;
      if (e.key === "Backspace" || e.key === "Delete") {
        e.preventDefault();
        emitRemove();
      }
    };
    el.addEventListener("keydown", onKeydown);
    const remove = el.querySelector('button[aria-label="Remove"]');
    const onRemoveClick = (e) => {
      e.stopPropagation();
      if (el.hasAttribute("disabled"))
        return;
      if (!el.hasAttribute("removable"))
        return;
      emitRemove();
    };
    remove?.addEventListener("click", onRemoveClick);
    return {
      destroy: () => {
        el.removeEventListener("keydown", onKeydown);
        remove?.removeEventListener("click", onRemoveClick);
      }
    };
  };
  // ../runtime/src/actions/chrome-offset.ts
  var chromeOffset = (node) => {
    const root = typeof document !== "undefined" ? document.documentElement : null;
    const publish = () => {
      if (!root)
        return;
      const h = Math.round(node.getBoundingClientRect().height);
      if (h > 0)
        root.style.setProperty("--chrome-offset", `${h}px`);
    };
    let ro;
    if (typeof requestAnimationFrame !== "undefined") {
      requestAnimationFrame(() => {
        publish();
        if (typeof ResizeObserver !== "undefined") {
          ro = new ResizeObserver(publish);
          ro.observe(node);
        }
      });
    } else {
      publish();
    }
    const onResize = () => publish();
    if (typeof window !== "undefined") {
      window.addEventListener("resize", onResize);
    }
    return {
      destroy: () => {
        ro?.disconnect();
        if (typeof window !== "undefined") {
          window.removeEventListener("resize", onResize);
        }
      }
    };
  };
  // ../runtime/src/actions/described-control.ts
  var _seq2 = 0;
  var describedControl = (node) => {
    const el = node;
    const n = ++_seq2;
    const control = el.children[1] ?? null;
    const hint = el.children[2] ?? null;
    const error = el.querySelector('[role="alert"]');
    const hintId = `sail-field-hint-${n}`;
    const errorId = `sail-field-error-${n}`;
    if (hint)
      hint.id = hintId;
    if (error)
      error.id = errorId;
    const CONTROL_SEL = 'input, textarea, select, [contenteditable], [role="textbox"], [role="combobox"], [role="spinbutton"], [data-sail-host="switch"]';
    const wire = () => {
      const first = control?.querySelector(CONTROL_SEL);
      if (!first)
        return;
      const inv = el.hasAttribute("invalid");
      const targetId = inv ? errorId : hintId;
      const prior = (first.getAttribute("aria-describedby") || "").split(/\s+/).filter((id) => id && id !== hintId && id !== errorId);
      const next = [...prior, targetId].join(" ").trim();
      if (next)
        first.setAttribute("aria-describedby", next);
      first.setAttribute("aria-invalid", inv ? "true" : "false");
    };
    wire();
    const obs = new MutationObserver(wire);
    obs.observe(el, { attributes: true, attributeFilter: ["invalid"] });
    if (control)
      obs.observe(control, { childList: true, subtree: true });
    return { destroy: () => obs.disconnect() };
  };
  // ../runtime/src/actions/details-toggle.ts
  var detailsToggle = (node) => {
    const el = node;
    const onToggle = () => {
      el.dispatchEvent(new CustomEvent("sail-toggle", { detail: { open: el.open }, bubbles: true, composed: true }));
    };
    el.addEventListener("toggle", onToggle);
    return {
      destroy: () => {
        el.removeEventListener("toggle", onToggle);
      }
    };
  };
  // ../runtime/src/actions/details-disabled.ts
  var detailsDisabled = (node) => {
    const el = node;
    const onClickCapture = (e) => {
      if (el.hasAttribute("disabled"))
        e.preventDefault();
    };
    const onKeydownCapture = (e) => {
      if (el.hasAttribute("disabled") && (e.key === "Enter" || e.key === " "))
        e.preventDefault();
    };
    el.addEventListener("click", onClickCapture, true);
    el.addEventListener("keydown", onKeydownCapture, true);
    return {
      destroy: () => {
        el.removeEventListener("click", onClickCapture, true);
        el.removeEventListener("keydown", onKeydownCapture, true);
      }
    };
  };
  // ../runtime/src/actions/dismiss-self.ts
  var dismissed = new WeakSet;
  function isDismissed(el) {
    return dismissed.has(el) || !el.isConnected;
  }
  function dismissSelf(el, eventName = "sail-toast-dismiss") {
    if (dismissed.has(el))
      return false;
    dismissed.add(el);
    el.dispatchEvent(new CustomEvent(eventName, { bubbles: true, composed: true }));
    el.remove();
    return true;
  }

  // ../runtime/src/actions/auto-dismiss.ts
  function readDuration(el) {
    const raw = el.getAttribute("duration");
    if (raw == null || raw === "")
      return 5000;
    const n = Number(raw);
    return Number.isFinite(n) ? n : 5000;
  }
  var autoDismiss = (node) => {
    const el = node;
    const duration = readDuration(el);
    let timer;
    const fire = () => {
      if (isDismissed(el))
        return;
      if (timer) {
        clearTimeout(timer);
        timer = undefined;
      }
      dismissSelf(el);
    };
    const start = () => {
      if (isDismissed(el) || duration <= 0)
        return;
      if (timer)
        clearTimeout(timer);
      timer = setTimeout(fire, duration);
    };
    const pause = () => {
      if (timer)
        clearTimeout(timer);
      timer = undefined;
    };
    el.addEventListener("pointerenter", pause);
    el.addEventListener("pointerleave", start);
    el.addEventListener("focusin", pause);
    el.addEventListener("focusout", start);
    start();
    return {
      destroy: () => {
        pause();
        el.removeEventListener("pointerenter", pause);
        el.removeEventListener("pointerleave", start);
        el.removeEventListener("focusin", pause);
        el.removeEventListener("focusout", start);
      }
    };
  };
  // ../runtime/src/actions/dismiss-control.ts
  var CLOSE_SEL = '[data-toast-close], [data-dismiss], button[aria-label="Dismiss"]';
  function isToastHost(el) {
    return el.hasAttribute("duration") || el.getAttribute("data-sail-host") === "toast";
  }
  var dismissControl = (node) => {
    const el = node;
    const onClick = (e) => {
      if (isDismissed(el))
        return;
      const close = e.target?.closest(CLOSE_SEL);
      if (!close || !el.contains(close))
        return;
      const toast = isToastHost(el);
      if (!toast && !el.hasAttribute("dismissible"))
        return;
      dismissSelf(el, toast ? "sail-toast-dismiss" : "sail-dismiss");
    };
    el.addEventListener("click", onClick);
    return {
      destroy: () => {
        el.removeEventListener("click", onClick);
      }
    };
  };
  // ../runtime/src/actions/live-region.ts
  var liveRegion = (node) => {
    const el = node;
    const syncLive = () => {
      const urgent = el.getAttribute("tone") === "warning" || el.getAttribute("tone") === "destructive";
      el.setAttribute("role", urgent ? "alert" : "status");
      el.setAttribute("aria-live", urgent ? "assertive" : "polite");
    };
    syncLive();
    const obs = new MutationObserver(syncLive);
    obs.observe(el, { attributes: true, attributeFilter: ["tone"] });
    return {
      destroy: () => {
        obs.disconnect();
      }
    };
  };
  // ../runtime/src/actions/range-fill.ts
  var rangeFill = (node) => {
    const el = node;
    const sync2 = () => {
      const min = parseFloat(el.min || "0");
      const max = parseFloat(el.max || "100");
      const val = parseFloat(el.value || "0");
      const pct = max > min ? (val - min) / (max - min) * 100 : 0;
      el.style.setProperty("--rng-pct", `${pct}%`);
    };
    sync2();
    el.addEventListener("input", sync2);
    const obs = new MutationObserver(sync2);
    obs.observe(el, { attributes: true, attributeFilter: ["value", "min", "max"] });
    return {
      destroy: () => {
        el.removeEventListener("input", sync2);
        obs.disconnect();
      }
    };
  };
  // ../runtime/src/actions/radio-group.ts
  var _seq3 = 0;
  var radioGroup = (node) => {
    const el = node;
    if (!el.hasAttribute("role"))
      el.setAttribute("role", "radiogroup");
    const fallbackName = `sail-radio-${++_seq3}`;
    const hidden = document.createElement("input");
    hidden.type = "hidden";
    el.appendChild(hidden);
    const radios = () => Array.from(el.querySelectorAll('[data-sail-host="radio"]'));
    const inputOf = (r) => r.querySelector('input[type="radio"]');
    const sync2 = () => {
      const val = el.getAttribute("value") ?? "";
      const groupDisabled = el.hasAttribute("disabled");
      const name = el.getAttribute("name") || fallbackName;
      for (const r of radios()) {
        const input = inputOf(r);
        if (!input)
          continue;
        const rval = r.getAttribute("value") ?? "";
        input.name = name;
        input.value = rval;
        input.checked = rval === val;
        input.disabled = groupDisabled || r.hasAttribute("disabled");
      }
      hidden.name = el.getAttribute("name") ?? "";
      hidden.value = val;
      const label = el.getAttribute("label");
      if (label)
        el.setAttribute("aria-label", label);
      else
        el.removeAttribute("aria-label");
      if (el.hasAttribute("required"))
        el.setAttribute("aria-required", "true");
      else
        el.removeAttribute("aria-required");
    };
    const onChange = (e) => {
      const input = e.target;
      if (!input || input.type !== "radio")
        return;
      const val = input.value;
      if (el.getAttribute("value") === val)
        return;
      el.setAttribute("value", val);
      sync2();
      el.dispatchEvent(new CustomEvent("sail-change", { detail: { value: val }, bubbles: true, composed: true }));
    };
    el.addEventListener("change", onChange);
    sync2();
    const obs = new MutationObserver(sync2);
    obs.observe(el, {
      attributes: true,
      attributeFilter: ["value", "disabled", "name", "label", "required"],
      childList: true
    });
    return {
      destroy: () => {
        el.removeEventListener("change", onChange);
        obs.disconnect();
        hidden.remove();
      }
    };
  };
  // ../runtime/src/actions/tabs.ts
  var host = (name) => `[data-sail-host="${name}"]`;
  var tabs = (node) => {
    const root = node;
    const keyOf = (el, i) => el.getAttribute("name") || String(i);
    const tabsOf = () => Array.from(root.querySelectorAll(host("tab")));
    const panelsOf = () => Array.from(root.querySelectorAll(host("tab-panel")));
    const listOf = () => root.querySelector(host("tab-list"));
    const activeName = () => root.getAttribute("active") ?? "";
    const orientationOf = () => root.getAttribute("orientation") === "vertical" ? "vertical" : "horizontal";
    const setAttr = (el, name, value) => {
      if (el.getAttribute(name) !== value)
        el.setAttribute(name, value);
    };
    const stampOrientation = () => {
      const orientation = orientationOf();
      const list = listOf();
      if (list)
        setAttr(list, "orientation", orientation);
      for (const p of panelsOf())
        setAttr(p, "orientation", orientation);
    };
    const sync2 = () => {
      mo.disconnect();
      try {
        stampOrientation();
        const tabs2 = tabsOf();
        let active = activeName();
        if (!active && tabs2.length) {
          active = keyOf(tabs2[0], 0);
          setAttr(root, "active", active);
        }
        for (const [i, t] of tabs2.entries()) {
          const name = keyOf(t, i);
          const on = name === active;
          setAttr(t, "role", "tab");
          setAttr(t, "aria-selected", on ? "true" : "false");
          setAttr(t, "tabindex", on ? "0" : "-1");
          t.toggleAttribute("active", on);
          if (name) {
            if (!t.id)
              t.id = `sail-tab-${name}`;
            setAttr(t, "aria-controls", `sail-tabpanel-${name}`);
          }
        }
        for (const [i, p] of panelsOf().entries()) {
          const name = keyOf(p, i);
          const on = name === active;
          setAttr(p, "role", "tabpanel");
          p.hidden = !on;
          setAttr(p, "tabindex", on ? "0" : "-1");
          if (name) {
            if (!p.id)
              p.id = `sail-tabpanel-${name}`;
            setAttr(p, "aria-labelledby", `sail-tab-${name}`);
          }
        }
      } finally {
        mo.observe(root, {
          childList: true,
          subtree: true,
          attributes: true,
          attributeFilter: ["orientation", "active"]
        });
      }
    };
    const focusTab = (name) => {
      tabsOf().find((t, i) => keyOf(t, i) === name)?.focus();
    };
    const activate = (name) => {
      if (!name)
        return;
      if (name !== activeName()) {
        root.setAttribute("active", name);
        sync2();
        root.dispatchEvent(new CustomEvent("sail-tab-change", {
          detail: { name },
          bubbles: true,
          composed: true
        }));
      }
      focusTab(name);
    };
    const tabFrom = (e) => {
      const t = e.target?.closest(host("tab")) ?? null;
      return t && root.contains(t) ? t : null;
    };
    const onClick = (e) => {
      const t = tabFrom(e);
      if (t && !t.hasAttribute("disabled"))
        activate(keyOf(t, tabsOf().indexOf(t)));
    };
    const onKeydown = (e) => {
      const t = tabFrom(e);
      if (!t)
        return;
      const enabled = tabsOf().filter((x) => !x.hasAttribute("disabled"));
      const i = enabled.indexOf(t);
      if (i < 0)
        return;
      const vertical = orientationOf() === "vertical";
      const [next, prev] = vertical ? ["ArrowDown", "ArrowUp"] : ["ArrowRight", "ArrowLeft"];
      let j = -1;
      if (e.key === next)
        j = (i + 1) % enabled.length;
      else if (e.key === prev)
        j = (i - 1 + enabled.length) % enabled.length;
      else if (e.key === "Home")
        j = 0;
      else if (e.key === "End")
        j = enabled.length - 1;
      else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        if (!t.hasAttribute("disabled"))
          activate(keyOf(t, tabsOf().indexOf(t)));
        return;
      }
      if (j >= 0) {
        e.preventDefault();
        activate(keyOf(enabled[j], tabsOf().indexOf(enabled[j])));
      }
    };
    root.addEventListener("click", onClick);
    root.addEventListener("keydown", onKeydown);
    const mo = new MutationObserver(() => sync2());
    sync2();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        root.removeEventListener("keydown", onKeydown);
        mo.disconnect();
      }
    };
  };
  // ../runtime/src/actions/toggle-group.ts
  var host2 = (name) => `[data-sail-host="${name}"]`;
  var toggleGroup = (node) => {
    const root = node;
    const isMulti = () => root.getAttribute("type") === "multiple";
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const togglesOf = () => Array.from(root.querySelectorAll(host2("toggle")));
    const activeSet = () => new Set((root.getAttribute("value") ?? "").split(/\s+/).filter(Boolean));
    const orientationOf = () => root.getAttribute("orientation") === "vertical" ? "vertical" : "horizontal";
    if (!root.hasAttribute("role"))
      root.setAttribute("role", "group");
    const stampOrientation = () => {
      const orientation = orientationOf();
      for (const t of togglesOf()) {
        t.setAttribute("orientation", orientation);
      }
    };
    const sync2 = () => {
      stampOrientation();
      root.setAttribute("aria-orientation", orientationOf());
      const active = activeSet();
      const toggles = togglesOf();
      for (const t of toggles) {
        t.setAttribute("role", "button");
        t.setAttribute("aria-pressed", active.has(valueOf(t)) ? "true" : "false");
      }
      const enabled = toggles.filter((t) => !t.hasAttribute("disabled"));
      const focusable = enabled.find((t) => active.has(valueOf(t))) ?? enabled[0];
      for (const t of toggles)
        t.setAttribute("tabindex", t === focusable ? "0" : "-1");
    };
    const commit = (v) => {
      root.setAttribute("value", v);
      sync2();
      root.dispatchEvent(new CustomEvent("sail-toggle-change", { detail: { value: v }, bubbles: true, composed: true }));
    };
    const toggleFrom = (e) => {
      const t = e.target?.closest(host2("toggle")) ?? null;
      return t && root.contains(t) ? t : null;
    };
    const onClick = (e) => {
      const t = toggleFrom(e);
      if (!t || t.hasAttribute("disabled"))
        return;
      const v = valueOf(t);
      const active = activeSet();
      if (isMulti()) {
        if (active.has(v))
          active.delete(v);
        else
          active.add(v);
        commit([...active].join(" "));
      } else {
        commit(active.has(v) ? "" : v);
      }
      t.focus();
    };
    const onKeydown = (e) => {
      const t = toggleFrom(e);
      if (!t)
        return;
      const enabled = togglesOf().filter((x) => !x.hasAttribute("disabled"));
      const i = enabled.indexOf(t);
      if (i < 0)
        return;
      const vertical = orientationOf() === "vertical";
      const [next, prev] = vertical ? ["ArrowDown", "ArrowUp"] : ["ArrowRight", "ArrowLeft"];
      let j = -1;
      if (e.key === next)
        j = (i + 1) % enabled.length;
      else if (e.key === prev)
        j = (i - 1 + enabled.length) % enabled.length;
      else if (e.key === "Home")
        j = 0;
      else if (e.key === "End")
        j = enabled.length - 1;
      if (j >= 0) {
        e.preventDefault();
        const to = enabled[j];
        t.setAttribute("tabindex", "-1");
        to.setAttribute("tabindex", "0");
        to.focus();
      }
    };
    root.addEventListener("click", onClick);
    root.addEventListener("keydown", onKeydown);
    const mo = new MutationObserver(() => sync2());
    mo.observe(root, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["orientation", "value", "type"]
    });
    sync2();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        root.removeEventListener("keydown", onKeydown);
        mo.disconnect();
      }
    };
  };
  // ../tokens/src/themes.ts
  var THEMES = [
    {
      id: "ship-light",
      label: "SHIP Light",
      dark: false,
      swatches: [
        "oklch(0.99 0.001 106.423)",
        "oklch(1 0 0)",
        "oklch(0.67 0.16 58)",
        "oklch(0.61 0.23 303)"
      ]
    },
    {
      id: "ship-dark",
      label: "SHIP Dark",
      dark: true,
      swatches: [
        "oklch(0.18 0.01 250)",
        "oklch(0.18 0.01 50)",
        "oklch(0.77 0.16 70)",
        "oklch(0.3 0.05 300)"
      ]
    },
    {
      id: "paper",
      label: "Paper",
      dark: false,
      swatches: [
        "oklch(0.97 0.014 80)",
        "oklch(0.99 0.010 80)",
        "oklch(0.55 0.14 35)",
        "oklch(0.48 0.09 55)"
      ]
    },
    {
      id: "halo",
      label: "Halo (Soft)",
      dark: false,
      swatches: [
        "oklch(0.92 0.008 230)",
        "oklch(0.92 0.008 230)",
        "oklch(0.62 0.14 50)",
        "oklch(0.55 0.05 230)"
      ]
    },
    {
      id: "console",
      label: "Console",
      dark: true,
      swatches: [
        "oklch(0.13 0.008 220)",
        "oklch(0.16 0.010 220)",
        "oklch(0.78 0.20 145)",
        "oklch(0.78 0.16 195)"
      ]
    },
    {
      id: "contrast",
      label: "High Contrast",
      dark: true,
      swatches: [
        "oklch(0.09 0 0)",
        "oklch(0.15 0 0)",
        "oklch(0.87 0.18 85)",
        "oklch(0.99 0 0)"
      ]
    },
    {
      id: "glass",
      label: "Glass",
      dark: false,
      swatches: [
        "oklch(0.95 0.04 230)",
        "oklch(0.99 0.005 240)",
        "oklch(0.62 0.16 50)",
        "oklch(0.62 0.14 280)"
      ]
    },
    {
      id: "neutral-light",
      label: "Neutral Light",
      dark: false,
      swatches: [
        "oklch(0.985 0.002 264)",
        "oklch(1 0 0)",
        "oklch(0.27 0.020 264)",
        "oklch(0.55 0.090 255)"
      ]
    },
    {
      id: "neutral-dark",
      label: "Neutral Dark",
      dark: true,
      swatches: [
        "oklch(0.18 0.008 264)",
        "oklch(0.22 0.010 264)",
        "oklch(0.92 0.008 264)",
        "oklch(0.70 0.100 255)"
      ]
    },
    {
      id: "brut",
      label: "Brutalist",
      dark: false,
      swatches: [
        "oklch(0.97 0 0)",
        "oklch(1 0 0)",
        "oklch(0.14 0 0)",
        "oklch(0.62 0.24 28)"
      ]
    },
    {
      id: "8bit",
      label: "8-Bit",
      dark: true,
      swatches: [
        "oklch(0.18 0.05 265)",
        "oklch(0.24 0.06 265)",
        "oklch(0.72 0.21 25)",
        "oklch(0.82 0.20 95)"
      ]
    },
    {
      id: "synthwave",
      label: "Synthwave",
      dark: true,
      swatches: [
        "oklch(0.16 0.06 285)",
        "oklch(0.22 0.08 285)",
        "oklch(0.65 0.28 340)",
        "oklch(0.75 0.22 190)"
      ]
    }
  ];
  var THEME_BY_ID = Object.fromEntries(THEMES.map((t) => [t.id, t]));
  var DEFAULT_THEME = "ship-light";
  var THEME_STORAGE_KEY = "ship-site-theme";
  var isValidTheme = (value) => typeof value === "string" && (value in THEME_BY_ID);
  var isDarkTheme = (id) => THEME_BY_ID[id]?.dark ?? false;
  function applyTheme(id, root = document.documentElement) {
    const next = isValidTheme(id) ? id : DEFAULT_THEME;
    const dark = isDarkTheme(next);
    root.dataset.theme = next;
    root.classList.toggle("dark", dark);
    if (typeof document !== "undefined" && typeof CustomEvent !== "undefined") {
      document.dispatchEvent(new CustomEvent("ship:theme:change", { detail: { id: next, dark } }));
    }
    return next;
  }
  var themeTransport = null;
  function persistTheme(id) {
    try {
      localStorage.setItem(THEME_STORAGE_KEY, id);
    } catch {}
    themeTransport?.save(id);
  }

  // ../runtime/src/actions/theme-select.ts
  var uid = 0;
  var themeSelect = (node) => {
    const root = node;
    const trigger = root.querySelector('button[popovertarget], button[type="button"]');
    const panel = root.querySelector('[popover], [role="listbox"]');
    if (panel) {
      const id = `sail-theme-panel-${++uid}`;
      panel.id = id;
      trigger?.setAttribute("popovertarget", id);
    }
    const options = () => Array.from(root.querySelectorAll('[role="option"]'));
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const current = () => {
      const attr = root.getAttribute("value") || document.documentElement.dataset.theme;
      return isValidTheme(attr) ? attr : "ship-light";
    };
    const sync2 = () => {
      const cur = current();
      const name = THEMES.find((t) => t.id === cur)?.label ?? cur;
      if (trigger) {
        trigger.setAttribute("aria-label", `Theme: ${name}`);
        trigger.title = `Theme: ${name}`;
      }
      for (const o of options()) {
        const on = valueOf(o) === cur;
        o.setAttribute("aria-selected", on ? "true" : "false");
        o.setAttribute("tabindex", on ? "0" : "-1");
      }
    };
    const select = (v) => {
      if (!isValidTheme(v))
        return;
      root.setAttribute("value", v);
      applyTheme(v);
      persistTheme(v);
      sync2();
      root.dispatchEvent(new CustomEvent("sail-theme-change", { detail: { theme: v }, bubbles: true, composed: true }));
      panel?.hidePopover?.();
      trigger?.focus();
    };
    const optionFrom = (e) => {
      const o = e.target?.closest('[role="option"]') ?? null;
      return o && root.contains(o) ? o : null;
    };
    const onClick = (e) => {
      const o = optionFrom(e);
      if (o)
        select(valueOf(o));
    };
    const onKeydown = (e) => {
      const o = optionFrom(e);
      if (!o)
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        select(valueOf(o));
        return;
      }
      const opts = options();
      const i = opts.indexOf(o);
      let j = -1;
      if (e.key === "ArrowDown" || e.key === "ArrowRight")
        j = (i + 1) % opts.length;
      else if (e.key === "ArrowUp" || e.key === "ArrowLeft")
        j = (i - 1 + opts.length) % opts.length;
      else if (e.key === "Home")
        j = 0;
      else if (e.key === "End")
        j = opts.length - 1;
      if (j >= 0) {
        e.preventDefault();
        opts[j].focus();
      }
    };
    const onToggle = (e) => {
      if (e.newState !== "open")
        return;
      sync2();
      const cur = current();
      (options().find((o) => valueOf(o) === cur) ?? options()[0])?.focus();
    };
    root.addEventListener("click", onClick);
    root.addEventListener("keydown", onKeydown);
    panel?.addEventListener("toggle", onToggle);
    sync2();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        root.removeEventListener("keydown", onKeydown);
        panel?.removeEventListener("toggle", onToggle);
      }
    };
  };
  // ../runtime/src/actions/pagination.ts
  var host3 = (name) => `[data-sail-host="${name}"]`;
  var pagination = (node) => {
    const root = node;
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const pagesOf = () => Array.from(root.querySelectorAll(host3("page")));
    const current = () => root.getAttribute("value") ?? "";
    if (!root.hasAttribute("aria-label"))
      root.setAttribute("aria-label", "Pagination");
    const sync2 = () => {
      const cur = current();
      for (const p of pagesOf()) {
        if (valueOf(p) === cur)
          p.setAttribute("aria-current", "page");
        else
          p.removeAttribute("aria-current");
      }
    };
    const setPage = (v) => {
      if (!v || v === current())
        return;
      root.setAttribute("value", v);
      sync2();
      root.dispatchEvent(new CustomEvent("sail-page-change", { detail: { page: v }, bubbles: true, composed: true }));
    };
    const pageFrom = (e) => {
      const p = e.target?.closest(host3("page")) ?? null;
      return p && root.contains(p) ? p : null;
    };
    const onClick = (e) => {
      const p = pageFrom(e);
      if (p && !p.hasAttribute("disabled"))
        setPage(valueOf(p));
    };
    root.addEventListener("click", onClick);
    const mo = new MutationObserver(() => sync2());
    mo.observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ["value"] });
    sync2();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        mo.disconnect();
      }
    };
  };
  // ../runtime/src/actions/menu.ts
  var host4 = (name) => `[data-sail-host="${name}"]`;
  var isOpen = (el) => {
    try {
      return el.matches(":popover-open");
    } catch {
      return false;
    }
  };
  var menu = (node) => {
    const root = node;
    const itemSel = host4("menu-item");
    const itemsOf = () => Array.from(root.querySelectorAll(itemSel)).filter((i) => !i.hasAttribute("disabled"));
    if (!root.hasAttribute("role"))
      root.setAttribute("role", "menu");
    const sync2 = () => {
      for (const i of root.querySelectorAll(itemSel)) {
        i.setAttribute("role", "menuitem");
        if (!i.hasAttribute("tabindex"))
          i.setAttribute("tabindex", "-1");
        if (i.hasAttribute("disabled"))
          i.setAttribute("aria-disabled", "true");
      }
    };
    const close = () => {
      try {
        root.hidePopover?.();
      } catch {}
    };
    const onKeydown = (e) => {
      const list = itemsOf();
      if (!list.length)
        return;
      const cur = list.indexOf(document.activeElement);
      let j = -1;
      if (e.key === "ArrowDown")
        j = cur < 0 ? 0 : (cur + 1) % list.length;
      else if (e.key === "ArrowUp")
        j = cur < 0 ? list.length - 1 : (cur - 1 + list.length) % list.length;
      else if (e.key === "Home")
        j = 0;
      else if (e.key === "End")
        j = list.length - 1;
      else if (e.key === "Escape") {
        close();
        return;
      }
      if (j >= 0) {
        e.preventDefault();
        list[j].focus();
      }
    };
    const onClick = (e) => {
      const item = e.target?.closest(itemSel);
      if (!item || !root.contains(item))
        return;
      if (item.hasAttribute("disabled")) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      root.dispatchEvent(new CustomEvent("sail-menu-select", {
        detail: { value: item.getAttribute("value") ?? item.textContent?.trim() ?? "" },
        bubbles: true,
        composed: true
      }));
      close();
    };
    const onToggle = () => {
      if (isOpen(root))
        itemsOf()[0]?.focus();
    };
    root.addEventListener("keydown", onKeydown);
    root.addEventListener("click", onClick);
    root.addEventListener("toggle", onToggle);
    const mo = new MutationObserver(() => sync2());
    mo.observe(root, { childList: true, subtree: true });
    sync2();
    return {
      destroy: () => {
        root.removeEventListener("keydown", onKeydown);
        root.removeEventListener("click", onClick);
        root.removeEventListener("toggle", onToggle);
        mo.disconnect();
      }
    };
  };
  // ../runtime/src/actions/context-menu.ts
  var contextMenu = (node) => {
    const root = node;
    const target = () => {
      const id = root.getAttribute("for");
      if (id)
        return document.getElementById(id);
      return root.parentElement;
    };
    const onContext = (e) => {
      e.preventDefault();
      const px2 = e.clientX;
      const py = e.clientY;
      setTimeout(() => {
        try {
          root.hidePopover?.();
        } catch {}
        try {
          root.showPopover?.();
        } catch {
          return;
        }
        const r = root.getBoundingClientRect();
        const x = Math.min(px2, Math.max(0, window.innerWidth - r.width - 8));
        const y = Math.min(py, Math.max(0, window.innerHeight - r.height - 8));
        root.style.setProperty("--anchor-left", `${Math.max(8, x)}px`);
        root.style.setProperty("--anchor-top", `${Math.max(8, y)}px`);
      }, 0);
    };
    const t = target();
    t?.addEventListener("contextmenu", onContext);
    return {
      destroy: () => {
        t?.removeEventListener("contextmenu", onContext);
      }
    };
  };
  // ../runtime/src/actions/virtual-row.ts
  var virtualRow = (node, initial) => {
    const el = node;
    const apply = (start) => {
      if (start == null || Number.isNaN(start)) {
        el.style.removeProperty("--row-y");
      } else {
        el.style.setProperty("--row-y", `${start}px`);
      }
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  var virtualSpacer = (node, initial) => {
    const el = node;
    const apply = (total) => {
      if (total == null || Number.isNaN(total)) {
        el.style.removeProperty("--rows-h");
      } else {
        el.style.setProperty("--rows-h", `${total}px`);
      }
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  var cellPad = (node, initial) => {
    const el = node;
    const apply = (indent) => {
      if (indent == null || Number.isNaN(indent)) {
        el.style.removeProperty("--cell-ps");
      } else {
        el.style.setProperty("--cell-ps", `calc(var(--density-row-px) + ${indent}px)`);
      }
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  // ../runtime/src/actions/event-span.ts
  var toValue = (v) => {
    if (v == null)
      return null;
    if (typeof v === "number")
      return Number.isFinite(v) ? `${v}px` : null;
    return v === "" ? null : v;
  };
  var eventSpan = (node, initial) => {
    const el = node;
    const apply = (rect) => {
      if (!rect) {
        el.style.removeProperty("--evt-top");
        el.style.removeProperty("--evt-h");
        el.style.removeProperty("--evt-left");
        el.style.removeProperty("--evt-w");
        return;
      }
      const set = (name, v) => {
        const val = toValue(v);
        if (val == null)
          el.style.removeProperty(name);
        else
          el.style.setProperty(name, val);
      };
      set("--evt-top", rect.top);
      set("--evt-h", rect.height);
      set("--evt-left", rect.left);
      set("--evt-w", rect.width);
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  // ../runtime/src/actions/rect-box.ts
  var toValue2 = (v) => {
    if (v == null)
      return null;
    if (typeof v === "number")
      return Number.isFinite(v) ? `${v}px` : null;
    return v === "" ? null : v;
  };
  var rect = (node, initial) => {
    const el = node;
    const apply = (box) => {
      if (!box) {
        el.style.removeProperty("--box-top");
        el.style.removeProperty("--box-left");
        el.style.removeProperty("--box-w");
        el.style.removeProperty("--box-h");
        return;
      }
      const set = (name, v) => {
        const val = toValue2(v);
        if (val == null)
          el.style.removeProperty(name);
        else
          el.style.setProperty(name, val);
      };
      set("--box-top", box.top);
      set("--box-left", box.left);
      set("--box-w", box.width);
      set("--box-h", box.height);
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  var pin = (node, initial) => {
    const el = node;
    const apply = (p) => {
      if (!p) {
        el.style.removeProperty("--pin-x");
        el.style.removeProperty("--pin-y");
        return;
      }
      el.style.setProperty("--pin-x", Number.isFinite(p.x) ? `${p.x}px` : "0");
      el.style.setProperty("--pin-y", Number.isFinite(p.y) ? `${p.y}px` : "0");
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  // ../runtime/src/actions/port-edge.ts
  var KEYS = ["--port-top", "--port-left", "--port-right", "--port-bottom"];
  var writeEdge = (el, side, parentW, parentH, selfW, selfH) => {
    if (!side) {
      for (const k of KEYS)
        el.style.removeProperty(k);
      return;
    }
    const halfW = selfW / 2;
    const halfH = selfH / 2;
    switch (side) {
      case "left": {
        el.style.setProperty("--port-top", `${parentH / 2 - halfH}px`);
        el.style.setProperty("--port-left", `${-halfW}px`);
        el.style.removeProperty("--port-right");
        el.style.removeProperty("--port-bottom");
        break;
      }
      case "right": {
        el.style.setProperty("--port-top", `${parentH / 2 - halfH}px`);
        el.style.removeProperty("--port-left");
        el.style.setProperty("--port-right", `-${halfW}px`);
        el.style.removeProperty("--port-bottom");
        break;
      }
      case "top": {
        el.style.setProperty("--port-top", `${-halfH}px`);
        el.style.setProperty("--port-left", `${parentW / 2 - halfW}px`);
        el.style.removeProperty("--port-right");
        el.style.removeProperty("--port-bottom");
        break;
      }
      case "bottom": {
        el.style.setProperty("--port-bottom", `-${halfH}px`);
        el.style.setProperty("--port-left", `${parentW / 2 - halfW}px`);
        el.style.removeProperty("--port-right");
        el.style.removeProperty("--port-top");
        break;
      }
    }
  };
  var portEdge = (node, initial) => {
    const el = node;
    let observer = null;
    let currentSide = initial;
    const recompute = () => {
      if (!currentSide) {
        writeEdge(el, currentSide, 0, 0, 0, 0);
        return;
      }
      const parent = el.parentElement;
      if (!parent) {
        writeEdge(el, currentSide, 0, 0, 0, 0);
        return;
      }
      const pr = parent.getBoundingClientRect();
      const sr = el.getBoundingClientRect();
      writeEdge(el, currentSide, pr.width, pr.height, sr.width, sr.height);
    };
    const startObserve = () => {
      if (typeof ResizeObserver === "undefined")
        return;
      observer = new ResizeObserver(recompute);
      const parent = el.parentElement;
      if (parent)
        observer.observe(parent);
    };
    startObserve();
    recompute();
    return {
      update(next) {
        currentSide = next;
        if (next && !observer)
          startObserve();
        recompute();
      },
      destroy() {
        observer?.disconnect();
        observer = null;
        for (const k of KEYS)
          el.style.removeProperty(k);
      }
    };
  };
  // ../runtime/src/actions/world-transform.ts
  var worldTransform = (node, initial) => {
    const el = node;
    const apply = (cssTransform) => {
      if (cssTransform == null || cssTransform === "") {
        el.style.removeProperty("--world-tx");
      } else {
        el.style.setProperty("--world-tx", cssTransform);
      }
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  // ../runtime/src/actions/zoom-pan.ts
  var zoomPan = (node, initial) => {
    const el = node;
    const compose = (s) => {
      const x = Number.isFinite(s.x) ? s.x : 0;
      const y = Number.isFinite(s.y) ? s.y : 0;
      const sc = Number.isFinite(s.scale) ? s.scale : 1;
      return `translate(${x}px, ${y}px) scale(${sc})`;
    };
    const apply = (state) => {
      if (!state) {
        el.style.removeProperty("--zoom-tx");
        return;
      }
      el.style.setProperty("--zoom-tx", compose(state));
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  var maxWidth = (node, initial) => {
    const el = node;
    const toValue3 = (v) => {
      if (v == null)
        return null;
      if (typeof v === "number")
        return Number.isFinite(v) ? `${v}px` : null;
      return v === "" ? null : v;
    };
    const apply = (v) => {
      const val = toValue3(v);
      if (val == null)
        el.style.removeProperty("--max-w");
      else
        el.style.setProperty("--max-w", val);
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  var swatch = (node, initial) => {
    const el = node;
    const apply = (c) => {
      if (c == null || c === "") {
        el.style.removeProperty("--swatch-color");
      } else {
        el.style.setProperty("--swatch-color", c);
      }
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  // ../runtime/src/actions/hover-surface.ts
  var hoverSurface = (node) => {
    const root = node;
    const openMs = Number(root.getAttribute("open-delay") ?? "") || 200;
    const closeMs = Number(root.getAttribute("close-delay") ?? "") || 150;
    const trigger = () => root.id ? document.querySelector(`[popovertarget="${root.id}"]`) : null;
    let openT;
    let closeT;
    const clear = () => {
      if (openT)
        clearTimeout(openT);
      if (closeT)
        clearTimeout(closeT);
      openT = closeT = undefined;
    };
    const show = () => {
      clear();
      openT = setTimeout(() => {
        try {
          root.showPopover?.();
        } catch {}
      }, openMs);
    };
    const hide = () => {
      clear();
      closeT = setTimeout(() => {
        try {
          root.hidePopover?.();
        } catch {}
      }, closeMs);
    };
    const t = trigger();
    t?.addEventListener("pointerenter", show);
    t?.addEventListener("pointerleave", hide);
    t?.addEventListener("focus", show);
    t?.addEventListener("blur", hide);
    root.addEventListener("pointerenter", clear);
    root.addEventListener("pointerleave", hide);
    return {
      destroy: () => {
        clear();
        t?.removeEventListener("pointerenter", show);
        t?.removeEventListener("pointerleave", hide);
        t?.removeEventListener("focus", show);
        t?.removeEventListener("blur", hide);
        root.removeEventListener("pointerenter", clear);
        root.removeEventListener("pointerleave", hide);
      }
    };
  };
  // ../runtime/src/actions/grid-tracks.ts
  var gridTracks = (node, initial) => {
    const el = node;
    const apply = (tracks) => {
      if (tracks == null || tracks === "") {
        el.style.removeProperty("--sail-grid-template");
      } else {
        el.style.setProperty("--sail-grid-template", tracks);
      }
    };
    apply(initial);
    return {
      update: (next) => apply(next)
    };
  };
  // ../runtime/src/actions/spotlight.ts
  var OVERLAY_STYLE_ID = "sail-spotlight-overlay-style";
  var overlayCss = `
[data-sail='spotlight-glow'] {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0;
  transition: opacity 400ms ease;
  background: radial-gradient(
    38rem circle at var(--spot-x, 50%) var(--spot-y, 40%),
    color-mix(in oklch, var(--primary) var(--spot-strength, 10%), transparent),
    transparent 65%
  );
}
@media (prefers-reduced-motion: reduce) {
  [data-sail='spotlight-glow'] { transition: none; }
}
[data-sail='spotlight-glow'][data-on='true'] { opacity: 1; }
`.trim();
  var styleInjected = null;
  function ensureOverlayCss(doc) {
    if (styleInjected && styleInjected.isConnected)
      return;
    let el = doc.getElementById(OVERLAY_STYLE_ID);
    if (!el) {
      el = doc.createElement("style");
      el.id = OVERLAY_STYLE_ID;
      el.textContent = overlayCss;
      doc.head.appendChild(el);
    }
    styleInjected = el;
  }
  var spotlight = (node) => {
    const host5 = node;
    ensureOverlayCss(document);
    const glow = document.createElement("div");
    glow.dataset.sail = "spotlight-glow";
    glow.setAttribute("aria-hidden", "true");
    const finePointer = matchMedia("(hover: hover) and (pointer: fine)").matches;
    if (!finePointer) {
      glow.dataset.on = "true";
      host5.prepend(glow);
      return { destroy: () => glow.remove() };
    }
    host5.prepend(glow);
    let raf = 0;
    const onMove = (e) => {
      if (raf)
        return;
      raf = requestAnimationFrame(() => {
        raf = 0;
        const r = host5.getBoundingClientRect();
        glow.style.setProperty("--spot-x", `${e.clientX - r.left}px`);
        glow.style.setProperty("--spot-y", `${e.clientY - r.top}px`);
        glow.style.setProperty("--spot-strength", "14");
        glow.dataset.on = "true";
      });
    };
    const onLeave = () => {
      glow.dataset.on = "false";
    };
    host5.addEventListener("pointermove", onMove);
    host5.addEventListener("pointerleave", onLeave);
    return {
      destroy: () => {
        host5.removeEventListener("pointermove", onMove);
        host5.removeEventListener("pointerleave", onLeave);
        if (raf)
          cancelAnimationFrame(raf);
        glow.remove();
      }
    };
  };
  // ../runtime/src/actions/target-cursor.ts
  var TARGETS = 'a, button, [role="tab"], input, summary, select, textarea, [data-cursor]';
  var IDLE_HALF = 14;
  var PAD = 8;
  var OVERLAY_STYLE_ID2 = "sail-target-cursor-overlay-style";
  var overlayCss2 = `
[data-sail='tc-layer'] {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 2147483000;
  opacity: 0;
  transition: opacity 200ms ease;
}
[data-sail='tc-layer'][data-on='true'] { opacity: 1; }
[data-sail='tc-bracket'] {
  position: fixed;
  width: 12px;
  height: 12px;
  border-top: 2px solid var(--primary);
  border-left: 2px solid var(--primary);
  /* The action sets --tc-x / --tc-y per-frame as custom properties; the
     translate consumes them. */
  translate: var(--tc-x, 0) var(--tc-y, 0);
  left: 0;
  top: 0;
  will-change: translate;
}
/* One mechanism: each corner is DRAWN by its two borders — no rotation. */
[data-sail='tc-bracket'][data-side='tr'] { border-left: 0; border-right: 2px solid var(--primary); }
[data-sail='tc-bracket'][data-side='br'] { border: 0; border-bottom: 2px solid var(--primary); border-right: 2px solid var(--primary); }
[data-sail='tc-bracket'][data-side='bl'] { border: 0; border-bottom: 2px solid var(--primary); border-left: 2px solid var(--primary); }
[data-sail='tc-dot'] {
  position: fixed;
  width: 4px;
  height: 4px;
  border-radius: 99px;
  background: var(--primary);
  translate: var(--tc-x, 0) var(--tc-y, 0);
  /* Hide the dot until the first pointermove lands the cursor. */
  opacity: 0;
  transition: opacity 120ms ease;
}
[data-sail='tc-layer'][data-on='true'] [data-sail='tc-dot'] { opacity: 1; }
@media (prefers-reduced-motion: reduce) {
  [data-sail='tc-layer'] { transition: none; }
  [data-sail='tc-dot'] { transition: none; }
}
`.trim();
  var hostCss = `
[data-tc-on='true'] { cursor: none; }
`.trim();
  var styleInjected2 = null;
  function ensureOverlayCss2(doc) {
    if (styleInjected2 && styleInjected2.isConnected)
      return;
    let el = doc.getElementById(OVERLAY_STYLE_ID2);
    if (!el) {
      el = doc.createElement("style");
      el.id = OVERLAY_STYLE_ID2;
      el.textContent = overlayCss2 + `
` + hostCss;
      doc.head.appendChild(el);
    }
    styleInjected2 = el;
  }
  var targetCursor = (node) => {
    const host5 = node;
    if (!matchMedia("(hover: hover) and (pointer: fine)").matches) {
      return { destroy: () => {} };
    }
    ensureOverlayCss2(document);
    const snap = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const layer = document.createElement("div");
    layer.dataset.sail = "tc-layer";
    layer.setAttribute("aria-hidden", "true");
    const sides = ["tl", "tr", "br", "bl"];
    const corners = sides.map((side) => {
      const c = document.createElement("div");
      c.dataset.sail = "tc-bracket";
      c.dataset.side = side;
      layer.appendChild(c);
      return c;
    });
    const dot = document.createElement("div");
    dot.dataset.sail = "tc-dot";
    layer.appendChild(dot);
    layer.dataset.on = "false";
    host5.dataset.tcOn = "false";
    document.body.appendChild(layer);
    let mx = -9999;
    let my = -9999;
    let seenPointer = false;
    let target = null;
    const cur = sides.map(() => ({ x: mx, y: my }));
    const goals = () => {
      if (target) {
        const r = target.getBoundingClientRect();
        return [
          { x: r.left - PAD, y: r.top - PAD },
          { x: r.right + PAD, y: r.top - PAD },
          { x: r.right + PAD, y: r.bottom + PAD },
          { x: r.left - PAD, y: r.bottom + PAD }
        ];
      }
      return [
        { x: mx - IDLE_HALF, y: my - IDLE_HALF },
        { x: mx + IDLE_HALF, y: my - IDLE_HALF },
        { x: mx + IDLE_HALF, y: my + IDLE_HALF },
        { x: mx - IDLE_HALF, y: my + IDLE_HALF }
      ];
    };
    let raf = 0;
    const ease = snap ? 1 : 0.22;
    const tick = () => {
      const hr = host5.getBoundingClientRect();
      const inside = seenPointer && mx >= hr.left && mx <= hr.right && my >= hr.top && my <= hr.bottom;
      layer.dataset.on = inside ? "true" : "false";
      host5.dataset.tcOn = inside ? "true" : "false";
      if (!inside)
        target = null;
      const g = goals();
      corners.forEach((c, i) => {
        const ci = cur[i];
        const gi = g[i];
        ci.x += (gi.x - ci.x) * ease;
        ci.y += (gi.y - ci.y) * ease;
        const ox = i === 1 || i === 2 ? -12 : 0;
        const oy = i === 2 || i === 3 ? -12 : 0;
        c.style.setProperty("--tc-x", `${ci.x + ox}px`);
        c.style.setProperty("--tc-y", `${ci.y + oy}px`);
      });
      dot.style.setProperty("--tc-x", `${mx - 2}px`);
      dot.style.setProperty("--tc-y", `${my - 2}px`);
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    const onMove = (e) => {
      mx = e.clientX;
      my = e.clientY;
      seenPointer = true;
    };
    const onOver = (e) => {
      const t = e.target?.closest?.(TARGETS);
      if (t)
        target = t;
    };
    const onOut = (e) => {
      if (target && e.target instanceof Element && e.target.closest(TARGETS) === target)
        target = null;
    };
    const onLeave = () => {
      mx = -9999;
      my = -9999;
      target = null;
    };
    host5.addEventListener("pointermove", onMove, { passive: true });
    host5.addEventListener("pointerover", onOver, { passive: true });
    host5.addEventListener("pointerout", onOut, { passive: true });
    host5.addEventListener("pointerleave", onLeave);
    return {
      destroy: () => {
        cancelAnimationFrame(raf);
        host5.removeEventListener("pointermove", onMove);
        host5.removeEventListener("pointerover", onOver);
        host5.removeEventListener("pointerout", onOut);
        host5.removeEventListener("pointerleave", onLeave);
        if (host5.dataset.tcOn !== undefined)
          delete host5.dataset.tcOn;
        layer.remove();
      }
    };
  };
  // ../runtime/src/actions/combobox.ts
  var uid2 = 0;
  var combobox = (node) => {
    const root = node;
    const input = root.querySelector("input");
    const list = root.querySelector("[popover]");
    if (!input || !list)
      return;
    const ph = root.getAttribute("placeholder");
    if (ph)
      input.placeholder = ph;
    const val = root.getAttribute("value");
    if (val) {
      const match = [...root.querySelectorAll("option")].find((o) => o.getAttribute("value") === val);
      if (match)
        input.value = match.textContent?.trim() ?? "";
    }
    const id = `sail-cb-${++uid2}`;
    list.id ||= `${id}-list`;
    input.id ||= `${id}-input`;
    input.setAttribute("popovertarget", list.id);
    input.setAttribute("role", "combobox");
    input.setAttribute("aria-controls", list.id);
    input.setAttribute("aria-autocomplete", "list");
    input.setAttribute("aria-expanded", "false");
    if (!input.hasAttribute("autocomplete"))
      input.setAttribute("autocomplete", "off");
    list.setAttribute("role", "listbox");
    const options = () => [...list.querySelectorAll("option")];
    const shown = () => options().filter((o) => !o.hidden && !o.hasAttribute("disabled"));
    options().forEach((o, i) => {
      o.id ||= `${id}-opt-${i}`;
      o.setAttribute("role", "option");
      o.setAttribute("aria-selected", "false");
    });
    let active = null;
    const setActive = (o) => {
      if (active) {
        active.removeAttribute("data-active");
        active.setAttribute("aria-selected", "false");
      }
      active = o;
      if (o) {
        o.setAttribute("data-active", "");
        o.setAttribute("aria-selected", "true");
        input.setAttribute("aria-activedescendant", o.id);
        o.scrollIntoView({ block: "nearest" });
      } else {
        input.removeAttribute("aria-activedescendant");
      }
    };
    const isOpen2 = () => list.matches(":popover-open");
    const open = () => {
      if (isOpen2())
        return;
      try {
        list.showPopover?.();
      } catch {}
    };
    const close = () => {
      if (!isOpen2())
        return;
      try {
        list.hidePopover?.();
      } catch {}
    };
    const filter = () => {
      const q = input.value.trim().toLowerCase();
      for (const o of options()) {
        const text = (o.textContent ?? "").toLowerCase();
        o.hidden = q !== "" && !text.includes(q);
      }
      const vis = shown();
      if (!vis.length) {
        close();
        setActive(null);
        return;
      }
      if (!active || active.hidden)
        setActive(vis[0] ?? null);
      open();
    };
    const commit = (o) => {
      if (!o)
        return;
      const value = o.getAttribute("value") ?? o.textContent ?? "";
      input.value = o.textContent?.trim() ?? "";
      root.setAttribute("value", value);
      for (const opt of options())
        opt.hidden = false;
      close();
      root.dispatchEvent(new CustomEvent("sail-combobox-select", {
        detail: { value, label: input.value },
        bubbles: true,
        composed: true
      }));
    };
    const move = (delta) => {
      const vis = shown();
      if (!vis.length)
        return;
      if (!isOpen2())
        open();
      const i = active ? vis.indexOf(active) : -1;
      const next = (i + delta + vis.length) % vis.length;
      setActive(vis[next] ?? null);
    };
    const onInput = () => filter();
    const onKeydown = (e) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          isOpen2() ? move(1) : (open(), setActive(shown()[0] ?? null));
          break;
        case "ArrowUp":
          e.preventDefault();
          move(-1);
          break;
        case "Home":
          if (isOpen2()) {
            e.preventDefault();
            setActive(shown()[0] ?? null);
          }
          break;
        case "End":
          if (isOpen2()) {
            e.preventDefault();
            const vis = shown();
            setActive(vis[vis.length - 1] ?? null);
          }
          break;
        case "Enter":
          if (isOpen2() && active) {
            e.preventDefault();
            commit(active);
          }
          break;
        case "Escape":
          if (isOpen2()) {
            e.preventDefault();
            close();
          }
          break;
        default:
          break;
      }
    };
    const onPointerdown = (e) => {
      const o = e.target?.closest("option");
      if (o && !o.hasAttribute("disabled")) {
        e.preventDefault();
        commit(o);
      }
    };
    const onToggle = (e) => {
      const open_ = e.newState === "open";
      input.setAttribute("aria-expanded", String(open_));
      if (!open_)
        return setActive(null);
      if (active)
        return;
      const sel = root.getAttribute("value");
      const cur = sel ? options().find((o) => o.getAttribute("value") === sel) ?? null : null;
      setActive(cur && !cur.hidden ? cur : shown()[0] ?? null);
    };
    input.addEventListener("input", onInput);
    input.addEventListener("keydown", onKeydown);
    list.addEventListener("pointerdown", onPointerdown);
    list.addEventListener("toggle", onToggle);
    return {
      destroy: () => {
        input.removeEventListener("input", onInput);
        input.removeEventListener("keydown", onKeydown);
        list.removeEventListener("pointerdown", onPointerdown);
        list.removeEventListener("toggle", onToggle);
      }
    };
  };
  // behavior-entry.ts
  var motion = { attach: __sail_motion_attach, runLeave };
  globalThis.SailActions = exports_actions;
  globalThis.SailBoundary = exports_error_boundary;
  globalThis.SailMotion = motion;
})();
