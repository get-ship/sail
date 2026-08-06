import { n as userError } from "../../design-hH4c3EtQ.js";
import { __tpl, bindAttr, bindFor, bindIf, bindProp, bindText, component, controller, delegateEvents, markHydrateText, on, prop, runAction } from "@ship/sail-runtime/internal";
import { mountAmbient, persisted, signal } from "@ship/sail-runtime";
import { navigate } from "@ship/deck/router";

//#region ../../packages/sail/runtime/src/actions/variant-live.ts
/**
* Observe `variant` and flip role/aria-live for connecting placeholders.
* Static defaults on the template (`role=note`, `aria-live=off`) keep the
* host inert before hydrate; this action only upgrades `connecting` and
* re-evaluates when the attribute changes.
*/
const variantLive = (node) => {
	const el = node;
	const sync = () => {
		const connecting = el.getAttribute("variant") === "connecting";
		el.setAttribute("role", connecting ? "status" : "note");
		el.setAttribute("aria-live", connecting ? "polite" : "off");
	};
	sync();
	const obs = new MutationObserver(sync);
	obs.observe(el, {
		attributes: true,
		attributeFilter: ["variant"]
	});
	return { destroy: () => obs.disconnect() };
};

//#endregion
//#region ../../packages/sail/runtime/src/actions/described-control.ts
let _seq = 0;
/**
* `use:describedControl` — form control ↔ hint/error association for the L1
* `<field>` host anatomy (structure under host, not class names):
*   children[0] label, [1] control, [2] hint, [3] error-gate → [role=alert]
*
* The hint↔error visibility swap is pure CSS; this action wires accessibility:
* give the hint and error regions stable ids, then point the first slotted
* control's aria-describedby at the active region (hint when valid, error when
* [invalid]) and set aria-invalid to match. Re-wires when [invalid] toggles or
* the control changes. Written once — no per-instance controller, compiles away.
*/
const describedControl = (node) => {
	const el = node;
	const n = ++_seq;
	const control = el.children[1] ?? null;
	const hint = el.children[2] ?? null;
	const error = el.querySelector("[role=\"alert\"]");
	const hintId = `sail-field-hint-${n}`;
	const errorId = `sail-field-error-${n}`;
	if (hint) hint.id = hintId;
	if (error) error.id = errorId;
	const CONTROL_SEL = "input, textarea, select, [contenteditable], [role=\"textbox\"], [role=\"combobox\"], [role=\"spinbutton\"], [data-sail-host=\"switch\"]";
	const wire = () => {
		const first = control?.querySelector(CONTROL_SEL);
		if (!first) return;
		const inv = el.hasAttribute("invalid");
		const targetId = inv ? errorId : hintId;
		const next = [...(first.getAttribute("aria-describedby") || "").split(/\s+/).filter((id) => id && id !== hintId && id !== errorId), targetId].join(" ").trim();
		if (next) first.setAttribute("aria-describedby", next);
		first.setAttribute("aria-invalid", inv ? "true" : "false");
	};
	wire();
	const obs = new MutationObserver(wire);
	obs.observe(el, {
		attributes: true,
		attributeFilter: ["invalid"]
	});
	if (control) obs.observe(control, {
		childList: true,
		subtree: true
	});
	return { destroy: () => obs.disconnect() };
};

//#endregion
//#region ../../packages/sail/runtime/src/actions/dismiss-self.ts
const dismissed = /* @__PURE__ */ new WeakSet();
/** True after a successful dismissSelf (or if the node is already detached). */
function isDismissed(el) {
	return dismissed.has(el) || !el.isConnected;
}
/**
* Emit a composed dismiss event then remove the element. Idempotent: a second
* call on the same node is a no-op.
*/
function dismissSelf(el, eventName = "sail-toast-dismiss") {
	if (dismissed.has(el)) return false;
	dismissed.add(el);
	el.dispatchEvent(new CustomEvent(eventName, {
		bubbles: true,
		composed: true
	}));
	el.remove();
	return true;
}

//#endregion
//#region ../../packages/sail/runtime/src/actions/auto-dismiss.ts
function readDuration(el) {
	const raw = el.getAttribute("duration");
	if (raw == null || raw === "") return 5e3;
	const n = Number(raw);
	return Number.isFinite(n) ? n : 5e3;
}
/**
* Auto-dismiss after `duration` ms (attr). 0 / negative = sticky (no timer).
* Pause while hovered or focused so a reader is never cut off mid-read.
*/
const autoDismiss = (node) => {
	const el = node;
	const duration = readDuration(el);
	let timer;
	const fire = () => {
		if (isDismissed(el)) return;
		if (timer) {
			clearTimeout(timer);
			timer = void 0;
		}
		dismissSelf(el);
	};
	const start = () => {
		if (isDismissed(el) || duration <= 0) return;
		if (timer) clearTimeout(timer);
		timer = setTimeout(fire, duration);
	};
	const pause = () => {
		if (timer) clearTimeout(timer);
		timer = void 0;
	};
	el.addEventListener("pointerenter", pause);
	el.addEventListener("pointerleave", start);
	el.addEventListener("focusin", pause);
	el.addEventListener("focusout", start);
	start();
	return { destroy: () => {
		pause();
		el.removeEventListener("pointerenter", pause);
		el.removeEventListener("pointerleave", start);
		el.removeEventListener("focusin", pause);
		el.removeEventListener("focusout", start);
	} };
};

//#endregion
//#region ../../packages/sail/runtime/src/actions/dismiss-control.ts
const CLOSE_SEL = "[data-toast-close], [data-dismiss], button[aria-label=\"Dismiss\"]";
function isToastHost(el) {
	return el.hasAttribute("duration") || el.getAttribute("data-sail-host") === "toast";
}
/**
* Click on a descendant close control dismisses the host: composed dismiss
* event, then `el.remove()`. Toast → `sail-toast-dismiss` (no dismissible
* gate). Alert / other → `sail-dismiss`, only while `[dismissible]`.
*/
const dismissControl = (node) => {
	const el = node;
	const onClick = (e) => {
		if (isDismissed(el)) return;
		const close = e.target?.closest(CLOSE_SEL);
		if (!close || !el.contains(close)) return;
		const toast = isToastHost(el);
		if (!toast && !el.hasAttribute("dismissible")) return;
		dismissSelf(el, toast ? "sail-toast-dismiss" : "sail-dismiss");
	};
	el.addEventListener("click", onClick);
	return { destroy: () => {
		el.removeEventListener("click", onClick);
	} };
};

