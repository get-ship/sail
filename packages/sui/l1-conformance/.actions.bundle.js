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
    emptyState: () => emptyState,
    dropzone: () => dropzone,
    contextMenu: () => contextMenu,
    collapsible: () => collapsible,
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
      summary.textContent = n === 0 ? summaryDefault : n === 1 ? input.files[0].name : `${n} files selected`;
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
  // ../runtime/src/actions/theme-picker.ts
  var themePicker = (node) => {
    const root = node;
    const target = () => document.documentElement;
    const optionsOf = () => Array.from(root.querySelectorAll(".sui-theme-option"));
    const valueOf = (el) => el.getAttribute("value") ?? "";
    const current = () => root.getAttribute("value") || target().getAttribute("data-theme") || "";
    if (!root.hasAttribute("role"))
      root.setAttribute("role", "radiogroup");
    const sync = () => {
      const cur = current();
      const opts = optionsOf();
      const focusable = opts.find((o) => valueOf(o) === cur) ?? opts[0];
      for (const o of opts) {
        o.setAttribute("role", "radio");
        o.setAttribute("aria-checked", valueOf(o) === cur ? "true" : "false");
        o.setAttribute("tabindex", o === focusable ? "0" : "-1");
      }
    };
    const setTheme = (v) => {
      if (!v)
        return;
      root.setAttribute("value", v);
      target().setAttribute("data-theme", v);
      sync();
      root.dispatchEvent(new CustomEvent("sui-theme-change", { detail: { theme: v }, bubbles: true, composed: true }));
    };
    const optFrom = (e) => {
      const o = e.target?.closest(".sui-theme-option") ?? null;
      return o && root.contains(o) ? o : null;
    };
    const onClick = (e) => {
      const o = optFrom(e);
      if (o)
        setTheme(valueOf(o));
    };
    const onKeydown = (e) => {
      const o = optFrom(e);
      if (!o)
        return;
      const opts = optionsOf();
      const i = opts.indexOf(o);
      let j = -1;
      if (e.key === "ArrowRight" || e.key === "ArrowDown")
        j = (i + 1) % opts.length;
      else if (e.key === "ArrowLeft" || e.key === "ArrowUp")
        j = (i - 1 + opts.length) % opts.length;
      else if (e.key === "Home")
        j = 0;
      else if (e.key === "End")
        j = opts.length - 1;
      if (j >= 0) {
        e.preventDefault();
        setTheme(valueOf(opts[j]));
        opts[j].focus();
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
  // behavior-entry.ts
  globalThis.SuiActions = exports_actions;
})();
