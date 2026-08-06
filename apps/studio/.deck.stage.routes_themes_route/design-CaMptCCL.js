//#region src/design/types.ts
/** Ordered radius scale steps (excludes full sentinel). */
const RADIUS_ORDERED = [
	"sm",
	"md",
	"lg",
	"xl"
];
/** Icon size-scale steps used by design surface family editor (subset of full ladder). */
const ICON_ORDERED = [
	"sm",
	"md",
	"lg",
	"xl"
];
const DECISION_IDS = [
	"color",
	"typography",
	"density",
	"radius",
	"icon"
];
const CATALOG_THEMES_DIR = "packages/sail/tokens/model/themes";

//#endregion
//#region src/design/scale.ts
/** Parse dimension-like value to px (aliases return null). */
function parseDimensionPx(raw) {
	const s = String(raw).trim();
	if (!s || s.startsWith("{")) return null;
	if (s.endsWith("px")) {
		const n = Number(s.slice(0, -2).trim());
		return Number.isFinite(n) ? n : null;
	}
	if (s.endsWith("rem")) {
		const n = Number(s.slice(0, -3).trim());
		return Number.isFinite(n) ? n * 16 : null;
	}
	if (s.endsWith("em")) {
		const n = Number(s.slice(0, -2).trim());
		return Number.isFinite(n) ? n * 16 : null;
	}
	const n = Number(s);
	return Number.isFinite(n) ? n : null;
}
function formatPx(n) {
	const r = Math.round(n * 1e3) / 1e3;
	return Number.isInteger(r) ? `${r}px` : `${r}px`;
}
/** Default radius family (base model values). */
function defaultRadiusFamily(scaleTogether = true) {
	const defaults = {
		sm: 4,
		md: 6,
		lg: 10,
		xl: 14
	};
	const ordered = [...RADIUS_ORDERED];
	return {
		path: "radius",
		ordered,
		scaleTogether,
		steps: ordered.map((key) => ({
			key,
			valuePx: defaults[key],
			raw: formatPx(defaults[key])
		}))
	};
}
/** Build ScaleFamily from a DTCG radius object. */
function radiusFamilyFromDoc(radiusObj, scaleTogether = true) {
	const base = defaultRadiusFamily(scaleTogether);
	if (!radiusObj || typeof radiusObj !== "object") return base;
	const ext = radiusObj.$extensions;
	let ordered = base.ordered;
	if (ext && Array.isArray(ext.ordered)) ordered = ext.ordered.filter((x) => typeof x === "string");
	const steps = [];
	for (const key of ordered) {
		const step = radiusObj[key];
		const raw = step && step.$value != null ? String(step.$value) : null;
		const px = raw != null ? parseDimensionPx(raw) : null;
		const fallback = base.steps.find((s) => s.key === key);
		steps.push({
			key,
			valuePx: px ?? fallback?.valuePx ?? 0,
			raw: raw ?? fallback?.raw ?? "0px"
		});
	}
	return {
		path: "radius",
		ordered,
		scaleTogether,
		steps
	};
}
/** Default icon size family (sm…xl subset). */
function defaultIconFamily(scaleTogether = true) {
	const defaults = {
		sm: 14,
		md: 16,
		lg: 18,
		xl: 20
	};
	const ordered = [...ICON_ORDERED];
	return {
		path: "icon",
		ordered,
		scaleTogether,
		steps: ordered.map((key) => ({
			key,
			valuePx: defaults[key],
			raw: formatPx(defaults[key])
		}))
	};
}
/** Build icon ScaleFamily from a DTCG icon object. */
function iconFamilyFromDoc(iconObj, scaleTogether = true) {
	const base = defaultIconFamily(scaleTogether);
	if (!iconObj || typeof iconObj !== "object") return base;
	const ext = iconObj.$extensions;
	let ordered = base.ordered;
	if (ext && Array.isArray(ext.ordered)) ordered = ext.ordered.filter((x) => typeof x === "string");
	const steps = [];
	for (const key of ordered) {
		const step = iconObj[key];
		const raw = step && step.$value != null ? String(step.$value) : null;
		const px = raw != null ? parseDimensionPx(raw) : null;
		const fallback = base.steps.find((s) => s.key === key);
		steps.push({
			key,
			valuePx: px ?? fallback?.valuePx ?? 0,
			raw: raw ?? fallback?.raw ?? "0px"
		});
	}
	return {
		path: "icon",
		ordered,
		scaleTogether,
		steps
	};
}

