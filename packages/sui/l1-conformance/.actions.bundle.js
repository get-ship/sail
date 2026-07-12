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

  // ../runtime/src/actions/index.ts
  var exports_actions = {};
  __export(exports_actions, {
    tooltip: () => tooltip,
    toggleGroup: () => toggleGroup,
    toast: () => toast,
    themePicker: () => themePicker,
    tabs: () => tabs,
    rangeFill: () => rangeFill,
    radioGroup: () => radioGroup,
    pagination: () => pagination,
    menu: () => menu,
    hoverCard: () => hoverCard,
    field: () => field,
    errorState: () => errorState,
    errorBoundary: () => errorBoundary,
    emptyState: () => emptyState,
    dropzone: () => dropzone,
    contextMenu: () => contextMenu,
    combobox: () => combobox,
    collapsible: () => collapsible,
    codeBlock: () => codeBlock,
    clickOutside: () => clickOutside,
    chip: () => chip,
    avatar: () => avatar,
    autosize: () => autosize,
    anchor: () => anchor,
    alert: () => alert
  });

  // ../runtime/src/actions/autosize.ts
  var autosize = (node) => {
    const el = node;
    const reflow = () => {
      el.style.height = "auto";
      el.style.height = `${el.scrollHeight}px`;
    };
    reflow();
    el.addEventListener("input", reflow);
    return {
      destroy: () => el.removeEventListener("input", reflow)
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
      el.style.position = "fixed";
      el.style.margin = "0";
      el.style.left = `${clamp(left, GAP, Math.max(GAP, vw - w - GAP))}px`;
      el.style.top = `${clamp(top, GAP, Math.max(GAP, vh - h - GAP))}px`;
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
  // ../runtime/src/actions/avatar.ts
  var avatar = (node) => {
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
  // ../runtime/src/actions/tooltip.ts
  var _seq = 0;
  var tooltip = (node) => {
    const el = node;
    const n = ++_seq;
    const bubbleId = `sui-tooltip-${n}`;
    el.style.setProperty("--sui-tooltip-anchor", `--sui-tooltip-${n}`);
    const bubble = el.querySelector(".bubble");
    const sync = () => {
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
    obs.observe(el, { attributes: true, attributeFilter: ["content"] });
    const onKeydown = (e) => {
      if (e.key === "Escape")
        el.blur();
    };
    el.addEventListener("keydown", onKeydown);
    return {
      destroy: () => {
        obs.disconnect();
        el.removeEventListener("keydown", onKeydown);
      }
    };
  };
  // ../runtime/src/actions/alert.ts
  var alert = (node) => {
    const el = node;
    const syncLive = () => {
      const urgent = el.getAttribute("tone") === "warning" || el.getAttribute("tone") === "destructive";
      el.setAttribute("role", urgent ? "alert" : "status");
      el.setAttribute("aria-live", urgent ? "assertive" : "polite");
    };
    syncLive();
    const obs = new MutationObserver(syncLive);
    obs.observe(el, { attributes: true, attributeFilter: ["tone"] });
    const close = el.querySelector(".close");
    const onClose = () => {
      if (!el.hasAttribute("dismissible"))
        return;
      el.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
      el.remove();
    };
    close?.addEventListener("click", onClose);
    return {
      destroy: () => {
        obs.disconnect();
        close?.removeEventListener("click", onClose);
      }
    };
  };
  // ../runtime/src/actions/empty-state.ts
  var emptyState = (node) => {
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
  // ../runtime/src/actions/error-state.ts
  var errorState = (node) => {
    const el = node;
    const code = el.querySelector(".code");
    const details = el.querySelector(".details");
    const toggle = el.querySelector(".details-toggle");
    const syncCode = () => {
      if (code)
        code.textContent = el.getAttribute("code") ?? "";
    };
    const hasDetails = () => !!details && (details.textContent ?? "").trim().length > 0;
    const syncDetails = () => {
      if (hasDetails()) {
        el.setAttribute("data-has-details", "");
      } else {
        el.removeAttribute("data-has-details");
        el.removeAttribute("data-details-open");
        if (toggle) {
          toggle.textContent = "Show details";
          toggle.setAttribute("aria-expanded", "false");
        }
      }
    };
    const onToggle = () => {
      const open = !el.hasAttribute("data-details-open");
      el.toggleAttribute("data-details-open", open);
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
    const firstAction = el.querySelector(".actions button, .actions [tabindex], .actions a");
    if (firstAction)
      requestAnimationFrame(() => firstAction.focus?.());
    return {
      destroy: () => {
        obs.disconnect();
        toggle?.removeEventListener("click", onToggle);
      }
    };
  };
  // ../runtime/src/error-boundary.ts
  var exports_error_boundary = {};
  __export(exports_error_boundary, {
    runProtected: () => runProtected,
    findNearestBoundary: () => findNearestBoundary,
    detachBoundary: () => detachBoundary,
    attachBoundary: () => attachBoundary
  });
  var BOUNDARY_KEY = Symbol.for("sui:error-boundary");
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
      console.error("[sui] uncaught throw (no error boundary):", err);
      return;
    }
  }

  // ../runtime/src/actions/error-boundary.ts
  var errorBoundary = (node) => {
    const el = node;
    const message = el.querySelector(".message");
    const retry = el.querySelector(".retry");
    const reset = () => {
      if (!el.hasAttribute("data-caught"))
        return;
      el.removeAttribute("data-caught");
      el.removeAttribute("data-error-message");
      el.dispatchEvent(new CustomEvent("sui-error-reset", { bubbles: true, composed: true }));
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
        el.dispatchEvent(new CustomEvent("sui-error-caught", {
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
  function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }
  var RE = new RegExp([
    String.raw`(?<comment><!--[\s\S]*?-->|\/\/[^\n]*|\/\*[\s\S]*?\*\/|#[^\n]*)`,
    String.raw`(?<string>\`(?:\\.|[^\`\\])*\`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')`,
    String.raw`(?<bind>\{[^{}]*\})`,
    String.raw`(?<tag><\/?[a-zA-Z][\w-]*|\/?>)`,
    String.raw`(?<num>\b\d+(?:\.\d+)?\b)`,
    String.raw`(?<word>[A-Za-z_$][\w$]*)`
  ].join("|"), "g");
  function highlight(code) {
    let out = "";
    let last = 0;
    for (const m of code.matchAll(RE)) {
      const i = m.index ?? 0;
      out += esc(code.slice(last, i));
      const g = m.groups ?? {};
      const text = m[0];
      if (g.comment != null)
        out += `<span class="tok-comment">${esc(text)}</span>`;
      else if (g.string != null)
        out += `<span class="tok-string">${esc(text)}</span>`;
      else if (g.bind != null)
        out += `<span class="tok-bind">${esc(text)}</span>`;
      else if (g.tag != null)
        out += `<span class="tok-tag">${esc(text)}</span>`;
      else if (g.num != null)
        out += `<span class="tok-number">${esc(text)}</span>`;
      else if (g.word != null) {
        if (KEYWORDS.has(text))
          out += `<span class="tok-keyword">${esc(text)}</span>`;
        else if (VERBS.has(text))
          out += `<span class="tok-fn">${esc(text)}</span>`;
        else
          out += esc(text);
      } else
        out += esc(text);
      last = i + text.length;
    }
    out += esc(code.slice(last));
    return out;
  }

  // ../runtime/src/actions/code-block.ts
  var codeBlock = (node) => {
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
  // ../runtime/src/actions/chip.ts
  var chip = (node) => {
    const el = node;
    if (!el.hasAttribute("role"))
      el.setAttribute("role", "button");
    const sync = () => {
      const disabled = el.hasAttribute("disabled");
      el.setAttribute("aria-pressed", el.hasAttribute("selected") ? "true" : "false");
      if (disabled) {
        el.setAttribute("aria-disabled", "true");
        el.setAttribute("tabindex", "-1");
      } else {
        el.removeAttribute("aria-disabled");
        if (el.getAttribute("tabindex") == null || el.getAttribute("tabindex") === "-1") {
          el.setAttribute("tabindex", "0");
        }
      }
    };
    sync();
    const obs = new MutationObserver(sync);
    obs.observe(el, { attributes: true, attributeFilter: ["selected", "disabled"] });
    const emitRemove = () => {
      el.dispatchEvent(new CustomEvent("sui-remove", { detail: {}, bubbles: true, composed: true }));
    };
    const onKeydown = (e) => {
      if (el.hasAttribute("disabled"))
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        el.click();
      } else if (el.hasAttribute("removable") && (e.key === "Backspace" || e.key === "Delete")) {
        e.preventDefault();
        emitRemove();
      }
    };
    el.addEventListener("keydown", onKeydown);
    const remove = el.querySelector(".remove");
    const onRemoveClick = (e) => {
      e.stopPropagation();
      if (el.hasAttribute("disabled"))
        return;
      emitRemove();
    };
    remove?.addEventListener("click", onRemoveClick);
    return {
      destroy: () => {
        obs.disconnect();
        el.removeEventListener("keydown", onKeydown);
        remove?.removeEventListener("click", onRemoveClick);
      }
    };
  };
  // ../runtime/src/actions/field.ts
  var _seq2 = 0;
  var field = (node) => {
    const el = node;
    const n = ++_seq2;
    const hint = el.querySelector(".hint");
    const error = el.querySelector(".error");
    const control = el.querySelector(".control");
    const hintId = `sui-field-hint-${n}`;
    const errorId = `sui-field-error-${n}`;
    if (hint)
      hint.id = hintId;
    if (error)
      error.id = errorId;
    const CONTROL_SEL = 'input, textarea, select, [contenteditable], [role="textbox"], [role="combobox"], [role="spinbutton"], .sui-switch';
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
  // ../runtime/src/actions/collapsible.ts
  var collapsible = (node) => {
    const el = node;
    const onToggle = () => {
      el.dispatchEvent(new CustomEvent("sui-toggle", { detail: { open: el.open }, bubbles: true, composed: true }));
    };
    const onClickCapture = (e) => {
      if (el.hasAttribute("disabled"))
        e.preventDefault();
    };
    const onKeydownCapture = (e) => {
      if (el.hasAttribute("disabled") && (e.key === "Enter" || e.key === " "))
        e.preventDefault();
    };
    el.addEventListener("toggle", onToggle);
    el.addEventListener("click", onClickCapture, true);
    el.addEventListener("keydown", onKeydownCapture, true);
    return {
      destroy: () => {
        el.removeEventListener("toggle", onToggle);
        el.removeEventListener("click", onClickCapture, true);
        el.removeEventListener("keydown", onKeydownCapture, true);
      }
    };
  };
  // ../runtime/src/actions/range-fill.ts
  var rangeFill = (node) => {
    const el = node;
    const sync = () => {
      const min = parseFloat(el.min || "0");
      const max = parseFloat(el.max || "100");
      const val = parseFloat(el.value || "0");
      const pct = max > min ? (val - min) / (max - min) * 100 : 0;
      el.style.setProperty("--rng-pct", `${pct}%`);
    };
    sync();
    el.addEventListener("input", sync);
    const obs = new MutationObserver(sync);
    obs.observe(el, { attributes: true, attributeFilter: ["value", "min", "max"] });
    return {
      destroy: () => {
        el.removeEventListener("input", sync);
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
    const fallbackName = `sui-radio-${++_seq3}`;
    const hidden = document.createElement("input");
    hidden.type = "hidden";
    el.appendChild(hidden);
    const radios = () => Array.from(el.querySelectorAll(".sui-radio"));
    const inputOf = (r) => r.querySelector('input[type="radio"]');
    const sync = () => {
      const val = el.getAttribute("value") ?? "";
      const groupDisabled = el.hasAttribute("disabled");
      const name = el.getAttribute("name") || fallbackName;
      const tone = el.getAttribute("tone");
      const size = el.getAttribute("size");
      for (const r of radios()) {
        const input = inputOf(r);
        if (!input)
          continue;
        const rval = r.getAttribute("value") ?? "";
        input.name = name;
        input.value = rval;
        input.checked = rval === val;
        input.disabled = groupDisabled || r.hasAttribute("disabled");
        if (tone && !r.hasAttribute("tone"))
          r.setAttribute("tone", tone);
        if (size && !r.hasAttribute("size"))
          r.setAttribute("size", size);
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
      sync();
      el.dispatchEvent(new CustomEvent("sui-change", { detail: { value: val }, bubbles: true, composed: true }));
    };
    el.addEventListener("change", onChange);
    sync();
    const obs = new MutationObserver(sync);
    obs.observe(el, {
      attributes: true,
      attributeFilter: ["value", "disabled", "name", "label", "required", "tone", "size"],
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
  var tabs = (node) => {
    const root = node;
    const nameOf = (el) => el.getAttribute("name") ?? "";
    const tabsOf = () => Array.from(root.querySelectorAll(".sui-tab"));
    const panelsOf = () => Array.from(root.querySelectorAll(".sui-tab-panel"));
    const activeName = () => root.getAttribute("active") ?? "";
    const sync = () => {
      const tabs2 = tabsOf();
      let active = activeName();
      if (!active && tabs2.length) {
        active = nameOf(tabs2[0]);
        if (active)
          root.setAttribute("active", active);
      }
      for (const t of tabs2) {
        const name = nameOf(t);
        const on = name === active;
        t.setAttribute("role", "tab");
        t.setAttribute("aria-selected", on ? "true" : "false");
        t.setAttribute("tabindex", on ? "0" : "-1");
        t.toggleAttribute("active", on);
        if (name) {
          if (!t.id)
            t.id = `sui-tab-${name}`;
          t.setAttribute("aria-controls", `sui-tabpanel-${name}`);
        }
      }
      for (const p of panelsOf()) {
        const name = nameOf(p);
        const on = name === active;
        p.setAttribute("role", "tabpanel");
        p.hidden = !on;
        p.setAttribute("tabindex", on ? "0" : "-1");
        if (name) {
          if (!p.id)
            p.id = `sui-tabpanel-${name}`;
          p.setAttribute("aria-labelledby", `sui-tab-${name}`);
        }
      }
    };
    const focusTab = (name) => {
      tabsOf().find((t) => nameOf(t) === name)?.focus();
    };
    const activate = (name) => {
      if (!name)
        return;
      if (name !== activeName()) {
        root.setAttribute("active", name);
        sync();
        root.dispatchEvent(new CustomEvent("sui-tab-change", {
          detail: { name },
          bubbles: true,
          composed: true
        }));
      }
      focusTab(name);
    };
    const tabFrom = (e) => {
      const t = e.target?.closest(".sui-tab") ?? null;
      return t && root.contains(t) ? t : null;
    };
    const onClick = (e) => {
      const t = tabFrom(e);
      if (t && !t.hasAttribute("disabled"))
        activate(nameOf(t));
    };
    const onKeydown = (e) => {
      const t = tabFrom(e);
      if (!t)
        return;
      const enabled = tabsOf().filter((x) => !x.hasAttribute("disabled"));
      const i = enabled.indexOf(t);
      if (i < 0)
        return;
      const vertical = root.getAttribute("orientation") === "vertical";
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
          activate(nameOf(t));
        return;
      }
      if (j >= 0) {
        e.preventDefault();
        activate(nameOf(enabled[j]));
      }
    };
    root.addEventListener("click", onClick);
    root.addEventListener("keydown", onKeydown);
    const mo = new MutationObserver(() => sync());
    mo.observe(root, { childList: true, subtree: true });
    sync();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        root.removeEventListener("keydown", onKeydown);
        mo.disconnect();
      }
    };
  };
  // ../runtime/src/actions/toggle-group.ts
  var toggleGroup = (node) => {
    const root = node;
    const isMulti = () => root.getAttribute("type") === "multiple";
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const togglesOf = () => Array.from(root.querySelectorAll(".sui-toggle"));
    const activeSet = () => new Set((root.getAttribute("value") ?? "").split(/\s+/).filter(Boolean));
    if (!root.hasAttribute("role"))
      root.setAttribute("role", "group");
    const sync = () => {
      root.setAttribute("aria-orientation", root.getAttribute("orientation") === "vertical" ? "vertical" : "horizontal");
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
      sync();
      root.dispatchEvent(new CustomEvent("sui-toggle-change", { detail: { value: v }, bubbles: true, composed: true }));
    };
    const toggleFrom = (e) => {
      const t = e.target?.closest(".sui-toggle") ?? null;
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
      const vertical = root.getAttribute("orientation") === "vertical";
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
    const mo = new MutationObserver(() => sync());
    mo.observe(root, { childList: true, subtree: true });
    sync();
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

  // ../runtime/src/actions/theme-picker.ts
  var themePicker = (node) => {
    const root = node;
    const trigger = root.querySelector(".sui-theme-trigger");
    const panel = root.querySelector(".sui-theme-panel");
    const options = () => Array.from(root.querySelectorAll(".sui-theme-option"));
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const current = () => {
      const attr = root.getAttribute("value") || document.documentElement.dataset.theme;
      return isValidTheme(attr) ? attr : "ship-light";
    };
    const sync = () => {
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
      sync();
      root.dispatchEvent(new CustomEvent("sui-theme-change", { detail: { theme: v }, bubbles: true, composed: true }));
      panel?.hidePopover?.();
      trigger?.focus();
    };
    const optionFrom = (e) => {
      const o = e.target?.closest(".sui-theme-option") ?? null;
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
      sync();
      const cur = current();
      (options().find((o) => valueOf(o) === cur) ?? options()[0])?.focus();
    };
    root.addEventListener("click", onClick);
    root.addEventListener("keydown", onKeydown);
    panel?.addEventListener("toggle", onToggle);
    sync();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        root.removeEventListener("keydown", onKeydown);
        panel?.removeEventListener("toggle", onToggle);
      }
    };
  };
  // ../runtime/src/actions/pagination.ts
  var pagination = (node) => {
    const root = node;
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const pagesOf = () => Array.from(root.querySelectorAll(".sui-page"));
    const current = () => root.getAttribute("value") ?? "";
    if (!root.hasAttribute("aria-label"))
      root.setAttribute("aria-label", "Pagination");
    const sync = () => {
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
      sync();
      root.dispatchEvent(new CustomEvent("sui-page-change", { detail: { page: v }, bubbles: true, composed: true }));
    };
    const pageFrom = (e) => {
      const p = e.target?.closest(".sui-page") ?? null;
      return p && root.contains(p) ? p : null;
    };
    const onClick = (e) => {
      const p = pageFrom(e);
      if (p && !p.hasAttribute("disabled"))
        setPage(valueOf(p));
    };
    root.addEventListener("click", onClick);
    const mo = new MutationObserver(() => sync());
    mo.observe(root, { childList: true, subtree: true });
    sync();
    return {
      destroy: () => {
        root.removeEventListener("click", onClick);
        mo.disconnect();
      }
    };
  };
  // ../runtime/src/actions/menu.ts
  var isOpen = (el) => {
    try {
      return el.matches(":popover-open");
    } catch {
      return false;
    }
  };
  var menu = (node) => {
    const root = node;
    const itemsOf = () => Array.from(root.querySelectorAll(".sui-menu-item")).filter((i) => !i.hasAttribute("disabled"));
    if (!root.hasAttribute("role"))
      root.setAttribute("role", "menu");
    const sync = () => {
      for (const i of root.querySelectorAll(".sui-menu-item")) {
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
      const item = e.target?.closest(".sui-menu-item");
      if (!item || !root.contains(item))
        return;
      if (item.hasAttribute("disabled")) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      root.dispatchEvent(new CustomEvent("sui-menu-select", {
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
    const mo = new MutationObserver(() => sync());
    mo.observe(root, { childList: true, subtree: true });
    sync();
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
      const px = e.clientX;
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
        root.style.position = "fixed";
        root.style.margin = "0";
        const r = root.getBoundingClientRect();
        const x = Math.min(px, Math.max(0, window.innerWidth - r.width - 8));
        const y = Math.min(py, Math.max(0, window.innerHeight - r.height - 8));
        root.style.left = `${Math.max(8, x)}px`;
        root.style.top = `${Math.max(8, y)}px`;
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
  // ../runtime/src/actions/hover-card.ts
  var hoverCard = (node) => {
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
  // ../runtime/src/actions/toast.ts
  var toast = (node) => {
    const el = node;
    const duration = Number(el.getAttribute("duration") ?? "") || 5000;
    let timer;
    let done = false;
    const dismiss = () => {
      if (done)
        return;
      done = true;
      if (timer)
        clearTimeout(timer);
      el.dispatchEvent(new CustomEvent("sui-toast-dismiss", { bubbles: true, composed: true }));
      el.remove();
    };
    const start = () => {
      if (done || duration <= 0)
        return;
      if (timer)
        clearTimeout(timer);
      timer = setTimeout(dismiss, duration);
    };
    const pause = () => {
      if (timer)
        clearTimeout(timer);
      timer = undefined;
    };
    const onClick = (e) => {
      const close = e.target?.closest("[data-toast-close]");
      if (close && el.contains(close))
        dismiss();
    };
    if (!el.hasAttribute("role"))
      el.setAttribute("role", "status");
    if (!el.hasAttribute("aria-live"))
      el.setAttribute("aria-live", "polite");
    el.addEventListener("click", onClick);
    el.addEventListener("pointerenter", pause);
    el.addEventListener("pointerleave", start);
    el.addEventListener("focusin", pause);
    el.addEventListener("focusout", start);
    start();
    return {
      destroy: () => {
        pause();
        el.removeEventListener("click", onClick);
        el.removeEventListener("pointerenter", pause);
        el.removeEventListener("pointerleave", start);
        el.removeEventListener("focusin", pause);
        el.removeEventListener("focusout", start);
      }
    };
  };
  // ../runtime/src/actions/combobox.ts
  var uid = 0;
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
    const id = `sui-cb-${++uid}`;
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
      root.dispatchEvent(new CustomEvent("sui-combobox-select", {
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

  // ../runtime/src/motion.ts
  function prefersReducedMotion() {
    return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  }
  var TRIGGER_EVENTS = {
    hover: ["mouseenter", "mouseleave"],
    press: ["pointerdown", "pointerup", "pointercancel"],
    focus: ["focus", "blur"],
    "focus-visible": ["focusin", "focusout"]
  };
  function isFocusVisible(ev) {
    const target = ev.target;
    if (target && typeof target.matches === "function") {
      try {
        return target.matches(":focus-visible");
      } catch {}
    }
    return focusSource === "keyboard";
  }
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
  function rehome(a) {
    if (typeof document === "undefined" || typeof Animation !== "function") {
      return a;
    }
    const timeline = document.timeline;
    if (!timeline || a.timeline === timeline)
      return a;
    const effect = a.effect;
    if (!effect)
      return a;
    a.cancel();
    return new Animation(effect, timeline);
  }
  function __sui_motion_attach(el, config) {
    const { trigger } = config;
    const rehomed = new Map;
    const fix = (a) => {
      if (a == null)
        return null;
      let r = rehomed.get(a);
      if (!r) {
        r = rehome(a);
        rehomed.set(a, r);
      }
      return r;
    };
    const enter = config.enter.map(fix).filter((a) => a != null);
    const leave = config.leave.map(fix).filter((a) => a != null);
    if (enter.length === 0)
      return () => {};
    for (const a of enter) {
      a.cancel();
    }
    const playForward = () => {
      for (const a of enter) {
        a.play();
      }
    };
    const playReverse = () => {
      enter.forEach((a, i) => {
        const b = leave[i] ?? a;
        try {
          a.cancel();
        } catch {}
        b.currentTime = 0;
        b.play();
      });
    };
    if (trigger === "view" || trigger === "scroll") {
      if (prefersReducedMotion()) {
        for (const a of enter)
          a.finish();
        return () => {
          for (const a of enter)
            a.cancel();
        };
      }
      const timeline = makeTimeline(trigger, el);
      if (!timeline) {
        if (trigger === "view" && typeof IntersectionObserver !== "undefined") {
          const io = new IntersectionObserver((entries) => {
            if (entries.some((e) => e.isIntersecting)) {
              io.disconnect();
              for (const a of enter)
                a.play();
            }
          }, { threshold: 0.15 });
          io.observe(el);
          return () => {
            io.disconnect();
            for (const a of enter)
              a.cancel();
          };
        }
        for (const a of enter)
          a.finish();
        return () => {
          for (const a of enter)
            a.cancel();
        };
      }
      for (const a of enter) {
        a.timeline = timeline;
        a.play();
      }
      return () => {
        for (const a of enter)
          a.cancel();
      };
    }
    if (trigger === "enter" || trigger === "exit") {
      installLifecycleRunner();
      if (trigger === "enter") {
        if (prefersReducedMotion()) {
          for (const a of enter)
            a.finish();
          return () => {
            for (const a of enter)
              a.cancel();
          };
        }
        if (config.stagger) {
          const delay = staggerOffset() * config.stagger;
          for (const a of enter) {
            a.effect?.updateTiming({ delay });
          }
        }
        playForward();
        return () => {
          for (const a of enter)
            a.cancel();
        };
      }
      for (const a of enter)
        a.cancel();
      EXITS.set(el, enter);
      return () => {
        EXITS.delete(el);
        for (const a of enter)
          a.cancel();
      };
    }
    const events = TRIGGER_EVENTS[trigger];
    if (!events) {
      return () => {
        for (const a of enter)
          a.cancel();
      };
    }
    if (trigger === "focus-visible")
      trackFocusSource();
    const [enterEvent, leaveEvent, cancelEvent] = events;
    const onEnter = (ev) => {
      if (prefersReducedMotion())
        return;
      if (trigger === "focus-visible" && !isFocusVisible(ev)) {
        return;
      }
      playForward();
    };
    const onLeave = () => {
      if (prefersReducedMotion())
        return;
      playReverse();
    };
    el.addEventListener(enterEvent, onEnter);
    el.addEventListener(leaveEvent, onLeave);
    if (cancelEvent) {
      el.addEventListener(cancelEvent, onLeave);
    }
    return () => {
      el.removeEventListener(enterEvent, onEnter);
      el.removeEventListener(leaveEvent, onLeave);
      if (cancelEvent) {
        el.removeEventListener(cancelEvent, onLeave);
      }
      for (const a of enter) {
        try {
          a.cancel();
        } catch {}
      }
    };
  }
  var EXITS = new WeakMap;
  var runnerInstalled = false;
  function installLifecycleRunner() {
    if (runnerInstalled)
      return;
    runnerInstalled = true;
    setTransitionRunner({
      runEnter: () => null,
      runLeave: (nodes) => {
        const playing = [];
        for (const n of nodes) {
          if (!(n instanceof Element))
            continue;
          const targets = EXITS.has(n) ? [n] : [...n.querySelectorAll("*")].filter((e) => EXITS.has(e));
          for (const t of targets) {
            const anims = EXITS.get(t);
            if (!anims)
              continue;
            for (const a of anims) {
              a.currentTime = 0;
              a.play();
              playing.push(a.finished.catch(() => {}));
            }
          }
        }
        if (playing.length === 0)
          return null;
        if (prefersReducedMotion())
          return null;
        return Promise.all(playing).then(() => {});
      },
      cancelTransitions: (node) => {
        if (!(node instanceof Element))
          return;
        const anims = EXITS.get(node);
        if (anims)
          for (const a of anims)
            a.cancel();
      }
    });
  }
  var waveCount = 0;
  var waveScheduled = false;
  function staggerOffset() {
    const i = waveCount++;
    if (!waveScheduled) {
      waveScheduled = true;
      queueMicrotask(() => {
        waveCount = 0;
        waveScheduled = false;
      });
    }
    return i;
  }
  function makeTimeline(kind, el) {
    const g = globalThis;
    try {
      if (kind === "view") {
        return g.ViewTimeline ? new g.ViewTimeline({ subject: el }) : null;
      }
      return g.ScrollTimeline ? new g.ScrollTimeline({ source: document.scrollingElement }) : null;
    } catch {
      return null;
    }
  }

  // behavior-entry.ts
  var motion = { attach: __sui_motion_attach, runLeave };
  globalThis.SuiActions = exports_actions;
  globalThis.SuiBoundary = exports_error_boundary;
  globalThis.SuiMotion = motion;
})();