//#endregion
//#region ../../packages/sail/runtime/src/actions/tabs.ts
const host = (name) => `[data-sail-host="${name}"]`;
/**
* Internal: L1 `<tabs>` closed behavior (not a public `use:`).
* Stdlib widget attaches this; authors use `<tabs>`, not `use:tabs`.
*
* Wires the ARIA tablist contract (role, aria-selected, aria-controls /
* aria-labelledby), roving tabindex, keyboard nav (Arrow per orientation,
* Home/End, Enter/Space), panel show/hide, and emits `sail-tab-change`. The
* active tab is the element's `active` attribute (the name of the active tab);
* unset defaults to the first tab.
*
* Orientation is structural; stamped onto tab-list / tab-panel so child axes
* resolve. Size/variant paint is authored on children (or host CVA).
*/
const tabs = (node) => {
	const root = node;
	const keyOf = (el, i) => el.getAttribute("name") || String(i);
	const tabsOf = () => Array.from(root.querySelectorAll(host("tab")));
	const panelsOf = () => Array.from(root.querySelectorAll(host("tab-panel")));
	const listOf = () => root.querySelector(host("tab-list"));
	const activeName = () => root.getAttribute("active") ?? "";
	const orientationOf = () => root.getAttribute("orientation") === "vertical" ? "vertical" : "horizontal";
	const setAttr = (el, name, value) => {
		if (el.getAttribute(name) !== value) el.setAttribute(name, value);
	};
	const stampOrientation = () => {
		const orientation = orientationOf();
		const list = listOf();
		if (list) setAttr(list, "orientation", orientation);
		for (const p of panelsOf()) setAttr(p, "orientation", orientation);
	};
	const sync = () => {
		mo.disconnect();
		try {
			stampOrientation();
			const tabs = tabsOf();
			let active = activeName();
			if (!active && tabs.length) {
				active = keyOf(tabs[0], 0);
				setAttr(root, "active", active);
			}
			for (const [i, t] of tabs.entries()) {
				const name = keyOf(t, i);
				const on = name === active;
				setAttr(t, "role", "tab");
				setAttr(t, "aria-selected", on ? "true" : "false");
				setAttr(t, "tabindex", on ? "0" : "-1");
				t.toggleAttribute("active", on);
				if (name) {
					if (!t.id) t.id = `sail-tab-${name}`;
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
					if (!p.id) p.id = `sail-tabpanel-${name}`;
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
		if (!name) return;
		if (name !== activeName()) {
			root.setAttribute("active", name);
			sync();
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
		if (t && !t.hasAttribute("disabled")) activate(keyOf(t, tabsOf().indexOf(t)));
	};
	const onKeydown = (e) => {
		const t = tabFrom(e);
		if (!t) return;
		const enabled = tabsOf().filter((x) => !x.hasAttribute("disabled"));
		const i = enabled.indexOf(t);
		if (i < 0) return;
		const [next, prev] = orientationOf() === "vertical" ? ["ArrowDown", "ArrowUp"] : ["ArrowRight", "ArrowLeft"];
		let j = -1;
		if (e.key === next) j = (i + 1) % enabled.length;
		else if (e.key === prev) j = (i - 1 + enabled.length) % enabled.length;
		else if (e.key === "Home") j = 0;
		else if (e.key === "End") j = enabled.length - 1;
		else if (e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			if (!t.hasAttribute("disabled")) activate(keyOf(t, tabsOf().indexOf(t)));
			return;
		}
		if (j >= 0) {
			e.preventDefault();
			activate(keyOf(enabled[j], tabsOf().indexOf(enabled[j])));
		}
	};
	root.addEventListener("click", onClick);
	root.addEventListener("keydown", onKeydown);
	const mo = new MutationObserver(() => sync());
	sync();
	return { destroy: () => {
		root.removeEventListener("click", onClick);
		root.removeEventListener("keydown", onKeydown);
		mo.disconnect();
	} };
};

//#endregion
//#region src/edit/explorer.ts
/** Parse SailInspect/Tree JSON into typed nodes. */
function parseTreeNodes(raw) {
	if (!Array.isArray(raw)) return [];
	return raw.map((r) => {
		const o = r;
		const span = o.span ?? {};
		const cf = o.control_flow;
		const attrsRaw = o.attrs;
		let attrs;
		if (attrsRaw && typeof attrsRaw === "object") {
			attrs = {};
			for (const [k, v] of Object.entries(attrsRaw)) attrs[k] = String(v ?? "");
		}
		return {
			id: String(o.id ?? ""),
			file: String(o.file ?? ""),
			span: {
				start: Number(span.start ?? 0),
				end: Number(span.end ?? 0),
				line: Number(span.line ?? 0),
				col: Number(span.col ?? 0),
				end_line: Number(span.end_line ?? span.line ?? 0),
				end_col: Number(span.end_col ?? span.col ?? 0)
			},
			tag: String(o.tag ?? ""),
			layer: String(o.layer ?? "unknown"),
			attrs,
			control_flow: cf ? {
				kind: String(cf.kind ?? ""),
				expr: cf.expr != null ? String(cf.expr) : void 0
			} : void 0,
			children: Array.isArray(o.children) ? o.children.map(String) : []
		};
	});
}
function pathToFileEntry(path, checks = 0) {
	const name = path.split("/").pop() ?? path;
	let group = "other";
	if (path.includes("/routes/") || path.startsWith("routes/")) group = "routes";
	else if (path.includes("/src/") || path.startsWith("src/")) group = "src";
	else if (path.includes("/packs/") || path.startsWith("packs/")) group = "packs";
	return {
		path,
		name,
		group,
		checks
	};
}

//#endregion
//#region src/edit/region-hit.ts
function parseRegions(raw) {
	if (!Array.isArray(raw)) return [];
	const out = [];
	for (const r of raw) {
		if (!r || typeof r !== "object") continue;
		const o = r;
		const id = String(o.nodeId ?? o.id ?? o.node_id ?? "");
		if (!id) continue;
		out.push({
			nodeId: id,
			x: Number(o.x ?? 0),
			y: Number(o.y ?? 0),
			w: Number(o.w ?? o.width ?? 0),
			h: Number(o.h ?? o.height ?? 0),
			tag: o.tag != null ? String(o.tag) : void 0,
			label: o.label != null ? String(o.label) : void 0
		});
	}
	return out;
}

//#endregion
//#region src/edit/views.ts
function parseBindings(raw) {
	if (!Array.isArray(raw)) return [];
	return raw.map((r) => {
		const o = r;
		const span = o.span ?? {};
		const edge = (e) => {
			const x = e;
			const s = x.span ?? {};
			return {
				id: String(x.id ?? ""),
				file: String(x.file ?? ""),
				span: {
					start: Number(s.start ?? 0),
					end: Number(s.end ?? 0),
					line: Number(s.line ?? 0),
					col: Number(s.col ?? 0),
					end_line: Number(s.end_line ?? 0),
					end_col: Number(s.end_col ?? 0)
				},
				kind: String(x.kind ?? ""),
				expr: x.expr != null ? String(x.expr) : void 0
			};
		};
		return {
			id: String(o.id ?? ""),
			file: String(o.file ?? ""),
			span: {
				start: Number(span.start ?? 0),
				end: Number(span.end ?? 0),
				line: Number(span.line ?? 0),
				col: Number(span.col ?? 0),
				end_line: Number(span.end_line ?? 0),
				end_col: Number(span.end_col ?? 0)
			},
			name: String(o.name ?? ""),
			kind: String(o.kind ?? ""),
			source: String(o.source ?? ""),
			ssr_tier: String(o.ssr_tier ?? "I"),
			readers: Array.isArray(o.readers) ? o.readers.map(edge) : [],
			writers: Array.isArray(o.writers) ? o.writers.map(edge) : []
		};
	});
}
function parseLayoutBoxes(raw) {
	return (Array.isArray(raw) ? raw : raw && typeof raw === "object" && Array.isArray(raw.boxes) ? raw.boxes : []).map((r) => {
		const o = r;
		const span = o.span ?? {};
		const vp = o.viewport ?? {};
		return {
			id: String(o.id ?? ""),
			file: String(o.file ?? ""),
			span: {
				start: Number(span.start ?? 0),
				end: Number(span.end ?? 0),
				line: Number(span.line ?? 0),
				col: Number(span.col ?? 0),
				end_line: Number(span.end_line ?? 0),
				end_col: Number(span.end_col ?? 0)
			},
			viewport: {
				width: Number(vp.width ?? 0),
				height: Number(vp.height ?? 0)
			},
			x: Number(o.x ?? 0),
			y: Number(o.y ?? 0),
			w: Number(o.w ?? 0),
			h: Number(o.h ?? 0),
			collapsed: Boolean(o.collapsed),
			overflow: Boolean(o.overflow),
			tag: String(o.tag ?? ""),
			unknown_join: Boolean(o.unknown_join)
		};
	});
}
function parseDataKeys(raw) {
	return (Array.isArray(raw) ? raw : raw && typeof raw === "object" && Array.isArray(raw.keys) ? raw.keys : []).map((r) => {
		const o = r;
		const span = o.span ?? {};
		return {
			id: String(o.id ?? ""),
			file: String(o.file ?? ""),
			span: {
				start: Number(span.start ?? 0),
				end: Number(span.end ?? 0),
				line: Number(span.line ?? 0),
				col: Number(span.col ?? 0),
				end_line: Number(span.end_line ?? 0),
				end_col: Number(span.end_col ?? 0)
			},
			kind: String(o.kind ?? "query"),
			key: String(o.key ?? ""),
			ssr_fetchable: Boolean(o.ssr_fetchable)
		};
	});
}
function parseThreads(raw, scopePath) {
	if (!Array.isArray(raw)) return [];
	return raw.map((t) => {
		const o = t;
		const id = String(o.id ?? "");
		const route = o.route != null ? String(o.route) : "";
		const nodeId = o.nodeId != null ? String(o.nodeId) : o.node_id != null ? String(o.node_id) : null;
		return {
			id,
			title: String(o.title ?? id),
			status: String(o.status ?? "open"),
			nodeId,
			route
		};
	}).filter((t) => {
		if (!scopePath) return true;
		const r = t.route ?? "";
		return !r || r.includes(scopePath) || scopePath.includes(r);
	}).map(({ id, title, status, nodeId }) => ({
		id,
		title,
		status,
		nodeId
	}));
}
function parseCheckDiagnostics(raw) {
	if (!Array.isArray(raw)) return [];
	return raw.map((d) => {
		const o = d;
		const col = o.column ?? o.col ?? o.startColumn ?? o.start_col;
		const endCol = o.endColumn ?? o.end_col ?? o.endCol;
		const endLine = o.endLine ?? o.end_line;
		return {
			line: Number(o.line ?? o.startLine ?? 1),
			column: col != null ? Number(col) : void 0,
			endLine: endLine != null ? Number(endLine) : void 0,
			endColumn: endCol != null ? Number(endCol) : void 0,
			severity: String(o.severity ?? "error"),
			message: String(o.message ?? ""),
			rule: o.rule != null ? String(o.rule) : void 0
		};
	});
}

//#endregion
//#region ../../packages/sail/studio-shell/src/selection.ts
let current = {
	nodeId: null,
	selector: null,
	source: "clear"
};
function getSelection() {
	return current;
}

//#endregion
//#region ../../packages/sail/studio-shell/src/model.ts
const CORE_ACTIVITIES = [
	"edit",
	"components",
	"themes",
	"tasks",
	"config"
];
const CORE_SET = new Set(CORE_ACTIVITIES);
const hasDom = () => typeof location !== "undefined" && typeof history !== "undefined";
function chromeFromHash() {
	if (!hasDom()) return {};
	const m = /^#\/([^/]+)(?:\/(.+))?$/.exec(location.hash);
	if (!m) return {};
	return {
		activity: decodeURIComponent(m[1]),
		tabId: m[2] ? decodeURIComponent(m[2]) : void 0
	};
}
function writeChromeHash() {
	if (!hasDom()) return;
	const parts = [state$3.activity];
	if (state$3.activeTabId && state$3.activeTabId !== "shell") parts.push(state$3.activeTabId);
	const hash = `#/${parts.map(encodeURIComponent).join("/")}`;
	if (location.hash !== hash) history.replaceState(null, "", hash);
}
const initialChrome = chromeFromHash();
let state$3 = {
	activity: initialChrome.activity && CORE_SET.has(initialChrome.activity) ? initialChrome.activity : "edit",
	tabs: [{
		id: "shell",
		label: "shell",
		dirty: false
	}],
	activeTabId: initialChrome.tabId ?? "shell",
	viewport: 1440,
	themeId: "ship",
	modeId: "dark",
	modelRev: 0,
	opsHealth: "ok",
	watchHealth: "ok",
	devUrl: "sail dev",
	fabVisible: true,
	autoFix: false,
	selection: getSelection(),
	checksBadge: 0,
	tasksBadge: 0,
	checksPanelOpen: false,
	regionNodes: [],
	batchCount: 0,
	sailPath: null,
	moduleActivities: [],
	moduleRoute: null,
	moduleUnsignedBanner: false
};
const listeners$2 = /* @__PURE__ */ new Set();
function emit$2() {
	for (const fn of listeners$2) fn(state$3);
}
function getShellState() {
	return state$3;
}
function openTab(tab) {
	const tabs = state$3.tabs.some((t) => t.id === tab.id) ? state$3.tabs.map((t) => t.id === tab.id ? {
		...t,
		...tab
	} : t) : [...state$3.tabs, tab];
	state$3 = {
		...state$3,
		tabs,
		activeTabId: tab.id
	};
	writeChromeHash();
	emit$2();
}
function setModelRev(rev) {
	state$3 = {
		...state$3,
		modelRev: rev
	};
	emit$2();
}
function setSailPath(path) {
	state$3 = {
		...state$3,
		sailPath: path
	};
	emit$2();
}

//#endregion
//#region ../../packages/sail/studio-shell/src/host-meta.ts
/**
* Host-stamped meta tags (deck / framework inject).
* Apps read project + app-route through this module — never raw document.querySelector.
*/
function readMeta(name) {
	if (typeof document === "undefined") return "";
	try {
		return document.querySelector(`meta[name="${name}"]`)?.getAttribute("content")?.trim() || "";
	} catch {
		return "";
	}
}
/** Workspace-relative project root from host stamp (`sail-project`). */
function hostSailProject() {
	return readMeta("sail-project").replace(/\/$/, "");
}

//#endregion
//#region src/edit/store.ts
/**
* Edit surface store — explorer + documents + view switcher + selection sync.
* Consumes SailInspect / SailRender / SailAuthoring via injected OpsClient (T-I2).
*/
let client$1 = null;
let state$2 = emptyState$2();
const listeners$1 = /* @__PURE__ */ new Set();
function emptyState$2() {
	return {
		projectRoot: "",
		filter: "",
		files: [],
		activePath: null,
		view: "source",
		docs: /* @__PURE__ */ new Map(),
		modelRev: 0,
		selectionNodeId: null,
		sourceScrollLine: null,
		sourceScrollCol: null,
		sourceScrollNonce: 0,
		hoverTip: {
			nodeId: "",
			label: "",
			visible: false
		},
		threads: [],
		autoFixMode: "off",
		error: null,
		loading: false
	};
}
function emit$1() {
	for (const fn of listeners$1) fn(state$2);
}
function getEditState() {
	return state$2;
}
function subscribeEdit(fn) {
	listeners$1.add(fn);
	return () => {
		listeners$1.delete(fn);
	};
}
function getOpsClient() {
	return client$1;
}
/** True for Sail sources that go through SailInspect. */
function isSailSourcePath(path) {
	return path.replace(/\\/g, "/").toLowerCase().endsWith(".sail");
}
function installOpenedDoc(path, doc, threads, gen) {
	if (gen !== previewGen) return null;
	const docs = new Map(state$2.docs);
	docs.set(path, doc);
	const rev = state$2.modelRev + 1;
	state$2 = {
		...state$2,
		docs,
		activePath: path,
		view: doc.view,
		modelRev: rev,
		loading: false,
		error: null,
		threads
	};
	setModelRev(rev);
	if (!state$2.files.some((f) => f.path === path)) state$2 = {
		...state$2,
		files: [...state$2.files, pathToFileEntry(path)]
	};
	emit$1();
	return state$2.docs.get(path) ?? doc;
}
/**
* Open a project path: .sail → SailInspect model (+ optional preview);
* other text files → source-only (no inspect tree). Always selects the path.
*/
async function openDocument(path, opts) {
	if (!client$1) {
		state$2 = {
			...state$2,
			error: "Studio is not connected yet. Try reloading."
		};
		emit$1();
		return null;
	}
	const gen = ++previewGen;
	state$2 = {
		...state$2,
		loading: true,
		error: null,
		activePath: path
	};
	emit$1();
	setSailPath(path);
	const openIds = [...getShellState().tabs.map((t) => t.id), path];
	const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
	const base = parts[parts.length - 1] ?? path;
	const label = openIds.filter((id) => {
		const p = id.replace(/\\/g, "/");
		return (p.split("/").pop() || p) === base;
	}).length > 1 && parts.length >= 2 ? `${parts[parts.length - 2]}/${parts[parts.length - 1]}` : base;
	openTab({
		id: path,
		label,
		dirty: false
	});
	try {
		if (!isSailSourcePath(path)) {
			let sourceText = opts?.sourceText ?? "";
			if (!sourceText && client$1.readSource) try {
				sourceText = await client$1.readSource(path);
			} catch {}
			if (!sourceText) sourceText = state$2.docs.get(path)?.source.text ?? "";
			if (gen !== previewGen) return null;
			const prior = state$2.docs.get(path);
			const docView = prior?.view ?? "source";
			const hasText = sourceText.length > 0;
			const source = {
				text: sourceText,
				dirty: false,
				status: hasText ? "Text file" : "Non-Sail file — source not available here (open .sail or use CLI ops)",
				cursorSpan: "—",
				checks: [],
				checksLabel: ""
			};
			return installOpenedDoc(path, {
				path,
				label,
				modelVersion: prior?.modelVersion ?? "text",
				tree: {
					roots: [],
					nodes: []
				},
				layout: { boxes: [] },
				state: { bindings: [] },
				data: { keys: [] },
				diagnostics: [],
				theme: prior?.theme,
				source,
				preview: emptyPreview(),
				view: docView,
				signalValues: prior?.signalValues ?? {}
			}, [], gen);
		}
		const model = await client$1.sailInspectModel({
			path,
			viewport: opts?.viewport
		});
		if (gen !== previewGen) return null;
		const treeRaw = model.tree;
		const nodes = parseTreeNodes(treeRaw?.nodes ?? []);
		const roots = treeRaw?.roots ?? [];
		const bindings = parseBindings(model.state?.bindings ?? []);
		const boxes = parseLayoutBoxes(model.layout);
		const keys = parseDataKeys(model.data);
		let sourceText = opts?.sourceText ?? "";
		if (!sourceText && typeof model.source === "string" && model.source.length) sourceText = model.source;
		if (!sourceText && client$1.readSource) try {
			sourceText = await client$1.readSource(path);
		} catch {}
		if (!sourceText) sourceText = state$2.docs.get(path)?.source.text ?? "";
		if (gen !== previewGen) return null;
		let checks = parseCheckDiagnostics([]);
		let checksLabel = "";
		if (client$1.sailAuthoringCheck) try {
			checks = parseCheckDiagnostics((await client$1.sailAuthoringCheck({ path })).diagnostics ?? []);
			const n = checks.filter((c) => c.severity === "error").length || checks.length;
			checksLabel = n > 0 ? `${n} check${n === 1 ? "" : "s"}` : "";
		} catch {}
		const source = {
			text: sourceText,
			dirty: false,
			status: sourceText ? "Saved" : "No source loaded",
			cursorSpan: "—",
			checks,
			checksLabel
		};
		const preview = emptyPreview(opts?.loadPreview !== false ? { loading: true } : void 0);
		let threads = [];
		if (client$1.sailTaskList) try {
			threads = parseThreads((await client$1.sailTaskList({ project: path.split("/").slice(0, 2).join("/") || state$2.projectRoot || void 0 })).tasks ?? [], path);
		} catch {
			threads = [];
		}
		if (gen !== previewGen) return null;
		const prior = state$2.docs.get(path);
		const docView = prior?.view ?? state$2.view;
		const doc = {
			path,
			label,
			modelVersion: model.modelVersion,
			tree: {
				roots,
				nodes
			},
			layout: { boxes },
			state: { bindings },
			data: { keys },
			diagnostics: model.diagnostics ?? [],
			theme: model.theme,
			source,
			preview,
			view: docView,
			signalValues: prior?.signalValues ?? {}
		};
		if (!installOpenedDoc(path, doc, threads, gen)) return null;
		if (gen !== previewGen) return null;
		if (opts?.loadPreview !== false) {
			await refreshPreview({
				theme: opts?.theme,
				mode: opts?.mode,
				viewport: opts?.viewport
			});
			if (state$2.activePath !== path) return null;
		}
		return state$2.docs.get(path) ?? doc;
	} catch (e) {
		if (gen !== previewGen) return null;
		state$2 = {
			...state$2,
			loading: false,
			error: userError(e, "Could not open that file — try again.")
		};
		emit$1();
		return null;
	}
}
/** Default cap so SailRender can never leave the surface on "Rendering…" forever. */
const PREVIEW_TIMEOUT_MS = 25e3;
let previewTimeoutMs = PREVIEW_TIMEOUT_MS;
/**
* Monotonic generation for preview loads. Each refreshPreview/openDocument bumps it;
* only the settle whose gen is still current may write preview state (drops stale races).
*/
let previewGen = 0;
function emptyPreview(partial) {
	return {
		imagePng: null,
		regions: [],
		dynamics: [],
		provenance: "Audit snapshot",
		loading: false,
		error: null,
		...partial
	};
}
/**
* Race a promise against a wall-clock timeout. Clears the timer in `finally`.
* Does **not** cancel the underlying ops work (no AbortSignal on SailRender yet);
* callers must use `previewGen` so late settles are ignored after supersede/retry.
*/
async function withTimeout(promise, ms, label) {
	let timer;
	try {
		return await Promise.race([promise, new Promise((_, reject) => {
			timer = setTimeout(() => reject(/* @__PURE__ */ new Error(`${label} timed out after ${ms}ms`)), ms);
		})]);
	} finally {
		if (timer !== void 0) clearTimeout(timer);
	}
}
async function loadPreview(path, opts) {
	if (!client$1) return emptyPreview({ error: "Studio is not connected yet." });
	const timeoutMs = opts?.timeoutMs ?? previewTimeoutMs;
	try {
		const vp = opts?.viewport ?? {
			width: 1440,
			height: 900
		};
		const ret = await withTimeout(client$1.sailRenderScreenshot({
			path,
			theme: opts?.theme ?? "ship",
			mode: opts?.mode ?? "dark",
			viewport: `${vp.width}x${vp.height}`
		}), timeoutMs, "Preview render");
		return emptyPreview({
			imagePng: ret.imagePng,
			regions: parseRegions(ret.regions ?? []),
			dynamics: (ret.dynamics ?? []).map((d) => {
				const o = d;
				return {
					nodeId: String(o.nodeId ?? o.id ?? ""),
					reason: o.reason != null ? String(o.reason) : void 0,
					tag: o.tag != null ? String(o.tag) : void 0
				};
			}),
			loading: false,
			error: null
		});
	} catch (e) {
		return emptyPreview({
			loading: false,
			error: userError(e, e instanceof Error && /timed out/i.test(e.message) ? "Preview timed out — try again, or open a smaller file." : "Could not render the audit snapshot — try again.")
		});
	}
}
/**
* Load/refresh audit screenshot for the active doc.
* Returns `{ applied: true }` only when this generation still owns the settle
* (stale concurrent refresh/open returns `{ applied: false }`).
*/
async function refreshPreview(opts) {
	const path = state$2.activePath;
	if (!path) return { applied: false };
	const doc = state$2.docs.get(path);
	if (!doc) return { applied: false };
	const gen = ++previewGen;
	const docsLoading = new Map(state$2.docs);
	docsLoading.set(path, {
		...doc,
		preview: {
			...doc.preview,
			loading: true,
			error: null
		}
	});
	state$2 = {
		...state$2,
		docs: docsLoading
	};
	emit$1();
	const preview = await loadPreview(path, opts);
	if (gen !== previewGen) {
		if (state$2.activePath !== path) {
			const stale = state$2.docs.get(path);
			if (stale?.preview.loading) {
				const docs = new Map(state$2.docs);
				docs.set(path, {
					...stale,
					preview: {
						...stale.preview,
						loading: false
					}
				});
				state$2 = {
					...state$2,
					docs
				};
				emit$1();
			}
		}
		return { applied: false };
	}
	const latest = state$2.docs.get(path);
	if (!latest || state$2.activePath !== path) return { applied: false };
	const docs = new Map(state$2.docs);
	docs.set(path, {
		...latest,
		preview
	});
	state$2 = {
		...state$2,
		docs
	};
	emit$1();
	return { applied: true };
}
/** Scroll source editor to a file span (Checks → Edit deep link). */
function requestSourceScroll(line, col = 1) {
	state$2 = {
		...state$2,
		sourceScrollLine: line,
		sourceScrollCol: col,
		sourceScrollNonce: state$2.sourceScrollNonce + 1
	};
	emit$1();
}

//#endregion
//#region src/tasks/parse.ts
function parseDrift(raw) {
	if (!raw || typeof raw !== "object") return null;
	const o = raw;
	return {
		status: String(o.status ?? "none"),
		span: o.span ?? null,
		liveNodeId: o.liveNodeId != null ? String(o.liveNodeId) : null,
		liveHash: o.liveHash != null ? String(o.liveHash) : null,
		spanDisplay: o.spanDisplay != null ? String(o.spanDisplay) : null
	};
}
function parseAnchor(raw) {
	if (!raw || typeof raw !== "object") return null;
	const o = raw;
	const nodeId = String(o.nodeId ?? o.node_id ?? "");
	if (!nodeId) return null;
	return {
		nodeId,
		elementPath: String(o.elementPath ?? o.element_path ?? ""),
		contentHash: String(o.contentHash ?? o.content_hash ?? ""),
		file: o.file != null ? String(o.file) : void 0,
		span: o.span
	};
}
function parseActor(raw) {
	if (!raw || typeof raw !== "object") return null;
	const o = raw;
	return {
		kind: String(o.kind ?? "human"),
		name: String(o.name ?? (o.kind === "agent" ? "agent" : "user"))
	};
}
function parseListItem(raw) {
	if (!raw || typeof raw !== "object") return null;
	const o = raw;
	const id = String(o.id ?? "");
	if (!id) return null;
	return {
		id,
		title: String(o.title ?? id),
		status: String(o.status ?? "todo"),
		updated_at: String(o.updated_at ?? ""),
		labels: Array.isArray(o.labels) ? o.labels.map(String) : [],
		created_by: parseActor(o.created_by) ?? void 0,
		assignee: parseActor(o.assignee),
		last_actor: parseActor(o.last_actor) ?? parseActor(o.lastActor),
		route: o.route != null ? String(o.route) : null,
		anchor: parseAnchor(o.anchor),
		artifact_count: typeof o.artifact_count === "number" ? o.artifact_count : void 0,
		drift: parseDrift(o.drift)
	};
}
function parseThread(raw) {
	if (!raw || typeof raw !== "object") return null;
	const o = raw;
	const id = String(o.id ?? "");
	if (!id) return null;
	const events = Array.isArray(o.events) ? o.events.map((e) => {
		const ev = e;
		return {
			id: String(ev.id ?? ""),
			at: String(ev.at ?? ""),
			type: String(ev.type ?? ""),
			actor: ev.actor ?? {
				kind: "human",
				name: "user"
			},
			body: ev.body != null ? String(ev.body) : null,
			meta: ev.meta ?? null
		};
	}) : [];
	const artifacts = Array.isArray(o.artifacts) ? o.artifacts.map((a) => {
		const ar = a;
		return {
			id: String(ar.id ?? ""),
			path: String(ar.path ?? ""),
			mime: String(ar.mime ?? ""),
			kind: String(ar.kind ?? "file"),
			label: ar.label != null ? String(ar.label) : null,
			region: ar.region ?? null,
			created_at: String(ar.created_at ?? "")
		};
	}) : [];
	return {
		id,
		title: String(o.title ?? id),
		status: String(o.status ?? "todo"),
		labels: Array.isArray(o.labels) ? o.labels.map(String) : [],
		created_at: String(o.created_at ?? ""),
		updated_at: String(o.updated_at ?? ""),
		created_by: o.created_by ?? {
			kind: "human",
			name: "user"
		},
		assignee: o.assignee ?? null,
		route: o.route != null ? String(o.route) : null,
		anchor: parseAnchor(o.anchor),
		artifacts,
		events,
		drift: parseDrift(o.drift)
	};
}

//#endregion
//#region src/tasks/store.ts
let client = null;
let state$1 = emptyState$1();
const listeners = /* @__PURE__ */ new Set();
function emptyState$1() {
	return {
		projectRoot: "",
		threads: [],
		selectedId: null,
		selected: null,
		openFilter: "open",
		actorFilter: "all",
		driftFilter: "all",
		replyDraft: "",
		loading: false,
		error: null,
		commitPath: null,
		lastBatchCount: 0
	};
}
function emit() {
	const snap = { ...state$1 };
	for (const fn of listeners) fn(snap);
}
function projectRootArg() {
	return state$1.projectRoot || void 0;
}
function projectArg() {
	return projectRootArg();
}
function humanActor() {
	return {
		kind: "human",
		name: "user"
	};
}
async function loadThreads() {
	if (!client) {
		state$1 = {
			...state$1,
			error: null,
			loading: false,
			threads: []
		};
		emit();
		return;
	}
	state$1 = {
		...state$1,
		loading: true,
		error: null
	};
	emit();
	try {
		const args = { project: projectArg() };
		if (state$1.actorFilter !== "all") args.actor = state$1.actorFilter;
		if (state$1.driftFilter !== "all") args.drift = state$1.driftFilter;
		if (state$1.openFilter === "open") args.open = true;
		const listFn = client.sailTaskThreadList ?? client.sailTaskList;
		if (!listFn) throw new Error("Could not load tasks — try again.");
		const threads = ((await listFn(args)).tasks ?? []).map(parseListItem).filter((t) => t != null);
		state$1 = {
			...state$1,
			threads,
			loading: false
		};
		if (state$1.selectedId && !threads.some((t) => t.id === state$1.selectedId)) state$1 = {
			...state$1,
			selectedId: null,
			selected: null
		};
		emit();
		if (state$1.selectedId) await selectThread(state$1.selectedId);
	} catch (e) {
		state$1 = {
			...state$1,
			loading: false,
			error: userError(e, "Could not load tasks — try again.")
		};
		emit();
	}
}
async function selectThread(id) {
	if (!id) {
		state$1 = {
			...state$1,
			selectedId: null,
			selected: null,
			replyDraft: "",
			commitPath: null
		};
		emit();
		return;
	}
	if (!client?.sailTaskGet && !client) {
		state$1 = {
			...state$1,
			selectedId: id,
			selected: null
		};
		emit();
		return;
	}
	state$1 = {
		...state$1,
		selectedId: id,
		loading: true,
		error: null
	};
	emit();
	try {
		const getFn = client.sailTaskGet;
		if (!getFn) throw new Error("Could not open that thread — try again.");
		const selected = parseThread((await getFn({
			project: projectArg(),
			id
		})).task);
		state$1 = {
			...state$1,
			selected,
			loading: false
		};
		emit();
	} catch (e) {
		state$1 = {
			...state$1,
			loading: false,
			error: userError(e, "Could not open that thread — try again.")
		};
		emit();
	}
}
async function createThread(opts) {
	if (!client) return null;
	const createFn = client.sailTaskThreadCreate ?? client.sailTaskCreate;
	if (!createFn) return null;
	try {
		const id = parseThread((await createFn({
			project: projectArg(),
			title: opts.title,
			body: opts.body,
			anchor: opts.anchor,
			actor: humanActor()
		})).task)?.id ?? null;
		await loadThreads();
		if (id) await selectThread(id);
		return id;
	} catch (e) {
		state$1 = {
			...state$1,
			error: userError(e, "Could not start that task — try again.")
		};
		emit();
		return null;
	}
}

//#endregion
//#region src/components/brief.ts
/** Build clipboard text for a library (or instance-mapped) component. */
function buildAgentBrief(input) {
	const { record } = input;
	const props = record.props.length > 0 ? record.props.map((p) => `${p.name}${p.required ? "!" : ""}`).join(", ") : "—";
	const issues = record.issueCount > 0 ? `${record.issueCount}${input.issuesSummary ? ` (${input.issuesSummary})` : ""}` : "0";
	const composition = input.composition && input.composition.length ? input.composition.join(", ") : "—";
	const intent = record.description?.trim() || "none";
	const pack = record.package && record.package !== "—" ? record.package : "—";
	const path = record.path?.trim() || "—";
	return [
		`Component: ${record.id} (${record.origin})`,
		`Path: ${path}`,
		`Pack: ${pack}`,
		`Props: ${props}`,
		`Issues: ${issues}`,
		`SSR: ${record.ssrTierSummary || "unknown"}`,
		`Composition: ${composition}`,
		`Intent: ${intent}`
	].join("\n");
}
/** Brief for New… scaffold (not yet a real component). */
function buildNewComponentBrief(opts) {
	return [
		`Component: ${opts.name.trim() || "unnamed"} (project)`,
		`Path: —`,
		`Pack: —`,
		`Props: —`,
		`Issues: 0`,
		`SSR: unknown`,
		`Composition: —`,
		`Intent: ${opts.intent.trim() || "none"}`,
		`Layer: ${opts.layer.trim() || "app"}`,
		`Status: draft brief (not scaffolded on disk)`
	].join("\n");
}

//#endregion
//#region src/components-session/components-activity.ts
/**
* Cross-route activity bag for Components (Library ↔ Inspect remounts).
* Backed by `persisted` (session storage) so remounts keep selection/filter.
*/
const empty = () => ({
	selectedId: "",
	filterQuery: ""
});
const bag = persisted("studio.components-activity", empty(), {
	storage: "session",
	debounceMs: 50
});
function readComponentsActivity() {
	return { ...bag() };
}
function writeComponentsActivity(patch) {
	const cur = bag();
	bag.set({
		selectedId: patch.selectedId !== void 0 ? patch.selectedId : cur.selectedId,
		filterQuery: patch.filterQuery !== void 0 ? patch.filterQuery : cur.filterQuery
	});
}

//#endregion
//#region src/checks/deeplink.ts
const DEFAULTS = {
	vscode: "vscode://file/{path}:{line}:{col}",
	cursor: "cursor://file/{path}:{line}:{col}",
	zed: "zed://file/{path}:{line}:{col}"
};
function defaultEditorSetting(scheme = "vscode") {
	const s = scheme.toLowerCase();
	return {
		scheme: s,
		template: DEFAULTS[s] ?? DEFAULTS.vscode
	};
}

//#endregion
//#region src/checks/store.ts
let state = emptyState();
function emptyState() {
	return {
		projectRoot: "",
		findings: [],
		selectedId: null,
		ruleFilter: null,
		fileFilter: null,
		autoFixMode: "off",
		editor: defaultEditorSetting("vscode"),
		layoutBothViewportsRan: false,
		errorCount: 0,
		warningCount: 0,
		statusNote: "All clear",
		loading: false,
		error: null,
		sourceByPath: /* @__PURE__ */ new Map(),
		autoFixAttributed: "project setting",
		reviewOpen: false,
		reviewItems: [],
		reviewNote: "",
		lastRunAt: null,
		lastCheckedPath: null,
		lastLayoutChecked: false,
		attributions: [],
		collapsedRules: []
	};
}

//#endregion
//#region src/shell/project.ts
/** Project root resolution + seed paths for Studio explorer. */
function projectPaths(proj) {
	const root = proj.replace(/\/$/, "");
	const paths = [`${root}/routes/page.sail`, `${root}/package.json`];
	if (root === "apps/studio" || root.endsWith("/apps/studio")) paths.push("apps/studio/test/fixtures/sample.sail", "apps/studio/test/fixtures/tier-s.sail", "apps/studio/test/fixtures/tier-i.sail", "apps/studio/test/fixtures/tier-c.sail", "apps/studio/src/edit-explorer/edit-explorer-vm.ts");
	return paths;
}
/**
* Ops target (workspace-relative).
* `?project=apps/…` → host sail-project meta → apps/studio dogfood.
* Meta is host-stamped; apps read via studio-shell, not document.querySelector.
*/
function resolveProjectRoot() {
	const meta = hostSailProject();
	if (meta) return meta;
	return "apps/studio";
}

//#endregion
//#region src/shell/shell-rail.ts
/** Multi-app mount base (e.g. `/_studio`) so rail links stay in-app. */
function appRouteBase() {
	if (typeof document === "undefined") return "";
	const raw = document.querySelector("meta[name=\"deck-app-route\"]")?.getAttribute("content")?.trim();
	if (!raw || raw === "/") return "";
	return raw.replace(/\/$/, "");
}
function hrefForActivity(id) {
	return `${appRouteBase()}${id === "edit" ? "/edit" : `/${id}`}`;
}

//#endregion
//#region src/components-session/components-nav.ts
/**
* Components route / mode helpers — href builders + path parse.
*/
function componentsHomeHref(mode) {
	const base = `${appRouteBase()}/components`;
	if (mode === "inspect") return `${base}/inspect`;
	return base;
}
function componentHref(id, mode) {
	if (mode === "inspect") return componentsHomeHref("inspect");
	if (!id) return componentsHomeHref("library");
	return `${appRouteBase()}/components/${encodeURIComponent(id)}`;
}
/** Component id from /components or /components/:id (app-base aware). */
function componentIdFromLocation(pathname = "") {
	const path = pathname;
	const base = appRouteBase();
	const rest = base && path.startsWith(base) ? path.slice(base.length) : path;
	if (rest === "/components/inspect" || rest.startsWith("/components/inspect/")) return "";
	const m = rest.match(/^\/components\/([^/]+)\/?$/);
	return m ? decodeURIComponent(m[1]) : "";
}
function modeFromLocation(search = "", pathname = "") {
	const path = pathname;
	const base = appRouteBase();
	const rest = base && path.startsWith(base) ? path.slice(base.length) : path;
	if (rest === "/components/inspect" || rest.startsWith("/components/inspect/")) return "inspect";
	const raw = search;
	try {
		if (new URLSearchParams(raw.startsWith("?") ? raw : `?${raw}`).get("mode") === "inspect") return "inspect";
	} catch {}
	return "library";
}
function themesComponentsHref(themeId = "studio") {
	return `${appRouteBase()}/themes/${encodeURIComponent(themeId)}/components`;
}
function editActivityHref() {
	return hrefForActivity("edit");
}
function tasksActivityHref() {
	return hrefForActivity("tasks");
}

//#endregion
//#region src/components-session/components-session.ts
/**
* Components page session — mode, selection, toast, open-edit, thread, brief.
* Domain modules register via onSync; no hand-rolled listener Sets as product stores.
*/
/**
* Programmatic clipboard write for session actions (copy brief).
* Prefer `use:clipboard` on click hosts; this path is for non-click handlers.
* Returns false when the Clipboard API is unavailable.
*/
async function writeClipboard(text) {
	try {
		const clip = globalThis.navigator?.clipboard;
		if (clip?.writeText) {
			await clip.writeText(text);
			return true;
		}
	} catch {}
	return false;
}
function createComponentsSession(opts) {
	const fixedMode = opts?.mode;
	const bag = readComponentsActivity();
	const mode = signal(fixedMode ?? "library");
	const selectedId = signal(bag.selectedId);
	const filterQuery = signal(bag.filterQuery);
	const detailTab = signal("overview");
	const isLibrary = signal((fixedMode ?? "library") === "library");
	const isInspect = signal((fixedMode ?? "library") === "inspect");
	const toastItems = signal([]);
	let toastSeq = 0;
	const showNewForm = signal(false);
	const newName = signal("");
	const newIntent = signal("");
	const newLayer = signal("app");
	const domainSyncs = /* @__PURE__ */ new Set();
	const setNote = (msg, tone = "muted") => {
		if (!msg) {
			toastItems.set([]);
			return;
		}
		toastSeq += 1;
		toastItems.set([{
			key: `toast-${toastSeq}`,
			message: msg,
			tone
		}]);
	};
	const onToastDismiss = () => {
		toastItems.set([]);
	};
	const notifyDomain = () => {
		for (const fn of domainSyncs) fn();
	};
	const onSync = (fn) => {
		domainSyncs.add(fn);
		return () => {
			domainSyncs.delete(fn);
		};
	};
	const applyMode = (m, opts) => {
		mode.set(m);
		isLibrary.set(m === "library");
		isInspect.set(m === "inspect");
		if (opts?.navigate !== false) {
			const id = selectedId();
			navigate(id ? componentHref(id, m) : componentsHomeHref(m));
		}
		notifyDomain();
	};
	const setSelectedId = (id, opts) => {
		selectedId.set(id);
		writeComponentsActivity({ selectedId: id });
		if (opts?.navigate !== false) {
			const m = mode();
			if (id) navigate(componentHref(id, m));
			else navigate(componentsHomeHref(m));
		}
		notifyDomain();
	};
	const syncFromRoute = () => {
		const m = fixedMode ?? modeFromLocation();
		const id = componentIdFromLocation();
		mode.set(m);
		isLibrary.set(m === "library");
		isInspect.set(m === "inspect");
		if (id && id !== "inspect") {
			selectedId.set(id);
			writeComponentsActivity({ selectedId: id });
		}
		notifyDomain();
	};
	const openInEdit = async (path, span) => {
		if (!path || path === "—" || path.includes("@ship/")) {
			setNote("No project path to open in Edit.", "warning");
			return false;
		}
		const shell = getShellState();
		try {
			if (!await openDocument(path, {
				loadPreview: false,
				theme: shell.themeId,
				mode: shell.modeId
			})) {
				setNote("Could not open in Edit — try again.", "destructive");
				return false;
			}
			if (span?.line && span.line > 0) requestSourceScroll(span.line, span.col ?? 1);
			navigate(editActivityHref());
			return true;
		} catch {
			setNote("Could not open in Edit — try again.", "destructive");
			return false;
		}
	};
	const startThread = async (record) => {
		if (!record) {
			setNote("Select a component first.", "warning");
			return null;
		}
		const id = await createThread({
			title: `Component: ${record.displayName || record.id}`,
			body: buildAgentBrief({ record }),
			anchor: {
				componentId: record.id,
				path: record.path,
				origin: record.origin,
				package: record.package
			}
		});
		if (!id) {
			setNote("Could not start a thread — tasks may be offline.", "warning");
			return null;
		}
		setNote("Thread started.", "success");
		navigate(tasksActivityHref());
		return id;
	};
	const copyAgentBrief = async (record) => {
		if (!record) {
			setNote("Select a component first.", "warning");
			return false;
		}
		const ok = await writeClipboard(buildAgentBrief({ record }));
		setNote(ok ? "Brief copied" : "Could not copy brief", ok ? "success" : "warning");
		return ok;
	};
	const onFilterInput = (e) => {
		const q = e.target.value ?? "";
		filterQuery.set(q);
		writeComponentsActivity({ filterQuery: q });
		notifyDomain();
	};
	const onModeLibrary = () => applyMode("library");
	const onModeInspect = () => applyMode("inspect");
	const onModeToggle = (e) => {
		const m = (e.target?.closest?.("[mode]"))?.dataset.mode;
		if (m === "library" || m === "inspect") applyMode(m);
	};
	const setDetailTab = (tab) => {
		detailTab.set(tab);
	};
	const onDetailTab = (e) => {
		const name = e.detail?.value ?? e.detail?.name ?? e.target?.getAttribute?.("value") ?? e.target?.getAttribute?.("name");
		if (name === "overview" || name === "contract" || name === "health" || name === "theme") detailTab.set(name);
	};
	const onNewOpen = () => {
		showNewForm.set(true);
		detailTab.set("overview");
		notifyDomain();
	};
	const onNewNameInput = (e) => {
		newName.set(e.target.value ?? "");
	};
	const onNewIntentInput = (e) => {
		newIntent.set(e.target.value ?? "");
	};
	const onNewLayerChange = (e) => {
		newLayer.set(e.target.value ?? "app");
	};
	const onCreateBrief = async () => {
		const ok = await writeClipboard(buildNewComponentBrief({
			name: newName(),
			intent: newIntent(),
			layer: newLayer()
		}));
		setNote(ok ? "Draft brief copied — scaffold is not written to disk yet." : "Could not copy brief", ok ? "success" : "warning");
	};
	const onStartThreadFromNew = async () => {
		const name = newName().trim() || "new-component";
		await startThread({
			id: name,
			displayName: name,
			origin: "project",
			package: "—",
			path: "",
			layer: newLayer() === "L2" ? "L2" : "app",
			description: newIntent(),
			props: [],
			slots: [],
			issueCount: 0,
			ssrTierSummary: "unknown",
			hasPreview: false,
			threadCount: 0,
			canOpenInEdit: false
		});
	};
	const goThemesComponents = () => {
		navigate(themesComponentsHref(getShellState().themeId || "studio"));
	};
	const activeEditPath = () => getEditState().activePath;
	syncFromRoute();
	mountAmbient(() => {
		syncFromRoute();
		return () => {};
	});
	return {
		mode,
		selectedId,
		filterQuery,
		detailTab,
		isLibrary,
		isInspect,
		toastItems,
		showNewForm,
		newName,
		newIntent,
		newLayer,
		setNote,
		onToastDismiss,
		onSync,
		notifyDomain,
		applyMode,
		setSelectedId,
		syncFromRoute,
		openInEdit,
		startThread,
		copyAgentBrief,
		onFilterInput,
		onModeLibrary,
		onModeInspect,
		onModeToggle,
		setDetailTab,
		onDetailTab,
		onNewOpen,
		onNewNameInput,
		onNewIntentInput,
		onNewLayerChange,
		onCreateBrief,
		onStartThreadFromNew,
		goThemesComponents,
		activeEditPath
	};
}

//#endregion
//#region components.gen.ts
const COMPONENTS_CATALOG = {
	"schemaVersion": "1.0.0",
	"modules": [
		{
			"kind": "javascript-module",
			"path": "sail-calendar.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailCalendar",
				"tagName": "sail-calendar",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "view",
						"type": { "text": "'month' | 'week' | 'day'" },
						"default": "month",
						"values": [
							"month",
							"week",
							"day"
						]
					},
					{
						"name": "events",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "focus-date",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "week-starts-on",
						"type": { "text": "number" },
						"default": "0"
					},
					{
						"name": "slot-minutes",
						"type": { "text": "number" },
						"default": "30"
					},
					{
						"name": "day-start-hour",
						"type": { "text": "number" },
						"default": "0"
					},
					{
						"name": "day-end-hour",
						"type": { "text": "number" },
						"default": "24"
					},
					{
						"name": "range-mode",
						"type": { "text": "'none' | 'date' | 'datetime'" },
						"default": "none",
						"values": [
							"none",
							"date",
							"datetime"
						]
					},
					{
						"name": "editable",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "label",
						"type": { "text": "string" },
						"default": ""
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "view",
						"type": { "text": "'month' | 'week' | 'day'" },
						"attribute": "view",
						"default": "month",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "events",
						"type": { "text": "string" },
						"attribute": "events",
						"default": ""
					},
					{
						"kind": "field",
						"name": "focusDate",
						"type": { "text": "string" },
						"attribute": "focus-date",
						"default": ""
					},
					{
						"kind": "field",
						"name": "weekStartsOn",
						"type": { "text": "number" },
						"attribute": "week-starts-on",
						"default": "0"
					},
					{
						"kind": "field",
						"name": "slotMinutes",
						"type": { "text": "number" },
						"attribute": "slot-minutes",
						"default": "30"
					},
					{
						"kind": "field",
						"name": "dayStartHour",
						"type": { "text": "number" },
						"attribute": "day-start-hour",
						"default": "0"
					},
					{
						"kind": "field",
						"name": "dayEndHour",
						"type": { "text": "number" },
						"attribute": "day-end-hour",
						"default": "24"
					},
					{
						"kind": "field",
						"name": "rangeMode",
						"type": { "text": "'none' | 'date' | 'datetime'" },
						"attribute": "range-mode",
						"default": "none",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "editable",
						"type": { "text": "boolean" },
						"attribute": "editable",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "label",
						"type": { "text": "string" },
						"attribute": "label",
						"default": ""
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["empty"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-calendar",
				"declaration": {
					"name": "SailCalendar",
					"module": "sail-calendar.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-citation.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailCitation",
				"tagName": "sail-citation",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [{
					"name": "source",
					"type": { "text": "string" },
					"default": ""
				}, {
					"name": "index",
					"type": { "text": "number" },
					"default": "0"
				}],
				"members": [{
					"kind": "field",
					"name": "source",
					"type": { "text": "string" },
					"attribute": "source",
					"default": ""
				}, {
					"kind": "field",
					"name": "index",
					"type": { "text": "number" },
					"attribute": "index",
					"default": "0"
				}],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-citation",
				"declaration": {
					"name": "SailCitation",
					"module": "sail-citation.sail"
				}
			}],
			"package": "@ship/sail-chat-elements"
		},
		{
			"kind": "javascript-module",
			"path": "sail-cmdk.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailCmdk",
				"tagName": "sail-cmdk",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "open",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "placeholder",
						"type": { "text": "string" },
						"default": "Type a command or search…"
					},
					{
						"name": "label",
						"type": { "text": "string" },
						"default": "Command palette"
					},
					{
						"name": "empty-label",
						"type": { "text": "string" },
						"default": "No results found."
					},
					{
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"default": "md",
						"values": [
							"md",
							"sm",
							"lg"
						]
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "open",
						"type": { "text": "boolean" },
						"attribute": "open",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "placeholder",
						"type": { "text": "string" },
						"attribute": "placeholder",
						"default": "Type a command or search…"
					},
					{
						"kind": "field",
						"name": "label",
						"type": { "text": "string" },
						"attribute": "label",
						"default": "Command palette"
					},
					{
						"kind": "field",
						"name": "emptyLabel",
						"type": { "text": "string" },
						"attribute": "empty-label",
						"default": "No results found."
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"attribute": "size",
						"default": "md",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-cmdk",
				"declaration": {
					"name": "SailCmdk",
					"module": "sail-cmdk.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-code-editor.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailCodeEditor",
				"tagName": "sail-code-editor",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "value",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "language",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "placeholder",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "readonly",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "line-numbers",
						"type": { "text": "boolean" },
						"default": "true"
					},
					{
						"name": "tone",
						"type": { "text": "'default' | 'info' | 'success' | 'warning' | 'destructive'" },
						"default": "default",
						"values": [
							"default",
							"info",
							"success",
							"warning",
							"destructive"
						]
					},
					{
						"name": "theme",
						"type": { "text": "'auto' | 'light' | 'dark'" },
						"default": "auto",
						"values": [
							"auto",
							"light",
							"dark"
						]
					},
					{
						"name": "wasm-url",
						"type": { "text": "string" },
						"default": "/sail_compiler_bg.wasm"
					},
					{
						"name": "ops-url",
						"type": { "text": "string" },
						"default": "",
						"description": "SailAuthoring ops base URL. Empty disables; \"/\" = same-origin."
					},
					{
						"name": "ops-path",
						"type": { "text": "string" },
						"default": "",
						"description": "Active file path for Hover/Complete/diagnostics RPCs."
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "value",
						"type": { "text": "string" },
						"attribute": "value",
						"default": ""
					},
					{
						"kind": "field",
						"name": "language",
						"type": { "text": "string" },
						"attribute": "language",
						"default": ""
					},
					{
						"kind": "field",
						"name": "placeholder",
						"type": { "text": "string" },
						"attribute": "placeholder",
						"default": ""
					},
					{
						"kind": "field",
						"name": "readonly",
						"type": { "text": "boolean" },
						"attribute": "readonly",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "lineNumbers",
						"type": { "text": "boolean" },
						"attribute": "line-numbers",
						"default": "true",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "tone",
						"type": { "text": "'default' | 'info' | 'success' | 'warning' | 'destructive'" },
						"attribute": "tone",
						"default": "default",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "theme",
						"type": { "text": "'auto' | 'light' | 'dark'" },
						"attribute": "theme",
						"default": "auto",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "wasmUrl",
						"type": { "text": "string" },
						"attribute": "wasm-url",
						"default": "/sail_compiler_bg.wasm"
					},
					{
						"kind": "field",
						"name": "opsUrl",
						"type": { "text": "string" },
						"attribute": "ops-url",
						"default": "",
						"description": "SailAuthoring ops base URL. Empty disables; \"/\" = same-origin."
					},
					{
						"kind": "field",
						"name": "opsPath",
						"type": { "text": "string" },
						"attribute": "ops-path",
						"default": "",
						"description": "Active file path for Hover/Complete/diagnostics RPCs."
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "{\n      md: 'text-sm leading-normal-plus',\n      sm: 'text-xs leading-normal',\n      lg: 'text-base leading-loose',\n    }" }
					},
					{
						"kind": "field",
						"name": "chrome",
						"type": { "text": "{\n      card: 'border-border bg-surface-1 rounded-lg shadow-xs',\n      flush: '',\n    }" }
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-code-editor",
				"declaration": {
					"name": "SailCodeEditor",
					"module": "sail-code-editor.sail"
				}
			}],
			"package": "@ship/sail-code-editor"
		},
		{
			"kind": "javascript-module",
			"path": "sail-color-picker.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailColorPicker",
				"tagName": "sail-color-picker",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "name",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "value",
						"type": { "text": "string" },
						"default": "#000000"
					},
					{
						"name": "disabled",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "showHex",
						"type": { "text": "boolean" },
						"default": "true"
					},
					{
						"name": "variant",
						"type": { "text": "'default' | 'swatch'" },
						"default": "default",
						"description": "swatch = color fill only (no hex field / \"Pick color\" chrome) — for design studio cards.",
						"values": ["default", "swatch"]
					},
					{
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"default": "default",
						"values": [
							"default",
							"primary",
							"info",
							"success",
							"warning",
							"destructive",
							"accent",
							"muted"
						]
					},
					{
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"default": "md",
						"values": [
							"md",
							"sm",
							"lg"
						]
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "name",
						"type": { "text": "string" },
						"attribute": "name",
						"default": ""
					},
					{
						"kind": "field",
						"name": "value",
						"type": { "text": "string" },
						"attribute": "value",
						"default": "#000000"
					},
					{
						"kind": "field",
						"name": "disabled",
						"type": { "text": "boolean" },
						"attribute": "disabled",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "showHex",
						"type": { "text": "boolean" },
						"attribute": "showHex",
						"default": "true",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "variant",
						"type": { "text": "'default' | 'swatch'" },
						"attribute": "variant",
						"default": "default",
						"description": "swatch = color fill only (no hex field / \"Pick color\" chrome) — for design studio cards.",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"attribute": "tone",
						"default": "default",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"attribute": "size",
						"default": "md",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-color-picker",
				"declaration": {
					"name": "SailColorPicker",
					"module": "sail-color-picker.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-component-viewer.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailComponentViewer",
				"tagName": "sail-component-viewer",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "name",
						"type": { "text": "string" },
						"default": "",
						"description": "Component name — the heading."
					},
					{
						"name": "blurb",
						"type": { "text": "string" },
						"default": "",
						"description": "Short blurb under the heading."
					},
					{
						"name": "pack",
						"type": { "text": "string" },
						"default": "",
						"description": "Pack label chip (e.g. \"L1 · language element\")."
					},
					{
						"name": "variant-attr",
						"type": { "text": "string" },
						"default": "",
						"description": "Which attribute the variant chips drive (empty = no chips)."
					},
					{
						"name": "usage",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "no-a11y-chip",
						"type": { "text": "boolean" },
						"default": "false",
						"description": "Hide the a11y assurance chip (fx actions and the like)."
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "name",
						"type": { "text": "string" },
						"attribute": "name",
						"default": "",
						"description": "Component name — the heading."
					},
					{
						"kind": "field",
						"name": "blurb",
						"type": { "text": "string" },
						"attribute": "blurb",
						"default": "",
						"description": "Short blurb under the heading."
					},
					{
						"kind": "field",
						"name": "pack",
						"type": { "text": "string" },
						"attribute": "pack",
						"default": "",
						"description": "Pack label chip (e.g. \"L1 · language element\")."
					},
					{
						"kind": "field",
						"name": "variantAttr",
						"type": { "text": "string" },
						"attribute": "variant-attr",
						"default": "",
						"description": "Which attribute the variant chips drive (empty = no chips)."
					},
					{
						"kind": "field",
						"name": "variants",
						"type": { "text": "string[]" },
						"description": "Variant values."
					},
					{
						"kind": "field",
						"name": "rows",
						"type": { "text": "PropRow[]" }
					},
					{
						"kind": "field",
						"name": "usage",
						"type": { "text": "string" },
						"attribute": "usage",
						"default": ""
					},
					{
						"kind": "field",
						"name": "noA11yChip",
						"type": { "text": "boolean" },
						"attribute": "no-a11y-chip",
						"default": "false",
						"description": "Hide the a11y assurance chip (fx actions and the like).",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["(default)", "stage"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-component-viewer",
				"declaration": {
					"name": "SailComponentViewer",
					"module": "sail-component-viewer.sail"
				}
			}],
			"package": "@ship/sail-viewer"
		},
		{
			"kind": "javascript-module",
			"path": "sail-composer.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailComposer",
				"tagName": "sail-composer",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "placeholder",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "disabled",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "multiline",
						"type": { "text": "boolean" },
						"default": "false"
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "placeholder",
						"type": { "text": "string" },
						"attribute": "placeholder",
						"default": ""
					},
					{
						"kind": "field",
						"name": "disabled",
						"type": { "text": "boolean" },
						"attribute": "disabled",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "multiline",
						"type": { "text": "boolean" },
						"attribute": "multiline",
						"default": "false",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["attachments", "trailing"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-composer",
				"declaration": {
					"name": "SailComposer",
					"module": "sail-composer.sail"
				}
			}],
			"package": "@ship/sail-chat-elements"
		},
		{
			"kind": "javascript-module",
			"path": "sail-date-picker.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailDatePicker",
				"tagName": "sail-date-picker",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "name",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "value",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "min",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "max",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "placeholder",
						"type": { "text": "string" },
						"default": "YYYY-MM-DD"
					},
					{
						"name": "disabled",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"default": "default",
						"values": [
							"default",
							"primary",
							"info",
							"success",
							"warning",
							"destructive",
							"accent",
							"muted"
						]
					},
					{
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"default": "md",
						"values": [
							"md",
							"sm",
							"lg"
						]
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "name",
						"type": { "text": "string" },
						"attribute": "name",
						"default": ""
					},
					{
						"kind": "field",
						"name": "value",
						"type": { "text": "string" },
						"attribute": "value",
						"default": ""
					},
					{
						"kind": "field",
						"name": "min",
						"type": { "text": "string" },
						"attribute": "min",
						"default": ""
					},
					{
						"kind": "field",
						"name": "max",
						"type": { "text": "string" },
						"attribute": "max",
						"default": ""
					},
					{
						"kind": "field",
						"name": "placeholder",
						"type": { "text": "string" },
						"attribute": "placeholder",
						"default": "YYYY-MM-DD"
					},
					{
						"kind": "field",
						"name": "disabled",
						"type": { "text": "boolean" },
						"attribute": "disabled",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"attribute": "tone",
						"default": "default",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"attribute": "size",
						"default": "md",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-date-picker",
				"declaration": {
					"name": "SailDatePicker",
					"module": "sail-date-picker.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-diff.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailDiff",
				"tagName": "sail-diff",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "from",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "to",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "value",
						"type": { "text": "string" },
						"default": ""
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "from",
						"type": { "text": "string" },
						"attribute": "from",
						"default": ""
					},
					{
						"kind": "field",
						"name": "to",
						"type": { "text": "string" },
						"attribute": "to",
						"default": ""
					},
					{
						"kind": "field",
						"name": "value",
						"type": { "text": "string" },
						"attribute": "value",
						"default": ""
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-diff",
				"declaration": {
					"name": "SailDiff",
					"module": "sail-diff.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-error-boundary.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailErrorBoundary",
				"tagName": "sail-error-boundary",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [{
					"name": "fallback-text",
					"type": { "text": "string" },
					"default": ""
				}],
				"members": [{
					"kind": "field",
					"name": "fallbackText",
					"type": { "text": "string" },
					"attribute": "fallback-text",
					"default": ""
				}],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["(default)"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-error-boundary",
				"declaration": {
					"name": "SailErrorBoundary",
					"module": "sail-error-boundary.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-file-upload.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailFileUpload",
				"tagName": "sail-file-upload",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "accept",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "multiple",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "max-size",
						"type": { "text": "number" },
						"default": "10485760"
					},
					{
						"name": "disabled",
						"type": { "text": "boolean" },
						"default": "false"
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "accept",
						"type": { "text": "string" },
						"attribute": "accept",
						"default": ""
					},
					{
						"kind": "field",
						"name": "multiple",
						"type": { "text": "boolean" },
						"attribute": "multiple",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "maxSize",
						"type": { "text": "number" },
						"attribute": "max-size",
						"default": "10485760"
					},
					{
						"kind": "field",
						"name": "disabled",
						"type": { "text": "boolean" },
						"attribute": "disabled",
						"default": "false",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-file-upload",
				"declaration": {
					"name": "SailFileUpload",
					"module": "sail-file-upload.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-json-view.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailJsonView",
				"tagName": "sail-json-view",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "data",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "expanded",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "depth",
						"type": { "text": "number" },
						"default": "1"
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "data",
						"type": { "text": "string" },
						"attribute": "data",
						"default": ""
					},
					{
						"kind": "field",
						"name": "expanded",
						"type": { "text": "boolean" },
						"attribute": "expanded",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "depth",
						"type": { "text": "number" },
						"attribute": "depth",
						"default": "1"
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-json-view",
				"declaration": {
					"name": "SailJsonView",
					"module": "sail-json-view.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-markdown.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailMarkdown",
				"tagName": "sail-markdown",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [{
					"name": "value",
					"type": { "text": "string" },
					"default": ""
				}],
				"members": [{
					"kind": "field",
					"name": "value",
					"type": { "text": "string" },
					"attribute": "value",
					"default": ""
				}],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-markdown",
				"declaration": {
					"name": "SailMarkdown",
					"module": "sail-markdown.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-message-list.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailMessageList",
				"tagName": "sail-message-list",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [{
					"name": "sticky-bottom",
					"type": { "text": "boolean" },
					"default": "true"
				}],
				"members": [{
					"kind": "field",
					"name": "stickyBottom",
					"type": { "text": "boolean" },
					"attribute": "sticky-bottom",
					"default": "true",
					"reflects": true
				}],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["empty"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-message-list",
				"declaration": {
					"name": "SailMessageList",
					"module": "sail-message-list.sail"
				}
			}],
			"package": "@ship/sail-chat-elements"
		},
		{
			"kind": "javascript-module",
			"path": "sail-message.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailMessage",
				"tagName": "sail-message",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "role",
						"type": { "text": "'assistant' | 'user' | 'system' | 'tool'" },
						"default": "assistant",
						"values": [
							"assistant",
							"user",
							"system",
							"tool"
						]
					},
					{
						"name": "content-type",
						"type": { "text": "'text' | 'code' | 'drawing' | 'terminal-snapshot' | 'tool-call' | 'citation' | 'image'" },
						"default": "text",
						"values": [
							"text",
							"code",
							"drawing",
							"terminal-snapshot",
							"tool-call",
							"citation",
							"image"
						]
					},
					{
						"name": "timestamp",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "status",
						"type": { "text": "'sent' | 'sending' | 'error'" },
						"default": "sent",
						"values": [
							"sent",
							"sending",
							"error"
						]
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "role",
						"type": { "text": "'assistant' | 'user' | 'system' | 'tool'" },
						"attribute": "role",
						"default": "assistant",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "contentType",
						"type": { "text": "'text' | 'code' | 'drawing' | 'terminal-snapshot' | 'tool-call' | 'citation' | 'image'" },
						"attribute": "content-type",
						"default": "text",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "timestamp",
						"type": { "text": "string" },
						"attribute": "timestamp",
						"default": ""
					},
					{
						"kind": "field",
						"name": "status",
						"type": { "text": "'sent' | 'sending' | 'error'" },
						"attribute": "status",
						"default": "sent",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["(default)", "trailing"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-message",
				"declaration": {
					"name": "SailMessage",
					"module": "sail-message.sail"
				}
			}],
			"package": "@ship/sail-chat-elements"
		},
		{
			"kind": "javascript-module",
			"path": "sail-render.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailRender",
				"tagName": "sail-render",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "mode",
						"type": { "text": "'source' | 'tag'" },
						"default": "source",
						"values": ["source", "tag"]
					},
					{
						"name": "component-name",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "viewport-width",
						"type": { "text": "'full' | 'desktop' | 'tablet' | 'mobile'" },
						"default": "full",
						"values": [
							"full",
							"desktop",
							"tablet",
							"mobile"
						]
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "mode",
						"type": { "text": "'source' | 'tag'" },
						"attribute": "mode",
						"default": "source",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "componentName",
						"type": { "text": "string" },
						"attribute": "component-name",
						"default": ""
					},
					{
						"kind": "field",
						"name": "viewportWidth",
						"type": { "text": "'full' | 'desktop' | 'tablet' | 'mobile'" },
						"attribute": "viewport-width",
						"default": "full",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-render",
				"declaration": {
					"name": "SailRender",
					"module": "sail-render.sail"
				}
			}],
			"package": "@ship/sail-viewer"
		},
		{
			"kind": "javascript-module",
			"path": "sail-thinking-dots.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailThinkingDots",
				"tagName": "sail-thinking-dots",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "label",
						"type": { "text": "string" },
						"default": "Assistant is thinking"
					},
					{
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"default": "md",
						"values": [
							"md",
							"sm",
							"lg"
						]
					},
					{
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"default": "default",
						"values": [
							"default",
							"primary",
							"info",
							"success",
							"warning",
							"destructive",
							"accent",
							"muted"
						]
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "label",
						"type": { "text": "string" },
						"attribute": "label",
						"default": "Assistant is thinking"
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"attribute": "size",
						"default": "md",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"attribute": "tone",
						"default": "default",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": []
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-thinking-dots",
				"declaration": {
					"name": "SailThinkingDots",
					"module": "sail-thinking-dots.sail"
				}
			}],
			"package": "@ship/sail-chat-elements"
		},
		{
			"kind": "javascript-module",
			"path": "sail-timeline-item.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailTimelineItem",
				"tagName": "sail-timeline-item",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"default": "default",
						"values": [
							"default",
							"primary",
							"info",
							"success",
							"warning",
							"destructive",
							"accent",
							"muted"
						]
					},
					{
						"name": "at",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "interactive",
						"type": { "text": "boolean" },
						"default": "false"
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "tone",
						"type": { "text": "'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'" },
						"attribute": "tone",
						"default": "default",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "at",
						"type": { "text": "string" },
						"attribute": "at",
						"default": ""
					},
					{
						"kind": "field",
						"name": "interactive",
						"type": { "text": "boolean" },
						"attribute": "interactive",
						"default": "false",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["(default)", "title"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-timeline-item",
				"declaration": {
					"name": "SailTimelineItem",
					"module": "sail-timeline-item.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-timeline.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailTimeline",
				"tagName": "sail-timeline",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [],
				"members": [],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["(default)"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-timeline",
				"declaration": {
					"name": "SailTimeline",
					"module": "sail-timeline.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-tool-call.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailToolCall",
				"tagName": "sail-tool-call",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "name",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "status",
						"type": { "text": "'running' | 'ok' | 'error'" },
						"default": "running",
						"values": [
							"running",
							"ok",
							"error"
						]
					},
					{
						"name": "open",
						"type": { "text": "boolean" },
						"default": "false"
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "name",
						"type": { "text": "string" },
						"attribute": "name",
						"default": ""
					},
					{
						"kind": "field",
						"name": "status",
						"type": { "text": "'running' | 'ok' | 'error'" },
						"attribute": "status",
						"default": "running",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "open",
						"type": { "text": "boolean" },
						"attribute": "open",
						"default": "false",
						"reflects": true
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["(default)", "result"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-tool-call",
				"declaration": {
					"name": "SailToolCall",
					"module": "sail-tool-call.sail"
				}
			}],
			"package": "@ship/sail-chat-elements"
		},
		{
			"kind": "javascript-module",
			"path": "sail-tree-grid.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailTreeGrid",
				"tagName": "sail-tree-grid",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "nodes",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "columns",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "label",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "selectable",
						"type": { "text": "'none' | 'single' | 'multi'" },
						"default": "none",
						"values": [
							"none",
							"single",
							"multi"
						]
					},
					{
						"name": "expand-on-select",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"default": "md",
						"values": [
							"md",
							"sm",
							"lg"
						]
					},
					{
						"name": "loading",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "error",
						"type": { "text": "string" },
						"default": ""
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "nodes",
						"type": { "text": "string" },
						"attribute": "nodes",
						"default": ""
					},
					{
						"kind": "field",
						"name": "columns",
						"type": { "text": "string" },
						"attribute": "columns",
						"default": ""
					},
					{
						"kind": "field",
						"name": "label",
						"type": { "text": "string" },
						"attribute": "label",
						"default": ""
					},
					{
						"kind": "field",
						"name": "selectable",
						"type": { "text": "'none' | 'single' | 'multi'" },
						"attribute": "selectable",
						"default": "none",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "expandOnSelect",
						"type": { "text": "boolean" },
						"attribute": "expand-on-select",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"attribute": "size",
						"default": "md",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "loading",
						"type": { "text": "boolean" },
						"attribute": "loading",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "error",
						"type": { "text": "string" },
						"attribute": "error",
						"default": ""
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["empty"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-tree-grid",
				"declaration": {
					"name": "SailTreeGrid",
					"module": "sail-tree-grid.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		},
		{
			"kind": "javascript-module",
			"path": "sail-tree.sail",
			"declarations": [{
				"kind": "class",
				"name": "SailTree",
				"tagName": "sail-tree",
				"customElement": true,
				"superclass": {
					"name": "HTMLElement",
					"package": "global"
				},
				"attributes": [
					{
						"name": "nodes",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "label",
						"type": { "text": "string" },
						"default": ""
					},
					{
						"name": "selectable",
						"type": { "text": "'none' | 'single' | 'multi'" },
						"default": "none",
						"values": [
							"none",
							"single",
							"multi"
						]
					},
					{
						"name": "expand-on-select",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "show-guides",
						"type": { "text": "boolean" },
						"default": "true"
					},
					{
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"default": "md",
						"values": [
							"md",
							"sm",
							"lg"
						]
					},
					{
						"name": "loading",
						"type": { "text": "boolean" },
						"default": "false"
					},
					{
						"name": "error",
						"type": { "text": "string" },
						"default": ""
					}
				],
				"members": [
					{
						"kind": "field",
						"name": "nodes",
						"type": { "text": "string" },
						"attribute": "nodes",
						"default": ""
					},
					{
						"kind": "field",
						"name": "label",
						"type": { "text": "string" },
						"attribute": "label",
						"default": ""
					},
					{
						"kind": "field",
						"name": "selectable",
						"type": { "text": "'none' | 'single' | 'multi'" },
						"attribute": "selectable",
						"default": "none",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "expandOnSelect",
						"type": { "text": "boolean" },
						"attribute": "expand-on-select",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "showGuides",
						"type": { "text": "boolean" },
						"attribute": "show-guides",
						"default": "true",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "size",
						"type": { "text": "'md' | 'sm' | 'lg'" },
						"attribute": "size",
						"default": "md",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "loading",
						"type": { "text": "boolean" },
						"attribute": "loading",
						"default": "false",
						"reflects": true
					},
					{
						"kind": "field",
						"name": "error",
						"type": { "text": "string" },
						"attribute": "error",
						"default": ""
					}
				],
				"anatomy": {
					"dom": "light",
					"classes": [],
					"slots": ["empty"]
				}
			}],
			"exports": [{
				"kind": "custom-element-definition",
				"name": "sail-tree",
				"declaration": {
					"name": "SailTree",
					"module": "sail-tree.sail"
				}
			}],
			"package": "@ship/sail-primitives"
		}
	]
};

//#endregion
//#region src/components/catalog.ts
function attrType(a) {
	if (!a.type) return "unknown";
	if (typeof a.type === "string") return a.type;
	return a.type.text || "unknown";
}
function mapProps(attrs) {
	if (!Array.isArray(attrs)) return [];
	return attrs.filter((a) => a?.name).map((a) => ({
		name: String(a.name),
		type: attrType(a),
		defaultValue: a.default != null ? String(a.default) : "",
		required: !!a.required,
		description: a.description ? String(a.description) : ""
	}));
}
function mapSlots(decl) {
	const raw = decl.slots;
	if (!Array.isArray(raw)) return [];
	return raw.map((s) => typeof s === "string" ? s : s?.name ? String(s.name) : "").filter(Boolean);
}
function displayFromTag(tag, name) {
	if (name && name.trim()) return name.trim();
	return tag.split("-").filter(Boolean).map((p) => p.charAt(0).toUpperCase() + p.slice(1)).join(" ");
}
/** Pack records from app components.gen CEM aggregate. */
function recordsFromCatalog(catalog) {
	const modules = catalog?.modules;
	if (!Array.isArray(modules)) return [];
	const out = [];
	const seen = /* @__PURE__ */ new Set();
	for (const mod of modules) {
		const pkg = mod.package ? String(mod.package) : "";
		const modPath = mod.path ? String(mod.path) : "";
		const decls = Array.isArray(mod.declarations) ? mod.declarations : [];
		for (const decl of decls) {
			const tag = decl.tagName ? String(decl.tagName).trim() : "";
			if (!tag) continue;
			if (decl.customElement === false) continue;
			if (seen.has(tag)) continue;
			seen.add(tag);
			const props = mapProps(decl.attributes);
			const slots = mapSlots(decl);
			const pathDisplay = pkg && modPath ? `${pkg}/${modPath}` : modPath || pkg || "";
			out.push({
				id: tag,
				displayName: displayFromTag(tag, decl.name),
				origin: "pack",
				package: pkg || "—",
				path: pathDisplay,
				layer: "L2",
				description: decl.description ? String(decl.description) : "",
				props,
				slots,
				issueCount: 0,
				ssrTierSummary: "unknown",
				hasPreview: false,
				threadCount: 0,
				canOpenInEdit: false
			});
		}
	}
	return out.sort((a, b) => a.id.localeCompare(b.id));
}
/**
* Project-authored component candidates from ListFiles (or seed paths).
* Heuristic only — no first-class project component index op (F15 candidate).
*/
function isProjectComponentPath(path) {
	const n = path.replace(/\\/g, "/");
	if (!n.endsWith(".sail")) return false;
	if (n.includes("/routes/")) return false;
	if (/\/page\.sail$/.test(n)) return false;
	if (n.includes("/test/") || n.includes("/fixtures/")) return false;
	if (n.includes("/src/")) return true;
	if (!n.includes("/node_modules/") && /\/[a-z0-9][\w-]*\.sail$/i.test(n)) {
		const base = n.split("/").pop() || "";
		return base !== "page.sail" && !base.startsWith("_");
	}
	return false;
}
/** Derive stable id from path: themes-surface.sail → themes-surface */
function projectComponentId(path) {
	return (path.replace(/\\/g, "/").split("/").pop() || path).replace(/\.sail$/i, "") || path;
}
function recordsFromProjectPaths(paths) {
	const out = [];
	const seen = /* @__PURE__ */ new Set();
	for (const path of paths) {
		if (!isProjectComponentPath(path)) continue;
		const id = projectComponentId(path);
		if (!id || seen.has(id)) continue;
		seen.add(id);
		out.push({
			id,
			displayName: displayFromTag(id),
			origin: "project",
			package: "—",
			path,
			layer: "app",
			description: "",
			props: [],
			slots: [],
			issueCount: 0,
			ssrTierSummary: "unknown",
			hasPreview: false,
			threadCount: 0,
			canOpenInEdit: true
		});
	}
	return out.sort((a, b) => a.id.localeCompare(b.id));
}
/** Merge project + pack; project wins on id collision. */
function mergeComponentRecords(project, packs) {
	const byId = /* @__PURE__ */ new Map();
	for (const p of packs) byId.set(p.id, p);
	for (const p of project) byId.set(p.id, p);
	return [...byId.values()].sort((a, b) => {
		if (a.origin !== b.origin) return a.origin === "project" ? -1 : 1;
		return a.id.localeCompare(b.id);
	});
}
function filterRecords(records, query) {
	const q = query.trim().toLowerCase();
	if (!q) return records;
	return records.filter((r) => {
		return [
			r.id,
			r.displayName,
			r.package,
			r.path,
			r.description
		].join(" ").toLowerCase().includes(q);
	});
}
function recordById(records, id) {
	if (!id) return null;
	return records.find((r) => r.id === id) ?? null;
}
/**
* Project path authority for catalog loads.
* - `replace`: gen-owned ListFiles/seed settle (full authority).
* - `union`: edit explorer tick — may only add paths, never drop ListFiles results.
*/
function mergeProjectPathAuthority(auth, incoming, mode) {
	if (mode === "replace") {
		const next = [...incoming];
		return {
			paths: next,
			changed: !(next.length === auth.length && next.every((p, i) => p === auth[i]))
		};
	}
	const set = new Set(auth);
	let changed = false;
	for (const p of incoming) {
		if (!p || set.has(p)) continue;
		set.add(p);
		changed = true;
	}
	if (!changed) return {
		paths: auth,
		changed: false
	};
	return {
		paths: [...set],
		changed: true
	};
}

//#endregion
//#region src/components-library/components-library.ts
/**
* Components library domain — catalog load, filter, groups, selection detail VMs.
* Single path authority: gen-owned loadCatalog replaces; edit ticks only union.
*/
function mapRow(r, sel) {
	const meta = r.origin === "project" ? r.path.split("/").slice(-2).join("/") || r.path : r.package && r.package !== "—" ? r.package.replace(/^@ship\//, "") : r.path;
	return {
		id: r.id,
		title: r.displayName || r.id,
		meta,
		path: r.path,
		origin: r.origin,
		package: r.package,
		selected: sel === r.id,
		originLabel: r.origin === "project" ? "project" : "pack",
		originTone: r.origin === "project" ? "primary" : "muted",
		issueLabel: r.issueCount > 0 ? String(r.issueCount) : "",
		hasIssues: r.issueCount > 0,
		key: r.id
	};
}
function createComponentsLibrary(session) {
	const allRecords = signal([]);
	const projectRows = signal([]);
	const packRows = signal([]);
	const hasProjectRows = signal(false);
	const hasPackRows = signal(false);
	const projectCountLabel = signal("0");
	const packCountLabel = signal("0");
	const isLoading = signal(false);
	const loadingLabel = signal("");
	const hasSelected = signal(false);
	const selectedRecord = signal(null);
	const detailTitle = signal("Select a component");
	const detailPath = signal("");
	const detailOriginLabel = signal("");
	const detailOriginTone = signal("muted");
	const detailPackage = signal("");
	const detailLayer = signal("");
	const detailIntent = signal("No intent documented.");
	const hasIntent = signal(false);
	const canOpenInEdit = signal(false);
	const propRows = signal([]);
	const hasProps = signal(false);
	const propsIncomplete = signal(false);
	const kvRows = signal([]);
	const compositionLabel = signal("—");
	const slotsLabel = signal("—");
	const previewTitle = signal("Preview");
	const previewHint = signal("Select a component to preview.");
	const emptyProject = signal(false);
	const emptyAll = signal(false);
	const schemaNote = signal("");
	/** Catalog load generation — stale ListFiles settles must not overwrite. */
	let loadGen = 0;
	/**
	* Authoritative project paths. Only gen-owned `loadCatalog` may replace.
	* Edit explorer ticks may only union (never shrink after ListFiles).
	*/
	let authProjectPaths = [];
	const recordsSnapshot = () => allRecords();
	const publishDetail = (rec) => {
		selectedRecord.set(rec);
		hasSelected.set(!!rec);
		if (!rec) {
			detailTitle.set(session.showNewForm() ? "New component" : "Select a component");
			detailPath.set("");
			detailOriginLabel.set("");
			detailOriginTone.set("muted");
			detailPackage.set("");
			detailLayer.set("");
			detailIntent.set(session.showNewForm() ? "Describe intent, then copy a brief or start a thread." : "Choose a component from This project or Installed packs.");
			hasIntent.set(false);
			canOpenInEdit.set(false);
			propRows.set([]);
			hasProps.set(false);
			propsIncomplete.set(false);
			kvRows.set([]);
			compositionLabel.set("—");
			slotsLabel.set("—");
			previewTitle.set("Preview");
			previewHint.set("Select a component to preview.");
			schemaNote.set("");
			return;
		}
		detailTitle.set(rec.displayName || rec.id);
		detailPath.set(rec.path || "—");
		detailOriginLabel.set(rec.origin === "project" ? "This project" : "Installed pack");
		detailOriginTone.set(rec.origin === "project" ? "primary" : "muted");
		detailPackage.set(rec.package || "—");
		detailLayer.set(rec.layer);
		const intent = rec.description?.trim();
		hasIntent.set(!!intent);
		detailIntent.set(intent || "No intent documented.");
		canOpenInEdit.set(!!rec.canOpenInEdit && !!rec.path && !rec.path.includes("@ship/"));
		const props = rec.props.map((p, i) => ({
			name: p.name,
			type: p.type,
			defaultValue: p.defaultValue || "—",
			requiredLabel: p.required ? "required" : "optional",
			description: p.description || "",
			key: `${p.name}-${i}`
		}));
		propRows.set(props);
		hasProps.set(props.length > 0);
		propsIncomplete.set(rec.origin === "project" && props.length === 0);
		schemaNote.set(rec.origin === "project" && props.length === 0 ? "Schema incomplete — project components have no CEM props yet." : props.length === 0 ? "No attributes declared in catalog." : "");
		kvRows.set([
			{
				label: "Id",
				value: rec.id,
				key: "id"
			},
			{
				label: "Origin",
				value: rec.origin,
				key: "origin"
			},
			{
				label: "Package",
				value: rec.package || "—",
				key: "package"
			},
			{
				label: "Path",
				value: rec.path || "—",
				key: "path"
			},
			{
				label: "Layer",
				value: rec.layer,
				key: "layer"
			},
			{
				label: "SSR",
				value: rec.ssrTierSummary || "unknown",
				key: "ssr"
			}
		].filter(Boolean));
		slotsLabel.set(rec.slots.length ? rec.slots.join(", ") : "—");
		compositionLabel.set(rec.slots.length ? `slots: ${rec.slots.join(", ")}` : "—");
		previewTitle.set(rec.displayName || rec.id);
		previewHint.set(rec.origin === "pack" ? "Pack preview stage is static for now — open in docs or Edit when path is available." : "Preview uses the app theme. Live mount is not wired for every project component yet.");
	};
	const syncRows = () => {
		const q = session.filterQuery();
		const sel = session.selectedId();
		const filtered = filterRecords(allRecords(), q);
		const proj = filtered.filter((r) => r.origin === "project").map((r) => mapRow(r, sel));
		const packs = filtered.filter((r) => r.origin === "pack").map((r) => mapRow(r, sel));
		projectRows.set(proj);
		packRows.set(packs);
		hasProjectRows.set(proj.length > 0);
		hasPackRows.set(packs.length > 0);
		projectCountLabel.set(String(proj.length));
		packCountLabel.set(String(packs.length));
		emptyProject.set(proj.length === 0);
		emptyAll.set(filtered.length === 0 && !isLoading());
		publishDetail(recordById(allRecords(), sel));
	};
	const ensureDefaultSelection = () => {
		const records = allRecords();
		if (!records.length) {
			if (session.selectedId()) session.setSelectedId("", { navigate: false });
			syncRows();
			return;
		}
		const cur = session.selectedId();
		if (cur && recordById(records, cur)) {
			syncRows();
			return;
		}
		const first = records.find((r) => r.origin === "project") ?? records[0];
		if (first) session.setSelectedId(first.id, { navigate: false });
		syncRows();
	};
	const rebuildFromAuth = () => {
		const packs = recordsFromCatalog(COMPONENTS_CATALOG);
		const project = recordsFromProjectPaths(authProjectPaths);
		allRecords.set(mergeComponentRecords(project, packs));
		ensureDefaultSelection();
	};
	/** Gen-owned replace of project path authority + rebuild. */
	const applyCatalog = (projectPathsList) => {
		const { paths } = mergeProjectPathAuthority(authProjectPaths, projectPathsList, "replace");
		authProjectPaths = paths;
		rebuildFromAuth();
	};
	/**
	* Edit explorer tick — union only. Never replace with a strict subset of
	* the last ListFiles authority (H1).
	*/
	const unionFromEditPaths = (paths) => {
		const { paths: next, changed } = mergeProjectPathAuthority(authProjectPaths, paths, "union");
		if (!changed) return false;
		authProjectPaths = next;
		rebuildFromAuth();
		return true;
	};
	const loadCatalog = async () => {
		const gen = ++loadGen;
		isLoading.set(true);
		loadingLabel.set("Loading components…");
		const root = resolveProjectRoot();
		let files = projectPaths(root);
		const editFiles = getEditState().files;
		if (editFiles?.length) files = editFiles.map((f) => f.path);
		try {
			try {
				const ops = getOpsClient();
				if (ops?.sailAuthoringListFiles) {
					const listed = await ops.sailAuthoringListFiles({ project: root });
					if (gen !== loadGen) return;
					if (listed.ok && listed.files?.length) files = listed.files;
				}
			} catch {}
			if (gen !== loadGen) return;
			applyCatalog(files);
		} finally {
			if (gen === loadGen) {
				isLoading.set(false);
				loadingLabel.set("");
			}
		}
	};
	const select = (id) => {
		if (!id) return;
		session.showNewForm.set(false);
		session.setSelectedId(id);
		syncRows();
	};
	const onListClick = (e) => {
		const el = e.target?.closest?.("[component-id]");
		if (!el?.dataset.componentId) return;
		const ctrl = e.target?.closest?.("button, a, input, [popover]");
		if (ctrl && ctrl !== el) return;
		select(el.dataset.componentId);
	};
	const onOpenInEdit = () => {
		const rec = selectedRecord();
		if (!rec?.canOpenInEdit || !rec.path) {
			session.setNote("No project path to open in Edit.", "warning");
			return;
		}
		session.openInEdit(rec.path);
	};
	const onStartThread = () => {
		session.startThread(selectedRecord());
	};
	const onCopyBrief = () => {
		session.copyAgentBrief(selectedRecord());
	};
	const getRecord = (id) => recordById(allRecords(), id);
	syncRows();
	mountAmbient(() => {
		const unsub = session.onSync(() => syncRows());
		const unsubEdit = subscribeEdit(() => {
			if (isLoading()) return;
			const files = getEditState().files;
			if (!files?.length) return;
			unionFromEditPaths(files.map((f) => f.path));
		});
		loadCatalog();
		return () => {
			unsub();
			unsubEdit();
			loadGen += 1;
		};
	});
	return {
		allRecords,
		projectRows,
		packRows,
		hasProjectRows,
		hasPackRows,
		projectCountLabel,
		packCountLabel,
		isLoading,
		loadingLabel,
		hasSelected,
		selectedRecord,
		detailTitle,
		detailPath,
		detailOriginLabel,
		detailOriginTone,
		detailPackage,
		detailLayer,
		detailIntent,
		hasIntent,
		canOpenInEdit,
		propRows,
		hasProps,
		propsIncomplete,
		kvRows,
		compositionLabel,
		slotsLabel,
		previewTitle,
		previewHint,
		emptyProject,
		emptyAll,
		schemaNote,
		loadCatalog,
		select,
		onListClick,
		onOpenInEdit,
		onStartThread,
		onCopyBrief,
		getRecord,
		recordsSnapshot,
		/** @internal tests */
		__applyCatalog: applyCatalog,
		__unionFromEditPaths: unionFromEditPaths,
		__authProjectPaths: () => authProjectPaths.slice(),
		__loadGen: () => loadGen,
		__bumpLoadGen: () => {
			loadGen += 1;
		}
	};
}

//#endregion
//#region src/components-surface/components-surface.sail.controller.ts
var components_surface_sail_controller_default = controller(() => {
	const session = createComponentsSession({ mode: "library" });
	return {
		session,
		library: createComponentsLibrary(session)
	};
});

//#endregion
//#region src/studio-empty/studio-empty.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0$3 = /*@__PURE__*/ __tpl("<div data-sail-host=\"empty-state\" role=\"note\" aria-live=\"off\" studio-empty=\"\" tone=\"default\" class=\"sail-root sx-wfull sail-empty-state sail-col sail-g-3 sail-ai-stretch sail-jc-center sx-fontFamilybody sx-fgemptystateforeground sx-textAligncenter sx-borderWidth1 sx-borderemptystateborder sx-radiuslg sx-bgemptystate sx-shadowxs sx-padY16 sx-padX4 sx-sizemd-padY16 sx-sizemd-padX4 sx-sizesm-padY7 sx-sizesm-padX4 sx-sizelg-padY24 sx-sizelg-padX6 sx-variantdefault-textAligncenter sx-variantdefault-borderWidth1 sx-variantdefault-borderemptystateborder sx-variantdefault-radiuslg sx-variantdefault-bgemptystate sx-variantdefault-shadowxs sx-variantinline-textAlignstart sx-variantinline-borderWidth1 sx-variantinline-borderdashed sx-variantinline-borderemptystateborder sx-variantinline-radiusmd sx-variantinline-bgmuted sx-variantinline-shadownone sx-variantinline-padY3 sx-variantinline-padX4 sx-variantplaceholder-textAligncenter sx-variantplaceholder-borderWidth0 sx-variantplaceholder-bgtransparent sx-variantplaceholder-radiusnone sx-variantplaceholder-shadownone sx-variantplaceholder-padY6 sx-variantconnecting-textAligncenter sx-variantconnecting-borderWidth0 sx-variantconnecting-bgtransparent sx-variantconnecting-radiusnone sx-variantconnecting-shadownone sx-variantconnecting-padY6 sx-forcedcolors-borderCanvasText sx-pad3 sx-borderbordersubtle sx-radiusmd\"><div aria-hidden=\"true\" class=\"sail-row-i sail-g-3 sail-ai-center sail-jc-center sail-nowrap sx-shrink1 sx-selfcenter sx-radiuslg sx-bgmuted sx-fgmutedforeground sx-size12 sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-toneprimary-bgmuted sx-toneprimary-fgprimary sx-toneinfo-bgmuted sx-toneinfo-fginfo sx-tonesuccess-bgmuted sx-tonesuccess-fgsuccess sx-tonewarning-bgmuted sx-tonewarning-fgwarning sx-tonedestructive-bgmuted sx-tonedestructive-fgdestructive sx-toneaccent-bgmuted sx-toneaccent-fgaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-empty-displaynone sx-forcedcolors-bgCanvas sx-forcedcolors-fgCanvasText\"></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0 sx-selfstretch\"><div class=\"sx-fontFamilyheading sx-fontWeightsemibold sx-fgforeground sx-leadingtight sx-fontSizemd sx-empty-displaynone\"><strong slot=\"title\"> </strong></div><div class=\"sx-leadingloose sx-fgmutedforeground sx-fontSizesm sx-maxWpanelchwmd sx-empty-displaynone\"><slot name=\"icon\"></slot><slot></slot><slot name=\"action\"></slot></div></div><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-center sx-selfcenter sx-empty-displaynone\"></div></div>");
const __default$2 = Object.assign(component((ctx) => {
	const { route, navigate, providers } = ctx;
	const $scope = Object.create(null);
	$scope.route = route;
	$scope.navigate = navigate;
	for (const __k of Object.keys(ctx)) {
		if (__k === "route" || __k === "navigate" || __k === "scope" || __k === "stateKeys" || __k === "host" || __k === "providers" || __k === "props") continue;
		$scope[__k] = ctx[__k];
	}
	if (providers && typeof providers === "object") Object.assign($scope, providers);
	const __props = ctx.props ?? {};
	$scope.title = __props.title;
	$scope.variant = __props.variant;
	$scope.size = __props.size;
	return Object.assign({}, $scope, {
		__mount(container) {
			const __cleanups = [];
			const __root = container.shadowRoot ?? container;
			const __e0 = __tpl0$3();
			runAction(__e0, variantLive, void 0);
			__cleanups.push(bindAttr(__e0, "variant", () => $scope.variant()));
			__cleanups.push(bindAttr(__e0, "size", () => $scope.size()));
			const __t4 = __e0.firstChild.nextSibling.firstChild.firstChild.firstChild;
			markHydrateText(__t4, "b2");
			__cleanups.push(bindText(__t4, () => $scope.title()));
			__root.appendChild(__e0);
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		},
		hydrate(root, ctx) {
			const __cleanups = [];
			const __t = collectTextOpens(root);
			const __m = collectElementMarkers(root);
			if (__m["b0"]) __cleanups.push(hydrateAttr(__m["b0"], "variant", () => $scope.variant()));
			if (__m["b1"]) __cleanups.push(hydrateAttr(__m["b1"], "size", () => $scope.size()));
			if (__t["b2"]) __cleanups.push(hydrateText(__t["b2"], () => $scope.title()));
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		}
	});
}), {
	_compilerVersion: {
		"cargo": "0.1.0",
		"git": "c11a87c8",
		"dirty": true,
		"builtAt": "2026-08-05T21:58:16Z"
	},
	clients: []
});
component("studio-empty", {
	props: {
		"title": prop.string({ default: "" }),
		"variant": prop.union([
			"default",
			"inline",
			"placeholder",
			"connecting"
		], {
			default: "placeholder",
			reflect: true
		}),
		"size": prop.union([
			"md",
			"sm",
			"lg"
		], {
			default: "sm",
			reflect: true
		})
	},
	styles: ".sx-pad3{padding:var(--space-3)}.sx-borderWidth1{border-width:1px}.sx-borderbordersubtle{border-color:var(--border-subtle)}.sx-radiusmd{border-radius:var(--radius-md)}.sx-wfull{width:100%}"
}, __default$2);

//#endregion
//#region src/studio-list-row/studio-list-row.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0$2 = /*@__PURE__*/ __tpl("<slot name=\"leading\"></slot>");
const __tpl1$1 = /*@__PURE__*/ __tpl("<p class=\"sx-grow1 sx-minW0 sx-fontSizesm sx-truncate\"> </p>");
const __tpl2$1 = /*@__PURE__*/ __tpl("<p studio-list-meta=\"\" class=\"sx-shrink0 sx-fgmutedforeground sx-fontSizexs\"> </p>");
const __tpl3$1 = /*@__PURE__*/ __tpl("<slot name=\"trailing\"></slot>");
const __tpl4$1 = /*@__PURE__*/ __tpl("<slot></slot>");
const __default$1 = Object.assign(component((ctx) => {
	const { route, navigate, providers } = ctx;
	const $scope = Object.create(null);
	$scope.route = route;
	$scope.navigate = navigate;
	for (const __k of Object.keys(ctx)) {
		if (__k === "route" || __k === "navigate" || __k === "scope" || __k === "stateKeys" || __k === "host" || __k === "providers" || __k === "props") continue;
		$scope[__k] = ctx[__k];
	}
	if (providers && typeof providers === "object") Object.assign($scope, providers);
	const __props = ctx.props ?? {};
	$scope.title = __props.title;
	$scope.meta = __props.meta;
	$scope.selected = __props.selected;
	const __wireBoundary_b2 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.meta()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag6 = document.createDocumentFragment();
			const __c7 = [];
			const __t8 = document.createTextNode("\n    ");
			__frag6.appendChild(__t8);
			const __e9 = __tpl2$1();
			const __t10 = __e9.firstChild;
			markHydrateText(__t10, "b3");
			__c7.push(bindText(__t10, () => $scope.meta()));
			__frag6.appendChild(__e9);
			const __t11 = document.createTextNode("\n  ");
			__frag6.appendChild(__t11);
			return {
				nodes: Array.from(__frag6.childNodes),
				cleanup: () => {
					for (const c of __c7) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	return Object.assign({}, $scope, {
		__mount(container) {
			const __cleanups = [];
			const __root = container.shadowRoot ?? container;
			const __e0 = document.createElement("div");
			__e0.setAttribute("class", "sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap sx-minW0 sx-h6 sx-padX3 sx-hover-bghoverbg sx-selected-bgsurface2");
			__e0.setAttribute("studio-list-row", "");
			__cleanups.push(bindAttr(__e0, "selected", () => $scope.selected() ? "true" : "false"));
			__cleanups.push(bindAttr(__e0, "aria-selected", () => {
				const __av1 = $scope.selected() ? "true" : "false";
				return __av1 == null || __av1 === false || __av1 === "" ? null : __av1 === true ? "true" : String(__av1);
			}));
			__e0.classList.add("sail-root");
			const __e2 = __tpl0$2();
			__e0.appendChild(__e2);
			const __e3 = __tpl1$1();
			const __t4 = __e3.firstChild;
			markHydrateText(__t4, "b1");
			__cleanups.push(bindText(__t4, () => $scope.title()));
			__e0.appendChild(__e3);
			const __ifAnchor5 = document.createComment("if");
			__e0.appendChild(__ifAnchor5);
			__cleanups.push(__wireBoundary_b2(__ifAnchor5));
			const __e12 = __tpl3$1();
			__e0.appendChild(__e12);
			const __e13 = __tpl4$1();
			__e0.appendChild(__e13);
			__root.appendChild(__e0);
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		},
		hydrate(root, ctx) {
			const __cleanups = [];
			const __t = collectTextOpens(root);
			const __m = collectElementMarkers(root);
			if (__m["b0"]) __cleanups.push(hydrateAttr(__m["b0"], "selected", () => $scope.selected() ? "true" : "false"));
			if (__t["b1"]) __cleanups.push(hydrateText(__t["b1"], () => $scope.title()));
			if (__m["b2"]) {
				const __t = __m["b2"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b2(__anchor));
			}
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		}
	});
}), {
	_compilerVersion: {
		"cargo": "0.1.0",
		"git": "c11a87c8",
		"dirty": true,
		"builtAt": "2026-08-05T21:58:16Z"
	},
	clients: []
});
component("studio-list-row", {
	props: {
		"title": prop.string({ default: "" }),
		"meta": prop.string({ default: "" }),
		"selected": prop.bool({
			default: false,
			reflect: true
		})
	},
	styles: ".sail-root { display: inline-flex; }.sx-h6{height:var(--space-6)}.sx-padX3{padding-inline:var(--space-3)}.sx-minW0{min-width:0}.sx-hover-bghoverbg:hover{background-color:var(--hover-bg)}.sx-selected-bgsurface2[aria-selected='true']{background-color:var(--surface-2)}.sx-fontSizesm{font-size:var(--text-sm)}.sx-truncate{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.sx-fgmutedforeground{color:var(--muted-foreground)}.sx-fontSizexs{font-size:var(--text-xs)}.sx-shrink0{flex-shrink:0}.sx-grow1{flex-grow:1}.sail-row{display:flex;flex-wrap:wrap;min-width:0}.sail-row>*{min-width:0}.sail-row[popover]:not(:popover-open){display:none!important}.sail-row[popover]:popover-open{display:flex;flex-wrap:wrap;min-width:0}.sail-g-2{gap:var(--space-2)}.sail-ai-center{align-items:center}.sail-jc-start{justify-content:flex-start}.sail-row.sail-nowrap{flex-wrap:nowrap}"
}, __default$1);

//#endregion
//#region src/components-surface/components-surface.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0$1 = /*@__PURE__*/ __tpl("<div components-nav-head=\"\" class=\"sail-col sail-g-2 sail-ai-stretch sx-shrink0 sx-padX4 sx-padT4 sx-padB3 sx-borderBWidth1 sx-borderBbordersubtle\"><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-between sail-nowrap\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n            Components</p><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" new-component=\"\" class=\"sx-shrink0 sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n            New…\n          </button></div><div mode-toggle=\"\" class=\"sail-row sail-g-1 sail-ai-center sail-jc-start sail-nowrap\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" mode=\"library\" variant=\"solid\" tone=\"primary\" aria-pressed=\"true\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n            Library\n          </button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" mode=\"inspect\" variant=\"ghost\" tone=\"muted\" aria-pressed=\"false\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-tonemutedvariantghost-fgmutedforeground\">\n            Inspect\n          </button></div><input data-sail-host=\"input\" type=\"search\" size=\"sm\" placeholder=\"Filter name, pack, path…\" components-filter=\"\" tone=\"default\" class=\"sail-input sx-typesearch-displayblock sx-typesearch-minHcontrolhmd sx-typesearch-padY15 sx-typesearch-padX3 sx-typesearch-fontSizebase sx-typesearch-fgforeground sx-typesearch-bgbackground sx-typesearch-borderWidth1 sx-typesearch-borderborder sx-typesearch-radiusmd sx-typesearch-shadowxs sx-typesearch-wfull sx-typesearchdisabled-cursornotallowed sx-typesearchdisabled-opacity50 sx-typesearchsizexs-minHcontrolh2xs sx-typesearchsizexs-padX2 sx-typesearchsizexs-fontSizexs sx-typesearchsizesm-minHcontrolhsm sx-typesearchsizesm-padX25 sx-typesearchsizesm-fontSizesm sx-typesearchsizemd-minHcontrolhmd sx-typesearchsizemd-padX3 sx-typesearchsizemd-fontSizebase sx-typesearchsizelg-minHcontrolhlg sx-typesearchsizelg-padX4 sx-typesearchsizelg-fontSizemd sx-invalidtrue-borderdestructive\"></div>");
const __tpl1 = /*@__PURE__*/ __tpl("<div class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sx-shrink0 sx-padX4 sx-padY2\"><span data-sail-host=\"spinner\" role=\"status\" aria-label=\"Loading\" size=\"sm\" tone=\"default\" class=\"sail-spinner sx-displayinlineblock sx-alignmiddle sx-sizeiconsizemd sx-fgspinnerforeground sx-sizemd-sizeiconsizemd sx-sizexs-sizeiconsize2xs sx-sizesm-sizeiconsizesm sx-sizelg-sizeiconsizexl sx-tonedefault-fgspinnerforeground sx-toneprimary-fgprimary sx-toneinfo-fginfo sx-tonesuccess-fgsuccess sx-tonewarning-fgwarning sx-tonedestructive-fgdestructive sx-toneaccent-fgaccent sx-centeredtrue-displayblock sx-centeredtrue-my8 sx-centeredtrue-mxauto\"><span aria-hidden=\"true\" class=\"sx-displayblock sx-sizefull sx-borderWidthcontrolstroke sx-bordersolid sx-borderspinnertrack sx-borderRcurrentColor sx-radiusfull sx-animatespin\">\n    </span></span><p class=\"sx-fgmutedforeground sx-fontSizesm\"> </p></div>");
const __tpl2 = /*@__PURE__*/ __tpl("<div class=\"sail-row sail-g-3 sail-ai-center sail-jc-between sail-nowrap\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                This project</p><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontFamilymono\"> </p></div>");
const __tpl3 = /*@__PURE__*/ __tpl("<studio-empty title=\"No project components\" size=\"sm\">\n                Author components under src/, or start a brief with New…\n                <button data-sail-host=\"button\" slot=\"action\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" new-component=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n                  New…\n                </button></studio-empty>");
const __tpl4 = /*@__PURE__*/ __tpl("<studio-list-row component-row=\"\"><span data-sail-host=\"badge\" slot=\"trailing\" size=\"sm\" variant=\"outline\" pill=\"\" class=\"sx-shrink0 sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\"> </span></studio-list-row>");
const __tpl5 = /*@__PURE__*/ __tpl("<div class=\"sail-row sail-g-3 sail-ai-center sail-jc-between sail-nowrap\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                Installed packs</p><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontFamilymono\"> </p></div>");
const __tpl6 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizesm\">\n                No installed packs</p>");
const __tpl7 = /*@__PURE__*/ __tpl("<studio-list-row component-row=\"\"><span data-sail-host=\"badge\" slot=\"trailing\" tone=\"muted\" size=\"sm\" variant=\"outline\" pill=\"\" class=\"sx-shrink0 sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">\n                  pack\n                </span></studio-list-row>");
const __tpl8 = /*@__PURE__*/ __tpl("<p l1-footnote=\"\" class=\"sx-padX1 sx-fgmutedforeground sx-fontSizexs sx-leadingsnug\">\n            \n            Element skins live under Themes → Components — not listed here.\n            <button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"ghost\" tone=\"muted\" themes-components=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-tonemutedvariantghost-fgmutedforeground\">\n              Open Themes\n            </button></p>");
const __tpl9 = /*@__PURE__*/ __tpl("<p detail-title=\"\" class=\"sx-fontSizelg sx-fontWeightsemibold sx-trackingtight sx-truncate\"> </p>");
const __tpl10 = /*@__PURE__*/ __tpl("<p detail-path=\"\" class=\"sx-fgmutedforeground sx-fontSizexs sx-fontFamilymono\"> </p>");
const __tpl11 = /*@__PURE__*/ __tpl("<span data-sail-host=\"badge\" size=\"sm\" variant=\"soft\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\"> </span>");
const __tpl12 = /*@__PURE__*/ __tpl("<span data-sail-host=\"badge\" tone=\"muted\" size=\"sm\" variant=\"outline\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\"> </span>");
const __tpl13 = /*@__PURE__*/ __tpl("<div class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap sx-shrink0\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" open-edit=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n              Open in Edit\n            </button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"default\" start-thread=\"\" tone=\"default\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantdefault-bgbackground sx-variantdefault-borderborder sx-variantdefault-fgforeground sx-variantdefault-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm\">\n              Start thread\n            </button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"ghost\" copy-brief=\"\" tone=\"default\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm\">\n              Copy agent brief\n            </button></div>");
const __tpl14 = /*@__PURE__*/ __tpl("<div data-sail-host=\"card\" new-form=\"\" padding=\"md\" tone=\"default\" variant=\"default\" accent-bar=\"off\" class=\"sail-card sx-positionrelative sx-isolate sx-overflowclip sx-radiuslg sx-border1border sx-fgcardforeground sx-shadowcardrest sx-transitioncolors sx-durationfast sx-easeout sx-bgcard sx-padY4 sx-padX5 sx-backdropcardbackdropfilter sx-borderImagecornerimage sx-variantdefault-bgcard sx-variantoutline-bgtransparent sx-paddingmd-padY4 sx-paddingmd-padX5 sx-paddingnone-pad0 sx-paddingsm-padY2 sx-paddingsm-padX3 sx-paddinglg-padY5 sx-paddinglg-padX6 sx-interactivetrue-cursorpointer sx-interactivetrue-tapHighlightnone sx-interactivetruehover-borderborderstrong sx-interactivetruehover-shadowhover sx-interactivetruehover-translateYpx sx-interactivetrueactive-shadowcardrest sx-interactivetrueactive-translateY0 sx-interactivetrueactive-durationinstant sx-interactivetruemotionreduce-translateY0 sx-interactivetruefocus-outline2 sx-interactivetrueforcedcolorsfocus-outlineCanvasText sx-filltrue-bgmuted sx-filltrue-borderborder sx-toneprimary-borderprimary sx-toneinfo-borderinfo sx-tonesuccess-bordersuccess sx-tonewarning-borderwarning sx-tonedestructive-borderdestructive sx-toneaccent-borderaccent sx-tonemuted-borderborder sx-statusrailtrue-borderS3borderstrong sx-statusrailtrue-padS4 sx-paddingsmstatusrailtrue-padS2 sx-paddingmdstatusrailtrue-padS4 sx-paddinglgstatusrailtrue-padS5 sx-paddingnonestatusrailtrue-padS0 sx-accentbarstaticbefore-contentempty sx-accentbarstaticbefore-positionabsolute sx-accentbarstaticbefore-left0 sx-accentbarstaticbefore-right0 sx-accentbarstaticbefore-top0 sx-accentbarstaticbefore-hhalf sx-accentbarstaticbefore-bgImageprimaryaccent sx-accentbarstaticbefore-opacity100 sx-accentbarstaticbefore-radiusTlg sx-accentbarstaticbefore-pointerEventsnone sx-accentbarstaticbefore-transitionopacity sx-accentbarstaticbefore-durationmedium sx-accentbarstaticbefore-easeout sx-accentbarhoverbefore-contentempty sx-accentbarhoverbefore-positionabsolute sx-accentbarhoverbefore-left0 sx-accentbarhoverbefore-right0 sx-accentbarhoverbefore-top0 sx-accentbarhoverbefore-hhalf sx-accentbarhoverbefore-bgImageprimaryaccent sx-accentbarhoverbefore-opacity0 sx-accentbarhoverbefore-radiusTlg sx-accentbarhoverbefore-pointerEventsnone sx-accentbarhoverbefore-transitionopacity sx-accentbarhoverbefore-durationmedium sx-accentbarhoverbefore-easeout sx-accentbarhoverhoverbefore-opacity100 sx-minW0\"><div class=\"sx-mt4 sx-mx5 sx-mb3 sx-padX5 sx-padY4 sx-borderB1bordersubtle sx-fontFamilyheading sx-empty-displaynone\"></div><div class=\"sail-col sail-g-3 sail-ai-stretch\"><p class=\"sx-fontSizemd sx-fontWeightsemibold\">\n                  New component</p><p class=\"sx-fgmutedforeground sx-fontSizesm\">\n                  Creates a draft brief only — does not write files yet.</p><div data-sail-host=\"field\" aria-label=\"Name\" class=\"sail-field sail-col sail-g-1 sail-ai-stretch sx-fontFamilybody sail-grp\"><div></div><div><input data-sail-host=\"input\" type=\"text\" size=\"sm\" placeholder=\"my-widget\" tone=\"default\" class=\"sail-input sx-typetext-displayblock sx-typetext-minHcontrolhmd sx-typetext-padY15 sx-typetext-padX3 sx-typetext-fontSizebase sx-typetext-fgforeground sx-typetext-bgbackground sx-typetext-borderWidth1 sx-typetext-borderborder sx-typetext-radiusmd sx-typetext-shadowxs sx-typetext-wfull sx-typetextdisabled-cursornotallowed sx-typetextdisabled-opacity50 sx-typetextsizexs-minHcontrolh2xs sx-typetextsizexs-padX2 sx-typetextsizexs-fontSizexs sx-typetextsizesm-minHcontrolhsm sx-typetextsizesm-padX25 sx-typetextsizesm-fontSizesm sx-typetextsizemd-minHcontrolhmd sx-typetextsizemd-padX3 sx-typetextsizemd-fontSizebase sx-typetextsizelg-minHcontrolhlg sx-typetextsizelg-padX4 sx-typetextsizelg-fontSizemd sx-invalidtrue-borderdestructive\"></div><div class=\"sx-mt1 sx-fontSizexs sx-leadingsnug sx-fgfieldhint sx-empty-displaynone sx-groupinvalid-displaynone\"></div><div class=\"sx-displaynone sx-groupinvalid-displaycontents\"><div role=\"alert\" aria-live=\"polite\" class=\"sail-row sail-g-1 sail-ai-start sail-jc-start sx-mt1 sx-fontSizexs sx-leadingsnug sx-fgfielderror sx-fontWeightmedium\"><svg viewbox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"sx-shrink0 sx-sizeiconsizesm\"><path d=\"M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z\"></path><path d=\"M12 9v4M12 17h.01\"></path></svg><span></span></div></div></div><div data-sail-host=\"field\" aria-label=\"Intent\" class=\"sail-field sail-col sail-g-1 sail-ai-stretch sx-fontFamilybody sail-grp\"><div></div><div><textarea data-sail-host=\"textarea\" size=\"sm\" rows=\"3\" placeholder=\"What should this component do?\" tone=\"default\" class=\"sail-textarea sx-displayblock sx-minHcontrolhmd sx-padY15 sx-padX3 sx-fontSizebase sx-fgforeground sx-bgbackground sx-borderWidth1 sx-borderborder sx-radiusmd sx-shadowxs sx-wfull sx-hover-borderborderstrong sx-disabled-cursornotallowed sx-disabled-opacity50 sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-invalidtrue-borderdestructive\"></textarea></div><div class=\"sx-mt1 sx-fontSizexs sx-leadingsnug sx-fgfieldhint sx-empty-displaynone sx-groupinvalid-displaynone\"></div><div class=\"sx-displaynone sx-groupinvalid-displaycontents\"><div role=\"alert\" aria-live=\"polite\" class=\"sail-row sail-g-1 sail-ai-start sail-jc-start sx-mt1 sx-fontSizexs sx-leadingsnug sx-fgfielderror sx-fontWeightmedium\"><svg viewbox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"sx-shrink0 sx-sizeiconsizesm\"><path d=\"M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z\"></path><path d=\"M12 9v4M12 17h.01\"></path></svg><span></span></div></div></div><div data-sail-host=\"field\" aria-label=\"Layer\" class=\"sail-field sail-col sail-g-1 sail-ai-stretch sx-fontFamilybody sail-grp\"><div></div><div><select data-sail-host=\"select\" size=\"sm\" tone=\"default\" class=\"sail-select sx-displayblock sx-minHcontrolhmd sx-padY15 sx-padS3 sx-padE8 sx-fontSizebase sx-fgforeground sx-bgbackground sx-borderWidth1 sx-borderborder sx-radiusmd sx-shadowxs sx-wfull sx-cursorpointer sx-hover-borderborderstrong sx-disabled-cursornotallowed sx-disabled-opacity50 sx-sizesm-minHcontrolhsm sx-sizesm-padS25 sx-sizesm-fontSizesm sx-invalidtrue-borderdestructive\"><option value=\"app\">app</option><option value=\"L2\">L2 pack</option></select></div><div class=\"sx-mt1 sx-fontSizexs sx-leadingsnug sx-fgfieldhint sx-empty-displaynone sx-groupinvalid-displaynone\"></div><div class=\"sx-displaynone sx-groupinvalid-displaycontents\"><div role=\"alert\" aria-live=\"polite\" class=\"sail-row sail-g-1 sail-ai-start sail-jc-start sx-mt1 sx-fontSizexs sx-leadingsnug sx-fgfielderror sx-fontWeightmedium\"><svg viewbox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"sx-shrink0 sx-sizeiconsizesm\"><path d=\"M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z\"></path><path d=\"M12 9v4M12 17h.01\"></path></svg><span></span></div></div></div><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" create-brief=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n                    Create brief\n                  </button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"default\" tone=\"default\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantdefault-bgbackground sx-variantdefault-borderborder sx-variantdefault-fgforeground sx-variantdefault-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm\">\n                    Start thread\n                  </button></div></div><div class=\"sx-mt3 sx-mx5 sx-mb4 sx-padX5 sx-padY4 sx-borderT1bordersubtle sx-empty-displaynone\"></div></div>");
const __tpl15 = /*@__PURE__*/ __tpl("<div data-sail-host=\"card\" components-preview=\"\" preview-specimen=\"\" padding=\"none\" tone=\"default\" variant=\"default\" accent-bar=\"off\" class=\"sail-card sx-positionrelative sx-isolate sx-overflowclip sx-radiuslg sx-border1border sx-fgcardforeground sx-shadowcardrest sx-transitioncolors sx-durationfast sx-easeout sx-bgcard sx-padY4 sx-padX5 sx-backdropcardbackdropfilter sx-borderImagecornerimage sx-variantdefault-bgcard sx-variantoutline-bgtransparent sx-paddingmd-padY4 sx-paddingmd-padX5 sx-paddingnone-pad0 sx-paddingsm-padY2 sx-paddingsm-padX3 sx-paddinglg-padY5 sx-paddinglg-padX6 sx-interactivetrue-cursorpointer sx-interactivetrue-tapHighlightnone sx-interactivetruehover-borderborderstrong sx-interactivetruehover-shadowhover sx-interactivetruehover-translateYpx sx-interactivetrueactive-shadowcardrest sx-interactivetrueactive-translateY0 sx-interactivetrueactive-durationinstant sx-interactivetruemotionreduce-translateY0 sx-interactivetruefocus-outline2 sx-interactivetrueforcedcolorsfocus-outlineCanvasText sx-filltrue-bgmuted sx-filltrue-borderborder sx-toneprimary-borderprimary sx-toneinfo-borderinfo sx-tonesuccess-bordersuccess sx-tonewarning-borderwarning sx-tonedestructive-borderdestructive sx-toneaccent-borderaccent sx-tonemuted-borderborder sx-statusrailtrue-borderS3borderstrong sx-statusrailtrue-padS4 sx-paddingsmstatusrailtrue-padS2 sx-paddingmdstatusrailtrue-padS4 sx-paddinglgstatusrailtrue-padS5 sx-paddingnonestatusrailtrue-padS0 sx-accentbarstaticbefore-contentempty sx-accentbarstaticbefore-positionabsolute sx-accentbarstaticbefore-left0 sx-accentbarstaticbefore-right0 sx-accentbarstaticbefore-top0 sx-accentbarstaticbefore-hhalf sx-accentbarstaticbefore-bgImageprimaryaccent sx-accentbarstaticbefore-opacity100 sx-accentbarstaticbefore-radiusTlg sx-accentbarstaticbefore-pointerEventsnone sx-accentbarstaticbefore-transitionopacity sx-accentbarstaticbefore-durationmedium sx-accentbarstaticbefore-easeout sx-accentbarhoverbefore-contentempty sx-accentbarhoverbefore-positionabsolute sx-accentbarhoverbefore-left0 sx-accentbarhoverbefore-right0 sx-accentbarhoverbefore-top0 sx-accentbarhoverbefore-hhalf sx-accentbarhoverbefore-bgImageprimaryaccent sx-accentbarhoverbefore-opacity0 sx-accentbarhoverbefore-radiusTlg sx-accentbarhoverbefore-pointerEventsnone sx-accentbarhoverbefore-transitionopacity sx-accentbarhoverbefore-durationmedium sx-accentbarhoverbefore-easeout sx-accentbarhoverhoverbefore-opacity100 sx-minW0 sx-overflowhidden sx-shadowlg\"><div class=\"sx-mt4 sx-mx5 sx-mb3 sx-padX5 sx-padY4 sx-borderB1bordersubtle sx-fontFamilyheading sx-empty-displaynone\"></div><div class=\"sail-col sail-g-0 sail-ai-stretch\"><div specimen-chrome=\"\" class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap sx-shrink0 sx-padX3 sx-h8 sx-minH8 sx-minW0 sx-borderBWidth1 sx-borderBbordersubtle sx-bgsurface1\"><div aria-hidden=\"true\" class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap sx-shrink0 sx-minW12\"><div class=\"sail-stackz sail-ai-stretch sail-jc-start sx-shrink0 sx-w2 sx-h2 sx-minW2 sx-minH2 sx-radiusfull sx-bgdestructive\">\n                    </div><div class=\"sail-stackz sail-ai-stretch sail-jc-start sx-shrink0 sx-w2 sx-h2 sx-minW2 sx-minH2 sx-radiusfull sx-bgwarning\">\n                    </div><div class=\"sail-stackz sail-ai-stretch sail-jc-start sx-shrink0 sx-w2 sx-h2 sx-minW2 sx-minH2 sx-radiusfull sx-bgsuccess\">\n                    </div></div><p class=\"sx-grow1 sx-minW0 sx-fgmutedforeground sx-fontSizexs sx-truncate\"> </p><span data-sail-host=\"badge\" tone=\"muted\" size=\"sm\" variant=\"outline\" pill=\"\" class=\"sx-shrink0 sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">preview</span></div><div mode=\"dark\" specimen-body=\"\" class=\"sail-col sail-g-2 sail-ai-stretch sx-pad6 sx-minH32 sx-bgbackground\"><p class=\"sx-fontSizemd sx-fontWeightsemibold sx-fontFamilyheading\"> </p><p class=\"sx-fgmutedforeground sx-fontSizesm sx-leadingrelaxed\"> </p></div></div><div class=\"sx-mt3 sx-mx5 sx-mb4 sx-padX5 sx-padY4 sx-borderT1bordersubtle sx-empty-displaynone\"></div></div>");
const __tpl16 = /*@__PURE__*/ __tpl("<div data-sail-host=\"tab-list\" role=\"tablist\" variant=\"line\" orientation=\"horizontal\" class=\"sail-tab-list sail-row sail-g-1 sail-ai-stretch sail-jc-start sx-borderBWidth1 sx-bordertablistborder sx-bgtablist sx-variantline-borderBWidth1 sx-variantline-bordertablistborder sx-variantpill-pad1 sx-variantpill-borderWidth0 sx-variantpill-radiusmd sx-variantpill-bgtablistrail sx-variantpill-borderBWidth0 sx-variantenclosed-borderBWidth1 sx-variantenclosed-bordertablistborder sx-stickytrue-positionsticky sx-stickytrue-top0 sx-stickytrue-zsticky sx-orientationvertical-borderBWidth0 sx-orientationvertical-borderEWidth1 sx-orientationvertical-bordertablistborder sx-orientationverticalvariantpill-borderEWidth0\"><button type=\"button\" data-sail-host=\"tab\" value=\"overview\" variant=\"line\" size=\"md\" class=\"sail-tab sx-fontFamilybody sx-fontWeightmedium sx-leadingnone sx-fgtabforeground sx-bgtransparent sx-cursorpointer sx-whitespacenowrap sx-selectnone sx-fontSizesm sx-padY2 sx-padX3 sx-borderWidth0 sx-borderBWidth2 sx-borderBtransparent sx-radiusTsm sx-tapHighlightnone sx-transitioncolors sx-durationfast sx-easeout sx-sizemd-fontSizesm sx-sizemd-padY2 sx-sizemd-padX3 sx-sizesm-fontSizexs sx-sizesm-padY15 sx-sizesm-padX25 sx-sizelg-fontSizebase sx-sizelg-padY25 sx-sizelg-padX4 sx-variantline-borderBtransparent sx-variantpill-borderBWidth0 sx-variantpill-radiussm sx-variantenclosed-borderWidth1 sx-variantenclosed-bordertransparent sx-variantenclosed-borderBWidth0 sx-variantenclosed-radiusTmd sx-hover-fgforeground sx-selected-fgtabselectedforeground sx-selected-fontWeightsemibold sx-selected-borderBprimary sx-focus-outlinesolid sx-focus-outlineringwidth sx-focus-outlinering sx-focus-outlineOffsetringinset sx-disabled-opacity50 sx-disabled-cursornotallowed sx-variantpillselected-fgtabselectedforeground sx-variantpillselected-bgtabselected sx-variantpillselected-shadowxs sx-variantpillselected-borderBtransparent sx-variantenclosedselected-fgtabselectedforeground sx-variantenclosedselected-borderborder sx-variantenclosedselected-bgtabselected sx-variantenclosedselected-mbpx\">Overview</button><button type=\"button\" data-sail-host=\"tab\" value=\"contract\" variant=\"line\" size=\"md\" class=\"sail-tab sx-fontFamilybody sx-fontWeightmedium sx-leadingnone sx-fgtabforeground sx-bgtransparent sx-cursorpointer sx-whitespacenowrap sx-selectnone sx-fontSizesm sx-padY2 sx-padX3 sx-borderWidth0 sx-borderBWidth2 sx-borderBtransparent sx-radiusTsm sx-tapHighlightnone sx-transitioncolors sx-durationfast sx-easeout sx-sizemd-fontSizesm sx-sizemd-padY2 sx-sizemd-padX3 sx-sizesm-fontSizexs sx-sizesm-padY15 sx-sizesm-padX25 sx-sizelg-fontSizebase sx-sizelg-padY25 sx-sizelg-padX4 sx-variantline-borderBtransparent sx-variantpill-borderBWidth0 sx-variantpill-radiussm sx-variantenclosed-borderWidth1 sx-variantenclosed-bordertransparent sx-variantenclosed-borderBWidth0 sx-variantenclosed-radiusTmd sx-hover-fgforeground sx-selected-fgtabselectedforeground sx-selected-fontWeightsemibold sx-selected-borderBprimary sx-focus-outlinesolid sx-focus-outlineringwidth sx-focus-outlinering sx-focus-outlineOffsetringinset sx-disabled-opacity50 sx-disabled-cursornotallowed sx-variantpillselected-fgtabselectedforeground sx-variantpillselected-bgtabselected sx-variantpillselected-shadowxs sx-variantpillselected-borderBtransparent sx-variantenclosedselected-fgtabselectedforeground sx-variantenclosedselected-borderborder sx-variantenclosedselected-bgtabselected sx-variantenclosedselected-mbpx\">Contract</button></div>");
const __tpl17 = /*@__PURE__*/ __tpl("<div class=\"sail-col sail-g-1 sail-ai-stretch\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                    Intent</p><p detail-intent=\"\" class=\"sx-fontSizesm sx-leadingrelaxed\"> </p></div>");
const __tpl18 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                    Meta</p>");
const __tpl19 = /*@__PURE__*/ __tpl("<div kv-row=\"\" class=\"sail-row sail-g-3 sail-ai-start sail-jc-start sail-nowrap\"><p class=\"sx-shrink0 sx-w24 sx-fgmutedforeground sx-fontSizexs\"> </p><p class=\"sx-grow1 sx-minW0 sx-fontSizesm sx-fontFamilymono sx-truncate\"> </p></div>");
const __tpl20 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                    Composition</p>");
const __tpl21 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontFamilymono\"> </p>");
const __tpl22 = /*@__PURE__*/ __tpl("<p schema-note=\"\" class=\"sx-fgmutedforeground sx-fontSizesm\"> </p>");
const __tpl23 = /*@__PURE__*/ __tpl("<studio-empty title=\"No props in catalog\" size=\"sm\">\n                    Schema incomplete or undeclared attributes.\n                  </studio-empty>");
const __tpl24 = /*@__PURE__*/ __tpl("<div class=\"sail-row sail-g-3 sail-ai-center sail-jc-start sail-nowrap sx-padY1 sx-borderBWidth1 sx-borderBbordersubtle\"><p class=\"sx-grow1 sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                        Name</p><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                        Type</p><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n                        Default</p></div>");
const __tpl25 = /*@__PURE__*/ __tpl("<div prop-row=\"\" class=\"sail-row sail-g-3 sail-ai-start sail-jc-start sail-nowrap sx-padY2 sx-borderBWidth1 sx-borderBbordersubtle\"><div class=\"sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minW0\"><p class=\"sx-fontSizesm sx-fontFamilymono\"> </p><p class=\"sx-fgmutedforeground sx-fontSizexs\"> </p></div><p class=\"sx-shrink0 sx-fgmutedforeground sx-fontSizexs sx-fontFamilymono\"> </p><p class=\"sx-shrink0 sx-fgmutedforeground sx-fontSizexs sx-fontFamilymono\"> </p></div>");
const __tpl26 = /*@__PURE__*/ __tpl("<div data-sail-host=\"toast\" role=\"status\" aria-live=\"polite\" components-toast=\"\" class=\"sail-toast sail-row sail-g-3 sail-ai-start sail-jc-start sx-fontFamilybody sx-fontSizesm sx-fgtoastforeground sx-bgtoast sx-radiusmd sx-shadowmd sx-padY3 sx-padX4 sx-borderWidth1 sx-bordertoastborder sx-borderSWidth3 sx-borderSforeground sx-pointerEventsauto sx-tonedefault-borderSforeground sx-toneprimary-borderSprimary sx-toneinfo-borderSinfo sx-tonesuccess-borderSsuccess sx-tonewarning-borderSwarning sx-tonedestructive-borderSdestructive sx-toneaccent-borderSaccent sx-tonemuted-borderSmutedforeground\"> <button data-toast-close=\"\" type=\"button\" anatomy=\"\" aria-label=\"Dismiss\" class=\"sx-shrink0 sx-grow0 sx-msauto sx-fgmutedforeground sx-bgtransparent sx-borderWidth0 sx-radiussm sx-cursorpointer sx-hover-fgforeground sx-hover-bgmuted\">\n    ✕</button></div>");
delegateEvents([
	"change",
	"click",
	"input"
]);
const __default = Object.assign(component((ctx) => {
	const { route, navigate, providers } = ctx;
	const $scope = Object.create(null);
	$scope.route = route;
	$scope.navigate = navigate;
	for (const __k of Object.keys(ctx)) {
		if (__k === "route" || __k === "navigate" || __k === "scope" || __k === "stateKeys" || __k === "host" || __k === "providers" || __k === "props") continue;
		$scope[__k] = ctx[__k];
	}
	if (providers && typeof providers === "object") Object.assign($scope, providers);
	if (components_surface_sail_controller_default && typeof components_surface_sail_controller_default.setup === "function") {
		ctx.scope = $scope;
		const __r = components_surface_sail_controller_default.setup(ctx);
		if (__r && typeof __r === "object") Object.assign($scope, __r);
	}
	const __wireBoundary_b4 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.library.isLoading()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag9 = document.createDocumentFragment();
			const __c10 = [];
			const __t11 = document.createTextNode("\n        ");
			__frag9.appendChild(__t11);
			const __e12 = __tpl1();
			const __t14 = __e12.firstChild.nextSibling.firstChild;
			markHydrateText(__t14, "b5");
			__c10.push(bindText(__t14, () => $scope.library.loadingLabel()));
			__frag9.appendChild(__e12);
			const __t15 = document.createTextNode("\n      ");
			__frag9.appendChild(__t15);
			return {
				nodes: Array.from(__frag9.childNodes),
				cleanup: () => {
					for (const c of __c10) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b8 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.library.emptyProject()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag23 = document.createDocumentFragment();
			const __c24 = [];
			const __t25 = document.createTextNode("\n              ");
			__frag23.appendChild(__t25);
			const __e26 = __tpl3();
			const __e27 = __e26.firstChild.nextSibling;
			__e27.$$click = (function() {
				const __h = $scope.session.onNewOpen;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__frag23.appendChild(__e26);
			const __t28 = document.createTextNode("\n            ");
			__frag23.appendChild(__t28);
			return {
				nodes: Array.from(__frag23.childNodes),
				cleanup: () => {
					for (const c of __c24) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b10 = (anchor) => bindFor({
		anchor,
		items: () => $scope.library.projectRows(),
		key: (row, __i) => row.key,
		render: (row, __i) => {
			const __childScope30 = Object.create($scope);
			__childScope30.row = row;
			__childScope30.__i = __i;
			return (($scope) => {
				const __frag31 = document.createDocumentFragment();
				const __c32 = [];
				const __t33 = document.createTextNode("\n              ");
				__frag31.appendChild(__t33);
				const __e34 = __tpl4();
				{
					const __av35 = $scope.row.id;
					if (__av35 == null || __av35 === false || __av35 === "") __e34.removeAttribute("component-id");
					else __e34.setAttribute("component-id", __av35 === true ? "" : String(__av35));
				}
				{
					const __av36 = $scope.row.title;
					if (__av36 == null || __av36 === false || __av36 === "") __e34.removeAttribute("title");
					else __e34.setAttribute("title", __av36 === true ? "" : String(__av36));
				}
				{
					const __av37 = $scope.row.meta;
					if (__av37 == null || __av37 === false || __av37 === "") __e34.removeAttribute("meta");
					else __e34.setAttribute("meta", __av37 === true ? "" : String(__av37));
				}
				{
					const __av38 = $scope.row.selected;
					if (__av38 == null || __av38 === false || __av38 === "") __e34.removeAttribute("selected");
					else __e34.setAttribute("selected", __av38 === true ? "" : String(__av38));
					if (__av38 == null || __av38 === false || __av38 === "") __e34.removeAttribute("aria-selected");
					else __e34.setAttribute("aria-selected", __av38 === true ? "true" : String(__av38));
				}
				const __e39 = __e34.firstChild;
				{
					const __av40 = $scope.row.originTone;
					if (__av40 == null || __av40 === false || __av40 === "") __e39.removeAttribute("tone");
					else __e39.setAttribute("tone", __av40 === true ? "" : String(__av40));
				}
				const __t41 = __e39.firstChild;
				markHydrateText(__t41, "b16");
				__t41.textContent = $scope.row.originLabel;
				__frag31.appendChild(__e34);
				const __t42 = document.createTextNode("\n            ");
				__frag31.appendChild(__t42);
				return {
					nodes: Array.from(__frag31.childNodes),
					cleanup: () => {
						for (const c of __c32) try {
							c();
						} catch (e) {}
					},
					update: (__new43, __newI44) => {
						const __old = $scope.row;
						$scope.row = __new43;
						$scope.__i = __newI44;
						if (__old.id !== $scope.row.id) {
							const __av35 = $scope.row.id;
							if (__av35 == null || __av35 === false || __av35 === "") __e34.removeAttribute("component-id");
							else __e34.setAttribute("component-id", __av35 === true ? "" : String(__av35));
						}
						if (__old.title !== $scope.row.title) {
							const __av36 = $scope.row.title;
							if (__av36 == null || __av36 === false || __av36 === "") __e34.removeAttribute("title");
							else __e34.setAttribute("title", __av36 === true ? "" : String(__av36));
						}
						if (__old.meta !== $scope.row.meta) {
							const __av37 = $scope.row.meta;
							if (__av37 == null || __av37 === false || __av37 === "") __e34.removeAttribute("meta");
							else __e34.setAttribute("meta", __av37 === true ? "" : String(__av37));
						}
						if (__old.selected !== $scope.row.selected) {
							const __av38 = $scope.row.selected;
							if (__av38 == null || __av38 === false || __av38 === "") __e34.removeAttribute("selected");
							else __e34.setAttribute("selected", __av38 === true ? "" : String(__av38));
							if (__av38 == null || __av38 === false || __av38 === "") __e34.removeAttribute("aria-selected");
							else __e34.setAttribute("aria-selected", __av38 === true ? "true" : String(__av38));
						}
						if (__old.originTone !== $scope.row.originTone) {
							const __av40 = $scope.row.originTone;
							if (__av40 == null || __av40 === false || __av40 === "") __e39.removeAttribute("tone");
							else __e39.setAttribute("tone", __av40 === true ? "" : String(__av40));
						}
						if (__old.originLabel !== $scope.row.originLabel) __t41.textContent = $scope.row.originLabel;
					}
				};
			})(__childScope30);
		}
	});
	const __wireBoundary_b18 = (anchor) => bindIf({
		anchor,
		test: () => {
			if (!$scope.library.hasPackRows()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag50 = document.createDocumentFragment();
			const __c51 = [];
			const __t52 = document.createTextNode("\n              ");
			__frag50.appendChild(__t52);
			const __e53 = __tpl6();
			__frag50.appendChild(__e53);
			const __t54 = document.createTextNode("\n            ");
			__frag50.appendChild(__t54);
			return {
				nodes: Array.from(__frag50.childNodes),
				cleanup: () => {
					for (const c of __c51) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b19 = (anchor) => bindFor({
		anchor,
		items: () => $scope.library.packRows(),
		key: (row, __i) => row.key,
		render: (row, __i) => {
			const __childScope56 = Object.create($scope);
			__childScope56.row = row;
			__childScope56.__i = __i;
			return (($scope) => {
				const __frag57 = document.createDocumentFragment();
				const __c58 = [];
				const __t59 = document.createTextNode("\n              ");
				__frag57.appendChild(__t59);
				const __e60 = __tpl7();
				{
					const __av61 = $scope.row.id;
					if (__av61 == null || __av61 === false || __av61 === "") __e60.removeAttribute("component-id");
					else __e60.setAttribute("component-id", __av61 === true ? "" : String(__av61));
				}
				{
					const __av62 = $scope.row.title;
					if (__av62 == null || __av62 === false || __av62 === "") __e60.removeAttribute("title");
					else __e60.setAttribute("title", __av62 === true ? "" : String(__av62));
				}
				{
					const __av63 = $scope.row.meta;
					if (__av63 == null || __av63 === false || __av63 === "") __e60.removeAttribute("meta");
					else __e60.setAttribute("meta", __av63 === true ? "" : String(__av63));
				}
				{
					const __av64 = $scope.row.selected;
					if (__av64 == null || __av64 === false || __av64 === "") __e60.removeAttribute("selected");
					else __e60.setAttribute("selected", __av64 === true ? "" : String(__av64));
					if (__av64 == null || __av64 === false || __av64 === "") __e60.removeAttribute("aria-selected");
					else __e60.setAttribute("aria-selected", __av64 === true ? "true" : String(__av64));
				}
				__frag57.appendChild(__e60);
				const __t65 = document.createTextNode("\n            ");
				__frag57.appendChild(__t65);
				return {
					nodes: Array.from(__frag57.childNodes),
					cleanup: () => {
						for (const c of __c58) try {
							c();
						} catch (e) {}
					},
					update: (__new66, __newI67) => {
						const __old = $scope.row;
						$scope.row = __new66;
						$scope.__i = __newI67;
						if (__old.id !== $scope.row.id) {
							const __av61 = $scope.row.id;
							if (__av61 == null || __av61 === false || __av61 === "") __e60.removeAttribute("component-id");
							else __e60.setAttribute("component-id", __av61 === true ? "" : String(__av61));
						}
						if (__old.title !== $scope.row.title) {
							const __av62 = $scope.row.title;
							if (__av62 == null || __av62 === false || __av62 === "") __e60.removeAttribute("title");
							else __e60.setAttribute("title", __av62 === true ? "" : String(__av62));
						}
						if (__old.meta !== $scope.row.meta) {
							const __av63 = $scope.row.meta;
							if (__av63 == null || __av63 === false || __av63 === "") __e60.removeAttribute("meta");
							else __e60.setAttribute("meta", __av63 === true ? "" : String(__av63));
						}
						if (__old.selected !== $scope.row.selected) {
							const __av64 = $scope.row.selected;
							if (__av64 == null || __av64 === false || __av64 === "") __e60.removeAttribute("selected");
							else __e60.setAttribute("selected", __av64 === true ? "" : String(__av64));
							if (__av64 == null || __av64 === false || __av64 === "") __e60.removeAttribute("aria-selected");
							else __e60.setAttribute("aria-selected", __av64 === true ? "true" : String(__av64));
						}
					}
				};
			})(__childScope56);
		}
	});
	const __wireBoundary_b27 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.library.hasSelected()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag80 = document.createDocumentFragment();
			const __c81 = [];
			const __t82 = document.createTextNode("\n                ");
			__frag80.appendChild(__t82);
			const __e83 = __tpl11();
			__c81.push(bindAttr(__e83, "tone", () => $scope.library.detailOriginTone()));
			const __t84 = __e83.firstChild;
			markHydrateText(__t84, "b29");
			__c81.push(bindText(__t84, () => $scope.library.detailOriginLabel()));
			__frag80.appendChild(__e83);
			const __t85 = document.createTextNode("\n                ");
			__frag80.appendChild(__t85);
			const __e86 = __tpl12();
			const __t87 = __e86.firstChild;
			markHydrateText(__t87, "b30");
			__c81.push(bindText(__t87, () => $scope.library.detailLayer()));
			__frag80.appendChild(__e86);
			const __t88 = document.createTextNode("\n              ");
			__frag80.appendChild(__t88);
			return {
				nodes: Array.from(__frag80.childNodes),
				cleanup: () => {
					for (const c of __c81) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b40 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.session.showNewForm()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag96 = document.createDocumentFragment();
			const __c97 = [];
			const __t98 = document.createTextNode("\n            ");
			__frag96.appendChild(__t98);
			const __e99 = __tpl14();
			const __e101 = __e99.firstChild.nextSibling.firstChild.nextSibling.nextSibling;
			runAction(__e101, describedControl, void 0);
			const __e103 = __e101.firstChild.nextSibling.firstChild;
			__c97.push(bindProp(__e103, "value", () => $scope.session.newName()));
			__e103.$$input = (function() {
				const __h = $scope.session.onNewNameInput;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e104 = __e101.nextSibling;
			runAction(__e104, describedControl, void 0);
			const __e106 = __e104.firstChild.nextSibling.firstChild;
			__c97.push(bindProp(__e106, "value", () => $scope.session.newIntent()));
			__e106.$$input = (function() {
				const __h = $scope.session.onNewIntentInput;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e107 = __e104.nextSibling;
			runAction(__e107, describedControl, void 0);
			const __e109 = __e107.firstChild.nextSibling.firstChild;
			__c97.push(bindProp(__e109, "value", () => $scope.session.newLayer()));
			__e109.$$change = (function() {
				const __h = $scope.session.onNewLayerChange;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e111 = __e107.nextSibling.firstChild;
			__e111.$$click = (function() {
				const __h = $scope.session.onCreateBrief;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e112 = __e111.nextSibling;
			__e112.$$click = (function() {
				const __h = $scope.session.onStartThreadFromNew;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__frag96.appendChild(__e99);
			const __t113 = document.createTextNode("\n          ");
			__frag96.appendChild(__t113);
			return {
				nodes: Array.from(__frag96.childNodes),
				cleanup: () => {
					for (const c of __c97) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b53 = (anchor) => bindFor({
		anchor,
		items: () => $scope.library.kvRows(),
		key: (kv, __i) => kv.key,
		render: (kv, __i) => {
			const __childScope134 = Object.create($scope);
			__childScope134.kv = kv;
			__childScope134.__i = __i;
			return (($scope) => {
				const __frag135 = document.createDocumentFragment();
				const __c136 = [];
				const __t137 = document.createTextNode("\n                    ");
				__frag135.appendChild(__t137);
				const __e138 = __tpl19();
				const __e139 = __e138.firstChild;
				const __t140 = __e139.firstChild;
				markHydrateText(__t140, "b54");
				__t140.textContent = $scope.kv.label;
				const __t142 = __e139.nextSibling.firstChild;
				markHydrateText(__t142, "b55");
				__t142.textContent = $scope.kv.value;
				__frag135.appendChild(__e138);
				const __t143 = document.createTextNode("\n                  ");
				__frag135.appendChild(__t143);
				return {
					nodes: Array.from(__frag135.childNodes),
					cleanup: () => {
						for (const c of __c136) try {
							c();
						} catch (e) {}
					},
					update: (__new144, __newI145) => {
						const __old = $scope.kv;
						$scope.kv = __new144;
						$scope.__i = __newI145;
						if (__old.label !== $scope.kv.label) __t140.textContent = $scope.kv.label;
						if (__old.value !== $scope.kv.value) __t142.textContent = $scope.kv.value;
					}
				};
			})(__childScope134);
		}
	});
	const __wireBoundary_b58 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.library.schemaNote()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag156 = document.createDocumentFragment();
			const __c157 = [];
			const __t158 = document.createTextNode("\n                  ");
			__frag156.appendChild(__t158);
			const __e159 = __tpl22();
			const __t160 = __e159.firstChild;
			markHydrateText(__t160, "b59");
			__c157.push(bindText(__t160, () => $scope.library.schemaNote()));
			__frag156.appendChild(__e159);
			const __t161 = document.createTextNode("\n                ");
			__frag156.appendChild(__t161);
			return {
				nodes: Array.from(__frag156.childNodes),
				cleanup: () => {
					for (const c of __c157) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b60 = (anchor) => bindIf({
		anchor,
		test: () => {
			if (!$scope.library.hasProps()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag163 = document.createDocumentFragment();
			const __c164 = [];
			const __t165 = document.createTextNode("\n                  ");
			__frag163.appendChild(__t165);
			const __e166 = __tpl23();
			__frag163.appendChild(__e166);
			const __t167 = document.createTextNode("\n                ");
			__frag163.appendChild(__t167);
			return {
				nodes: Array.from(__frag163.childNodes),
				cleanup: () => {
					for (const c of __c164) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b62 = (anchor) => bindFor({
		anchor,
		items: () => $scope.library.propRows(),
		key: (prop, __i) => prop.key,
		render: (prop, __i) => {
			const __childScope175 = Object.create($scope);
			__childScope175.prop = prop;
			__childScope175.__i = __i;
			return (($scope) => {
				const __frag176 = document.createDocumentFragment();
				const __c177 = [];
				const __t178 = document.createTextNode("\n                      ");
				__frag176.appendChild(__t178);
				const __e179 = __tpl25();
				const __e180 = __e179.firstChild;
				const __e181 = __e180.firstChild;
				const __t182 = __e181.firstChild;
				markHydrateText(__t182, "b63");
				__t182.textContent = $scope.prop.name;
				const __t184 = __e181.nextSibling.firstChild;
				markHydrateText(__t184, "b64");
				__t184.textContent = $scope.prop.requiredLabel;
				const __e185 = __e180.nextSibling;
				const __t186 = __e185.firstChild;
				markHydrateText(__t186, "b65");
				__t186.textContent = $scope.prop.type;
				const __t188 = __e185.nextSibling.firstChild;
				markHydrateText(__t188, "b66");
				__t188.textContent = $scope.prop.defaultValue;
				__frag176.appendChild(__e179);
				const __t189 = document.createTextNode("\n                    ");
				__frag176.appendChild(__t189);
				return {
					nodes: Array.from(__frag176.childNodes),
					cleanup: () => {
						for (const c of __c177) try {
							c();
						} catch (e) {}
					},
					update: (__new190, __newI191) => {
						const __old = $scope.prop;
						$scope.prop = __new190;
						$scope.__i = __newI191;
						if (__old.name !== $scope.prop.name) __t182.textContent = $scope.prop.name;
						if (__old.requiredLabel !== $scope.prop.requiredLabel) __t184.textContent = $scope.prop.requiredLabel;
						if (__old.type !== $scope.prop.type) __t186.textContent = $scope.prop.type;
						if (__old.defaultValue !== $scope.prop.defaultValue) __t188.textContent = $scope.prop.defaultValue;
					}
				};
			})(__childScope175);
		}
	});
	const __wireBoundary_b61 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.library.hasProps()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag169 = document.createDocumentFragment();
			const __c170 = [];
			const __t171 = document.createTextNode("\n                  ");
			__frag169.appendChild(__t171);
			const __e172 = document.createElement("div");
			__e172.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch");
			__e172.setAttribute("props-table", "");
			const __e173 = __tpl24();
			__e172.appendChild(__e173);
			const __forAnchor174 = document.createComment("for");
			__e172.appendChild(__forAnchor174);
			__c170.push(__wireBoundary_b62(__forAnchor174));
			__frag169.appendChild(__e172);
			const __t192 = document.createTextNode("\n                ");
			__frag169.appendChild(__t192);
			return {
				nodes: Array.from(__frag169.childNodes),
				cleanup: () => {
					for (const c of __c170) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b67 = (anchor) => bindFor({
		anchor,
		items: () => $scope.session.toastItems(),
		key: (t, __i) => t.key,
		render: (t, __i) => {
			const __childScope195 = Object.create($scope);
			__childScope195.t = t;
			__childScope195.__i = __i;
			return (($scope) => {
				const __frag196 = document.createDocumentFragment();
				const __c197 = [];
				const __t198 = document.createTextNode("\n      ");
				__frag196.appendChild(__t198);
				const __e199 = __tpl26();
				runAction(__e199, autoDismiss, void 0);
				runAction(__e199, dismissControl, void 0);
				{
					const __av200 = $scope.t.tone;
					if (__av200 == null || __av200 === false || __av200 === "") __e199.removeAttribute("tone");
					else __e199.setAttribute("tone", __av200 === true ? "" : String(__av200));
				}
				__c197.push(on(__e199, "sail-toast-dismiss", (function() {
					const __h = $scope.session.onToastDismiss;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})(), { modifiers: [] }));
				const __t201 = __e199.firstChild;
				markHydrateText(__t201, "b70");
				__t201.textContent = $scope.t.message;
				__frag196.appendChild(__e199);
				const __t202 = document.createTextNode("\n    ");
				__frag196.appendChild(__t202);
				return {
					nodes: Array.from(__frag196.childNodes),
					cleanup: () => {
						for (const c of __c197) try {
							c();
						} catch (e) {}
					},
					update: (__new203, __newI204) => {
						const __old = $scope.t;
						$scope.t = __new203;
						$scope.__i = __newI204;
						if (__old.tone !== $scope.t.tone) {
							const __av200 = $scope.t.tone;
							if (__av200 == null || __av200 === false || __av200 === "") __e199.removeAttribute("tone");
							else __e199.setAttribute("tone", __av200 === true ? "" : String(__av200));
						}
						if (__old.message !== $scope.t.message) __t201.textContent = $scope.t.message;
					}
				};
			})(__childScope195);
		}
	});
	return Object.assign({}, $scope, {
		__mount(container) {
			const __cleanups = [];
			const __root = container.shadowRoot ?? container;
			const __e0 = document.createElement("div");
			__e0.setAttribute("class", "sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minH0 sx-hfull sx-minW0 sx-overflowhidden sx-bgbackground");
			__e0.setAttribute("surface", "components");
			__e0.setAttribute("components-surface", "");
			__e0.setAttribute("components-library", "");
			__e0.classList.add("sail-root");
			const __e1 = document.createElement("div");
			__e1.setAttribute("class", "sail-row sail-g-0 sail-ai-stretch sail-jc-start sail-nowrap sx-grow1 sx-minH0 sx-basis0 sx-hfull sx-minW0 sx-overflowhidden");
			__e1.setAttribute("components-workbench", "");
			const __e2 = document.createElement("div");
			__e2.setAttribute("class", "sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minH0 sx-hfull sx-minWpanelminwmd sx-maxWpanelmaxwmd sx-basis0 sx-overflowhidden sx-borderRWidth1 sx-borderRbordersubtle sx-bgsurface0");
			__e2.setAttribute("components-nav", "");
			const __e3 = __tpl0$1();
			const __e4 = __e3.firstChild;
			const __e5 = __e4.firstChild.nextSibling;
			__e5.$$click = (function() {
				const __h = $scope.session.onNewOpen;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e6 = __e4.nextSibling;
			__e6.$$click = (function() {
				const __h = $scope.session.onModeToggle;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e7 = __e6.nextSibling;
			__cleanups.push(bindProp(__e7, "value", () => $scope.session.filterQuery()));
			__e7.$$input = (function() {
				const __h = $scope.session.onFilterInput;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__e2.appendChild(__e3);
			const __ifAnchor8 = document.createComment("if");
			__e2.appendChild(__ifAnchor8);
			__cleanups.push(__wireBoundary_b4(__ifAnchor8));
			const __e16 = document.createElement("div");
			__e16.setAttribute("class", "sx-grow1 sail-scroll-area sx-overscrollcontain sx-scrollbarThin sx-scrollbarThumbscrollareathumb sx-scrollbarTracktransparent sx-orientationvertical-overflowXhidden sx-orientationvertical-overflowYauto sx-orientationhorizontal-overflowXauto sx-orientationhorizontal-overflowYhidden sx-orientationboth-overflowauto sx-hover-scrollbarThumbscrollareathumbhover sx-minH0 sx-basis0 sx-hfull");
			__e16.setAttribute("data-sail-host", "scroll-area");
			__e16.setAttribute("components-list", "");
			__e16.$$click = (function() {
				const __h = $scope.library.onListClick;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__e16.setAttribute("orientation", "vertical");
			const __e17 = document.createElement("div");
			__e17.setAttribute("class", "sail-col sail-g-3 sail-ai-stretch sx-pad3");
			const __e18 = document.createElement("div");
			__e18.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch");
			__e18.setAttribute("list-group", "project");
			const __e19 = __tpl2();
			const __t21 = __e19.firstChild.nextSibling.firstChild;
			markHydrateText(__t21, "b7");
			__cleanups.push(bindText(__t21, () => $scope.library.projectCountLabel()));
			__e18.appendChild(__e19);
			const __ifAnchor22 = document.createComment("if");
			__e18.appendChild(__ifAnchor22);
			__cleanups.push(__wireBoundary_b8(__ifAnchor22));
			const __forAnchor29 = document.createComment("for");
			__e18.appendChild(__forAnchor29);
			__cleanups.push(__wireBoundary_b10(__forAnchor29));
			__e17.appendChild(__e18);
			const __e45 = document.createElement("div");
			__e45.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch");
			__e45.setAttribute("list-group", "packs");
			const __e46 = __tpl5();
			const __t48 = __e46.firstChild.nextSibling.firstChild;
			markHydrateText(__t48, "b17");
			__cleanups.push(bindText(__t48, () => $scope.library.packCountLabel()));
			__e45.appendChild(__e46);
			const __ifAnchor49 = document.createComment("if");
			__e45.appendChild(__ifAnchor49);
			__cleanups.push(__wireBoundary_b18(__ifAnchor49));
			const __forAnchor55 = document.createComment("for");
			__e45.appendChild(__forAnchor55);
			__cleanups.push(__wireBoundary_b19(__forAnchor55));
			__e17.appendChild(__e45);
			const __e68 = __tpl8();
			const __e69 = __e68.firstChild.nextSibling;
			__e69.$$click = (function() {
				const __h = $scope.session.goThemesComponents;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__e17.appendChild(__e68);
			__e16.appendChild(__e17);
			__e2.appendChild(__e16);
			__e1.appendChild(__e2);
			const __e70 = document.createElement("div");
			__e70.setAttribute("class", "sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minH0 sx-hfull sx-minW0 sx-basis0 sx-overflowhidden sx-bgbackground");
			__e70.setAttribute("components-detail", "");
			const __e71 = document.createElement("div");
			__e71.setAttribute("class", "sail-col sail-g-3 sail-ai-stretch sx-shrink0 sx-padX5 sx-padY4 sx-borderBWidth1 sx-borderBbordersubtle sx-bgsurface0");
			__e71.setAttribute("components-detail-head", "");
			const __e72 = document.createElement("div");
			__e72.setAttribute("class", "sail-row sail-g-3 sail-ai-start sail-jc-between sail-nowrap");
			const __e73 = document.createElement("div");
			__e73.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0");
			const __e74 = __tpl9();
			const __t75 = __e74.firstChild;
			markHydrateText(__t75, "b25");
			__cleanups.push(bindText(__t75, () => $scope.library.detailTitle()));
			__e73.appendChild(__e74);
			const __e76 = __tpl10();
			const __t77 = __e76.firstChild;
			markHydrateText(__t77, "b26");
			__cleanups.push(bindText(__t77, () => $scope.library.detailPath()));
			__e73.appendChild(__e76);
			const __e78 = document.createElement("div");
			__e78.setAttribute("class", "sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap");
			const __ifAnchor79 = document.createComment("if");
			__e78.appendChild(__ifAnchor79);
			__cleanups.push(__wireBoundary_b27(__ifAnchor79));
			__e73.appendChild(__e78);
			__e72.appendChild(__e73);
			const __e89 = __tpl13();
			const __e90 = __e89.firstChild;
			__cleanups.push(bindProp(__e90, "disabled", () => !$scope.library.canOpenInEdit()));
			__cleanups.push(bindAttr(__e90, "title", () => $scope.library.canOpenInEdit() ? "Open this component source in Edit" : "No project path to open in Edit"));
			__e90.$$click = (function() {
				const __h = $scope.library.onOpenInEdit;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e91 = __e90.nextSibling;
			__cleanups.push(bindProp(__e91, "disabled", () => !$scope.library.hasSelected()));
			__cleanups.push(bindAttr(__e91, "title", () => $scope.library.hasSelected() ? "Start a Work thread for this component" : "Select a component first"));
			__e91.$$click = (function() {
				const __h = $scope.library.onStartThread;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e92 = __e91.nextSibling;
			__cleanups.push(bindProp(__e92, "disabled", () => !$scope.library.hasSelected()));
			__cleanups.push(bindAttr(__e92, "title", () => $scope.library.hasSelected() ? "Copy a structured brief for an agent" : "Select a component first"));
			__e92.$$click = (function() {
				const __h = $scope.library.onCopyBrief;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__e72.appendChild(__e89);
			__e71.appendChild(__e72);
			__e70.appendChild(__e71);
			const __e93 = document.createElement("div");
			__e93.setAttribute("class", "sx-grow1 sail-scroll-area sx-overscrollcontain sx-scrollbarThin sx-scrollbarThumbscrollareathumb sx-scrollbarTracktransparent sx-orientationvertical-overflowXhidden sx-orientationvertical-overflowYauto sx-orientationhorizontal-overflowXauto sx-orientationhorizontal-overflowYhidden sx-orientationboth-overflowauto sx-hover-scrollbarThumbscrollareathumbhover sx-minH0 sx-basis0 sx-hfull");
			__e93.setAttribute("data-sail-host", "scroll-area");
			__e93.setAttribute("components-detail-body", "");
			__e93.setAttribute("orientation", "vertical");
			const __e94 = document.createElement("div");
			__e94.setAttribute("class", "sail-col sail-g-4 sail-ai-stretch sx-pad5 sx-minW0");
			const __ifAnchor95 = document.createComment("if");
			__e94.appendChild(__ifAnchor95);
			__cleanups.push(__wireBoundary_b40(__ifAnchor95));
			const __e114 = __tpl15();
			const __e116 = __e114.firstChild.nextSibling.firstChild;
			const __t118 = __e116.firstChild.nextSibling.firstChild;
			markHydrateText(__t118, "b49");
			__cleanups.push(bindText(__t118, () => $scope.library.previewTitle()));
			const __e120 = __e116.nextSibling.firstChild;
			const __t121 = __e120.firstChild;
			markHydrateText(__t121, "b50");
			__cleanups.push(bindText(__t121, () => $scope.library.previewTitle()));
			const __t123 = __e120.nextSibling.firstChild;
			markHydrateText(__t123, "b51");
			__cleanups.push(bindText(__t123, () => $scope.library.previewHint()));
			__e94.appendChild(__e114);
			const __e124 = document.createElement("div");
			__e124.setAttribute("class", "sail-tabs sail-col sail-g-3 sail-ai-stretch sx-minW0");
			runAction(__e124, tabs, void 0);
			__e124.setAttribute("data-sail-host", "tabs");
			__e124.setAttribute("components-tabs", "");
			__e124.setAttribute("default-value", "overview");
			__e124.setAttribute("size", "md");
			__e124.setAttribute("tone", "default");
			__e124.setAttribute("orientation", "horizontal");
			const __e125 = __tpl16();
			__e124.appendChild(__e125);
			const __e126 = document.createElement("div");
			__e126.setAttribute("class", "sail-tab-panel sx-minW0 sx-orientationhorizontal-padT4 sx-orientationhorizontal-minW0 sx-orientationvertical-padT0 sx-orientationvertical-padS4 sx-orientationvertical-minW0");
			__e126.setAttribute("data-sail-host", "tab-panel");
			__e126.setAttribute("value", "overview");
			__e126.setAttribute("orientation", "horizontal");
			const __e127 = document.createElement("div");
			__e127.setAttribute("class", "sail-col sail-g-3 sail-ai-stretch sx-padY3");
			const __e128 = __tpl17();
			const __t130 = __e128.firstChild.nextSibling.firstChild;
			markHydrateText(__t130, "b52");
			__cleanups.push(bindText(__t130, () => $scope.library.detailIntent()));
			__e127.appendChild(__e128);
			const __e131 = document.createElement("div");
			__e131.setAttribute("class", "sail-col sail-g-2 sail-ai-stretch");
			const __e132 = __tpl18();
			__e131.appendChild(__e132);
			const __forAnchor133 = document.createComment("for");
			__e131.appendChild(__forAnchor133);
			__cleanups.push(__wireBoundary_b53(__forAnchor133));
			__e127.appendChild(__e131);
			const __e146 = document.createElement("div");
			__e146.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch");
			const __e147 = __tpl20();
			__e146.appendChild(__e147);
			const __e148 = __tpl21();
			const __t149 = __e148.firstChild;
			markHydrateText(__t149, "b56");
			__cleanups.push(bindText(__t149, () => $scope.library.compositionLabel()));
			__e146.appendChild(__e148);
			const __e150 = document.createElement("p");
			__e150.setAttribute("class", "sx-fgmutedforeground sx-fontSizesm");
			const __t151 = document.createTextNode("\n                    Slots: ");
			__e150.appendChild(__t151);
			const __t152 = document.createTextNode("");
			markHydrateText(__t152, "b57");
			__e150.appendChild(__t152);
			__cleanups.push(bindText(__t152, () => $scope.library.slotsLabel()));
			__e146.appendChild(__e150);
			__e127.appendChild(__e146);
			__e126.appendChild(__e127);
			__e124.appendChild(__e126);
			const __e153 = document.createElement("div");
			__e153.setAttribute("class", "sail-tab-panel sx-minW0 sx-orientationhorizontal-padT4 sx-orientationhorizontal-minW0 sx-orientationvertical-padT0 sx-orientationvertical-padS4 sx-orientationvertical-minW0");
			__e153.setAttribute("data-sail-host", "tab-panel");
			__e153.setAttribute("value", "contract");
			__e153.setAttribute("orientation", "horizontal");
			const __e154 = document.createElement("div");
			__e154.setAttribute("class", "sail-col sail-g-3 sail-ai-stretch sx-padY3");
			const __ifAnchor155 = document.createComment("if");
			__e154.appendChild(__ifAnchor155);
			__cleanups.push(__wireBoundary_b58(__ifAnchor155));
			const __ifAnchor162 = document.createComment("if");
			__e154.appendChild(__ifAnchor162);
			__cleanups.push(__wireBoundary_b60(__ifAnchor162));
			const __ifAnchor168 = document.createComment("if");
			__e154.appendChild(__ifAnchor168);
			__cleanups.push(__wireBoundary_b61(__ifAnchor168));
			__e153.appendChild(__e154);
			__e124.appendChild(__e153);
			__e94.appendChild(__e124);
			__e93.appendChild(__e94);
			__e70.appendChild(__e93);
			__e1.appendChild(__e70);
			__e0.appendChild(__e1);
			const __e193 = document.createElement("div");
			__e193.setAttribute("class", "sail-toast-region sail-col sail-g-2 sail-ai-stretch sx-positionfixed sx-zztoast sx-pad4 sx-pointerEventsnone sx-top0 sx-rightauto sx-bottomauto sx-left0 sx-wpanelwtoast sx-positiontopstart-top0 sx-positiontopstart-rightauto sx-positiontopstart-bottomauto sx-positiontopstart-left0 sx-positiontopend-top0 sx-positiontopend-right0 sx-positiontopend-bottomauto sx-positiontopend-leftauto sx-positionbottomstart-topauto sx-positionbottomstart-rightauto sx-positionbottomstart-bottom0 sx-positionbottomstart-left0 sx-positionbottomstart-flexDircolumnreverse sx-positionbottomend-topauto sx-positionbottomend-right0 sx-positionbottomend-bottom0 sx-positionbottomend-leftauto sx-positionbottomend-flexDircolumnreverse");
			__e193.setAttribute("data-sail-host", "toast-region");
			__e193.setAttribute("position", "bottom-end");
			__e193.setAttribute("components-toast-region", "");
			const __forAnchor194 = document.createComment("for");
			__e193.appendChild(__forAnchor194);
			__cleanups.push(__wireBoundary_b67(__forAnchor194));
			__e0.appendChild(__e193);
			__root.appendChild(__e0);
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		},
		hydrate(root, ctx) {
			const __cleanups = [];
			const __t = collectTextOpens(root);
			const __m = collectElementMarkers(root);
			if (__m["b0"]) __cleanups.push(hydrateOn(__m["b0"], "click", (function() {
				const __h = $scope.session.onNewOpen;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b1"]) __cleanups.push(hydrateOn(__m["b1"], "click", (function() {
				const __h = $scope.session.onModeToggle;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b2"]) __cleanups.push(hydrateProp(__m["b2"], "value", () => $scope.session.filterQuery()));
			if (__m["b3"]) __cleanups.push(hydrateOn(__m["b3"], "input", (function() {
				const __h = $scope.session.onFilterInput;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b4"]) {
				const __t = __m["b4"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b4(__anchor));
			}
			if (__m["b6"]) __cleanups.push(hydrateOn(__m["b6"], "click", (function() {
				const __h = $scope.library.onListClick;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__t["b7"]) __cleanups.push(hydrateText(__t["b7"], () => $scope.library.projectCountLabel()));
			if (__m["b8"]) {
				const __t = __m["b8"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b8(__anchor));
			}
			if (__m["b10"]) {
				const __t = __m["b10"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b10(__anchor));
			}
			if (__t["b17"]) __cleanups.push(hydrateText(__t["b17"], () => $scope.library.packCountLabel()));
			if (__m["b18"]) {
				const __t = __m["b18"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b18(__anchor));
			}
			if (__m["b19"]) {
				const __t = __m["b19"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b19(__anchor));
			}
			if (__m["b24"]) __cleanups.push(hydrateOn(__m["b24"], "click", (function() {
				const __h = $scope.session.goThemesComponents;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__t["b25"]) __cleanups.push(hydrateText(__t["b25"], () => $scope.library.detailTitle()));
			if (__t["b26"]) __cleanups.push(hydrateText(__t["b26"], () => $scope.library.detailPath()));
			if (__m["b27"]) {
				const __t = __m["b27"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b27(__anchor));
			}
			if (__m["b31"]) __cleanups.push(hydrateProp(__m["b31"], "disabled", () => !$scope.library.canOpenInEdit()));
			if (__m["b32"]) __cleanups.push(hydrateAttr(__m["b32"], "title", () => $scope.library.canOpenInEdit() ? "Open this component source in Edit" : "No project path to open in Edit"));
			if (__m["b33"]) __cleanups.push(hydrateOn(__m["b33"], "click", (function() {
				const __h = $scope.library.onOpenInEdit;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b34"]) __cleanups.push(hydrateProp(__m["b34"], "disabled", () => !$scope.library.hasSelected()));
			if (__m["b35"]) __cleanups.push(hydrateAttr(__m["b35"], "title", () => $scope.library.hasSelected() ? "Start a Work thread for this component" : "Select a component first"));
			if (__m["b36"]) __cleanups.push(hydrateOn(__m["b36"], "click", (function() {
				const __h = $scope.library.onStartThread;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b37"]) __cleanups.push(hydrateProp(__m["b37"], "disabled", () => !$scope.library.hasSelected()));
			if (__m["b38"]) __cleanups.push(hydrateAttr(__m["b38"], "title", () => $scope.library.hasSelected() ? "Copy a structured brief for an agent" : "Select a component first"));
			if (__m["b39"]) __cleanups.push(hydrateOn(__m["b39"], "click", (function() {
				const __h = $scope.library.onCopyBrief;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b40"]) {
				const __t = __m["b40"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b40(__anchor));
			}
			if (__t["b49"]) __cleanups.push(hydrateText(__t["b49"], () => $scope.library.previewTitle()));
			if (__t["b50"]) __cleanups.push(hydrateText(__t["b50"], () => $scope.library.previewTitle()));
			if (__t["b51"]) __cleanups.push(hydrateText(__t["b51"], () => $scope.library.previewHint()));
			if (__t["b52"]) __cleanups.push(hydrateText(__t["b52"], () => $scope.library.detailIntent()));
			if (__m["b53"]) {
				const __t = __m["b53"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b53(__anchor));
			}
			if (__t["b56"]) __cleanups.push(hydrateText(__t["b56"], () => $scope.library.compositionLabel()));
			if (__t["b57"]) __cleanups.push(hydrateText(__t["b57"], () => $scope.library.slotsLabel()));
			if (__m["b58"]) {
				const __t = __m["b58"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b58(__anchor));
			}
			if (__m["b60"]) {
				const __t = __m["b60"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b60(__anchor));
			}
			if (__m["b61"]) {
				const __t = __m["b61"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b61(__anchor));
			}
			if (__m["b67"]) {
				const __t = __m["b67"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b67(__anchor));
			}
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		}
	});
}), {
	_compilerVersion: {
		"cargo": "0.1.0",
		"git": "c11a87c8",
		"dirty": true,
		"builtAt": "2026-08-05T21:58:16Z"
	},
	clients: []
});
component("components-surface", { styles: ".sx-minH0{min-height:0}.sx-hfull{height:100%}.sx-minW0{min-width:0}.sx-overflowhidden{overflow:hidden}.sx-bgbackground{background-color:var(--background)}.sx-basis0{flex-basis:0}.sx-minWpanelminwmd{min-width:var(--panel-min-w-md)}.sx-maxWpanelmaxwmd{max-width:var(--panel-max-w-md)}.sx-borderRWidth1{border-right-width:1px}.sx-borderRbordersubtle{border-right-color:var(--border-subtle)}.sx-bgsurface0{background-color:var(--surface-0)}.sx-padX4{padding-inline:var(--space-4)}.sx-padT4{padding-top:var(--space-4)}.sx-padB3{padding-bottom:var(--space-3)}.sx-borderBWidth1{border-bottom-width:1px}.sx-borderBbordersubtle{border-bottom-color:var(--border-subtle)}.sx-fgmutedforeground{color:var(--muted-foreground)}.sx-fontSizexs{font-size:var(--text-xs)}.sx-fontWeightsemibold{font-weight:var(--font-weight-semibold)}.sx-textCaseuppercase{text-transform:uppercase}.sx-trackingwide{letter-spacing:var(--tracking-wide)}.sx-padY2{padding-block:var(--space-2)}.sx-fontSizesm{font-size:var(--text-sm)}.sx-pad3{padding:var(--space-3)}.sx-fontFamilymono{font-family:var(--mono-font)}.sx-padX1{padding-inline:var(--space-1)}.sx-leadingsnug{line-height:var(--leading-snug)}.sx-padX5{padding-inline:var(--space-5)}.sx-padY4{padding-block:var(--space-4)}.sx-fontSizelg{font-size:var(--text-lg)}.sx-trackingtight{letter-spacing:var(--tracking-tight)}.sx-truncate{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.sx-pad5{padding:var(--space-5)}.sx-fontSizemd{font-size:var(--text-md)}.sx-shadowlg{box-shadow:var(--shadow-lg)}.sx-padX3{padding-inline:var(--space-3)}.sx-h8{height:var(--space-8)}.sx-minH8{min-height:var(--space-8)}.sx-bgsurface1{background-color:var(--surface-1)}.sx-minW12{min-width:var(--space-12)}.sx-w2{width:var(--space-2)}.sx-h2{height:var(--space-2)}.sx-minW2{min-width:var(--space-2)}.sx-minH2{min-height:var(--space-2)}.sx-radiusfull{border-radius:var(--radius-full)}.sx-bgdestructive{background-color:var(--destructive)}.sx-bgwarning{background-color:var(--warning)}.sx-bgsuccess{background-color:var(--success)}.sx-pad6{padding:var(--space-6)}.sx-minH32{min-height:var(--space-32)}.sx-fontFamilyheading{font-family:var(--heading-font)}.sx-leadingrelaxed{line-height:var(--leading-relaxed)}.sx-padY3{padding-block:var(--space-3)}.sx-w24{width:var(--space-24)}.sx-padY1{padding-block:var(--space-1)}.sx-shrink0{flex-shrink:0}.sx-grow1{flex-grow:1}.sail-col{display:flex;flex-direction:column;min-width:0}.sail-col>*{min-width:0}.sail-col[popover]:not(:popover-open){display:none!important}.sail-col[popover]:popover-open{display:flex;flex-direction:column;min-width:0}.sail-g-0{gap:0}.sail-ai-stretch{align-items:stretch}.sail-row{display:flex;flex-wrap:wrap;min-width:0}.sail-row>*{min-width:0}.sail-row[popover]:not(:popover-open){display:none!important}.sail-row[popover]:popover-open{display:flex;flex-wrap:wrap;min-width:0}.sail-jc-start{justify-content:flex-start}.sail-row.sail-nowrap{flex-wrap:nowrap}.sail-g-2{gap:var(--space-2)}.sail-ai-center{align-items:center}.sail-jc-between{justify-content:space-between}.sail-g-1{gap:var(--space-1)}.sail-g-3{gap:var(--space-3)}.sail-ai-start{align-items:flex-start}.sail-g-4{gap:var(--space-4)}.sail-stackz{display:grid}.sail-stackz>*{grid-area:1/1;position:relative}" }, __default);

//#endregion
//#region routes/components/page.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0 = /*@__PURE__*/ __tpl("<components-surface class=\"sx-grow1 sx-minH0 sx-hfull sx-minW0 sx-overflowhidden\">\n  </components-surface>");
var page_default = Object.assign(component((ctx) => {
	const { route, navigate, providers } = ctx;
	const $scope = Object.create(null);
	$scope.route = route;
	$scope.navigate = navigate;
	for (const __k of Object.keys(ctx)) {
		if (__k === "route" || __k === "navigate" || __k === "scope" || __k === "stateKeys" || __k === "host" || __k === "providers" || __k === "props") continue;
		$scope[__k] = ctx[__k];
	}
	if (providers && typeof providers === "object") Object.assign($scope, providers);
	return Object.assign({}, $scope, { __mount(container) {
		const __cleanups = [];
		const __root = container.shadowRoot ?? container;
		const __suiStylesKey = "37f1aea1";
		const __suiStyles = "@layer sail {\n.sx-minH0{min-height:0}.sx-hfull{height:100%}.sx-minW0{min-width:0}.sx-overflowhidden{overflow:hidden}.sx-grow1{flex-grow:1}\n}";
		{
			let __existing = null;
			let __n = container.firstChild;
			while (__n) {
				if (__n.nodeType === 1 && __n.tagName === "STYLE" && __n.getAttribute("data-sail-style") === __suiStylesKey) {
					__existing = __n;
					break;
				}
				__n = __n.nextSibling;
			}
			if (!__existing) {
				const __el = document.createElement("style");
				__el.setAttribute("data-sail-style", __suiStylesKey);
				__el.textContent = __suiStyles;
				container.insertBefore(__el, container.firstChild);
			}
		}
		const __e0 = __tpl0();
		__root.appendChild(__e0);
		return () => {
			for (const c of __cleanups) try {
				c();
			} catch (e) {}
		};
	} });
}), {
	_compilerVersion: {
		"cargo": "0.1.0",
		"git": "c11a87c8",
		"dirty": true,
		"builtAt": "2026-08-05T21:58:16Z"
	},
	clients: []
});

//#endregion
export { page_default as default };