//#endregion
//#region src/design/theme-doc.ts
function parseThemeJson(content) {
	const v = JSON.parse(content);
	if (!v || typeof v !== "object" || Array.isArray(v)) throw new Error("theme JSON root must be an object");
	return v;
}
function stringifyTheme(doc) {
	return `${JSON.stringify(doc, null, 2)}\n`;
}
function themeIdOf(doc, fallback) {
	const id = (doc.$extensions?.ship)?.themeId;
	return typeof id === "string" && id ? id : fallback;
}
function setThemeId(doc, id) {
	const next = structuredClone(doc);
	const ext = { ...next.$extensions ?? {} };
	ext.ship = {
		...ext.ship ?? {},
		themeId: id,
		layer: ext.ship?.layer ?? "theme"
	};
	next.$extensions = ext;
	return next;
}
function densityOf(doc) {
	if (!doc) return "default";
	const d = (doc.$extensions?.ship)?.density;
	if (d === "compact" || d === "spacious" || d === "default") return d;
	return "default";
}
/** Read a scale group object from root or modes (prefer active mode). */
function extractScaleGroup(doc, group, modeId = "light") {
	if (!doc) return null;
	const modes = doc.modes;
	const active = modes?.[modeId];
	if (active?.[group] && typeof active[group] === "object") return active[group];
	if (doc[group] && typeof doc[group] === "object" && !Array.isArray(doc[group])) return doc[group];
	const fallback = modes?.[modeId === "dark" ? "light" : "dark"];
	if (fallback?.[group] && typeof fallback[group] === "object") return fallback[group];
	return null;
}
/** Read radius family from active mode, then root, then other mode. */
function extractRadius(doc, modeId = "light") {
	return radiusFamilyFromDoc(extractScaleGroup(doc, "radius", modeId));
}
function extractIconScale(doc, modeId = "light") {
	return iconFamilyFromDoc(extractScaleGroup(doc, "icon", modeId));
}
/**
* Inheritance chain for radius: base catalog → theme scale → component override.
* Components: button, card, input (specimen-relevant).
*/
/**
* Full Sail L1 stdlib catalog (Themes Components rail SSOT).
* Keep in sync with `crates/sail/compiler/src/stdlib.rs` `SOURCES` tags.
* Tag `a` is the anchor element (file: anchor.sail).
*/
const L1_CATALOG = [
	"spinner",
	"button",
	"input",
	"textarea",
	"select",
	"dialog",
	"details",
	"summary",
	"table",
	"fieldset",
	"legend",
	"progress",
	"meter",
	"a",
	"separator",
	"icon",
	"label",
	"card",
	"switch",
	"badge",
	"avatar",
	"tooltip",
	"alert",
	"empty-state",
	"error-state",
	"error-boundary",
	"code-block",
	"chip",
	"field",
	"collapsible",
	"slider",
	"radio",
	"radio-group",
	"tabs",
	"tab-list",
	"tab",
	"tab-panel",
	"toggle-group",
	"toggle",
	"scroll-area",
	"ticker",
	"theme-picker",
	"navbar",
	"pagination",
	"page",
	"breadcrumb",
	"popover",
	"prose",
	"combobox",
	"dropdown",
	"menu-item",
	"context-menu",
	"hover-card",
	"toast-region",
	"toast"
];
/** Display label for an L1 tag (`a` → Anchor, `menu-item` → Menu Item). */
function l1Label(id) {
	if (id === "a") return "Anchor";
	return id.split("-").map((p) => p ? p.charAt(0).toUpperCase() + p.slice(1) : p).join(" ");
}
/**
* Inheritance chain for radius: base catalog → theme scale → per-L1 override.
* One node per L1 in {@link L1_CATALOG} so Themes Components + Design stay aligned.
*/
function radiusInheritance(doc, modeId = "light") {
	const globalVal = extractRadius(doc, modeId).steps.find((s) => s.key === "lg")?.raw ?? "10px";
	const nodes = [{
		id: "base",
		label: "Base · catalog",
		value: "10px",
		isOverride: false,
		path: "radius.lg",
		canReset: false
	}, {
		id: "theme",
		label: `Theme · ${parseDimensionPx(globalVal) ?? globalVal}`,
		value: globalVal,
		isOverride: false,
		path: "radius.lg",
		canReset: false
	}];
	for (const comp of L1_CATALOG) {
		const path = `component.${comp}.radius`;
		const raw = getPathValue(doc, path, modeId);
		const isOverride = raw != null;
		const name = l1Label(comp);
		nodes.push({
			id: comp,
			label: isOverride ? `Override · ${name} · ${raw}` : `Override · ${name} · inherits theme`,
			value: raw,
			isOverride,
			path,
			canReset: isOverride
		});
	}
	return nodes;
}
function walkPath(doc, path) {
	if (!doc || !path) return null;
	const parts = path.split(".");
	let cur = doc;
	for (const p of parts) {
		if (!cur || typeof cur !== "object" || Array.isArray(cur)) return null;
		cur = cur[p];
	}
	return cur;
}
function leafValue(node) {
	if (!node || typeof node !== "object" || Array.isArray(node)) return null;
	const v = node.$value;
	return v != null ? String(v) : null;
}
/**
* Read candidates for a logical token path.
* Prefer root, then active mode, then the other mode (read fallback only).
*/
function candidatePaths(path, modeId = "light") {
	if (!path || path.startsWith("$") || path.startsWith("modes.")) return [path];
	const other = modeId === "dark" ? "light" : "dark";
	const out = [
		path,
		`modes.${modeId}.${path}`,
		`modes.${other}.${path}`
	];
	return [...new Set(out)];
}
/**
* Write candidates — never the inactive mode.
* Prefer existing root leaf, then active mode leaf; create under active when modes-in-file.
*/
function writeCandidatePaths(path, modeId = "light") {
	if (!path || path.startsWith("$") || path.startsWith("modes.")) return [path];
	return [path, `modes.${modeId}.${path}`];
}
/** Resolve $value at logical path, walking modes when needed (read may fall back). */
function getPathValue(doc, path, modeId = "light") {
	if (!doc) return null;
	for (const p of candidatePaths(path, modeId)) {
		const v = leafValue(walkPath(doc, p));
		if (v != null) return v;
	}
	return null;
}
/** Physical path for write-back — never clobbers the inactive mode. */
function resolveWritePath(doc, path, modeId = "light") {
	if (!doc) return path;
	for (const p of writeCandidatePaths(path, modeId)) if (leafValue(walkPath(doc, p)) != null) return p;
	if (doc.modes && typeof doc.modes === "object") return `modes.${modeId}.${path}`;
	return path;
}
/** Set $value at dotted path, creating intermediate objects. Mode-aware. */
function setPathValue(doc, path, value, type = "dimension", modeId = "light", cssName) {
	const target = resolveWritePath(doc, path, modeId);
	const next = structuredClone(doc);
	const parts = target.split(".");
	let cur = next;
	for (let i = 0; i < parts.length - 1; i++) {
		const p = parts[i];
		const child = cur[p];
		if (!child || typeof child !== "object" || Array.isArray(child)) cur[p] = {};
		cur = cur[p];
	}
	const leaf = parts[parts.length - 1];
	const prev = cur[leaf] ?? {};
	const prevExt = prev.$extensions && typeof prev.$extensions === "object" && !Array.isArray(prev.$extensions) ? prev.$extensions : {};
	const prevShip = prevExt.ship && typeof prevExt.ship === "object" && !Array.isArray(prevExt.ship) ? prevExt.ship : {};
	const ship = cssName ? {
		...prevShip,
		cssName
	} : prevShip;
	const extensions = cssName || Object.keys(prevExt).length > 0 ? {
		...prevExt,
		...Object.keys(ship).length ? { ship } : {}
	} : prev.$extensions;
	cur[leaf] = {
		...prev,
		$value: value,
		$type: prev.$type ?? type,
		...extensions ? { $extensions: extensions } : {}
	};
	return next;
}
/**
* Public cssName → DTCG path for type-role leaves (token-registry paths).
* Used to walk `{alias}` when extracting faces from a theme doc.
*/
const FONT_CSS_TO_PATH = {
	"display-font": "typography.displayFont",
	"heading-font": "typography.heading.font",
	"body-font": "typography.body.font",
	"mono-font": "typography.monoFont",
	"font-display": "typography.font.display",
	"font-display-app": "typography.font.display-app",
	"font-body": "typography.font.body",
	"font-body-app": "typography.font.body-app",
	"font-mono": "typography.font.mono",
	"font-mono-app": "typography.font.mono-app"
};
/**
* Base-layer stack names that themes alias into (tokens model base / fonts.ts).
* Not product CSS — identity for honest role labels when the DTCG leaf is `{cssName}`.
*/
const PLATFORM_STACK_FACE = {
	"font-press-start": "Press Start 2P",
	"font-press-start-app": "Press Start 2P",
	"font-display-app": "Syne",
	"font-display": "Syne",
	"font-body-app": "DM Sans",
	"font-body": "DM Sans",
	"font-mono-app": "JetBrains Mono",
	"font-mono": "JetBrains Mono"
};
function extractTypeRoles(doc, modeId = "light") {
	const defaults = {
		display: "Fraunces",
		body: "DM Sans",
		mono: "JetBrains Mono"
	};
	if (!doc) return defaults;
	const displayRaw = getPathValue(doc, "typography.displayFont", modeId) ?? getPathValue(doc, "typography.heading.font", modeId) ?? getPathValue(doc, "typography.font.display", modeId) ?? getPathValue(doc, "typography.font.display-app", modeId);
	const bodyRaw = getPathValue(doc, "typography.body.font", modeId) ?? getPathValue(doc, "typography.font.body", modeId) ?? getPathValue(doc, "typography.font.body-app", modeId);
	const monoRaw = getPathValue(doc, "typography.monoFont", modeId) ?? getPathValue(doc, "typography.font.mono", modeId) ?? getPathValue(doc, "typography.font.mono-app", modeId) ?? getPathValue(doc, "typography.mono.font", modeId);
	return {
		display: resolveFaceName(doc, displayRaw, modeId) || defaults.display,
		body: resolveFaceName(doc, bodyRaw, modeId) || defaults.body,
		mono: resolveFaceName(doc, monoRaw, modeId) || defaults.mono
	};
}
/** First family name from a concrete stack: `"Fraunces", …` or bare `Fraunces, serif`. */
function familyName(raw) {
	if (!raw) return "";
	const t = raw.trim();
	if (!t || t.startsWith("{")) return "";
	const m = t.match(/"([^"]+)"/);
	if (m) return m[1].replace(/\s+Variable$/, "");
	return t.split(",")[0]?.replace(/"/g, "").trim() ?? "";
}
/**
* Resolve a DTCG font $value to a UI face name.
* Walks whole-value `{alias}` through the same doc + known base stacks.
* Never invents Fraunces when the theme aliases to Press Start / mono.
*/
function resolveFaceName(doc, raw, modeId, depth = 0) {
	if (!raw) return "";
	const t = raw.trim();
	if (!t) return "";
	if (t.startsWith("{") && t.endsWith("}")) {
		if (depth > 6) return "";
		const ref = t.slice(1, -1);
		if (!ref) return "";
		const platform = PLATFORM_STACK_FACE[ref];
		if (platform) return platform;
		const path = FONT_CSS_TO_PATH[ref] ?? (ref.includes(".") ? ref : null);
		if (path) return resolveFaceName(doc, getPathValue(doc, path, modeId), modeId, depth + 1);
		return "";
	}
	return familyName(t);
}
/** Patch typography faces into public role leaves + app stacks (mode-aware). */
function patchTypeRoles(doc, roles, modeId = "light") {
	const stack = (name, fallback) => `"${name}", "${fallback}", sans-serif`;
	const displayStack = (name) => `"${name}", ui-serif, Georgia, serif`;
	const monoStack = (name) => `"${name}", ui-monospace, monospace`;
	let next = doc;
	next = setPathValue(next, "typography.displayFont", displayStack(roles.display), "fontFamily", modeId, "display-font");
	next = setPathValue(next, "typography.heading.font", displayStack(roles.display), "fontFamily", modeId, "heading-font");
	next = setPathValue(next, "typography.body.font", stack(roles.body, "Inter"), "fontFamily", modeId, "body-font");
	next = setPathValue(next, "typography.monoFont", monoStack(roles.mono), "fontFamily", modeId, "mono-font");
	next = setPathValue(next, "typography.font.display-app", stack(roles.display, "Inter"), "fontFamily", modeId, "font-display-app");
	next = setPathValue(next, "typography.font.display", stack(roles.display, "Inter"), "fontFamily", modeId, "font-display");
	next = setPathValue(next, "typography.font.body-app", stack(roles.body, "Inter"), "fontFamily", modeId, "font-body-app");
	next = setPathValue(next, "typography.font.body", stack(roles.body, "Inter"), "fontFamily", modeId, "font-body");
	next = setPathValue(next, "typography.font.mono-app", monoStack(roles.mono), "fontFamily", modeId, "font-mono-app");
	return setPathValue(next, "typography.font.mono", monoStack(roles.mono), "fontFamily", modeId, "font-mono");
}
/** Diff two theme docs as path → $value rows (shallow leaf walk + ship density). */
function diffThemeDocs(from, to) {
	const a = flattenValues(from);
	const b = flattenValues(to);
	const keys = new Set([...a.keys(), ...b.keys()]);
	const rows = [];
	for (const path of [...keys].sort()) {
		const fa = a.get(path) ?? "";
		const tb = b.get(path) ?? "";
		if (fa !== tb) rows.push({
			path,
			from: fa,
			to: tb
		});
	}
	return rows;
}
/**
* Flatten DTCG leaves for export/diff.
* Includes `$value` tokens and intentional ship extensions (`$extensions.ship.density`).
* Never reads CSS.
*/
function flattenValues(doc, prefix = "", out = /* @__PURE__ */ new Map()) {
	if (!doc) return out;
	for (const [k, v] of Object.entries(doc)) {
		if (k === "$extensions" && v && typeof v === "object" && !Array.isArray(v)) {
			const ship = v.ship;
			if (ship && typeof ship === "object" && !Array.isArray(ship)) {
				const dens = ship.density;
				if (typeof dens === "string" && dens) {
					const densPath = prefix ? `${prefix}.$extensions.ship.density` : "$extensions.ship.density";
					out.set(densPath, dens);
				}
			}
			continue;
		}
		if (k.startsWith("$") && k !== "$value") continue;
		const path = prefix ? `${prefix}.${k}` : k;
		if (v && typeof v === "object" && !Array.isArray(v)) {
			const obj = v;
			if ("$value" in obj) out.set(path, String(obj.$value));
			else flattenValues(obj, path, out);
		}
	}
	return out;
}
/** Dirty path count vs baseline, or component + density overrides. */
function countOverrides(doc, baseline = null) {
	if (!doc) return 0;
	if (baseline) return diffThemeDocs(baseline, doc).length;
	let n = 0;
	const flat = flattenValues(doc);
	for (const path of flat.keys()) if (path.startsWith("component.")) n++;
	if (densityOf(doc) !== "default") n++;
	return n;
}
/** Token inventory rows (search + overrides-only). */
function listTokenRows(doc, baseline, opts) {
	const flat = flattenValues(doc);
	const base = flattenValues(baseline);
	const q = (opts?.search ?? "").trim().toLowerCase();
	const rows = [];
	for (const [path, value] of [...flat.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
		const dirty = (base.get(path) ?? "") !== value || base.has(path) === false && !!value;
		const isOverride = baseline != null ? dirty : path.startsWith("component.") || path === "$extensions.ship.density";
		if (opts?.overridesOnly && !isOverride) continue;
		if (q && !path.toLowerCase().includes(q) && !value.toLowerCase().includes(q)) continue;
		rows.push({
			path,
			value,
			dirty: baseline != null ? dirty : isOverride
		});
	}
	return rows;
}
/** Catalog theme basenames → ThemeFile list. */
function themeFilesFromList(files, dir, catalog) {
	return files.filter((f) => f.endsWith(".json") && !f.startsWith(".")).map((file) => ({
		file,
		id: file.replace(/\.json$/, ""),
		path: `${dir.replace(/\/$/, "")}/${file}`,
		catalog
	})).sort((a, b) => a.id.localeCompare(b.id));
}

//#endregion
//#region src/design/specimen.ts
/** Inline .sail specimen for headless SailRender audit (secondary path). */
function specimenSource(opts) {
	const density = opts.density === "default" ? "" : ` data-density="${opts.density}"`;
	const display = opts.typeRoles?.display ?? "Fraunces";
	const body = opts.typeRoles?.body ?? "DM Sans";
	const mono = opts.typeRoles?.mono ?? "JetBrains Mono";
	return `<!-- design specimen -->
<div${density} style={style.pad(6).bg(background).fg(foreground)}>
  <column gap="5">
    <column gap="2">
      <span style={style.fontSize(xs).fg(muted-foreground).fontWeight(semibold)}>Buttons</span>
      <row gap="2" align="center">
        <button type="button" tone="primary">Primary</button>
        <button type="button" variant="outline">Outline</button>
        <button type="button" tone="destructive">Destructive</button>
      </row>
    </column>
    <column gap="2">
      <span style={style.fontSize(xs).fg(muted-foreground).fontWeight(semibold)}>Card</span>
      <card style={style.pad(4).radius(lg).borderWidth(1).border(border-subtle).bg(surface-1)}>
        <p style={style.fontWeight(medium)}>${escapeText(opts.note ?? "Sample card")}</p>
        <p style={style.fontSize(sm).fg(muted-foreground).padT(1)}>Density: ${opts.density}</p>
      </card>
    </column>
    <column gap="2">
      <span style={style.fontSize(xs).fg(muted-foreground).fontWeight(semibold)}>Badge · alert · field</span>
      <row gap="2" align="center">
        <badge tone="primary" size="sm">primary</badge>
        <badge tone="success" size="sm">success</badge>
        <badge tone="warning" size="sm">warning</badge>
      </row>
      <alert tone="info">Alert · status copy</alert>
      <input type="search" placeholder="Search…" />
    </column>
    <column gap="2" style={style.pad(4).radius(md).bg(surface-0).fg(foreground)}>
      <span style={style.fontSize(xs).fg(muted-foreground)}>Type</span>
      <p style={style.fontSize(xl).fontWeight(bold).fontFamily(display)}>Display / ${escapeText(display)}</p>
      <p style={style.fontSize(lg).fontWeight(semibold).fontFamily(body)}>Heading / ${escapeText(body)}</p>
      <p style={style.fontSize(sm).fontFamily(body)}>Body — quick brown fox</p>
      <p style={style.fontSize(xs).fontFamily(mono)}>code / ${escapeText(mono)}</p>
    </column>
  </column>
</div>
`;
}
function escapeText(s) {
	return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
const SPECIMEN_VIEWPORT = {
	width: 720,
	height: 640
};

//#endregion
//#region ../../packages/sail/tokens/src/token-registry.json
var tokens = {
	"radius": {
		"path": "radius.default",
		"cssName": "radius",
		"type": "dimension",
		"value": "10px"
	},
	"radius-sm": {
		"path": "radius.sm",
		"cssName": "radius-sm",
		"type": "dimension",
		"value": "4px"
	},
	"radius-md": {
		"path": "radius.md",
		"cssName": "radius-md",
		"type": "dimension",
		"value": "6px"
	},
	"radius-lg": {
		"path": "radius.lg",
		"cssName": "radius-lg",
		"type": "dimension",
		"value": "10px"
	},
	"radius-xl": {
		"path": "radius.xl",
		"cssName": "radius-xl",
		"type": "dimension",
		"value": "14px"
	},
	"radius-full": {
		"path": "radius.full",
		"cssName": "radius-full",
		"type": "dimension",
		"value": "9999px"
	},
	"space-1": {
		"path": "space.1",
		"cssName": "space-1",
		"type": "dimension",
		"value": "4px"
	},
	"space-2": {
		"path": "space.2",
		"cssName": "space-2",
		"type": "dimension",
		"value": "8px"
	},
	"space-3": {
		"path": "space.3",
		"cssName": "space-3",
		"type": "dimension",
		"value": "12px"
	},
	"space-4": {
		"path": "space.4",
		"cssName": "space-4",
		"type": "dimension",
		"value": "16px"
	},
	"space-5": {
		"path": "space.5",
		"cssName": "space-5",
		"type": "dimension",
		"value": "20px"
	},
	"space-6": {
		"path": "space.6",
		"cssName": "space-6",
		"type": "dimension",
		"value": "24px"
	},
	"space-7": {
		"path": "space.7",
		"cssName": "space-7",
		"type": "dimension",
		"value": "28px"
	},
	"space-8": {
		"path": "space.8",
		"cssName": "space-8",
		"type": "dimension",
		"value": "32px"
	},
	"space-9": {
		"path": "space.9",
		"cssName": "space-9",
		"type": "dimension",
		"value": "36px"
	},
	"space-10": {
		"path": "space.10",
		"cssName": "space-10",
		"type": "dimension",
		"value": "40px"
	},
	"space-11": {
		"path": "space.11",
		"cssName": "space-11",
		"type": "dimension",
		"value": "44px"
	},
	"space-12": {
		"path": "space.12",
		"cssName": "space-12",
		"type": "dimension",
		"value": "48px"
	},
	"space-14": {
		"path": "space.14",
		"cssName": "space-14",
		"type": "dimension",
		"value": "56px"
	},
	"space-16": {
		"path": "space.16",
		"cssName": "space-16",
		"type": "dimension",
		"value": "64px"
	},
	"space-20": {
		"path": "space.20",
		"cssName": "space-20",
		"type": "dimension",
		"value": "80px"
	},
	"space-24": {
		"path": "space.24",
		"cssName": "space-24",
		"type": "dimension",
		"value": "96px"
	},
	"space-28": {
		"path": "space.28",
		"cssName": "space-28",
		"type": "dimension",
		"value": "112px"
	},
	"space-32": {
		"path": "space.32",
		"cssName": "space-32",
		"type": "dimension",
		"value": "128px"
	},
	"space-36": {
		"path": "space.36",
		"cssName": "space-36",
		"type": "dimension",
		"value": "144px"
	},
	"space-40": {
		"path": "space.40",
		"cssName": "space-40",
		"type": "dimension",
		"value": "160px"
	},
	"space-44": {
		"path": "space.44",
		"cssName": "space-44",
		"type": "dimension",
		"value": "176px"
	},
	"space-48": {
		"path": "space.48",
		"cssName": "space-48",
		"type": "dimension",
		"value": "192px"
	},
	"space-52": {
		"path": "space.52",
		"cssName": "space-52",
		"type": "dimension",
		"value": "208px"
	},
	"space-56": {
		"path": "space.56",
		"cssName": "space-56",
		"type": "dimension",
		"value": "224px"
	},
	"space-60": {
		"path": "space.60",
		"cssName": "space-60",
		"type": "dimension",
		"value": "240px"
	},
	"space-64": {
		"path": "space.64",
		"cssName": "space-64",
		"type": "dimension",
		"value": "256px"
	},
	"space-px": {
		"path": "space.px",
		"cssName": "space-px",
		"type": "dimension",
		"value": "1px"
	},
	"space-half": {
		"path": "space.half",
		"cssName": "space-half",
		"type": "dimension",
		"value": "2px"
	},
	"space-0_75": {
		"path": "space.0_75",
		"cssName": "space-0_75",
		"type": "dimension",
		"value": "3px"
	},
	"space-1_25": {
		"path": "space.1_25",
		"cssName": "space-1_25",
		"type": "dimension",
		"value": "5px"
	},
	"space-1_5": {
		"path": "space.1_5",
		"cssName": "space-1_5",
		"type": "dimension",
		"value": "6px"
	},
	"space-2_5": {
		"path": "space.2_5",
		"cssName": "space-2_5",
		"type": "dimension",
		"value": "10px"
	},
	"space-3_5": {
		"path": "space.3_5",
		"cssName": "space-3_5",
		"type": "dimension",
		"value": "14px"
	},
	"space-4_5": {
		"path": "space.4_5",
		"cssName": "space-4_5",
		"type": "dimension",
		"value": "18px"
	},
	"icon-size-3xs": {
		"path": "icon.3xs",
		"cssName": "icon-size-3xs",
		"type": "dimension",
		"value": "8px"
	},
	"icon-size-2xs": {
		"path": "icon.2xs",
		"cssName": "icon-size-2xs",
		"type": "dimension",
		"value": "10px"
	},
	"icon-size-xs": {
		"path": "icon.xs",
		"cssName": "icon-size-xs",
		"type": "dimension",
		"value": "12px"
	},
	"icon-size-sm": {
		"path": "icon.sm",
		"cssName": "icon-size-sm",
		"type": "dimension",
		"value": "14px"
	},
	"icon-size-md": {
		"path": "icon.md",
		"cssName": "icon-size-md",
		"type": "dimension",
		"value": "16px"
	},
	"icon-size-lg": {
		"path": "icon.lg",
		"cssName": "icon-size-lg",
		"type": "dimension",
		"value": "18px"
	},
	"icon-size-xl": {
		"path": "icon.xl",
		"cssName": "icon-size-xl",
		"type": "dimension",
		"value": "20px"
	},
	"icon-size-2xl": {
		"path": "icon.2xl",
		"cssName": "icon-size-2xl",
		"type": "dimension",
		"value": "24px"
	},
	"icon-size-3xl": {
		"path": "icon.3xl",
		"cssName": "icon-size-3xl",
		"type": "dimension",
		"value": "28px"
	},
	"control-h-2xs": {
		"path": "control.h-2xs",
		"cssName": "control-h-2xs",
		"type": "dimension",
		"value": "22px"
	},
	"control-h-xs": {
		"path": "control.h-xs",
		"cssName": "control-h-xs",
		"type": "dimension",
		"value": "24px"
	},
	"control-h-sm": {
		"path": "control.h-sm",
		"cssName": "control-h-sm",
		"type": "dimension",
		"value": "28px"
	},
	"control-h-md": {
		"path": "control.h-md",
		"cssName": "control-h-md",
		"type": "dimension",
		"value": "34px"
	},
	"control-h-lg": {
		"path": "control.h-lg",
		"cssName": "control-h-lg",
		"type": "dimension",
		"value": "40px"
	},
	"control-h-xl": {
		"path": "control.h-xl",
		"cssName": "control-h-xl",
		"type": "dimension",
		"value": "44px"
	},
	"control-stroke": {
		"path": "control.stroke",
		"cssName": "control-stroke",
		"type": "dimension",
		"value": "2px"
	},
	"control-frame": {
		"path": "control.frame",
		"cssName": "control-frame",
		"type": "shadow",
		"value": "0 0 0 0 transparent"
	},
	"control-lift": {
		"path": "control.lift",
		"cssName": "control-lift",
		"type": "dimension",
		"value": "-1px"
	},
	"press-translate": {
		"path": "control.press.translate",
		"cssName": "press-translate",
		"type": "dimension",
		"value": "1px"
	},
	"press-scale": {
		"path": "control.press.scale",
		"cssName": "press-scale",
		"type": "number",
		"value": "0.98"
	},
	"press-shadow": {
		"path": "control.press.shadow",
		"cssName": "press-shadow",
		"type": "shadow",
		"value": "0 0 0 0 transparent"
	},
	"panel-max-w-sm": {
		"path": "panel.max-w-sm",
		"cssName": "panel-max-w-sm",
		"type": "dimension",
		"value": "320px"
	},
	"panel-max-w-md": {
		"path": "panel.max-w-md",
		"cssName": "panel-max-w-md",
		"type": "dimension",
		"value": "480px"
	},
	"panel-max-w-lg": {
		"path": "panel.max-w-lg",
		"cssName": "panel-max-w-lg",
		"type": "dimension",
		"value": "720px"
	},
	"panel-max-w-xs": {
		"path": "panel.max-w-xs",
		"cssName": "panel-max-w-xs",
		"type": "dimension",
		"value": "20rem"
	},
	"panel-max-h-sm": {
		"path": "panel.max-h-sm",
		"cssName": "panel-max-h-sm",
		"type": "dimension",
		"value": "240px"
	},
	"panel-max-h-md": {
		"path": "panel.max-h-md",
		"cssName": "panel-max-h-md",
		"type": "dimension",
		"value": "480px"
	},
	"panel-max-h-lg": {
		"path": "panel.max-h-lg",
		"cssName": "panel-max-h-lg",
		"type": "dimension",
		"value": "720px"
	},
	"panel-min-w-sm": {
		"path": "panel.min-w-sm",
		"cssName": "panel-min-w-sm",
		"type": "dimension",
		"value": "180px"
	},
	"panel-min-w-md": {
		"path": "panel.min-w-md",
		"cssName": "panel-min-w-md",
		"type": "dimension",
		"value": "240px"
	},
	"panel-min-w-lg": {
		"path": "panel.min-w-lg",
		"cssName": "panel-min-w-lg",
		"type": "dimension",
		"value": "320px"
	},
	"panel-vh-sm": {
		"path": "panel.vh-sm",
		"cssName": "panel-vh-sm",
		"type": "dimension",
		"value": "50vh"
	},
	"panel-vh-md": {
		"path": "panel.vh-md",
		"cssName": "panel-vh-md",
		"type": "dimension",
		"value": "80vh"
	},
	"panel-vh-lg": {
		"path": "panel.vh-lg",
		"cssName": "panel-vh-lg",
		"type": "dimension",
		"value": "90vh"
	},
	"panel-ch-w-md": {
		"path": "panel.ch-w-md",
		"cssName": "panel-ch-w-md",
		"type": "dimension",
		"value": "38ch"
	},
	"panel-ch-w-lg": {
		"path": "panel.ch-w-lg",
		"cssName": "panel-ch-w-lg",
		"type": "dimension",
		"value": "60ch"
	},
	"panel-rail-w": {
		"path": "panel.rail-w",
		"cssName": "panel-rail-w",
		"type": "dimension",
		"value": "14rem"
	},
	"menu-min-w": {
		"path": "panel.menuMinW",
		"cssName": "menu-min-w",
		"type": "dimension",
		"value": "12rem"
	},
	"panel-pad-x": {
		"path": "panel.pad-x",
		"cssName": "panel-pad-x",
		"type": "dimension",
		"value": "var(--space-6)"
	},
	"panel-head-pad-y": {
		"path": "panel.head-pad-y",
		"cssName": "panel-head-pad-y",
		"type": "dimension",
		"value": "18px"
	},
	"panel-body-pad-y": {
		"path": "panel.body-pad-y",
		"cssName": "panel-body-pad-y",
		"type": "dimension",
		"value": "var(--space-4)"
	},
	"panel-foot-pad-y": {
		"path": "panel.foot-pad-y",
		"cssName": "panel-foot-pad-y",
		"type": "dimension",
		"value": "var(--space-4)"
	},
	"panel-close-inset": {
		"path": "panel.close-inset",
		"cssName": "panel-close-inset",
		"type": "dimension",
		"value": "var(--space-7)"
	},
	"panel-pad-card": {
		"path": "panel.pad-card",
		"cssName": "panel-pad-card",
		"type": "dimension",
		"value": "var(--space-2)"
	},
	"panel-pad-menu": {
		"path": "panel.pad-menu",
		"cssName": "panel-pad-menu",
		"type": "dimension",
		"value": "var(--space-1)"
	},
	"panel-w-side": {
		"path": "panel.w-side",
		"cssName": "panel-w-side",
		"type": "dimension",
		"value": "420px"
	},
	"panel-w-toast": {
		"path": "panel.w-toast",
		"cssName": "panel-w-toast",
		"type": "dimension",
		"value": "min(24rem, calc(100vw - var(--space-8)))"
	},
	"panel-min-h-cell": {
		"path": "panel.min-h-cell",
		"cssName": "panel-min-h-cell",
		"type": "dimension",
		"value": "84px"
	},
	"panel-min-h-daycol": {
		"path": "panel.min-h-daycol",
		"cssName": "panel-min-h-daycol",
		"type": "dimension",
		"value": "864px"
	},
	"leading-loose": {
		"path": "typography.leading.loose",
		"cssName": "leading-loose",
		"type": "number",
		"value": "1.55"
	},
	"leading-none": {
		"path": "typography.leading.none",
		"cssName": "leading-none",
		"type": "number",
		"value": "1"
	},
	"leading-tight": {
		"path": "typography.leading.tight",
		"cssName": "leading-tight",
		"type": "number",
		"value": "1.2"
	},
	"leading-snug": {
		"path": "typography.leading.snug",
		"cssName": "leading-snug",
		"type": "number",
		"value": "1.35"
	},
	"leading-normal": {
		"path": "typography.leading.normal",
		"cssName": "leading-normal",
		"type": "number",
		"value": "1.5"
	},
	"leading-normal-plus": {
		"path": "typography.leading.normal-plus",
		"cssName": "leading-normal-plus",
		"type": "number",
		"value": "1.55"
	},
	"leading-code": {
		"path": "typography.leading.code",
		"cssName": "leading-code",
		"type": "number",
		"value": "1.6"
	},
	"leading-relaxed": {
		"path": "typography.leading.relaxed",
		"cssName": "leading-relaxed",
		"type": "number",
		"value": "1.7"
	},
	"text-xs": {
		"path": "typography.text.xs",
		"cssName": "text-xs",
		"type": "dimension",
		"value": "11px"
	},
	"text-sm": {
		"path": "typography.text.sm",
		"cssName": "text-sm",
		"type": "dimension",
		"value": "12px"
	},
	"text-base": {
		"path": "typography.text.base",
		"cssName": "text-base",
		"type": "dimension",
		"value": "13px"
	},
	"text-md": {
		"path": "typography.text.md",
		"cssName": "text-md",
		"type": "dimension",
		"value": "14px"
	},
	"text-lg": {
		"path": "typography.text.lg",
		"cssName": "text-lg",
		"type": "dimension",
		"value": "16px"
	},
	"text-xl": {
		"path": "typography.text.xl",
		"cssName": "text-xl",
		"type": "dimension",
		"value": "20px"
	},
	"text-2xl": {
		"path": "typography.text.2xl",
		"cssName": "text-2xl",
		"type": "dimension",
		"value": "24px"
	},
	"text-3xl": {
		"path": "typography.text.3xl",
		"cssName": "text-3xl",
		"type": "dimension",
		"value": "28px"
	},
	"text-4xl": {
		"path": "typography.text.4xl",
		"cssName": "text-4xl",
		"type": "dimension",
		"value": "36px"
	},
	"text-display": {
		"path": "typography.text.display",
		"cssName": "text-display",
		"type": "dimension",
		"value": "clamp(3rem, 9vw, 8.5rem)"
	},
	"font-weight-normal": {
		"path": "typography.font.weight-normal",
		"cssName": "font-weight-normal",
		"type": "number",
		"value": "400"
	},
	"font-weight-medium": {
		"path": "typography.font.weight-medium",
		"cssName": "font-weight-medium",
		"type": "number",
		"value": "500"
	},
	"font-weight-semibold": {
		"path": "typography.font.weight-semibold",
		"cssName": "font-weight-semibold",
		"type": "number",
		"value": "600"
	},
	"font-weight-bold": {
		"path": "typography.font.weight-bold",
		"cssName": "font-weight-bold",
		"type": "number",
		"value": "700"
	},
	"font-display-app": {
		"path": "typography.font.display-app",
		"cssName": "font-display-app",
		"type": "fontFamily",
		"value": "\"Syne Variable\", \"Syne\", \"Inter\", sans-serif"
	},
	"font-body-app": {
		"path": "typography.font.body-app",
		"cssName": "font-body-app",
		"type": "fontFamily",
		"value": "\"DM Sans Variable\", \"DM Sans\", \"Inter\", sans-serif"
	},
	"font-mono-app": {
		"path": "typography.font.mono-app",
		"cssName": "font-mono-app",
		"type": "fontFamily",
		"value": "\"JetBrains Mono Variable\", \"JetBrains Mono\", ui-monospace, \"SFMono-Regular\", Menlo, Consolas, monospace"
	},
	"font-press-start-app": {
		"path": "typography.font.press-start-app",
		"cssName": "font-press-start-app",
		"type": "fontFamily",
		"value": "\"Press Start 2P\", \"Press Start 2P Fallback\", ui-monospace, \"SFMono-Regular\", Menlo, Consolas, monospace"
	},
	"font-display": {
		"path": "typography.font.display",
		"cssName": "font-display",
		"type": "fontFamily",
		"value": "var(--font-display-app)"
	},
	"font-body": {
		"path": "typography.font.body",
		"cssName": "font-body",
		"type": "fontFamily",
		"value": "var(--font-body-app)"
	},
	"font-mono": {
		"path": "typography.font.mono",
		"cssName": "font-mono",
		"type": "fontFamily",
		"value": "var(--font-mono-app)"
	},
	"font-press-start": {
		"path": "typography.font.press-start",
		"cssName": "font-press-start",
		"type": "fontFamily",
		"value": "var(--font-press-start-app)"
	},
	"tracking-tighter": {
		"path": "typography.tracking.tighter",
		"cssName": "tracking-tighter",
		"type": "dimension",
		"value": "-0.03em"
	},
	"tracking-tight": {
		"path": "typography.tracking.tight",
		"cssName": "tracking-tight",
		"type": "dimension",
		"value": "-0.01em"
	},
	"tracking-normal": {
		"path": "typography.tracking.normal",
		"cssName": "tracking-normal",
		"type": "dimension",
		"value": "0"
	},
	"tracking-wide": {
		"path": "typography.tracking.wide",
		"cssName": "tracking-wide",
		"type": "dimension",
		"value": "0.04em"
	},
	"tracking-wider": {
		"path": "typography.tracking.wider",
		"cssName": "tracking-wider",
		"type": "dimension",
		"value": "0.1em"
	},
	"heading-font": {
		"path": "typography.heading.font",
		"cssName": "heading-font",
		"type": "fontFamily",
		"value": "var(--font-display)"
	},
	"heading-weight": {
		"path": "typography.heading.weight",
		"cssName": "heading-weight",
		"type": "number",
		"value": "var(--font-weight-semibold)"
	},
	"heading-tracking": {
		"path": "typography.heading.tracking",
		"cssName": "heading-tracking",
		"type": "dimension",
		"value": "var(--tracking-tight)"
	},
	"heading-transform": {
		"path": "typography.heading.transform",
		"cssName": "heading-transform",
		"type": "string",
		"value": "none"
	},
	"body-font": {
		"path": "typography.body.font",
		"cssName": "body-font",
		"type": "fontFamily",
		"value": "var(--font-body)"
	},
	"body-weight": {
		"path": "typography.body.weight",
		"cssName": "body-weight",
		"type": "number",
		"value": "var(--font-weight-normal)"
	},
	"body-tracking": {
		"path": "typography.body.tracking",
		"cssName": "body-tracking",
		"type": "dimension",
		"value": "var(--tracking-normal)"
	},
	"mono-font": {
		"path": "typography.monoFont",
		"cssName": "mono-font",
		"type": "fontFamily",
		"value": "var(--font-mono)"
	},
	"label-font": {
		"path": "typography.label.font",
		"cssName": "label-font",
		"type": "fontFamily",
		"value": "var(--font-body)"
	},
	"label-weight": {
		"path": "typography.label.weight",
		"cssName": "label-weight",
		"type": "number",
		"value": "var(--font-weight-semibold)"
	},
	"label-tracking": {
		"path": "typography.label.tracking",
		"cssName": "label-tracking",
		"type": "dimension",
		"value": "var(--tracking-wide)"
	},
	"label-transform": {
		"path": "typography.label.transform",
		"cssName": "label-transform",
		"type": "string",
		"value": "uppercase"
	},
	"display-font": {
		"path": "typography.displayFont",
		"cssName": "display-font",
		"type": "fontFamily",
		"value": "var(--font-display)"
	},
	"edge-highlight": {
		"path": "color.edgeHighlight",
		"cssName": "edge-highlight",
		"type": "shadow",
		"value": "inset 0 1px 0 0 var(--hover-bg)"
	},
	"ring-width": {
		"path": "color.ring.width",
		"cssName": "ring-width",
		"type": "dimension",
		"value": "3px"
	},
	"ring-offset": {
		"path": "color.ring.offset",
		"cssName": "ring-offset",
		"type": "dimension",
		"value": "2px"
	},
	"ring-opacity": {
		"path": "color.ring.opacity",
		"cssName": "ring-opacity",
		"type": "number",
		"value": "25%"
	},
	"ring": {
		"path": "color.ring.default",
		"cssName": "ring",
		"type": "color",
		"value": "oklch(0.67 0.16 var(--brand-hue, 58)/ 40%)"
	},
	"ring-color": {
		"path": "color.ring.color",
		"cssName": "ring-color",
		"type": "color",
		"value": "var(--primary)"
	},
	"surface-0": {
		"path": "color.surface.0",
		"cssName": "surface-0",
		"type": "color",
		"value": "oklch(0.97 0.004 80)"
	},
	"surface-1": {
		"path": "color.surface.1",
		"cssName": "surface-1",
		"type": "color",
		"value": "oklch(1 0 0)"
	},
	"surface-2": {
		"path": "color.surface.2",
		"cssName": "surface-2",
		"type": "color",
		"value": "oklch(1 0 0)"
	},
	"surface-3": {
		"path": "color.surface.3",
		"cssName": "surface-3",
		"type": "color",
		"value": "oklch(1 0 0)"
	},
	"surface-overlay": {
		"path": "color.surface.overlay",
		"cssName": "surface-overlay",
		"type": "string",
		"value": "none"
	},
	"surface-overlay-blend": {
		"path": "color.surface.overlay-blend",
		"cssName": "surface-overlay-blend",
		"type": "string",
		"value": "overlay"
	},
	"surface-overlay-opacity": {
		"path": "color.surface.overlay-opacity",
		"cssName": "surface-overlay-opacity",
		"type": "number",
		"value": "1"
	},
	"surface-inverse": {
		"path": "color.surface.inverse",
		"cssName": "surface-inverse",
		"type": "color",
		"value": "var(--foreground)"
	},
	"border-width": {
		"path": "color.border.width",
		"cssName": "border-width",
		"type": "dimension",
		"value": "1px"
	},
	"border": {
		"path": "color.border.default",
		"cssName": "border",
		"type": "color",
		"value": "oklch(0.147 0.004 49.25 / 25%)"
	},
	"border-subtle": {
		"path": "color.border.subtle",
		"cssName": "border-subtle",
		"type": "color",
		"value": "color-mix(in oklch, var(--border) 60%, transparent)"
	},
	"border-strong": {
		"path": "color.border.strong",
		"cssName": "border-strong",
		"type": "color",
		"value": "color-mix(in oklch, var(--foreground) 55%, transparent)"
	},
	"corner-image": {
		"path": "color.cornerImage",
		"cssName": "corner-image",
		"type": "string",
		"value": "none"
	},
	"gradient-primary": {
		"path": "color.gradient.primary",
		"cssName": "gradient-primary",
		"type": "dimension",
		"value": "linear-gradient(180deg, var(--primary) 0%, transparent 92%)"
	},
	"gradient-accent": {
		"path": "color.gradient.accent",
		"cssName": "gradient-accent",
		"type": "dimension",
		"value": "linear-gradient(180deg, var(--accent) 0%, transparent 92%)"
	},
	"gradient-surface": {
		"path": "color.gradient.surface",
		"cssName": "gradient-surface",
		"type": "string",
		"value": "none"
	},
	"glow-primary": {
		"path": "color.glow.primary",
		"cssName": "glow-primary",
		"type": "dimension",
		"value": "radial-gradient(circle, color-mix(in oklch, var(--primary) 8%, transparent), transparent 60%)"
	},
	"glow-accent": {
		"path": "color.glow.accent",
		"cssName": "glow-accent",
		"type": "dimension",
		"value": "radial-gradient(circle, color-mix(in oklch, var(--accent) 8%, transparent), transparent 60%)"
	},
	"glow-default": {
		"path": "color.glow.default",
		"cssName": "glow-default",
		"type": "string",
		"value": "none"
	},
	"glow-muted": {
		"path": "color.glow.muted",
		"cssName": "glow-muted",
		"type": "string",
		"value": "none"
	},
	"glow-info": {
		"path": "color.glow.info",
		"cssName": "glow-info",
		"type": "string",
		"value": "none"
	},
	"glow-success": {
		"path": "color.glow.success",
		"cssName": "glow-success",
		"type": "string",
		"value": "none"
	},
	"glow-warning": {
		"path": "color.glow.warning",
		"cssName": "glow-warning",
		"type": "string",
		"value": "none"
	},
	"glow-destructive": {
		"path": "color.glow.destructive",
		"cssName": "glow-destructive",
		"type": "string",
		"value": "none"
	},
	"shape-pill": {
		"path": "color.shapePill",
		"cssName": "shape-pill",
		"type": "dimension",
		"value": "9999px"
	},
	"background": {
		"path": "color.background",
		"cssName": "background",
		"type": "color",
		"value": "oklch(0.975 0.002 106.423)"
	},
	"foreground": {
		"path": "color.foreground",
		"cssName": "foreground",
		"type": "color",
		"value": "oklch(0.147 0.004 49.25)"
	},
	"primary": {
		"path": "color.primary.default",
		"cssName": "primary",
		"type": "color",
		"value": "oklch(0.67 0.16 var(--brand-hue, 58))"
	},
	"primary-foreground": {
		"path": "color.primary.foreground",
		"cssName": "primary-foreground",
		"type": "color",
		"value": "oklch(0.99 0.02 95)"
	},
	"primary-hover": {
		"path": "color.primary.hover",
		"cssName": "primary-hover",
		"type": "color",
		"value": "color-mix(in oklch, var(--primary) 88%, var(--foreground))"
	},
	"secondary": {
		"path": "color.secondary.default",
		"cssName": "secondary",
		"type": "color",
		"value": "oklch(0.95 0.02 302)"
	},
	"secondary-foreground": {
		"path": "color.secondary.foreground",
		"cssName": "secondary-foreground",
		"type": "color",
		"value": "oklch(0.33 0.11 302)"
	},
	"muted": {
		"path": "color.muted.default",
		"cssName": "muted",
		"type": "color",
		"value": "oklch(0.95 0.003 106.424)"
	},
	"muted-foreground": {
		"path": "color.muted.foreground",
		"cssName": "muted-foreground",
		"type": "color",
		"value": "oklch(0.40 0.02 58.071)"
	},
	"accent": {
		"path": "color.brandAccent.default",
		"cssName": "accent",
		"type": "color",
		"value": "oklch(0.61 0.23 var(--brand-hue, 303))"
	},
	"accent-foreground": {
		"path": "color.brandAccent.foreground",
		"cssName": "accent-foreground",
		"type": "color",
		"value": "oklch(0.98 0.02 301)"
	},
	"destructive": {
		"path": "color.destructive.default",
		"cssName": "destructive",
		"type": "color",
		"value": "oklch(0.577 0.245 27.325)"
	},
	"destructive-foreground": {
		"path": "color.destructive.foreground",
		"cssName": "destructive-foreground",
		"type": "color",
		"value": "oklch(0.99 0.02 25)"
	},
	"input": {
		"path": "color.input",
		"cssName": "input",
		"type": "color",
		"value": "oklch(0.147 0.004 49.25 / 25%)"
	},
	"success": {
		"path": "color.success.default",
		"cssName": "success",
		"type": "color",
		"value": "oklch(0.65 0.2 150)"
	},
	"success-foreground": {
		"path": "color.success.foreground",
		"cssName": "success-foreground",
		"type": "color",
		"value": "oklch(0.99 0.02 150)"
	},
	"warning": {
		"path": "color.warning.default",
		"cssName": "warning",
		"type": "color",
		"value": "oklch(0.75 0.15 75)"
	},
	"warning-foreground": {
		"path": "color.warning.foreground",
		"cssName": "warning-foreground",
		"type": "color",
		"value": "oklch(0.2 0.04 75)"
	},
	"info": {
		"path": "color.info.default",
		"cssName": "info",
		"type": "color",
		"value": "oklch(0.6 0.18 260)"
	},
	"info-foreground": {
		"path": "color.info.foreground",
		"cssName": "info-foreground",
		"type": "color",
		"value": "oklch(0.99 0.02 260)"
	},
	"status-gray": {
		"path": "color.status.gray",
		"cssName": "status-gray",
		"type": "color",
		"value": "oklch(0.5 0.01 58)"
	},
	"status-blue": {
		"path": "color.status.blue",
		"cssName": "status-blue",
		"type": "color",
		"value": "oklch(0.6 0.18 260)"
	},
	"status-red": {
		"path": "color.status.red",
		"cssName": "status-red",
		"type": "color",
		"value": "oklch(0.55 0.25 25)"
	},
	"status-green": {
		"path": "color.status.green",
		"cssName": "status-green",
		"type": "color",
		"value": "oklch(0.65 0.2 150)"
	},
	"status-yellow": {
		"path": "color.status.yellow",
		"cssName": "status-yellow",
		"type": "color",
		"value": "oklch(0.7 0.18 55)"
	},
	"status-purple": {
		"path": "color.status.purple",
		"cssName": "status-purple",
		"type": "color",
		"value": "oklch(0.6 0.2 300)"
	},
	"status-orange": {
		"path": "color.status.orange",
		"cssName": "status-orange",
		"type": "color",
		"value": "oklch(0.6 0.2 35)"
	},
	"status-amber": {
		"path": "color.status.amber",
		"cssName": "status-amber",
		"type": "color",
		"value": "oklch(0.75 0.15 75)"
	},
	"syntax-keyword": {
		"path": "color.syntax.keyword",
		"cssName": "syntax-keyword",
		"type": "color",
		"value": "oklch(0.55 0.20 280)"
	},
	"syntax-string": {
		"path": "color.syntax.string",
		"cssName": "syntax-string",
		"type": "color",
		"value": "oklch(0.50 0.15 152)"
	},
	"syntax-number": {
		"path": "color.syntax.number",
		"cssName": "syntax-number",
		"type": "color",
		"value": "oklch(0.50 0.16 250)"
	},
	"syntax-function": {
		"path": "color.syntax.function",
		"cssName": "syntax-function",
		"type": "color",
		"value": "oklch(0.55 0.22 310)"
	},
	"syntax-type": {
		"path": "color.syntax.type",
		"cssName": "syntax-type",
		"type": "color",
		"value": "oklch(0.55 0.18 200)"
	},
	"syntax-tag": {
		"path": "color.syntax.tag",
		"cssName": "syntax-tag",
		"type": "color",
		"value": "oklch(0.50 0.20 25)"
	},
	"syntax-attr": {
		"path": "color.syntax.attr",
		"cssName": "syntax-attr",
		"type": "color",
		"value": "oklch(0.55 0.16 60)"
	},
	"syntax-comment": {
		"path": "color.syntax.comment",
		"cssName": "syntax-comment",
		"type": "color",
		"value": "oklch(0.55 0.02 250)"
	},
	"syntax-variable": {
		"path": "color.syntax.variable",
		"cssName": "syntax-variable",
		"type": "color",
		"value": "var(--foreground)"
	},
	"sidebar": {
		"path": "color.sidebar.default",
		"cssName": "sidebar",
		"type": "color",
		"value": "oklch(0.99 0.001 106.423)"
	},
	"sidebar-foreground": {
		"path": "color.sidebar.foreground",
		"cssName": "sidebar-foreground",
		"type": "color",
		"value": "oklch(0.147 0.004 49.25)"
	},
	"sidebar-primary": {
		"path": "color.sidebar.primary",
		"cssName": "sidebar-primary",
		"type": "color",
		"value": "oklch(0.67 0.16 var(--brand-hue, 58))"
	},
	"sidebar-primary-foreground": {
		"path": "color.sidebar.primary-foreground",
		"cssName": "sidebar-primary-foreground",
		"type": "color",
		"value": "oklch(0.99 0.02 95)"
	},
	"sidebar-accent": {
		"path": "color.sidebar.accent",
		"cssName": "sidebar-accent",
		"type": "color",
		"value": "oklch(0.95 0.001 106)"
	},
	"sidebar-accent-foreground": {
		"path": "color.sidebar.accent-foreground",
		"cssName": "sidebar-accent-foreground",
		"type": "color",
		"value": "oklch(0.147 0.004 49.25)"
	},
	"sidebar-border": {
		"path": "color.sidebar.border",
		"cssName": "sidebar-border",
		"type": "color",
		"value": "oklch(0.147 0.004 49.25 / 12%)"
	},
	"sidebar-ring": {
		"path": "color.sidebar.ring",
		"cssName": "sidebar-ring",
		"type": "color",
		"value": "oklch(0.67 0.16 58 / 20%)"
	},
	"hue-red": {
		"path": "color.hue.red",
		"cssName": "hue-red",
		"type": "color",
		"value": "oklch(0.62 0.21 25)"
	},
	"hue-orange": {
		"path": "color.hue.orange",
		"cssName": "hue-orange",
		"type": "color",
		"value": "oklch(0.68 0.18 50)"
	},
	"hue-amber": {
		"path": "color.hue.amber",
		"cssName": "hue-amber",
		"type": "color",
		"value": "oklch(0.74 0.16 75)"
	},
	"hue-yellow": {
		"path": "color.hue.yellow",
		"cssName": "hue-yellow",
		"type": "color",
		"value": "oklch(0.82 0.16 95)"
	},
	"hue-lime": {
		"path": "color.hue.lime",
		"cssName": "hue-lime",
		"type": "color",
		"value": "oklch(0.78 0.18 130)"
	},
	"hue-green": {
		"path": "color.hue.green",
		"cssName": "hue-green",
		"type": "color",
		"value": "oklch(0.62 0.16 150)"
	},
	"hue-teal": {
		"path": "color.hue.teal",
		"cssName": "hue-teal",
		"type": "color",
		"value": "oklch(0.62 0.13 185)"
	},
	"hue-cyan": {
		"path": "color.hue.cyan",
		"cssName": "hue-cyan",
		"type": "color",
		"value": "oklch(0.68 0.13 220)"
	},
	"hue-sky": {
		"path": "color.hue.sky",
		"cssName": "hue-sky",
		"type": "color",
		"value": "oklch(0.65 0.16 235)"
	},
	"hue-blue": {
		"path": "color.hue.blue",
		"cssName": "hue-blue",
		"type": "color",
		"value": "oklch(0.55 0.20 260)"
	},
	"hue-indigo": {
		"path": "color.hue.indigo",
		"cssName": "hue-indigo",
		"type": "color",
		"value": "oklch(0.50 0.20 280)"
	},
	"hue-violet": {
		"path": "color.hue.violet",
		"cssName": "hue-violet",
		"type": "color",
		"value": "oklch(0.55 0.22 295)"
	},
	"hue-purple": {
		"path": "color.hue.purple",
		"cssName": "hue-purple",
		"type": "color",
		"value": "oklch(0.55 0.22 310)"
	},
	"hue-fuchsia": {
		"path": "color.hue.fuchsia",
		"cssName": "hue-fuchsia",
		"type": "color",
		"value": "oklch(0.60 0.25 330)"
	},
	"hue-pink": {
		"path": "color.hue.pink",
		"cssName": "hue-pink",
		"type": "color",
		"value": "oklch(0.68 0.20 345)"
	},
	"hue-rose": {
		"path": "color.hue.rose",
		"cssName": "hue-rose",
		"type": "color",
		"value": "oklch(0.62 0.21 5)"
	},
	"backdrop": {
		"path": "color.backdrop",
		"cssName": "backdrop",
		"type": "color",
		"value": "oklch(0 0 0 / 40%)"
	},
	"scrim": {
		"path": "color.scrim",
		"cssName": "scrim",
		"type": "color",
		"value": "oklch(0 0 0 / 20%)"
	},
	"divider": {
		"path": "color.divider",
		"cssName": "divider",
		"type": "color",
		"value": "var(--border-subtle)"
	},
	"accent-dashboard": {
		"path": "color.accent.dashboard",
		"cssName": "accent-dashboard",
		"type": "color",
		"value": "oklch(0.62 0.18 260)"
	},
	"accent-pipelines": {
		"path": "color.accent.pipelines",
		"cssName": "accent-pipelines",
		"type": "color",
		"value": "oklch(0.67 0.16 58)"
	},
	"accent-agents": {
		"path": "color.accent.agents",
		"cssName": "accent-agents",
		"type": "color",
		"value": "oklch(0.58 0.22 295)"
	},
	"accent-skills": {
		"path": "color.accent.skills",
		"cssName": "accent-skills",
		"type": "color",
		"value": "oklch(0.62 0.16 152)"
	},
	"accent-git": {
		"path": "color.accent.git",
		"cssName": "accent-git",
		"type": "color",
		"value": "oklch(0.58 0.21 25)"
	},
	"accent-collections": {
		"path": "color.accent.collections",
		"cssName": "accent-collections",
		"type": "color",
		"value": "oklch(0.65 0.15 200)"
	},
	"accent-canvas": {
		"path": "color.accent.canvas",
		"cssName": "accent-canvas",
		"type": "color",
		"value": "oklch(0.72 0.16 80)"
	},
	"accent-settings": {
		"path": "color.accent.settings",
		"cssName": "accent-settings",
		"type": "color",
		"value": "oklch(0.55 0.02 60)"
	},
	"accent-chat": {
		"path": "color.accent.chat",
		"cssName": "accent-chat",
		"type": "color",
		"value": "oklch(0.62 0.13 280)"
	},
	"accent-events": {
		"path": "color.accent.events",
		"cssName": "accent-events",
		"type": "color",
		"value": "oklch(0.62 0.18 30)"
	},
	"accent-specs": {
		"path": "color.accent.specs",
		"cssName": "accent-specs",
		"type": "color",
		"value": "oklch(0.55 0.05 240)"
	},
	"accent-theme": {
		"path": "color.accent.theme",
		"cssName": "accent-theme",
		"type": "color",
		"value": "oklch(0.66 0.20 320)"
	},
	"accent-setup": {
		"path": "color.accent.setup",
		"cssName": "accent-setup",
		"type": "color",
		"value": "oklch(0.68 0.16 110)"
	},
	"accent-overview": {
		"path": "color.accent.overview",
		"cssName": "accent-overview",
		"type": "color",
		"value": "oklch(0.60 0.10 230)"
	},
	"accent-mesh": {
		"path": "color.accent.mesh",
		"cssName": "accent-mesh",
		"type": "color",
		"value": "oklch(0.64 0.14 195)"
	},
	"mix-bg": {
		"path": "color.mix.bgMix",
		"cssName": "mix-bg",
		"type": "number",
		"value": "18%"
	},
	"mix-border": {
		"path": "color.mix.borderMix",
		"cssName": "mix-border",
		"type": "number",
		"value": "35%"
	},
	"mix-hover": {
		"path": "color.mix.hoverMix",
		"cssName": "mix-hover",
		"type": "number",
		"value": "24%"
	},
	"z-base": {
		"path": "z.base",
		"cssName": "z-base",
		"type": "number",
		"value": "0"
	},
	"z-elevated": {
		"path": "z.elevated",
		"cssName": "z-elevated",
		"type": "number",
		"value": "1"
	},
	"z-dropdown": {
		"path": "z.dropdown",
		"cssName": "z-dropdown",
		"type": "number",
		"value": "10"
	},
	"z-sticky": {
		"path": "z.sticky",
		"cssName": "z-sticky",
		"type": "number",
		"value": "20"
	},
	"z-popover": {
		"path": "z.popover",
		"cssName": "z-popover",
		"type": "number",
		"value": "30"
	},
	"z-dialog": {
		"path": "z.dialog",
		"cssName": "z-dialog",
		"type": "number",
		"value": "40"
	},
	"z-toast": {
		"path": "z.toast",
		"cssName": "z-toast",
		"type": "number",
		"value": "50"
	},
	"z-max": {
		"path": "z.max",
		"cssName": "z-max",
		"type": "number",
		"value": "100"
	},
	"ease-out": {
		"path": "motion.ease.out",
		"cssName": "ease-out",
		"type": "cubicBezier",
		"value": "cubic-bezier(0.16, 1, 0.3, 1)"
	},
	"ease-out-soft": {
		"path": "motion.ease.out-soft",
		"cssName": "ease-out-soft",
		"type": "cubicBezier",
		"value": "cubic-bezier(0.33, 1, 0.68, 1)"
	},
	"ease-in": {
		"path": "motion.ease.in",
		"cssName": "ease-in",
		"type": "cubicBezier",
		"value": "cubic-bezier(0.7, 0, 0.84, 0)"
	},
	"ease-in-out": {
		"path": "motion.ease.in-out",
		"cssName": "ease-in-out",
		"type": "cubicBezier",
		"value": "cubic-bezier(0.65, 0, 0.35, 1)"
	},
	"ease-spring": {
		"path": "motion.ease.spring",
		"cssName": "ease-spring",
		"type": "cubicBezier",
		"value": "cubic-bezier(0.34, 1.56, 0.64, 1)"
	},
	"duration-instant": {
		"path": "motion.duration.instant",
		"cssName": "duration-instant",
		"type": "duration",
		"value": "60ms"
	},
	"duration-fast": {
		"path": "motion.duration.fast",
		"cssName": "duration-fast",
		"type": "duration",
		"value": "120ms"
	},
	"duration-medium": {
		"path": "motion.duration.medium",
		"cssName": "duration-medium",
		"type": "duration",
		"value": "180ms"
	},
	"duration-slow": {
		"path": "motion.duration.slow",
		"cssName": "duration-slow",
		"type": "duration",
		"value": "280ms"
	},
	"duration-deliberate": {
		"path": "motion.duration.deliberate",
		"cssName": "duration-deliberate",
		"type": "duration",
		"value": "420ms"
	},
	"duration-glacial": {
		"path": "motion.duration.glacial",
		"cssName": "duration-glacial",
		"type": "duration",
		"value": "14000ms"
	},
	"tooltip-show-delay": {
		"path": "motion.tooltip.show-delay",
		"cssName": "tooltip-show-delay",
		"type": "duration",
		"value": "300ms"
	},
	"tooltip-hide-delay": {
		"path": "motion.tooltip.hide-delay",
		"cssName": "tooltip-hide-delay",
		"type": "duration",
		"value": "100ms"
	},
	"focus-ring": {
		"path": "interaction.focus.ring",
		"cssName": "focus-ring",
		"type": "dimension",
		"value": "2px solid var(--ring-color)"
	},
	"focus-ring-offset": {
		"path": "interaction.focus.ring-offset",
		"cssName": "focus-ring-offset",
		"type": "dimension",
		"value": "2px"
	},
	"hover-strength": {
		"path": "interaction.hover.strength",
		"cssName": "hover-strength",
		"type": "number",
		"value": "6%"
	},
	"hover-bg": {
		"path": "interaction.hover.bg",
		"cssName": "hover-bg",
		"type": "color",
		"value": "color-mix(in oklch, var(--foreground) var(--hover-strength), transparent)"
	},
	"active-strength": {
		"path": "interaction.active.strength",
		"cssName": "active-strength",
		"type": "number",
		"value": "10%"
	},
	"active-bg": {
		"path": "interaction.active.bg",
		"cssName": "active-bg",
		"type": "color",
		"value": "color-mix(in oklch, var(--foreground) var(--active-strength), transparent)"
	},
	"selected-strength": {
		"path": "interaction.selected.strength",
		"cssName": "selected-strength",
		"type": "number",
		"value": "14%"
	},
	"selected-bg": {
		"path": "interaction.selected.bg",
		"cssName": "selected-bg",
		"type": "color",
		"value": "color-mix(in oklch, var(--primary) var(--selected-strength), transparent)"
	},
	"selected-border": {
		"path": "interaction.selected.border",
		"cssName": "selected-border",
		"type": "color",
		"value": "color-mix(in oklch, var(--primary) 40%, transparent)"
	},
	"breakpoint-mobile": {
		"path": "breakpoint.mobile",
		"cssName": "breakpoint-mobile",
		"type": "dimension",
		"value": "390px"
	},
	"breakpoint-tablet": {
		"path": "breakpoint.tablet",
		"cssName": "breakpoint-tablet",
		"type": "dimension",
		"value": "768px"
	},
	"breakpoint-desktop": {
		"path": "breakpoint.desktop",
		"cssName": "breakpoint-desktop",
		"type": "dimension",
		"value": "1280px"
	},
	"density-row-py": {
		"path": "density.row-py",
		"cssName": "density-row-py",
		"type": "dimension",
		"value": "var(--space-2)"
	},
	"density-row-px": {
		"path": "density.row-px",
		"cssName": "density-row-px",
		"type": "dimension",
		"value": "var(--space-3)"
	},
	"density-row-gap": {
		"path": "density.row-gap",
		"cssName": "density-row-gap",
		"type": "dimension",
		"value": "var(--space-2)"
	},
	"density-text": {
		"path": "density.text",
		"cssName": "density-text",
		"type": "dimension",
		"value": "var(--text-base)"
	},
	"density-icon": {
		"path": "density.icon",
		"cssName": "density-icon",
		"type": "dimension",
		"value": "16px"
	},
	"shadow-xs": {
		"path": "shadow.xs",
		"cssName": "shadow-xs",
		"type": "shadow",
		"value": "0 1px 2px 0 oklch(0 0 0 / 5%)"
	},
	"shadow-sm": {
		"path": "shadow.sm",
		"cssName": "shadow-sm",
		"type": "shadow",
		"value": "0 1px 3px 0 oklch(0 0 0 / 7%), 0 1px 2px -1px oklch(0 0 0 / 5%)"
	},
	"shadow-md": {
		"path": "shadow.md",
		"cssName": "shadow-md",
		"type": "shadow",
		"value": "0 4px 12px -2px oklch(0 0 0 / 9%), 0 2px 4px -2px oklch(0 0 0 / 6%)"
	},
	"shadow-lg": {
		"path": "shadow.lg",
		"cssName": "shadow-lg",
		"type": "shadow",
		"value": "0 12px 32px -6px oklch(0 0 0 / 13%), 0 4px 12px -4px oklch(0 0 0 / 8%)"
	},
	"shadow-xl": {
		"path": "shadow.xl",
		"cssName": "shadow-xl",
		"type": "shadow",
		"value": "0 24px 64px -12px oklch(0 0 0 / 18%), 0 8px 24px -8px oklch(0 0 0 / 10%)"
	},
	"shadow-card-rest": {
		"path": "shadow.card-rest",
		"cssName": "shadow-card-rest",
		"type": "shadow",
		"value": "0 1px 3px oklch(0.20 0.04 60 / 6%), 0 0 0 1px oklch(0.20 0.04 60 / 6%)"
	},
	"shadow-hover": {
		"path": "shadow.hover",
		"cssName": "shadow-hover",
		"type": "shadow",
		"value": "0 8px 20px oklch(0.20 0.04 60 / 12%), 0 0 0 1px oklch(0.20 0.04 60 / 8%)"
	},
	"shadow-sticky": {
		"path": "shadow.sticky",
		"cssName": "shadow-sticky",
		"type": "shadow",
		"value": "0 2px 6px -2px oklch(0 0 0 / 8%), 0 1px 2px oklch(0 0 0 / 4%)"
	},
	"shadow-accent": {
		"path": "shadow.accent",
		"cssName": "shadow-accent",
		"type": "shadow",
		"value": "0 0 0 3px color-mix(in oklch, var(--primary) 22%, transparent)"
	},
	"shadow-inset-soft": {
		"path": "shadow.inset-soft",
		"cssName": "shadow-inset-soft",
		"type": "shadow",
		"value": "inset 0 1px 2px oklch(0 0 0 / 5%)"
	},
	"shadow-inset-ring": {
		"path": "shadow.inset-ring",
		"cssName": "shadow-inset-ring",
		"type": "shadow",
		"value": "inset 0 0 0 1px var(--border), inset 0 1px 0 var(--background)"
	},
	"shadow-edge-top": {
		"path": "shadow.edge-top",
		"cssName": "shadow-edge-top",
		"type": "shadow",
		"value": "0 -8px 24px -8px oklch(0 0 0 / 8%), 0 -1px 0 oklch(0 0 0 / 4%)"
	},
	"scrollbar-thumb": {
		"path": "scrollbar.thumb",
		"cssName": "scrollbar-thumb",
		"type": "color",
		"value": "color-mix(in oklab, var(--foreground) 22%, transparent)"
	},
	"scrollbar-thumb-hover": {
		"path": "scrollbar.thumb-hover",
		"cssName": "scrollbar-thumb-hover",
		"type": "color",
		"value": "color-mix(in oklab, var(--foreground) 38%, transparent)"
	},
	"scrollbar-track": {
		"path": "scrollbar.track",
		"cssName": "scrollbar-track",
		"type": "string",
		"value": "transparent"
	},
	"scrollbar-width": {
		"path": "scrollbar.width",
		"cssName": "scrollbar-width",
		"type": "dimension",
		"value": "10px"
	},
	"scrollbar-width-thin": {
		"path": "scrollbar.width-thin",
		"cssName": "scrollbar-width-thin",
		"type": "dimension",
		"value": "6px"
	},
	"scrollbar-radius": {
		"path": "scrollbar.radius",
		"cssName": "scrollbar-radius",
		"type": "dimension",
		"value": "999px"
	},
	"alert": {
		"path": "component.alert.bg",
		"cssName": "alert",
		"type": "color",
		"value": "var(--surface-1)"
	},
	"alert-foreground": {
		"path": "component.alert.fg",
		"cssName": "alert-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"alert-stripe": {
		"path": "component.alert.stripe",
		"cssName": "alert-stripe",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"avatar": {
		"path": "component.avatar.bg",
		"cssName": "avatar",
		"type": "color",
		"value": "var(--muted)"
	},
	"avatar-foreground": {
		"path": "component.avatar.fg",
		"cssName": "avatar-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"badge": {
		"path": "component.badge.bg",
		"cssName": "badge",
		"type": "color",
		"value": "var(--muted)"
	},
	"badge-foreground": {
		"path": "component.badge.fg",
		"cssName": "badge-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"badge-border": {
		"path": "component.badge.border",
		"cssName": "badge-border",
		"type": "color",
		"value": "var(--border)"
	},
	"card": {
		"path": "component.card.bg",
		"cssName": "card",
		"type": "color",
		"value": "var(--surface-1)"
	},
	"card-foreground": {
		"path": "component.card.fg",
		"cssName": "card-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"card-backdrop-filter": {
		"path": "component.card.backdrop-filter",
		"cssName": "card-backdrop-filter",
		"type": "string",
		"value": "none"
	},
	"card-texture": {
		"path": "component.card.texture",
		"cssName": "card-texture",
		"type": "string",
		"value": "none"
	},
	"card-texture-blend": {
		"path": "component.card.texture-blend",
		"cssName": "card-texture-blend",
		"type": "string",
		"value": "overlay"
	},
	"card-texture-opacity": {
		"path": "component.card.texture-opacity",
		"cssName": "card-texture-opacity",
		"type": "number",
		"value": "1"
	},
	"chip": {
		"path": "component.chip.bg",
		"cssName": "chip",
		"type": "color",
		"value": "var(--muted)"
	},
	"chip-foreground": {
		"path": "component.chip.fg",
		"cssName": "chip-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"chip-border": {
		"path": "component.chip.border",
		"cssName": "chip-border",
		"type": "color",
		"value": "var(--border)"
	},
	"code-block": {
		"path": "component.code-block.bg",
		"cssName": "code-block",
		"type": "color",
		"value": "var(--surface-1)"
	},
	"code-block-foreground": {
		"path": "component.code-block.fg",
		"cssName": "code-block-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"combobox": {
		"path": "component.combobox.bg",
		"cssName": "combobox",
		"type": "color",
		"value": "var(--popover)"
	},
	"combobox-foreground": {
		"path": "component.combobox.fg",
		"cssName": "combobox-foreground",
		"type": "color",
		"value": "var(--popover-foreground)"
	},
	"context-menu": {
		"path": "component.context-menu.bg",
		"cssName": "context-menu",
		"type": "color",
		"value": "var(--background)"
	},
	"context-menu-foreground": {
		"path": "component.context-menu.fg",
		"cssName": "context-menu-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"dropdown": {
		"path": "component.dropdown.bg",
		"cssName": "dropdown",
		"type": "color",
		"value": "var(--background)"
	},
	"dropdown-foreground": {
		"path": "component.dropdown.fg",
		"cssName": "dropdown-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"empty-state": {
		"path": "component.empty-state.bg",
		"cssName": "empty-state",
		"type": "color",
		"value": "var(--muted)"
	},
	"empty-state-foreground": {
		"path": "component.empty-state.fg",
		"cssName": "empty-state-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"empty-state-border": {
		"path": "component.empty-state.border",
		"cssName": "empty-state-border",
		"type": "color",
		"value": "var(--border)"
	},
	"error-boundary": {
		"path": "component.error-boundary.bg",
		"cssName": "error-boundary",
		"type": "color",
		"value": "var(--muted)"
	},
	"error-boundary-border": {
		"path": "component.error-boundary.border",
		"cssName": "error-boundary-border",
		"type": "color",
		"value": "var(--destructive)"
	},
	"error-boundary-foreground": {
		"path": "component.error-boundary.fg",
		"cssName": "error-boundary-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"error-boundary-message-foreground": {
		"path": "component.error-boundary.message-fg",
		"cssName": "error-boundary-message-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"error-boundary-icon": {
		"path": "component.error-boundary.icon-fg",
		"cssName": "error-boundary-icon",
		"type": "color",
		"value": "var(--destructive)"
	},
	"error-boundary-retry-bg": {
		"path": "component.error-boundary.retry-bg",
		"cssName": "error-boundary-retry-bg",
		"type": "color",
		"value": "var(--surface-1)"
	},
	"error-boundary-retry-foreground": {
		"path": "component.error-boundary.retry-fg",
		"cssName": "error-boundary-retry-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"error-state": {
		"path": "component.error-state.bg",
		"cssName": "error-state",
		"type": "color",
		"value": "var(--muted)"
	},
	"error-state-foreground": {
		"path": "component.error-state.fg",
		"cssName": "error-state-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"error-state-border": {
		"path": "component.error-state.border",
		"cssName": "error-state-border",
		"type": "color",
		"value": "var(--border)"
	},
	"field-hint": {
		"path": "component.field.hint",
		"cssName": "field-hint",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"field-error": {
		"path": "component.field.error",
		"cssName": "field-error",
		"type": "color",
		"value": "var(--destructive)"
	},
	"hover-card": {
		"path": "component.hover-card.bg",
		"cssName": "hover-card",
		"type": "color",
		"value": "var(--popover)"
	},
	"hover-card-foreground": {
		"path": "component.hover-card.fg",
		"cssName": "hover-card-foreground",
		"type": "color",
		"value": "var(--popover-foreground)"
	},
	"label-foreground": {
		"path": "component.label.fg",
		"cssName": "label-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"label-required": {
		"path": "component.label.required",
		"cssName": "label-required",
		"type": "color",
		"value": "var(--destructive)"
	},
	"menu-item-foreground": {
		"path": "component.menu-item.fg",
		"cssName": "menu-item-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"menu-item-hover-bg": {
		"path": "component.menu-item.hover-bg",
		"cssName": "menu-item-hover-bg",
		"type": "color",
		"value": "var(--muted)"
	},
	"navbar": {
		"path": "component.navbar.bg",
		"cssName": "navbar",
		"type": "color",
		"value": "var(--background)"
	},
	"navbar-foreground": {
		"path": "component.navbar.fg",
		"cssName": "navbar-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"page-foreground": {
		"path": "component.page.fg",
		"cssName": "page-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"page-hover-bg": {
		"path": "component.page.hover-bg",
		"cssName": "page-hover-bg",
		"type": "color",
		"value": "var(--muted)"
	},
	"page-current-bg": {
		"path": "component.page.current-bg",
		"cssName": "page-current-bg",
		"type": "color",
		"value": "var(--primary)"
	},
	"page-current-foreground": {
		"path": "component.page.current-fg",
		"cssName": "page-current-foreground",
		"type": "color",
		"value": "var(--primary-foreground)"
	},
	"popover": {
		"path": "component.popover.bg",
		"cssName": "popover",
		"type": "color",
		"value": "var(--surface-1)"
	},
	"popover-foreground": {
		"path": "component.popover.fg",
		"cssName": "popover-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"prose-foreground": {
		"path": "component.prose.fg",
		"cssName": "prose-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"radio-foreground": {
		"path": "component.radio.fg",
		"cssName": "radio-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"scroll-area-thumb": {
		"path": "component.scroll-area.thumb",
		"cssName": "scroll-area-thumb",
		"type": "color",
		"value": "var(--border)"
	},
	"scroll-area-thumb-hover": {
		"path": "component.scroll-area.thumb-hover",
		"cssName": "scroll-area-thumb-hover",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"separator-line": {
		"path": "component.separator.line",
		"cssName": "separator-line",
		"type": "color",
		"value": "var(--border)"
	},
	"spinner-foreground": {
		"path": "component.spinner.fg",
		"cssName": "spinner-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"spinner-track": {
		"path": "component.spinner.track",
		"cssName": "spinner-track",
		"type": "color",
		"value": "var(--border-subtle)"
	},
	"switch": {
		"path": "component.switch.track",
		"cssName": "switch",
		"type": "color",
		"value": "var(--muted)"
	},
	"switch-border": {
		"path": "component.switch.border",
		"cssName": "switch-border",
		"type": "color",
		"value": "var(--border-strong)"
	},
	"switch-thumb": {
		"path": "component.switch.thumb",
		"cssName": "switch-thumb",
		"type": "color",
		"value": "var(--surface-1)"
	},
	"tab-list": {
		"path": "component.tab-list.bg",
		"cssName": "tab-list",
		"type": "color",
		"value": "var(--background)"
	},
	"tab-list-border": {
		"path": "component.tab-list.border",
		"cssName": "tab-list-border",
		"type": "color",
		"value": "var(--border)"
	},
	"tab-list-rail": {
		"path": "component.tab-list.rail",
		"cssName": "tab-list-rail",
		"type": "color",
		"value": "var(--muted)"
	},
	"tab-foreground": {
		"path": "component.tab.fg",
		"cssName": "tab-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"tab-selected-foreground": {
		"path": "component.tab.selected-fg",
		"cssName": "tab-selected-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"tab-selected": {
		"path": "component.tab.selected-bg",
		"cssName": "tab-selected",
		"type": "color",
		"value": "var(--background)"
	},
	"theme-picker": {
		"path": "component.theme-picker.bg",
		"cssName": "theme-picker",
		"type": "color",
		"value": "var(--popover)"
	},
	"theme-picker-foreground": {
		"path": "component.theme-picker.fg",
		"cssName": "theme-picker-foreground",
		"type": "color",
		"value": "var(--popover-foreground)"
	},
	"theme-picker-trigger-foreground": {
		"path": "component.theme-picker.trigger-fg",
		"cssName": "theme-picker-trigger-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"theme-picker-option-foreground": {
		"path": "component.theme-picker.option-fg",
		"cssName": "theme-picker-option-foreground",
		"type": "color",
		"value": "var(--muted-foreground)"
	},
	"theme-picker-option-hover-bg": {
		"path": "component.theme-picker.option-hover-bg",
		"cssName": "theme-picker-option-hover-bg",
		"type": "color",
		"value": "var(--hover-bg)"
	},
	"toast-region-width": {
		"path": "component.toast-region.width",
		"cssName": "toast-region-width",
		"type": "dimension",
		"value": "var(--panel-w-toast)"
	},
	"toast-region-z": {
		"path": "component.toast-region.z",
		"cssName": "toast-region-z",
		"type": "number",
		"value": "var(--z-toast)"
	},
	"toast": {
		"path": "component.toast.bg",
		"cssName": "toast",
		"type": "color",
		"value": "var(--background)"
	},
	"toast-foreground": {
		"path": "component.toast.fg",
		"cssName": "toast-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"toast-border": {
		"path": "component.toast.border",
		"cssName": "toast-border",
		"type": "color",
		"value": "var(--border)"
	},
	"toggle-group": {
		"path": "component.toggle-group.bg",
		"cssName": "toggle-group",
		"type": "color",
		"value": "var(--muted)"
	},
	"toggle-group-border": {
		"path": "component.toggle-group.border",
		"cssName": "toggle-group-border",
		"type": "color",
		"value": "var(--border)"
	},
	"toggle": {
		"path": "component.toggle.bg",
		"cssName": "toggle",
		"type": "color",
		"value": "var(--background)"
	},
	"toggle-foreground": {
		"path": "component.toggle.fg",
		"cssName": "toggle-foreground",
		"type": "color",
		"value": "var(--foreground)"
	},
	"toggle-border": {
		"path": "component.toggle.border",
		"cssName": "toggle-border",
		"type": "color",
		"value": "var(--border)"
	},
	"tooltip": {
		"path": "component.tooltip.bg",
		"cssName": "tooltip",
		"type": "color",
		"value": "var(--foreground)"
	},
	"tooltip-foreground": {
		"path": "component.tooltip.fg",
		"cssName": "tooltip-foreground",
		"type": "color",
		"value": "var(--background)"
	},
	"app-accent-agents": {
		"path": "color.accent.agents",
		"cssName": "app-accent-agents",
		"type": "color",
		"value": "var(--accent-agents)",
		"aliasOf": "accent-agents"
	},
	"app-accent-canvas": {
		"path": "color.accent.canvas",
		"cssName": "app-accent-canvas",
		"type": "color",
		"value": "var(--accent-canvas)",
		"aliasOf": "accent-canvas"
	},
	"app-accent-chat": {
		"path": "color.accent.chat",
		"cssName": "app-accent-chat",
		"type": "color",
		"value": "var(--accent-chat)",
		"aliasOf": "accent-chat"
	},
	"app-accent-collections": {
		"path": "color.accent.collections",
		"cssName": "app-accent-collections",
		"type": "color",
		"value": "var(--accent-collections)",
		"aliasOf": "accent-collections"
	},
	"app-accent-dashboard": {
		"path": "color.accent.dashboard",
		"cssName": "app-accent-dashboard",
		"type": "color",
		"value": "var(--accent-dashboard)",
		"aliasOf": "accent-dashboard"
	},
	"app-accent-events": {
		"path": "color.accent.events",
		"cssName": "app-accent-events",
		"type": "color",
		"value": "var(--accent-events)",
		"aliasOf": "accent-events"
	},
	"app-accent-git": {
		"path": "color.accent.git",
		"cssName": "app-accent-git",
		"type": "color",
		"value": "var(--accent-git)",
		"aliasOf": "accent-git"
	},
	"app-accent-mesh": {
		"path": "color.accent.mesh",
		"cssName": "app-accent-mesh",
		"type": "color",
		"value": "var(--accent-mesh)",
		"aliasOf": "accent-mesh"
	},
	"app-accent-overview": {
		"path": "color.accent.overview",
		"cssName": "app-accent-overview",
		"type": "color",
		"value": "var(--accent-overview)",
		"aliasOf": "accent-overview"
	},
	"app-accent-pipelines": {
		"path": "color.accent.pipelines",
		"cssName": "app-accent-pipelines",
		"type": "color",
		"value": "var(--accent-pipelines)",
		"aliasOf": "accent-pipelines"
	},
	"app-accent-settings": {
		"path": "color.accent.settings",
		"cssName": "app-accent-settings",
		"type": "color",
		"value": "var(--accent-settings)",
		"aliasOf": "accent-settings"
	},
	"app-accent-setup": {
		"path": "color.accent.setup",
		"cssName": "app-accent-setup",
		"type": "color",
		"value": "var(--accent-setup)",
		"aliasOf": "accent-setup"
	},
	"app-accent-skills": {
		"path": "color.accent.skills",
		"cssName": "app-accent-skills",
		"type": "color",
		"value": "var(--accent-skills)",
		"aliasOf": "accent-skills"
	},
	"app-accent-specs": {
		"path": "color.accent.specs",
		"cssName": "app-accent-specs",
		"type": "color",
		"value": "var(--accent-specs)",
		"aliasOf": "accent-specs"
	},
	"app-accent-theme": {
		"path": "color.accent.theme",
		"cssName": "app-accent-theme",
		"type": "color",
		"value": "var(--accent-theme)",
		"aliasOf": "accent-theme"
	}
};

//#endregion
//#region src/design/token-groups.ts
/**
* Token group inventory from the DTCG registry (~400 leaves).
* Groups drive Design primary nav; knobs are type-aware editors.
*/
const GROUP_ORDER = [
	"color",
	"space",
	"typography",
	"radius",
	"shadow",
	"motion",
	"component",
	"icon",
	"density",
	"control",
	"interaction",
	"panel",
	"z",
	"scrollbar",
	"breakpoint"
];
const GROUP_LABELS = {
	color: "Color",
	space: "Space",
	typography: "Text",
	radius: "Radius",
	shadow: "Shadow",
	motion: "Motion",
	component: "Components",
	icon: "Icons",
	density: "Density",
	control: "Controls",
	interaction: "Interaction",
	panel: "Panels",
	z: "Stacking",
	scrollbar: "Scrollbar",
	breakpoint: "Breakpoints"
};
function classify(type, value) {
	const t = (type || "").toLowerCase();
	if (t === "color") return "color";
	if (t === "dimension") return "dimension";
	if (t === "duration") return "duration";
	if (t === "fontfamily" || t === "font-family") return "fontFamily";
	if (t === "number") return "number";
	if (/^\d+(\.\d+)?(px|rem|em|ms|s|%)$/.test(value.trim())) return "dimension";
	if (value.startsWith("#") || value.startsWith("oklch") || value.startsWith("rgb") || value.startsWith("color-mix")) return "color";
	return "other";
}
function parseDim(value) {
	const m = String(value).trim().match(/^(-?\d+(?:\.\d+)?)(px|rem|em|ms|s|%)?$/);
	if (!m) return null;
	return {
		num: Number(m[1]),
		unit: m[2] || ""
	};
}
function dimBounds(unit, path) {
	if (unit === "ms") return {
		min: 0,
		max: 2e3,
		step: 10
	};
	if (unit === "s") return {
		min: 0,
		max: 5,
		step: .05
	};
	if (unit === "%") return {
		min: 0,
		max: 100,
		step: 1
	};
	if (path.startsWith("space.") || path.startsWith("radius.")) return {
		min: 0,
		max: 64,
		step: 1
	};
	if (path.startsWith("icon.")) return {
		min: 8,
		max: 48,
		step: 1
	};
	return {
		min: 0,
		max: 128,
		step: 1
	};
}
/** Human label from DTCG path — never the raw oklch string. */
function humanLabel(path) {
	const parts = path.split(".").filter((p) => p && p !== "default");
	if (parts.length === 0) return path;
	const tail = parts[parts.length - 1];
	const prev = parts.length > 1 ? parts[parts.length - 2] : "";
	if (/^\d+$/.test(tail)) return prev ? `${titleCase(prev)} ${tail}` : tail;
	if (tail === "sm" || tail === "md" || tail === "lg" || tail === "xl") {
		const size = tail === "sm" ? "Small" : tail === "md" ? "Medium" : tail === "lg" ? "Large" : "Extra large";
		return prev ? `${titleCase(prev)} · ${size}` : size;
	}
	if (parts[0] === "component" && parts.length >= 2) return parts.slice(1).map(titleCase).join(" · ");
	return titleCase(tail);
}
function titleCase(s) {
	return s.replace(/[-_]/g, " ").replace(/([a-z])([A-Z])/g, "$1 $2").split(" ").filter(Boolean).map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
}
/**
* Short color readout for UI — never primary-display raw oklch.
* Falls back to css var name.
*/
function colorSummary(value, cssName) {
	const v = value.trim();
	const m = v.match(/^oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)/i);
	if (m) {
		const L = Number(m[1]);
		const C = Number(m[2]);
		const H = Number(m[3]);
		return `${L >= .75 ? "light" : L >= .45 ? "mid" : "dark"} ${C < .04 ? "neutral" : C < .12 ? "soft" : "vivid"} · hue ${Math.round(H)}`;
	}
	if (v.startsWith("color-mix") || v.startsWith("var(")) return cssName;
	if (v.startsWith("#")) return cssName;
	return cssName || "color";
}
function pathGroup(path) {
	return path.split(".")[0] || "other";
}
function buildKnobs() {
	const raw = tokens || {};
	const out = [];
	for (const [cssKey, entry] of Object.entries(raw)) {
		const path = entry.path || cssKey;
		const cssName = entry.cssName || cssKey;
		const type = entry.type || "unknown";
		const base = String(entry.value ?? "");
		const kind = classify(type, base);
		const dim = parseDim(base);
		const knob = {
			path,
			cssName,
			type,
			kind,
			base,
			label: humanLabel(path)
		};
		if (dim && (kind === "dimension" || kind === "duration" || kind === "number")) {
			const b = dimBounds(dim.unit, path);
			knob.unit = dim.unit;
			knob.min = b.min;
			knob.max = b.max;
			knob.step = b.step;
		}
		out.push(knob);
	}
	out.sort((a, b) => a.path.localeCompare(b.path));
	return out;
}
const ALL_KNOBS = buildKnobs();
const BY_PATH = new Map(ALL_KNOBS.map((k) => [k.path, k]));
const BY_CSS = new Map(ALL_KNOBS.map((k) => [k.cssName, k]));
function listTokenGroups() {
	const map = /* @__PURE__ */ new Map();
	for (const k of ALL_KNOBS) {
		const id = pathGroup(k.path);
		if (!map.has(id)) map.set(id, []);
		map.get(id).push(k);
	}
	return [...GROUP_ORDER.filter((g) => map.has(g)), ...[...map.keys()].filter((g) => !GROUP_ORDER.includes(g)).sort()].map((id) => ({
		id,
		label: GROUP_LABELS[id] || titleCase(id),
		knobs: map.get(id) || []
	}));
}
function groupKnobs(groupId) {
	return ALL_KNOBS.filter((k) => pathGroup(k.path) === groupId);
}
function getKnob(path) {
	return BY_PATH.get(path) || BY_CSS.get(path);
}
/** Groups that use the scale-family slider engine. */
function isScaleFamilyGroup(groupId) {
	return groupId === "radius" || groupId === "icon";
}
const DEFAULT_GROUP_ID = "color";

//#endregion
//#region src/design/color.ts
function parseOklch(s) {
	const m = /^oklch\(\s*([\d.]+%?)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+%?))?\s*\)/i.exec(String(s ?? "").trim());
	if (!m) return null;
	const Lraw = m[1];
	const L = Lraw.endsWith("%") ? Number(Lraw.slice(0, -1)) / 100 : Number(Lraw);
	const C = Number(m[2]);
	const H = Number(m[3]);
	let a;
	if (m[4] != null) {
		const ar = m[4];
		a = ar.endsWith("%") ? Number(ar.slice(0, -1)) / 100 : Number(ar);
	}
	if (![
		L,
		C,
		H
	].every(Number.isFinite)) return null;
	return {
		L,
		C,
		H,
		a
	};
}
function formatOklch(c) {
	const fmt = (n) => String(Math.round(n * 1e3) / 1e3);
	if (c.a != null && c.a < 1) return `oklch(${fmt(c.L)} ${fmt(c.C)} ${fmt(c.H)} / ${fmt(c.a)})`;
	return `oklch(${fmt(c.L)} ${fmt(c.C)} ${fmt(c.H)})`;
}
const clamp255 = (n) => Math.max(0, Math.min(255, Math.round(n)));
function srgbGamma(c) {
	return c <= .0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - .055;
}
function oklchToRgb(c) {
	const hr = c.H * Math.PI / 180;
	const a = c.C * Math.cos(hr);
	const bb = c.C * Math.sin(hr);
	const l_ = c.L + .3963377774 * a + .2158037573 * bb;
	const m_ = c.L - .1055613458 * a - .0638541728 * bb;
	const s_ = c.L - .0894841775 * a - 1.291485548 * bb;
	const lc = l_ ** 3;
	const mc = m_ ** 3;
	const sc = s_ ** 3;
	const r = 4.0767416621 * lc - 3.3077115913 * mc + .2309699292 * sc;
	const g = -1.2684380046 * lc + 2.6097574011 * mc - .3413193965 * sc;
	const b = -.0041960863 * lc - .7034186147 * mc + 1.707614701 * sc;
	return {
		r: clamp255(srgbGamma(r) * 255),
		g: clamp255(srgbGamma(g) * 255),
		b: clamp255(srgbGamma(b) * 255)
	};
}
function rgbToHex(r, g, b) {
	const to2 = (n) => clamp255(n).toString(16).padStart(2, "0");
	return `#${to2(r)}${to2(g)}${to2(b)}`;
}
function isHex(s) {
	return /^#[0-9a-f]{6}$/i.test(String(s ?? "").trim());
}
function normHex(s) {
	const t = String(s ?? "").trim().toLowerCase();
	if (/^#[0-9a-f]{6}$/.test(t)) return t;
	if (/^[0-9a-f]{6}$/.test(t)) return `#${t}`;
	if (/^#[0-9a-f]{3}$/.test(t)) return `#${t[1]}${t[1]}${t[2]}${t[2]}${t[3]}${t[3]}`;
	return "";
}
/**
* OKLCH (or hex) → `#rrggbb` for sail-color-picker.
* Returns null when the value cannot be converted — **never** invent mid-gray.
* Callers must treat null as "unresolved / not pickable as hex".
*/
function tryPickerHex(value) {
	const v = String(value ?? "").trim();
	if (!v) return null;
	if (isHex(v)) return v.toLowerCase();
	const n = normHex(v);
	if (n) return n;
	if (/var\(|color-mix\(|gradient\(|url\(/i.test(v)) return null;
	const ok = parseOklch(v);
	if (!ok) return null;
	const rgb = oklchToRgb(ok);
	return rgbToHex(rgb.r, rgb.g, rgb.b);
}
/**
* OKLCH (or hex) → `#rrggbb`. Empty string when unconvertible (honest UI).
* Prefer {@link tryPickerHex} when you need null vs empty.
*/
function toPickerHex(value) {
	return tryPickerHex(value) ?? "";
}
const lin = (c) => {
	const x = c / 255;
	return x <= .04045 ? x / 12.92 : ((x + .055) / 1.055) ** 2.4;
};
/** Derive a 5-step brand ramp around primary (monotonic L). */
function deriveBrandRamp(primary) {
	const ok = parseOklch(primary) ?? {
		L: .62,
		C: .16,
		H: 38
	};
	return [
		{
			key: "soft",
			label: "Soft",
			dL: .22,
			dC: -.04
		},
		{
			key: "light",
			label: "Light",
			dL: .1,
			dC: -.01
		},
		{
			key: "base",
			label: "Brand",
			dL: 0,
			dC: 0
		},
		{
			key: "deep",
			label: "Deep",
			dL: -.1,
			dC: .02
		},
		{
			key: "ink",
			label: "Ink",
			dL: -.22,
			dC: .01
		}
	].map((s) => {
		const value = formatOklch({
			L: Math.max(.08, Math.min(.96, ok.L + s.dL)),
			C: Math.max(0, ok.C + s.dC),
			H: ok.H
		});
		return {
			key: s.key,
			label: s.label,
			value,
			hex: toPickerHex(value)
		};
	});
}
function relativeLuminance(hex) {
	const h = normHex(hex) || hex;
	const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(h);
	if (!m) return 0;
	const r = lin(parseInt(m[1], 16));
	const g = lin(parseInt(m[2], 16));
	const b = lin(parseInt(m[3], 16));
	return .2126 * r + .7152 * g + .0722 * b;
}
/** WCAG contrast ratio between two color strings (oklch/hex). 0 when either is unconvertible. */
function contrastRatio(fg, bg) {
	const a = tryPickerHex(fg);
	const b = tryPickerHex(bg);
	if (!a || !b) return 0;
	const L1 = relativeLuminance(a);
	const L2 = relativeLuminance(b);
	const lighter = Math.max(L1, L2);
	const darker = Math.min(L1, L2);
	return (lighter + .05) / (darker + .05);
}
/** Check brand pairs used by specimen controls. */
function brandContrastFlags(opts) {
	return [
		{
			id: "primary-on-bg",
			label: "Brand on page",
			fg: opts.primary,
			bg: opts.background
		},
		{
			id: "fg-on-primary",
			label: "Label on brand",
			fg: opts.primaryFg,
			bg: opts.primary
		},
		{
			id: "body-on-bg",
			label: "Body on page",
			fg: opts.foreground,
			bg: opts.background
		}
	].map((p) => {
		const ratio = Math.round(contrastRatio(p.fg, p.bg) * 100) / 100;
		return {
			id: p.id,
			label: p.label,
			ratio,
			passAA: ratio >= 4.5
		};
	});
}
const BRAND_PRIMARY_PATH = "color.primary.default";
const BRAND_FG_PATH = "color.primary.foreground";

//#endregion
//#region src/design/ui.ts
/**
* Design surface view-models — pure projections of store state.
*/
function matchesQuery(k, q) {
	if (!q) return true;
	return k.path.toLowerCase().includes(q) || k.label.toLowerCase().includes(q) || k.cssName.toLowerCase().includes(q) || k.base.toLowerCase().includes(q);
}
function pathGroupId(path) {
	return path.split(".")[0] || "other";
}
/** Human-readable value for UI (colors → summary, never raw oklch primary). */
function displayValue(value, path, cssName) {
	const knob = getKnob(path);
	const name = cssName ?? knob?.cssName ?? path;
	if (knob?.kind === "color" || value.startsWith("oklch") || value.startsWith("#") || value.startsWith("rgb") || value.startsWith("color-mix")) return colorSummary(value, name);
	if (!value) return "—";
	return value;
}
function displayPathLabel(path) {
	const knob = getKnob(path);
	if (knob?.label) return knob.label;
	if (path === "$extensions.ship.density") return "Density";
	if (path.startsWith("$extensions.")) return humanPath(path.replace(/^\$extensions\./, ""));
	return humanLabel(path) || humanPath(path);
}
/** One human line for Changes panel + clipboard export. */
function formatDiffLine(row) {
	return `${displayPathLabel(row.path)}: ${displayValue(row.from, row.path)} → ${displayValue(row.to, row.path)}`;
}
function uiTokenGroups(state) {
	const q = state.tokenSearch.trim().toLowerCase();
	return listTokenGroups().map((g) => {
		let count = g.knobs.length;
		if (q) count = g.knobs.filter((k) => matchesQuery(k, q)).length;
		return {
			id: g.id,
			label: g.label,
			count: String(count),
			active: state.groupId === g.id,
			key: g.id
		};
	});
}
function currentValue(knob, state, saved) {
	const mode = state.modeId;
	const fromDoc = getPathValue(state.themeDoc, knob.path, mode);
	const fromSaved = getPathValue(saved, knob.path, mode);
	return {
		value: fromDoc ?? knob.base,
		dirty: fromDoc != null && fromSaved != null ? fromDoc !== fromSaved : fromDoc != null && fromDoc !== knob.base
	};
}
const DENSITY_LABEL = {
	compact: "Compact",
	default: "Comfortable",
	spacious: "Roomy"
};
const DECISION_META = {
	color: {
		label: "Brand color",
		hint: "Primary accent"
	},
	typography: {
		label: "Type",
		hint: "Display · body · mono"
	},
	density: {
		label: "Spacing density",
		hint: "List + control spacing"
	},
	radius: {
		label: "Corners",
		hint: "Radius scale"
	},
	icon: {
		label: "Icons",
		hint: "Pack + sizes"
	}
};
/** Five front-door decision cards with live summaries. */
function uiDecisionCards(state) {
	const brand = getPathValue(state.themeDoc, "color.primary.default", state.modeId) ?? "";
	return DECISION_IDS.map((id) => {
		const meta = DECISION_META[id];
		let summary = meta.hint;
		let swatch = "";
		if (id === "color") {
			summary = brand ? colorSummary(brand, "primary") : meta.hint;
			swatch = brand ? toPickerHex(brand) : "";
		} else if (id === "typography") summary = state.typeRoles.display;
		else if (id === "density") summary = DENSITY_LABEL[state.density] ?? state.density;
		else if (id === "radius") summary = `${state.radius.steps[0]?.valuePx ?? 0}–${state.radius.steps[state.radius.steps.length - 1]?.valuePx ?? 0}px`;
		else if (id === "icon") {
			const md = state.iconScale.steps.find((s) => s.key === "md");
			summary = `${state.icons.pack} · ${md?.raw ?? "16px"}`;
		}
		const active = state.decisionId === id || state.groupId === id;
		return {
			id,
			label: meta.label,
			summary,
			active,
			key: id,
			tone: active ? "primary" : "default",
			isColor: id === "color",
			swatch
		};
	});
}
function uiBrandPanel(state) {
	const mode = state.modeId;
	const primary = getPathValue(state.themeDoc, "color.primary.default", mode) ?? "oklch(0.72 0.17 38)";
	const primaryFg = getPathValue(state.themeDoc, "color.primary.foreground", mode) ?? "oklch(0.18 0.02 48)";
	const background = getPathValue(state.themeDoc, "color.background.default", mode) ?? getPathValue(state.themeDoc, "color.background", mode) ?? "oklch(0.16 0.01 48)";
	const foreground = getPathValue(state.themeDoc, "color.foreground.default", mode) ?? getPathValue(state.themeDoc, "color.foreground", mode) ?? "oklch(0.95 0.01 90)";
	const ramp = deriveBrandRamp(primary);
	const contrast = brandContrastFlags({
		primary,
		primaryFg,
		background,
		foreground
	});
	return {
		primary,
		primaryHex: toPickerHex(primary),
		ramp,
		contrast,
		contrastFailCount: contrast.filter((c) => !c.passAA).length
	};
}
function densityLabel(d) {
	return DENSITY_LABEL[d] ?? d;
}
function knobToRow(k, state, saved, groupLabel, groupId) {
	const { value, dirty } = currentValue(k, state, saved);
	if (state.overridesOnly && !dirty) return null;
	const dim = (() => {
		const m = String(value).match(/^(-?\d+(?:\.\d+)?)/);
		return m ? Number(m[1]) : 0;
	})();
	const isColor = k.kind === "color";
	const isDim = k.kind === "dimension" || k.kind === "duration" || k.kind === "number";
	return {
		path: k.path,
		label: k.label,
		cssName: k.cssName,
		value,
		display: isColor ? colorSummary(value, k.cssName) : value,
		dirty,
		isColor,
		isDim,
		isText: !isColor && !isDim,
		num: dim,
		unit: k.unit || "",
		min: k.min ?? 0,
		max: k.max ?? 64,
		step: k.step ?? 1,
		key: k.path,
		groupLabel,
		groupId,
		pickerHex: isColor ? toPickerHex(value) : ""
	};
}
/**
* Token rows for the editor.
* Empty search → active group only.
* Non-empty search → flat cross-group results (filter everything).
*/
function uiGroupTokenRows(state, saved) {
	const q = state.tokenSearch.trim().toLowerCase();
	const groups = listTokenGroups();
	const rows = [];
	if (q) {
		for (const g of groups) for (const k of g.knobs) {
			if (!matchesQuery(k, q)) continue;
			const row = knobToRow(k, state, saved, g.label, g.id);
			if (row) rows.push(row);
		}
		return rows;
	}
	const label = groups.find((g) => g.id === state.groupId)?.label ?? state.groupId;
	for (const k of groupKnobs(state.groupId)) {
		const row = knobToRow(k, state, saved, label, state.groupId);
		if (row) rows.push(row);
	}
	return rows;
}
/** Advanced all-tokens list (search + overrides-only) with human labels. */
function uiTokenRowsLabeled(state, saved) {
	return listTokenRows(state.themeDoc, saved, {
		search: state.tokenSearch,
		overridesOnly: state.overridesOnly
	}).map((r) => {
		const knob = getKnob(r.path);
		const isColor = knob?.kind === "color" || r.value.startsWith("oklch") || r.value.startsWith("#") || r.value.startsWith("rgb");
		const gid = pathGroupId(r.path);
		const gLabel = listTokenGroups().find((g) => g.id === gid)?.label ?? gid;
		return {
			path: r.path,
			label: knob?.label ?? humanPath(r.path),
			cssName: knob?.cssName ?? r.path,
			value: r.value,
			display: displayValue(r.value, r.path, knob?.cssName),
			dirty: r.dirty,
			isColor,
			isDim: knob?.kind === "dimension" || knob?.kind === "duration",
			isText: !isColor && knob?.kind !== "dimension" && knob?.kind !== "duration",
			num: 0,
			unit: knob?.unit ?? "",
			min: knob?.min ?? 0,
			max: knob?.max ?? 64,
			step: knob?.step ?? 1,
			key: r.path,
			groupLabel: gLabel,
			groupId: gid,
			pickerHex: isColor ? toPickerHex(r.value) : ""
		};
	});
}
function humanPath(path) {
	return path.split(".").filter((p) => p && p !== "default" && !p.startsWith("$")).map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" · ");
}
function uiRadiusSteps$1(state) {
	return state.radius.steps.map((s) => ({
		key: s.key,
		valuePx: s.valuePx,
		raw: s.raw,
		label: s.key
	}));
}
function uiIconScaleSteps$1(state) {
	return state.iconScale.steps.map((s) => ({
		key: s.key,
		valuePx: s.valuePx,
		raw: s.raw,
		label: s.key
	}));
}
function uiInheritance$1(state) {
	return state.inheritance;
}
function uiThemeOptions$1(state) {
	const seen = /* @__PURE__ */ new Set();
	const out = [];
	for (const t of [...state.projectThemes, ...state.catalogThemes]) {
		if (seen.has(t.id)) continue;
		seen.add(t.id);
		out.push({
			id: t.id,
			label: t.catalog ? t.id : `${t.id} (yours)`,
			catalog: t.catalog
		});
	}
	return out;
}
function uiHeader$1(state) {
	const forkLabel = state.forkedFrom ? `Based on ${state.forkedFrom}` : state.themePath ? "Project theme" : "Catalog";
	return {
		themeLabel: state.themeId,
		forkLabel,
		overridesLabel: `${state.overrideCount} change${state.overrideCount === 1 ? "" : "s"}`,
		canSave: !!state.themeDoc
	};
}
function uiActiveGroupMeta(state) {
	const g = listTokenGroups().find((x) => x.id === state.groupId);
	const id = g?.id ?? state.groupId;
	return {
		id,
		label: g?.label ?? id,
		isScaleFamily: isScaleFamilyGroup(id),
		isRadius: id === "radius",
		isIcon: id === "icon",
		isType: id === "typography",
		isDensity: id === "density",
		isColor: id === "color"
	};
}

//#endregion
//#region src/user-error.ts
/**
* Map caught exceptions to fixed user copy.
* Raw machine messages (ops, refused, writeSource, …) never reach the UI.
*/
function userError(e, fallback) {
	try {
		if (typeof console !== "undefined" && typeof console.warn === "function") console.warn("[studio]", e instanceof Error ? e.message : e);
	} catch {}
	return fallback;
}

//#endregion
//#region src/design/edit-actions.ts
async function setTypeRoleOn(ctx, role, family) {
	const state = ctx.getState();
	if (!state.themeDoc) {
		ctx.setState({
			...state,
			typeRoles: {
				...state.typeRoles,
				[role]: family
			}
		});
		ctx.emit();
		return;
	}
	const roles = {
		...state.typeRoles,
		[role]: family
	};
	const doc = patchTypeRoles(state.themeDoc, roles, state.modeId);
	ctx.recomputeDerived(doc);
	ctx.setState({
		...ctx.getState(),
		typeRoles: roles,
		lastWriteError: null
	});
	ctx.emit();
	await ctx.repaintSpecimen();
}
async function repaintSpecimenOn(ctx) {
	const client = ctx.getClient();
	const state = ctx.getState();
	if (!client?.sailRenderScreenshot) {
		ctx.setState({
			...state,
			specimenPng: null,
			specimenMs: null
		});
		ctx.emit();
		return;
	}
	const source = specimenSource({
		density: state.density,
		typeRoles: state.typeRoles
	});
	try {
		const t0 = Date.now();
		const r = await client.sailRenderScreenshot({
			source,
			theme: state.themeId,
			mode: state.modeId,
			viewport: SPECIMEN_VIEWPORT
		});
		ctx.setState({
			...ctx.getState(),
			specimenPng: r.imagePng ?? null,
			specimenMs: r.ms ?? Date.now() - t0
		});
		ctx.emit();
	} catch (e) {
		ctx.setState({
			...ctx.getState(),
			specimenPng: null,
			specimenMs: null,
			error: ctx.getState().error ?? userError(e, "Could not update the preview — try again.")
		});
		ctx.emit();
	}
}
/** Product copy for token usage ("Used by") — never "blast radius" in UI. */
function formatUsedByLabel(opts) {
	if (opts.offline) return "Usage unavailable offline";
	if (opts.components <= 0) return "Not used in this project yet";
	const n = opts.components;
	const f = Math.max(1, opts.files);
	if ((opts.dirtyCount ?? 0) > 0) return `Save updates ${n} component${n === 1 ? "" : "s"} across ${f} file${f === 1 ? "" : "s"}`;
	return `Used by · ${n} component${n === 1 ? "" : "s"}`;
}
/**
* Aggregate token-usage for dirty-path tokens (save-review / Used by).
* @param isCurrent — generation guard; stale async settles must no-op.
*/
async function refreshBlastAggregateOn(ctx, tokens, dirtyCount = 0, isCurrent = () => true) {
	const client = ctx.getClient();
	const list = tokens.length > 0 ? tokens : ["primary"];
	const apply = (blast) => {
		if (!isCurrent()) return;
		ctx.setState({
			...ctx.getState(),
			blast
		});
		ctx.emit();
	};
	if (!client?.sailInspectTokenUsage) {
		apply({
			token: list.join(","),
			components: 0,
			componentIds: [],
			sites: [],
			label: formatUsedByLabel({
				components: 0,
				files: 0,
				dirtyCount,
				offline: true
			})
		});
		return;
	}
	try {
		const componentIds = /* @__PURE__ */ new Set();
		const sites = [];
		let components = 0;
		for (const token of list.slice(0, 6)) {
			if (!isCurrent()) return;
			const r = await client.sailInspectTokenUsage({ token });
			components += r.components ?? 0;
			for (const id of r.componentIds ?? []) componentIds.add(id);
			for (const s of r.sites ?? []) {
				const o = s;
				sites.push({
					file: o.file ?? "",
					span: o.span
				});
			}
		}
		const uniqueComponents = componentIds.size || components;
		const files = new Set(sites.map((s) => s.file).filter(Boolean)).size;
		apply({
			token: list.join(","),
			components: uniqueComponents,
			componentIds: [...componentIds],
			sites,
			label: formatUsedByLabel({
				components: uniqueComponents,
				files,
				dirtyCount
			})
		});
	} catch {
		apply({
			token: list.join(","),
			components: 0,
			componentIds: [],
			sites: [],
			label: formatUsedByLabel({
				components: 0,
				files: 0,
				dirtyCount,
				offline: true
			})
		});
	}
}

//#endregion
//#region src/design/store-ui.ts
function makeStoreUi(access) {
	const uiGroups = () => uiTokenGroups(access.getState());
	const uiActiveGroup = () => uiActiveGroupMeta(access.getState());
	const uiGroupTokens = () => uiGroupTokenRows(access.getState(), access.getSaved());
	const uiRadiusSteps = () => uiRadiusSteps$1(access.getState());
	const uiInheritance = () => uiInheritance$1(access.getState());
	const uiIconScaleSteps = () => uiIconScaleSteps$1(access.getState());
	const uiThemeOptions = () => uiThemeOptions$1(access.getState());
	const uiHeader = () => uiHeader$1(access.getState());
	const uiDecisions = () => uiDecisionCards(access.getState());
	const uiBrand = () => uiBrandPanel(access.getState());
	const uiDensityLabel = () => densityLabel(access.getState().density);
	const uiTokenRows = () => uiTokenRowsLabeled(access.getState(), access.getSaved()).map((r) => ({
		path: r.path,
		value: r.value,
		dirty: r.dirty,
		key: r.path,
		label: r.label,
		display: r.display,
		isColor: r.isColor,
		pickerHex: r.pickerHex
	}));
	const exportDiffText = () => {
		const rows = access.getState().diffRows;
		if (!rows.length) return "No edits\n";
		return rows.map((r) => formatDiffLine(r)).join("\n") + "\n";
	};
	const filteredIconGlyphs = () => {
		const s = access.getState();
		const q = s.icons.query.trim().toLowerCase();
		const all = s.icons.glyphs;
		if (!q) return all.slice(0, 80);
		return all.filter((g) => g.includes(q)).slice(0, 80);
	};
	const searchFonts = async (query, opts) => {
		const empty = {
			families: [],
			hits: [],
			count: 0,
			catalogSize: 0,
			source: "none"
		};
		const client = access.getClient();
		if (!client?.sailThemeFontSearch) return empty;
		try {
			const r = await client.sailThemeFontSearch({
				query,
				category: opts?.category,
				limit: opts?.limit ?? 40
			});
			const families = (r.families ?? []).filter(Boolean);
			const hits = (r.hits ?? []).map((h) => ({
				family: String(h.family ?? ""),
				category: String(h.category ?? "Other"),
				popularity: Number(h.popularity ?? 99999),
				mono: Boolean(h.mono)
			})).filter((h) => h.family);
			const resolvedHits = hits.length > 0 ? hits : families.map((family, i) => ({
				family,
				category: "Other",
				popularity: i,
				mono: /mono|code/i.test(family)
			}));
			return {
				families: families.length ? families : resolvedHits.map((h) => h.family),
				hits: resolvedHits,
				count: Number(r.count ?? resolvedHits.length),
				catalogSize: Number(r.catalogSize ?? 0),
				source: String(r.source ?? "unknown")
			};
		} catch {
			return empty;
		}
	};
	const addFontFamily = async (family, role = "body") => {
		const client = access.getClient();
		if (client?.sailThemeFontAdd) try {
			await client.sailThemeFontAdd({ family });
			await access.setTypeRole(role, family);
			access.setState({
				...access.getState(),
				statusNote: `Installed ${family} → ${role}`
			});
			access.emit();
			return {
				ok: true,
				message: `Installed ${family} and set ${role}`
			};
		} catch (e) {
			return {
				ok: false,
				message: userError(e, `Could not install ${family} — try again.`)
			};
		}
		await access.setTypeRole(role, family);
		return {
			ok: false,
			message: `Ops FontAdd unavailable; ${role} set to ${family} (not vendored)`
		};
	};
	const loadIconGlyphs = async (names) => {
		const glyphs = names && names.length > 0 ? names : await fetchIconRegistryNames();
		access.setState({
			...access.getState(),
			icons: {
				...access.getState().icons,
				glyphs
			}
		});
		access.emit();
	};
	return {
		uiGroups,
		uiActiveGroup,
		uiGroupTokens,
		uiRadiusSteps,
		uiInheritance,
		uiIconScaleSteps,
		uiThemeOptions,
		uiHeader,
		uiDecisions,
		uiBrand,
		uiDensityLabel,
		uiTokenRows,
		exportDiffText,
		filteredIconGlyphs,
		searchFonts,
		addFontFamily,
		loadIconGlyphs
	};
}
async function fetchIconRegistryNames() {
	return await fetchJsonNameList("/_deck/icons/registry.json");
}
async function fetchJsonNameList(_url) {
	return [];
}

//#endregion
//#region src/design/theme-io.ts
/** After project JSON write — regenerate app theme tables (not the edit-loop paint). */
async function generateAppThemes(ctx, noteOnFail = true) {
	const client = ctx.getClient();
	const state = ctx.getState();
	if (!client?.sailThemeGenerate) {
		if (noteOnFail) ctx.setState({
			...ctx.getState(),
			statusNote: (ctx.getState().statusNote || "Theme saved") + " · tables not regenerated (Generate unavailable)"
		});
		return;
	}
	try {
		await client.sailThemeGenerate({
			mode: "app",
			project: state.projectRoot || void 0
		});
	} catch (e) {
		if (noteOnFail) {
			ctx.setState({
				...ctx.getState(),
				lastWriteError: userError(e, "Theme JSON saved, but app theme tables failed to regenerate.")
			});
			ctx.emit();
		}
	}
}
async function loadThemesOn(ctx, opts) {
	const client = ctx.getClient();
	let state = ctx.getState();
	if (!client) {
		ctx.setState({
			...state,
			error: null,
			loading: false
		});
		ctx.emit();
		return;
	}
	ctx.setState({
		...state,
		loading: true,
		error: null
	});
	ctx.emit();
	state = ctx.getState();
	try {
		const [cat, proj] = await Promise.all([client.sailThemeList({ path: CATALOG_THEMES_DIR }), client.sailThemeList({ path: state.themesDir }).catch(() => ({ files: [] }))]);
		const catalogThemes = themeFilesFromList(cat.files ?? [], CATALOG_THEMES_DIR, true);
		const projectThemes = themeFilesFromList(proj.files ?? [], state.themesDir, false);
		ctx.setState({
			...ctx.getState(),
			catalogThemes,
			projectThemes
		});
		state = ctx.getState();
		const prefer = opts?.themeId || state.themeId || projectThemes[0]?.id || catalogThemes.find((t) => t.id === "studio")?.id || projectThemes.find((t) => t.id === "studio")?.id || catalogThemes.find((t) => t.id === "ship")?.id || catalogThemes[0]?.id || "studio";
		if (state.dirty && state.themeDoc && !opts?.themeId) {
			ctx.setState({
				...ctx.getState(),
				loading: false
			});
			ctx.emit();
			return;
		}
		if (!opts?.themeId && state.themeDoc && state.themeId === prefer) {
			ctx.setState({
				...ctx.getState(),
				loading: false
			});
			ctx.emit();
			return;
		}
		await openThemeOn(ctx, prefer);
	} catch (e) {
		ctx.setState({
			...ctx.getState(),
			loading: false,
			error: userError(e, "Could not load themes — try again.")
		});
		ctx.emit();
	}
}
async function openThemeOn(ctx, themeId) {
	const client = ctx.getClient();
	if (!client) return;
	ctx.setState({
		...ctx.getState(),
		loading: true,
		error: null
	});
	ctx.emit();
	try {
		const state = ctx.getState();
		const proj = state.projectThemes.find((t) => t.id === themeId);
		const cat = state.catalogThemes.find((t) => t.id === themeId);
		const file = proj ?? cat;
		if (!file) {
			const path = `${CATALOG_THEMES_DIR}/${themeId}.json`;
			const doc = parseThemeJson((await client.sailThemeRead({ path })).content);
			ctx.setSavedDoc(structuredClone(doc));
			ctx.setState({
				...ctx.getState(),
				themeId: themeIdOf(doc, themeId),
				forkedFrom: null,
				themePath: null,
				loading: false
			});
			ctx.recomputeDerived(doc);
			ctx.emit();
			await ctx.repaintSpecimen();
			return;
		}
		const doc = parseThemeJson((await client.sailThemeRead({ path: file.path })).content);
		ctx.setSavedDoc(structuredClone(doc));
		ctx.setState({
			...ctx.getState(),
			themeId: themeIdOf(doc, file.id),
			forkedFrom: file.catalog ? null : ctx.getState().forkedFrom,
			themePath: file.catalog ? null : file.path,
			loading: false
		});
		ctx.recomputeDerived(doc);
		ctx.emit();
		await ctx.repaintSpecimen();
		if (ctx.getState().groupId === "radius") await ctx.refreshBlast("radius-lg");
	} catch (e) {
		ctx.setState({
			...ctx.getState(),
			loading: false,
			error: userError(e, "Could not open that theme — try again.")
		});
		ctx.emit();
	}
}
/**
* Add a catalog (or project) theme into the project as `newId`.
* Product UI: "Add to project" — never "fork".
*/
async function forkThemeOn(ctx, newId, sourceId) {
	const client = ctx.getClient();
	if (!client) {
		ctx.setState({
			...ctx.getState(),
			lastWriteError: "Could not save themes right now"
		});
		ctx.emit();
		return false;
	}
	const id = newId.trim();
	if (!/^[a-z][a-z0-9-]*$/.test(id)) {
		ctx.setState({
			...ctx.getState(),
			lastWriteError: "Use lowercase letters, numbers, and hyphens"
		});
		ctx.emit();
		return false;
	}
	const state = ctx.getState();
	if (state.projectThemes.some((t) => t.id === id)) {
		ctx.setState({
			...ctx.getState(),
			lastWriteError: `Project already has a theme named “${id}”.`
		});
		ctx.emit();
		return false;
	}
	const srcId = sourceId || state.themeId;
	const source_path = (state.projectThemes.find((t) => t.id === srcId) || state.catalogThemes.find((t) => t.id === srcId))?.path || `${"packages/sail/tokens/model/themes"}/${srcId}.json`;
	const dest_path = `${state.themesDir}/${id}.json`;
	try {
		let content;
		if (state.themeDoc && (!sourceId || sourceId === state.themeId)) content = stringifyTheme(setThemeId(state.themeDoc, id));
		await client.sailThemeFork({
			source_path,
			dest_path,
			new_id: id,
			content
		});
		ctx.setState({
			...ctx.getState(),
			forkedFrom: srcId,
			themePath: dest_path,
			themeId: id,
			lastWriteError: null
		});
		await generateAppThemes(ctx, false);
		await loadThemesOn(ctx, { themeId: id });
		return true;
	} catch (e) {
		ctx.setState({
			...ctx.getState(),
			lastWriteError: userError(e, "Could not add that theme to the project — try again.")
		});
		ctx.emit();
		return false;
	}
}
/** Delete a project theme file (catalog themes refused by ops). */
async function deleteThemeOn(ctx, themeId) {
	const client = ctx.getClient();
	const state = ctx.getState();
	if (!client?.sailThemeDelete) {
		ctx.setState({
			...state,
			lastWriteError: "Theme delete is not available offline."
		});
		ctx.emit();
		return false;
	}
	const row = state.projectThemes.find((t) => t.id === themeId) || (state.themeId === themeId && state.themePath ? {
		id: themeId,
		path: state.themePath,
		file: "",
		catalog: false
	} : null);
	if (!row?.path || row.catalog) {
		ctx.setState({
			...state,
			lastWriteError: "Only project themes can be deleted."
		});
		ctx.emit();
		return false;
	}
	try {
		await client.sailThemeDelete({ path: row.path });
		if (state.themeId === themeId) {
			ctx.setSavedDoc(null);
			ctx.setState({
				...ctx.getState(),
				themeId: "",
				themePath: null,
				themeDoc: null,
				dirty: false,
				diffRows: [],
				lastWriteError: null,
				statusNote: "Theme deleted"
			});
		} else ctx.setState({
			...ctx.getState(),
			lastWriteError: null,
			statusNote: "Theme deleted"
		});
		ctx.emit();
		await generateAppThemes(ctx, false);
		await loadThemesOn(ctx);
		return true;
	} catch (e) {
		ctx.setState({
			...ctx.getState(),
			lastWriteError: userError(e, "Could not delete the theme — try again.")
		});
		ctx.emit();
		return false;
	}
}
function emptyDesignState() {
	return {
		projectRoot: "",
		themesDir: "apps/studio/tokens/themes",
		groupId: DEFAULT_GROUP_ID,
		decisionId: "color",
		density: "default",
		themeId: "ship",
		modeId: "dark",
		forkedFrom: null,
		themePath: null,
		themeDoc: null,
		dirty: false,
		saveReviewOpen: false,
		catalogThemes: [],
		projectThemes: [],
		radius: {
			path: "radius",
			ordered: [
				"sm",
				"md",
				"lg",
				"xl"
			],
			scaleTogether: true,
			steps: [
				{
					key: "sm",
					valuePx: 4,
					raw: "4px"
				},
				{
					key: "md",
					valuePx: 6,
					raw: "6px"
				},
				{
					key: "lg",
					valuePx: 10,
					raw: "10px"
				},
				{
					key: "xl",
					valuePx: 14,
					raw: "14px"
				}
			]
		},
		inheritance: [],
		typeRoles: {
			display: "Fraunces",
			body: "DM Sans",
			mono: "JetBrains Mono"
		},
		icons: {
			pack: "lucide",
			glyphs: [],
			query: ""
		},
		iconScale: {
			path: "icon",
			ordered: [
				"sm",
				"md",
				"lg",
				"xl"
			],
			scaleTogether: true,
			steps: [
				{
					key: "sm",
					valuePx: 14,
					raw: "14px"
				},
				{
					key: "md",
					valuePx: 16,
					raw: "16px"
				},
				{
					key: "lg",
					valuePx: 20,
					raw: "20px"
				},
				{
					key: "xl",
					valuePx: 24,
					raw: "24px"
				}
			]
		},
		blast: null,
		inspectOpen: false,
		overridesOnly: false,
		tokenSearch: "",
		overrideCount: 0,
		specimenPng: null,
		specimenMs: null,
		loading: false,
		error: null,
		lastWriteError: null,
		lastClampNote: null,
		diffRows: [],
		statusNote: "Pick a decision or token group",
		canUndo: false,
		canRedo: false
	};
}

//#endregion
//#region src/design/store.ts
/**
* Theme domain store — product UI is Themes only.
* (Legacy path name `design`; writers shared by theme-session facets.)
*
* Draft model: one themeDoc in RAM; dirty ⇔ doc ≠ baseline (savedDoc);
* Save = JSON write + SailTheme Generate mode=app. Live preview via
* theme-session/theme-draft-paint (applyScope when available).
*/
/** Lazy import to avoid circular design ↔ theme-session at module init. */
function scheduleDraftPaint() {
	import("./theme-draft-paint-CrLocIlo.js").then((m) => m.scheduleDraftPaint()).catch(() => {});
}
let client = null;
let state = bootState();
let savedDoc = null;
const listeners = /* @__PURE__ */ new Set();
/** Monotonic gen for usage refresh — stale async settles must not overwrite. */
let blastGen = 0;
/** Undo/redo stacks — JSON snapshots of themeDoc (project theme edits only). */
const historyPast = [];
const historyFuture = [];
let lastHistorySnap = "";
function snapshotDoc(doc) {
	return doc ? JSON.stringify(doc) : "";
}
function beginBlastRefresh() {
	const gen = ++blastGen;
	return () => gen === blastGen;
}
function bootState() {
	return {
		...emptyDesignState(),
		radius: defaultRadiusFamily(true),
		iconScale: defaultIconFamily(true),
		inheritance: radiusInheritance(null),
		groupId: DEFAULT_GROUP_ID,
		statusNote: "Pick a decision or token group"
	};
}
function emit() {
	for (const fn of listeners) fn(state);
	scheduleDraftPaint();
}
function clearHistory() {
	historyPast.length = 0;
	historyFuture.length = 0;
	lastHistorySnap = "";
	state = {
		...state,
		canUndo: false,
		canRedo: false
	};
}
function recomputeDerived(doc) {
	const mode = state.modeId;
	const radius = extractRadius(doc, mode);
	radius.scaleTogether = state.radius.scaleTogether;
	const iconScale = extractIconScale(doc, mode);
	iconScale.scaleTogether = state.iconScale.scaleTogether;
	state = {
		...state,
		themeDoc: doc,
		density: densityOf(doc),
		radius,
		iconScale,
		inheritance: radiusInheritance(doc, mode),
		typeRoles: extractTypeRoles(doc, mode),
		overrideCount: countOverrides(doc, savedDoc),
		diffRows: diffThemeDocs(savedDoc, doc),
		dirty: !!doc && JSON.stringify(doc) !== JSON.stringify(savedDoc)
	};
}
function themeCtx() {
	return {
		getClient: () => client,
		getState: () => state,
		setState: (s) => {
			state = s;
		},
		getSavedDoc: () => savedDoc,
		setSavedDoc: (d) => {
			savedDoc = d;
		},
		recomputeDerived,
		emit,
		repaintSpecimen: () => repaintSpecimen(),
		refreshBlast: (t) => refreshBlast(t)
	};
}
function editCtx() {
	return {
		getClient: () => client,
		getState: () => state,
		setState: (s) => {
			state = s;
		},
		recomputeDerived,
		emit,
		repaintSpecimen: () => repaintSpecimen(),
		refreshBlast: (t) => refreshBlast(t),
		reflectCss
	};
}
function reflectCss(_cssName, _value) {}
const ui = makeStoreUi({
	getState: () => state,
	getSaved: () => savedDoc,
	getClient: () => client,
	setState: (s) => {
		state = s;
	},
	emit,
	setTypeRole: (role, family) => setTypeRole(role, family)
});
function getDesignState() {
	return state;
}
function subscribeDesign(fn) {
	listeners.add(fn);
	return () => {
		listeners.delete(fn);
	};
}
async function loadThemes(opts) {
	await loadThemesOn(themeCtx(), opts);
}
async function openTheme(themeId) {
	clearHistory();
	await openThemeOn(themeCtx(), themeId);
	lastHistorySnap = snapshotDoc(state.themeDoc);
	state = {
		...state,
		canUndo: false,
		canRedo: false,
		saveReviewOpen: false
	};
	emit();
}
/**
* Add catalog/source theme into the project (UI: "Add to project").
* `forkTheme` kept as alias for existing call sites.
*/
async function addThemeToProject(newId, sourceId) {
	return forkThemeOn(themeCtx(), newId, sourceId);
}
/** Delete a project theme file (catalog refused). Reloads theme list. */
async function deleteTheme(themeId) {
	return deleteThemeOn(themeCtx(), themeId);
}
async function setTypeRole(role, family) {
	await setTypeRoleOn(editCtx(), role, family);
}
const searchFonts = ui.searchFonts;
const addFontFamily = ui.addFontFamily;
const loadIconGlyphs = ui.loadIconGlyphs;
const filteredIconGlyphs = ui.filteredIconGlyphs;
const exportDiffText = ui.exportDiffText;
const uiGroups = ui.uiGroups;
const uiActiveGroup = ui.uiActiveGroup;
const uiGroupTokens = ui.uiGroupTokens;
const uiRadiusSteps = ui.uiRadiusSteps;
const uiInheritance = ui.uiInheritance;
const uiIconScaleSteps = ui.uiIconScaleSteps;
const uiThemeOptions = ui.uiThemeOptions;
const uiHeader = ui.uiHeader;
const uiDecisions = ui.uiDecisions;
const uiBrand = ui.uiBrand;
const uiDensityLabel = ui.uiDensityLabel;
const uiTokenRows = ui.uiTokenRows;
async function refreshBlast(token) {
	const isCurrent = beginBlastRefresh();
	await refreshBlastAggregateOn(editCtx(), [token], 0, isCurrent);
}
async function repaintSpecimen() {
	await repaintSpecimenOn(editCtx());
}

//#endregion
export { openTheme as a, loadThemes as i, deleteTheme as n, subscribeDesign as o, getDesignState as r, addThemeToProject as t };