import { t as getDesignState } from "./design-hH4c3EtQ.js";
import { theme } from "@ship/sail-runtime";

//#region src/theme-session/theme-draft-paint.ts
/**
* Theme draft paint adapter — single Studio entry for live preview on hosts.
*
* Edit loop (dirty draft):
*   RAM draftDoc → theme.applyScope(host, { doc, mode }) → host only
*   No Generate, no HMR, no :root product CSSOM.
*
* Default applier is framework `theme.applyScope` (T1). Tests may inject
* via setDraftPaintApplier. Without a doc, falls back to theme/mode attrs
* for registered-table cold path only (never theme).
*
* Wire:
*   bindPreviewHost(el) once per specimen; store emit → scheduleDraftPaint.
*/
const hosts = /* @__PURE__ */ new Set();
const teardowns = /* @__PURE__ */ new Map();
/** null = use default applyScope; function = inject; false sentinel not used */
let applier = void 0;
/** Optional resolve of registered paint id when host has no paintId override. */
let paintIdResolver = null;
let raf = 0;
let lastPaintKey = "";
function defaultApplyScope(host, opts) {
	if (!opts.doc) {
		applyRegisteredAttrs(host, opts);
		return () => {};
	}
	return theme.applyScope(host, {
		doc: opts.doc,
		mode: opts.mode
	});
}
function getDraftPaintApplier() {
	if (applier === void 0) return defaultApplyScope;
	return applier;
}
/** Re-apply after draft mutations (batched to rAF for continuous inputs). */
function scheduleDraftPaint() {
	if (typeof requestAnimationFrame === "undefined") {
		flushDraftPaint();
		return;
	}
	if (raf) return;
	raf = requestAnimationFrame(() => {
		raf = 0;
		flushDraftPaint();
	});
}
/** Synchronous apply — tests + callers that need paint before next frame. */
function flushDraftPaint(override) {
	if (hosts.size === 0 && !override) return;
	const ds = getDesignState();
	const themeId = override?.themeId ?? ds.themeId ?? "";
	const mode = override?.mode ?? ds.modeId ?? "light";
	const doc = override?.doc !== void 0 ? override.doc : ds.themeDoc;
	const forkedFrom = ds.forkedFrom;
	const paintId = override?.paintId ?? (paintIdResolver ? paintIdResolver(themeId, forkedFrom) : themeId || "studio");
	const opts = {
		doc,
		mode,
		themeId,
		paintId
	};
	const key = `${themeId}|${mode}|${paintId}|${doc ? JSON.stringify(doc).length : 0}|${ds.dirty}`;
	if (!ds.dirty && key === lastPaintKey && !override) return;
	lastPaintKey = key;
	for (const host of hosts) applyOne(host, opts);
}
function applyOne(host, opts) {
	const prev = teardowns.get(host);
	if (prev) {
		try {
			prev();
		} catch {}
		teardowns.delete(host);
	}
	const fn = getDraftPaintApplier();
	if (fn) {
		host.setAttribute("draft-paint", opts.doc ? "apply" : "registered");
		try {
			const td = fn(host, opts);
			if (typeof td === "function") teardowns.set(host, td);
		} catch (e) {
			host.setAttribute("draft-paint", "error");
			console.warn("[theme-draft-paint] applier failed", e);
		}
		return;
	}
	host.setAttribute("draft-paint", "registered");
	applyRegisteredAttrs(host, opts);
}
function applyRegisteredAttrs(host, opts) {
	const id = opts.paintId || opts.themeId || "studio";
	host.setAttribute("theme", id);
	host.removeAttribute("theme");
	host.removeAttribute("theme-preview");
	host.setAttribute("mode", opts.mode);
}

//#endregion
export { scheduleDraftPaint };