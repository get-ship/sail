import { __tpl, component, controller, mountAmbient } from "@ship/sail-runtime/internal";
import { navigate } from "@ship/deck/router";

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
const initialChrome = chromeFromHash();
let state = {
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
const listeners = /* @__PURE__ */ new Set();
function emit() {
	for (const fn of listeners) fn(state);
}
function setChecksPanelOpen(open) {
	if (state.checksPanelOpen === open) return;
	state = {
		...state,
		checksPanelOpen: open
	};
	emit();
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

//#endregion
//#region routes/checks/page.sail.controller.ts
var page_sail_controller_default = controller(() => {
	mountAmbient(() => {
		setChecksPanelOpen(true);
		navigate(`${appRouteBase()}/edit`, { replace: true });
	});
	return {};
});

//#endregion
//#region routes/checks/page.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0 = /*@__PURE__*/ __tpl("<div studio-redirect=\"\" checks-redirect=\"\" class=\"sail-col sail-g-4 sail-ai-center sail-jc-center sx-grow1 sx-minH0 sx-hfull sx-bgbackground\"><p class=\"sx-fgmutedforeground sx-fontSizesm\">\n    Opening Checks panel…</p></div>");
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
	if (page_sail_controller_default && typeof page_sail_controller_default.setup === "function") {
		ctx.scope = $scope;
		const __r = page_sail_controller_default.setup(ctx);
		if (__r && typeof __r === "object") Object.assign($scope, __r);
	}
	return Object.assign({}, $scope, { __mount(container) {
		const __cleanups = [];
		const __root = container.shadowRoot ?? container;
		const __suiStylesKey = "2849ddb6";
		const __suiStyles = "@layer sail {\n.sx-minH0{min-height:0}.sx-hfull{height:100%}.sx-bgbackground{background-color:var(--background)}.sx-fgmutedforeground{color:var(--muted-foreground)}.sx-fontSizesm{font-size:var(--text-sm)}.sx-grow1{flex-grow:1}.sail-col{display:flex;flex-direction:column;min-width:0}.sail-col>*{min-width:0}.sail-col[popover]:not(:popover-open){display:none!important}.sail-col[popover]:popover-open{display:flex;flex-direction:column;min-width:0}.sail-g-4{gap:var(--space-4)}.sail-ai-center{align-items:center}.sail-jc-center{justify-content:center}\n}";
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