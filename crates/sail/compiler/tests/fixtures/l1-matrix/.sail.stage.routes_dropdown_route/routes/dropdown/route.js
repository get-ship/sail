import { __tpl, component, mountComponent } from "@ship/sail-runtime/internal";
import __mod_dropdown from "@sail/element/dropdown";
import __mod_menu_item from "@sail/element/menu-item";

//#region routes/dropdown/page.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"1991ce2b","dirty":true,"builtAt":"2026-08-23T00:35:39Z"} */
const __tpl0 = /*@__PURE__*/ __tpl("<div class=\"sail-col sail-g-2 sail-ai-stretch s-size-max-w-breakpoint-desktop-w-100pct\"><h1 class=\"s-type-family-heading s-type-size-4xl s-type-leading-tight\">dropdown</h1><p class=\"s-type-family-body s-type-size-base s-type-leading-normal\">Specimens for <code class=\"s-type-family-mono s-type-size-sm s-type-leading-relaxed\">dropdown</code> — rest state and axis sweeps.</p></div>");
const __tpl1 = /*@__PURE__*/ __tpl("<h2 class=\"s-type-family-heading s-type-size-2xl s-type-leading-tight\">dropdown rest</h2>");
const __tpl2 = /*@__PURE__*/ __tpl("<button data-sail-host=\"button\" data-sail-bridge=\"tone\" popovertarget=\"dropdown-rest\" style=\"--tone: var(--primary); --tone-foreground: var(--primary-foreground); --tone-hover: var(--primary-hover);\" variant=\"solid\" tone=\"primary\" size=\"md\" class=\"sail-button s-display-inline-flex s-position-scheme-relative s-appearance-none s-radius-md s-border-style-solid-width-control-stroke s-isolate s-control-frame s-select-none s-size-min-w-max-w-max s-type-leading-none-weight-medium-wrap-nowrap s-play-control-fast s-hover-control-lift s-active-press s-disabled-false-cursor-pointer s-disabled-cursor-not-allowed s-disabled-filter-opacity50 s-variant-solid-bg-foreground s-variant-solid-border-foreground s-variant-solid-fg-background s-variant-solid-elevation-xs s-variant-solid-hover-filter-opacity90 s-variant-solid-hover-elevation-sm s-variant-solid-bg-tone s-variant-solid-border-tone s-variant-solid-fg-tone-foreground s-size-md-size-min-h-control-h-md s-size-md-pad-x3 s-size-md-type-size-base s-pill-radius-full s-loading-cursor-wait s-loading-pointer-events-none\"><div class=\"sail-row-i sail-g-2 sail-ai-center sail-jc-start sail-nowrap sail-cluster\">Open dropdown rest</div></button>");
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
		const __suiStylesKey = "1aeccb6c";
		const __suiStyles = "@layer sail {\n.s-pad-6{padding:var(--space-6)}.s-bg-background{background-color:var(--background)}.s-size-max-w-breakpoint-desktop-w-100pct{max-width:var(--breakpoint-desktop);width:100%}.s-type-family-heading{font-family:var(--heading-font)}.s-type-size-4xl{font-size:var(--text-4xl)}.s-type-leading-tight{line-height:var(--leading-tight)}.s-type-family-body{font-family:var(--body-font)}.s-type-size-base{font-size:var(--text-base)}.s-type-leading-normal{line-height:var(--leading-normal)}.s-type-family-mono{font-family:var(--mono-font)}.s-type-size-sm{font-size:var(--text-sm)}.s-type-leading-relaxed{line-height:var(--leading-relaxed)}.s-type-size-2xl{font-size:var(--text-2xl)}.sail-col{display:flex;flex-direction:column;min-width:0}.sail-col>*{min-width:0}.sail-col>.s-size-hug{min-width:auto;min-height:auto;flex:0 0 auto}.sail-col[popover]:not(:popover-open){display:none!important}.sail-col[popover]:popover-open{display:flex;flex-direction:column;min-width:0}.sail-g-8{gap:var(--space-8)}.sail-ai-stretch{align-items:stretch}.sail-g-2{gap:var(--space-2)}.sail-row-i{display:inline-flex;flex-wrap:wrap;min-width:0}.sail-row-i>*{min-width:0}.sail-row-i>.s-size-hug{min-width:auto;min-height:auto;flex:0 0 auto}.sail-ai-center{align-items:center}.sail-jc-start{justify-content:flex-start}.sail-row-i.sail-nowrap{flex-wrap:nowrap}.sail-row-i.sail-cluster>*{min-width:auto;flex:0 0 auto}\n}";
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
		const __e0 = document.createElement("div");
		__e0.setAttribute("class", "sail-col sail-g-8 sail-ai-stretch s-pad-6 s-bg-background");
		__e0.setAttribute("role", "region");
		const __e1 = __tpl0();
		__e0.appendChild(__e1);
		const __e2 = document.createElement("div");
		__e2.setAttribute("class", "sail-col sail-g-2 sail-ai-stretch");
		const __e3 = __tpl1();
		__e2.appendChild(__e3);
		const __e4 = __tpl2();
		__e2.appendChild(__e4);
		const __h5 = document.createElement("div");
		__h5.setAttribute("data-component", "dropdown");
		__h5.classList.add("sail-dropdown");
		const __p6 = Object.create(null);
		__p6.label = "dropdown rest";
		__p6.panelId = "dropdown-rest";
		const __h7 = document.createElement("div");
		__h7.setAttribute("data-component", "menu-item");
		__h7.classList.add("sail-menu-item");
		const __p8 = Object.create(null);
		__p8.value = "edit";
		const __t9 = document.createTextNode("Edit");
		__h7.appendChild(__t9);
		__h5.appendChild(__h7);
		const __h10 = document.createElement("div");
		__h10.setAttribute("data-component", "menu-item");
		__h10.classList.add("sail-menu-item");
		const __p11 = Object.create(null);
		__p11.value = "dup";
		const __t12 = document.createTextNode("Duplicate");
		__h10.appendChild(__t12);
		__h5.appendChild(__h10);
		const __m14 = mountComponent(__mod_dropdown, __h5, { props: __p6 });
		__cleanups.push(__m14);
		__e2.appendChild(__h5);
		const __x15 = { props: __p8 };
		if (__p8.names) __x15.names = __p8.names;
		if (__p8.labels) __x15.labels = __p8.labels;
		{
			const __v = __m14.instance.disabled;
			__x15.disabled = __v;
			__p8.disabled = __v;
		}
		{
			const __v = __m14.instance.onSelect;
			__x15.onSelect = __v;
			__p8.onSelect = __v;
		}
		const __m16 = mountComponent(__mod_menu_item, __h7, __x15);
		__cleanups.push(__m16);
		const __x17 = { props: __p11 };
		if (__p11.names) __x17.names = __p11.names;
		if (__p11.labels) __x17.labels = __p11.labels;
		{
			const __v = __m14.instance.disabled;
			__x17.disabled = __v;
			__p11.disabled = __v;
		}
		{
			const __v = __m14.instance.onSelect;
			__x17.onSelect = __v;
			__p11.onSelect = __v;
		}
		const __m18 = mountComponent(__mod_menu_item, __h10, __x17);
		__cleanups.push(__m18);
		__e0.appendChild(__e2);
		__root.appendChild(__e0);
		return () => {
			for (const c of __cleanups) try {
				c();
			} catch (e) {}
		};
	} });
}), { _compilerVersion: {
	"cargo": "0.1.0",
	"git": "1991ce2b",
	"dirty": true,
	"builtAt": "2026-08-23T00:35:39Z"
} });

//#endregion
export { page_default as default };