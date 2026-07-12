var He=Object.defineProperty;var Ae=(e,n,t)=>n in e?He(e,n,{enumerable:!0,configurable:!0,writable:!0,value:t}):e[n]=t;var w=(e,n,t)=>Ae(e,typeof n!="symbol"?n+"":n,t);var De=Symbol.for("sui:error-boundary");function D(e){let n=e;for(;n;){if(n instanceof Element){let r=n[De];if(r)return r}let t=n.parentNode;if(t){n=t;continue}let o=n;if(o.host){n=o.host;continue}n=null}return null}var M=null;function C(e,n){let t=M;M=e;try{return n()}finally{M=t}}function H(e){let n=M;n&&n.cleanup(e)}function z(){let e=[],n=!1;return{cleanup(o){if(n){try{o()}catch(r){console.error("[sui/scope] late cleanup threw:",r)}return}e.push(o)},dispose(){if(!n){n=!0;for(let o=e.length-1;o>=0;o--)try{e[o]()}catch(r){console.error("[sui/scope] cleanup threw:",r)}e.length=0}},get disposed(){return n}}}function ee(e){let n=e??M,t=z();return n&&n.cleanup(()=>t.dispose()),t}var E=0,Y=1,k=2;function Me(e){return{cleanup(n){e.cleanups.push(n)},dispose(){},disposed:!1}}var b=null,Pe=0,P=[],G=!1,L=[];function O(e,n){L.push(e);try{return n()}finally{L.pop()}}function Le(){return L.length>0?L[L.length-1]:null}function T(e){return typeof e=="function"&&"set"in e&&typeof e.set=="function"}function Oe(e){let n=b;n!==null&&(e.observers.has(n)||(e.observers.add(n),n.sources.push(e)))}function te(e,n){if(e.state>=n)return;let t=e.state===E;if(e.state=n,e.isEffect)t&&e.active&&P.push(e);else if(t)for(let o of e.observers)te(o,Y)}function oe(e){return e.computation===!0}function K(e){let n=e.cleanups;if(n.length!==0){e.cleanups=[];for(let t=0;t<n.length;t++)try{n[t]()}catch(o){console.error("[sui] cleanup threw:",o)}}}function U(e){let n=e.sources;for(let t=0;t<n.length;t++)n[t].observers.delete(e);e.sources=[]}function Ne(e){K(e),U(e),e.state=E;let n=b;b=e;let t;try{t=e.fn()}finally{b=n}if(!Object.is(t,e.value)){e.value=t;for(let o of e.observers)o!==b&&o.state<k&&(o.state=k)}}function re(e){K(e),U(e),e.state=E;let n=e.scope??(e.scope=Me(e)),t=b;b=e;try{let o=C(n,()=>e.fn());typeof o=="function"&&e.cleanups.push(o)}catch(o){let r=e.host?D(e.host):null;r?r.catch(o,"effect"):console.error("[sui] effect threw:",o)}finally{b=t}}function ie(e){if(e.state!==E){if(e.state===Y){let n=e.sources;for(let t=0;t<n.length;t++){let o=n[t];if(oe(o)&&(ie(o),e.state===k))break}}e.state===k&&Ne(e),e.state=E}}function Be(e){if(e.active){if(e.state===Y){let n=e.sources;for(let t=0;t<n.length;t++){let o=n[t];if(oe(o)&&(ie(o),e.state===k))break}}e.state===k?re(e):e.state=E}}var ne=1e5;function ze(){if(G)return;G=!0;let e=0;try{for(;P.length>0;){if(++e>ne)throw P=[],new Error(`[sui] reactivity: aborting flush after ${ne} rounds \u2014 a reaction keeps scheduling itself (likely an effect writing a signal it reads with no stable fixpoint).`);let n=P;P=[];for(let t=0;t<n.length;t++)Be(n[t])}}finally{G=!1}}function N(e){let n={value:e,observers:new Set},t=(()=>(Oe(n),n.value));return t.set=o=>{let r=typeof o=="function"?o(n.value):o;if(!Object.is(r,n.value)){n.value=r;for(let i of n.observers)te(i,k);Pe===0&&ze()}},t.__subs=n.observers,t.__peek=()=>n.value,t}function _(e){let n={fn:e,state:k,sources:[],observers:new Set,cleanups:[],isEffect:!0,active:!0,computation:!0,host:Le()};re(n);let t=()=>{n.active&&(n.active=!1,K(n),U(n),n.state=E)};return b!==null&&H(t),t}function W(e){let n=b;b=null;try{return e()}finally{b=n}}var A="$signals";function q(e,n){return new Proxy(e,{get(t,o){return o===A?n:o in n?n[o]():t[o]},set(t,o,r){if(o===A)return console.warn(`[ship-set] ${A} is the read-only signal-map accessor; the assignment was ignored.`),!0;let i=n[o];return i?(typeof r=="function"?i.set(()=>r):i.set(r),!0):(console.warn(`[ship-set] s.${String(o)} = \u2026 was ignored \u2014 ${String(o)} is not a writable signal. Declare it as <state name="${String(o)}"> in the template or return it from the scope factory.`),!0)},has(t,o){return o===A||o in t},ownKeys(){return[A,...Object.keys(n)]},getOwnPropertyDescriptor(t,o){if(o===A)return{configurable:!0,enumerable:!0,writable:!1,value:n};let r=n[o];if(r)return{configurable:!0,enumerable:!0,writable:!0,value:r()}}})}function je(e){let n={};for(let[t,o]of Object.entries(e))T(o)&&(n[t]=o);return n}function se(e){return q(e,je(e))}var Fe=Symbol("sui.shared");function Ve(e){return!!e?.[Fe]}function ae(e,n,t){e[`__sui_ctx_${n}`]=t}function ce(e){let n={};for(let t in e)Object.prototype.hasOwnProperty.call(e,t)&&Ve(e[t])&&(n[t]=e[t]);return n}function S(e,n,t){if(typeof n!="function")return;let o,r=!1;_(()=>{let i=t?t():void 0;r?o&&o.update?.(i):(r=!0,o=n(e,i))}),H(()=>{o&&o.destroy?.()})}var de=(e,n,t)=>Math.max(n,Math.min(e,t)),j=(e,n)=>{let t=e,o=n??(t.getAttribute("placement")||"bottom"),r=()=>{if(!t.id)return null;let a=`[popovertarget="${t.id}"]`,c=t.getRootNode(),u=c.querySelector?.(a);return u||(c.host?.querySelector(a)??null)},i=!1,s=()=>{try{return t.matches(":popover-open")}catch{return i}},l=()=>{if(!s())return;let a=r();if(!a)return;let c=a.getBoundingClientRect(),u=t.offsetWidth,p=t.offsetHeight,v=window.innerWidth,x=window.innerHeight,f=o.split("-")[0],g=o.endsWith("-end"),h,m;f==="top"||f==="bottom"?(h=f==="bottom"?c.bottom+4:c.top-p-4,f==="bottom"&&h+p>x&&c.top-p-4>=0?h=c.top-p-4:f==="top"&&h<0&&c.bottom+p+4<=x&&(h=c.bottom+4),m=g?c.right-u:c.left):(m=f==="right"?c.right+4:c.left-u-4,f==="right"&&m+u>v&&c.left-u-4>=0?m=c.left-u-4:f==="left"&&m<0&&c.right+u+4<=v&&(m=c.right+4),h=g?c.bottom-p:c.top),t.style.position="fixed",t.style.margin="0",t.style.left=`${de(m,4,Math.max(4,v-u-4))}px`,t.style.top=`${de(h,4,Math.max(4,x-p-4))}px`},d=a=>{let c=a.newState==="open";i=c,c&&requestAnimationFrame(l)};return t.addEventListener("toggle",d),window.addEventListener("resize",l),window.addEventListener("scroll",l,!0),{update:a=>{o=a??"bottom",l()},destroy:()=>{t.removeEventListener("toggle",d),window.removeEventListener("resize",l),window.removeEventListener("scroll",l,!0)}}};function $e(e,n,t){e.dispatchEvent(new CustomEvent(n,{detail:t,bubbles:!0,composed:!0}))}function Ie(e,n){let t=e,o=Object.keys(n);for(let r of o)t[r]=n[r];H(()=>{for(let r of o)if(t[r]===n[r])try{delete t[r]}catch{t[r]=void 0}})}function J(e){let n=e;n.emit=function(t,o){$e(e,t,o)},n.expose=function(t){Ie(e,t)}}function X(e){let n;return()=>{if(n===void 0){let t=document.createElement("template");t.innerHTML=e,n=t.content.firstElementChild}return n.cloneNode(!0)}}var le=e=>e??"",ue=e=>e===null||e===""?NaN:Number(e),pe=e=>!(e===null||e.toLowerCase()==="false");function fe(e,n){let t=new Set(e);return o=>o!==null&&t.has(o)?o:n}var F=e=>{switch(e.type){case"string":return le;case"bool":return pe;case"number":return ue;case"union":{let n=e.values??[],t=e.default??n[0]??"";return fe(n,t)}case"property":return n=>n}};function me(e,n,t,o){if(!n)return;let r={},i,s=t.formValue;if(s!==void 0){let{sig:d,propDef:a}=ve(t,o,s,"formValue"),c=a.default,u=F(a);i=_(()=>{let p=d();typeof n.setFormValue=="function"&&n.setFormValue(Ge(p))}),r.reset=()=>{d.set(c)},r.restore=p=>{d.set(u(p))}}let l=t.formDisabled;if(l!==void 0){let{sig:d,propDef:a}=ve(t,o,l,"formDisabled");if(a.type!=="bool")throw new Error(`[sui-runtime] formDisabled: "${l}" must be a bool-typed prop`);r.disabled=c=>{d.set(!!c)}}return e.__shipSetFormHooks=r,i&&(e.__shipSetFormValueCleanup=i),i}function ge(e){let n=e;delete n.__shipSetFormHooks,delete n.__shipSetFormValueCleanup}function Ge(e){return e==null?"":typeof e=="boolean"?e?"on":"":String(e)}function ve(e,n,t,o){let r=e.props[t];if(!r)throw new Error(`[sui-runtime] ${o}: "${t}" \u2014 no matching prop on this component`);let i=n[B(t)];if(!i)throw new Error(`[sui-runtime] ${o}: "${t}" \u2014 prop signal missing`);return{sig:i,propDef:r}}var V="__shipSetLifecycle";function be(e,n){let t=Ye(e,n),o={ship:n.ship,clients:n.clients??{},...n.clients??{},route:n.route??{},navigate:n.navigate??(a=>{}),...n.scope??{}},r=t.scope?t.scope(o,n.host):{};Object.assign(o,r);let i=Ke(o,n.stateKeys,r),s=q(o,i),l=Ue(t,s),d={...r,...l};if(Object.keys(r).length>0&&n.host){let a=ce(r);if(Object.keys(a).length>0){let c=n.host,u=c.tagName.toLowerCase();ae(c,u,a)}}if(t.mount||t.unmount){let a={state:s};t.mount&&(a.mount=t.mount),t.unmount&&(a.unmount=t.unmount),Object.defineProperty(d,V,{value:a,enumerable:!0,configurable:!0,writable:!1})}return d}function Ye(e,n){return typeof e=="function"?e({ship:n.ship,clients:n.clients??{}}):e}function Ke(e,n,t){let o={},r=n!==void 0,i=t?Object.keys(t):[];if(r||i.length>0){if(n)for(let s of n){let l=e[s];T(l)&&(o[s]=l)}for(let s of i){if(s in o)continue;let l=e[s];T(l)&&(o[s]=l)}return o}for(let[s,l]of Object.entries(e))T(l)&&(o[s]=l);return o}function Ue(e,n){let t=e.on??{},o={};for(let[r,i]of Object.entries(t))o[r]=(...s)=>i(n,...s);return o}function Q(e){return{setup:n=>be(e,n),__def:e}}function he(e,n,t){let o=ee(),r,i=[];return C(o,()=>O(n,()=>{let s=e.setup(t);if(!s)return;let l=s.__mount;typeof l=="function"&&(r=l(n));let d=Object.getOwnPropertyDescriptor(s,V)?.value;if(d){let a=se(s);if(d.mount){let c=d.mount(a,n);typeof c=="function"&&i.push(c)}if(d.unmount){let c=d.unmount;i.push(()=>c(a))}}})),()=>{for(let s of i.splice(0))try{s()}catch(l){console.error("[sui/compose] lifecycle cleanup threw:",l)}try{typeof r=="function"&&r()}catch(s){console.error("[sui/compose] child unmount threw:",s)}o.dispose()}}var We=e=>Array.from(e.querySelectorAll("slot")).filter(n=>n.closest("[data-component]")===e),qe=e=>e.nodeType===Node.ELEMENT_NODE?!0:e.nodeType===Node.TEXT_NODE?(e.textContent??"").trim().length>0:!1;function ye(e,n){let t=We(e);if(t.length===0){for(let s of n)s.remove?.();return}let o=new Map,r=null;for(let s of t){let l=s.getAttribute("name");l?o.set(l,s):r??(r=s)}let i=new Map;for(let s of n){let l=s.nodeType===Node.ELEMENT_NODE?s.getAttribute("slot"):null,d=l?o.get(l)??null:r;if(d&&(l||qe(s))){let a=i.get(d)??[];a.push(s),i.set(d,a)}else s.remove?.()}for(let[s,l]of i)s.replaceChildren(...l)}var we=null,xe=!1;function ke(){if(we){we();return}xe||typeof document>"u"||(xe=!0,!document.querySelector("link[data-sui-contracts], style[data-sui-contracts]")&&document.documentElement.getAttribute("data-sui-contracts")!=="none"&&console.warn('[sui] A component mounted but the framework contracts are not present. Either link the stylesheet (`<link rel="stylesheet" href="\u2026/contracts.css" data-sui-contracts>`, shipped as @ship/sui-tokens/contracts.css) or `import "@ship/sui-runtime/contracts"` to inject it from JS. Linking is preferred: it keeps ~30KB of CSS out of your JS bundle.'))}var Je=(e,n)=>{if(n==null)return null;switch(e.type){case"string":return n===""?null:String(n);case"number":return Number.isNaN(n)?null:String(n);case"bool":return n?"":null;case"union":return String(n);case"property":return null}},Xe=(e,n)=>{let t=`ship-set-style-${e}`;if(document.getElementById(t))return;let o=document.createElement("style");o.id=t,o.textContent=`@scope ([data-component="${e}"]) to ([data-component]) { ${n} }`,document.head.appendChild(o)},Qe=(e,n,t,o)=>{let r=e.getAttribute(n),i=r!==null?o(r):t.default;return N(i)},Ze=e=>{let n=[],t={};for(let[i,s]of Object.entries(e.props??{}))s.type!=="property"&&n.push(i),t[i]={camel:B(i),decode:F(s)};let o=e.formAssociated===!0||e.formValue!==void 0||e.formDisabled!==void 0,r={observedAttrs:n,props:t};return o&&(r.formAssociated=!0),r};function $(e,n,t){if(typeof e=="function")return{setup:e};let o=e,r=n,i=t,s=Ze(r),l=Object.entries(r.props??{});Se(o,s,(d,a,c)=>{let u=null,p=null;if(r.shadow){if(u=d.shadowRoot??d.attachShadow({mode:"open"}),r.styles){let f=document.createElement("style");f.textContent=r.styles,u.appendChild(f)}}else p=Array.from(d.childNodes),d.style.display="contents",ke(),r.styles&&Xe(o,r.styles);let v=[];for(let[f,g]of l){let h=B(f),m=s.props[f].decode,y=Qe(d,f,g,m);if(a[h]=y,g.reflect){let R=!0,_e=_(()=>{let Re=y();if(R){R=!1;return}let Z=Je(g,Re);Z===null?d.removeAttribute(f):d.setAttribute(f,Z)});v.push(_e)}}if(r.formValue!==void 0||r.formDisabled!==void 0){let f=me(d,c,r,a);typeof f=="function"&&v.push(f)}let x={props:a,host:d};return c&&(x.internals=c),v.push(he(i,d,x)),p&&ye(d,p),()=>{ge(d);for(let f of v.splice(0))try{f()}catch(g){console.error(`[ship-set:${o}] cleanup threw:`,g)}}})}var Ce=new Set,Ee=new Map;function Se(e,n,t){let o=Ee.get(e);if(o){o.factory=t,o.schema=n;for(let s of o.instances)try{s.__suiReRun()}catch(l){console.error(`[ship-set:${e}] HMR re-run threw:`,l)}return}if(customElements.get(e)){Ce.has(e)||(Ce.add(e),console.warn(`[sui-runtime] customElements.define("${e}", \u2026) called twice. The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.`));return}let r={schema:n,factory:t,instances:new Set};if(Ee.set(e,r),n.formAssociated){class s extends HTMLElement{constructor(){super();w(this,"cleanup");w(this,"scope");w(this,"propSignals",{});w(this,"pendingProps");w(this,"internals");this.internals=this.attachInternals()}static get observedAttributes(){return r.schema.observedAttrs}runSetup(){this.scope=z();try{this.cleanup=C(this.scope,()=>O(this,()=>r.factory(this,this.propSignals,this.internals)))}catch(a){let c=D(this.parentNode);c?c.catch(a,"mount"):console.error(`[ship-set:${e}] setup threw:`,a)}if(this.pendingProps){for(let a in this.pendingProps)this.propSignals[a]?.set(this.pendingProps[a]);this.pendingProps=void 0}}teardown(){try{this.cleanup?.()}catch(a){console.error(`[ship-set:${e}] cleanup threw:`,a)}this.cleanup=void 0;try{this.scope?.dispose()}catch(a){console.error(`[ship-set:${e}] scope dispose threw:`,a)}this.scope=void 0}connectedCallback(){this.dataset.component=e,J(this),r.instances.add(this),this.runSetup()}disconnectedCallback(){r.instances.delete(this),this.teardown()}__suiReRun(){this.teardown(),(this.shadowRoot??this).replaceChildren(),this.propSignals={},this.runSetup()}attributeChangedCallback(a,c,u){let p=r.schema.props[a];if(!p)return;let v=this.propSignals[p.camel];v&&v.set(p.decode(u))}formResetCallback(){let a=this.__shipSetFormHooks;try{a?.reset?.()}catch(c){console.error(`[ship-set:${e}] formResetCallback threw:`,c)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(a,c){let u=this.__shipSetFormHooks;try{u?.restore?.(a)}catch(p){console.error(`[ship-set:${e}] formStateRestoreCallback threw:`,p)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:a,mode:c}}))}formDisabledCallback(a){let c=this.__shipSetFormHooks;try{c?.disabled?.(a)}catch(u){console.error(`[ship-set:${e}] formDisabledCallback threw:`,u)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:a}}))}}w(s,"formAssociated",!0),Te(s.prototype,n),customElements.define(e,s);return}class i extends HTMLElement{constructor(){super(...arguments);w(this,"cleanup");w(this,"scope");w(this,"propSignals",{});w(this,"pendingProps")}static get observedAttributes(){return r.schema.observedAttrs}runSetup(){this.scope=z();try{if(this.cleanup=C(this.scope,()=>O(this,()=>r.factory(this,this.propSignals))),this.pendingProps){for(let d in this.pendingProps)this.propSignals[d]?.set(this.pendingProps[d]);this.pendingProps=void 0}}catch(d){let a=D(this.parentNode);a?a.catch(d,"mount"):console.error(`[ship-set:${e}] setup threw:`,d)}}teardown(){try{this.cleanup?.()}catch(d){console.error(`[ship-set:${e}] cleanup threw:`,d)}this.cleanup=void 0;try{this.scope?.dispose()}catch(d){console.error(`[ship-set:${e}] scope dispose threw:`,d)}this.scope=void 0}connectedCallback(){this.dataset.component=e,J(this),r.instances.add(this),this.runSetup()}disconnectedCallback(){r.instances.delete(this),this.teardown()}__suiReRun(){this.teardown(),(this.shadowRoot??this).replaceChildren(),this.propSignals={},this.runSetup()}attributeChangedCallback(d,a,c){let u=r.schema.props[d];if(!u)return;let p=this.propSignals[u.camel];p&&p.set(u.decode(c))}}Te(i.prototype,n),customElements.define(e,i)}function Te(e,n){for(let t of Object.keys(n.props)){let o=n.props[t];if(!o)continue;let r=o.camel;r in e||Object.defineProperty(e,r,{configurable:!0,enumerable:!1,get(){let i=this.propSignals[r];return i?W(()=>i()):void 0},set(i){let s=this.propSignals[r];s?s.set(i):(this.pendingProps??(this.pendingProps={}))[r]=i}})}}function B(e){return e.replace(/-([a-z])/g,(n,t)=>t.toUpperCase())}{let e=globalThis;e.__suiRuntimeInstances=(e.__suiRuntimeInstances??0)+1,e.__suiRuntimeInstances>1&&console.warn("[sui-runtime] loaded %d times. In production this means a bundle is not externalizing @ship/sui-runtime (split-brain signals); under vite shell-dev it is the expected @fs-source + /_deck-bundle duplication.",e.__suiRuntimeInstances)}var nn=N(!1),I=Q({scope:()=>({open:nn})});var tn=X('<div class="sui-col sui-g-3 sui-ai-stretch"><div class="sui-card"><div role="status" aria-live="polite" aria-atomic="true" tone="info" class="sui-alert"><span class="ico"></span><div class="content"><div class="title"></div><div class="body">Heads up</div><div class="actions"></div></div><button type="button" aria-label="Dismiss" class="close"><svg viewbox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" fill="none"></path></svg></button></div><div class="sui-row sui-g-2 sui-ai-center sui-jc-start"><button variant="solid" tone="primary">Save</button><span tone="success" class="sui-badge">ok</span><span class="sui-chip"><span class="leading"></span>filter<button type="button" aria-label="Remove" tabindex="-1" class="remove"><svg viewbox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8"></path></svg></button></span><input type="checkbox" role="switch" class="sui-switch"></div><div class="sui-field"><div class="label"></div><div class="control"><label for="n">Name</label><input id="n" placeholder="name"></div><div class="hint msg"></div><div role="alert" aria-live="polite" class="error msg"><svg viewbox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path><path d="M12 9v4M12 17h.01"></path></svg><span></span></div></div><button popovertarget="p1">Menu</button><div popover="" id="p1" class="sui-dropdown"><button type="button" value="a" class="sui-menu-item">Edit</button><button type="button" value="b" class="sui-menu-item">Delete</button></div><button command="show-modal" commandfor="d1">Open</button><dialog id="d1"><header>Hi</header><section>Body</section></dialog><div class="sui-tabs"><div role="tablist" class="sui-tab-list"><button type="button" class="sui-tab">One</button><button type="button" class="sui-tab">Two</button></div><div class="sui-tab-panel">1</div><div class="sui-tab-panel">2</div></div></div></div>'),Lt=Object.assign($(e=>{let{ship:n,route:t,navigate:o}=e,r=Object.create(null);if(r.ship=n,r.route=t,r.navigate=o,I&&typeof I.setup=="function"){e.scope=r;let i=I.setup(e);i&&typeof i=="object"&&Object.assign(r,i)}return Object.assign({},r,{__mount(i){let s=[],l=i.shadowRoot??i,d="6785cde6",a=`.sui-col{display:flex;flex-direction:column;min-width:0}.sui-col>*{min-width:0}.sui-g-3{gap:var(--space-3)}.sui-ai-stretch{align-items:stretch}.sui-row{display:flex;flex-wrap:wrap;min-width:0}.sui-row>*{min-width:0}.sui-g-2{gap:var(--space-2)}.sui-ai-center{align-items:center}.sui-jc-start{justify-content:flex-start}label { display: inline-block; }


  label {
    display: inline-block;
    font-family: var(--font-body);
    font-weight: 600;
    letter-spacing: 0;
    color: var(--foreground);
    cursor: pointer;
  }
  label[hidden] { display: none; }

  label:not([size]),
  label[size='md'] { font-size: var(--text-base); }
  label[size='sm'],
  label[size='lg'] { font-size: var(--text-sm); }

  .req {
    display: none;
    color: var(--destructive);
    margin-left: var(--space-1);
    font-weight: 700;
  }
  label[required] .req { display: inline; }

  .sui-card {
    display: block;
    position: relative;
    background: var(--surface-1);
    color: var(--card-foreground);
    border: 1px solid var(--border);
    --card-radius: var(--radius-lg);
    border-radius: var(--card-radius);
    overflow: clip;
    /* Frost on glass themes (--card-backdrop-filter resolves to a blur there,
       \`none\` everywhere else). isolation establishes a stacking context so the
       backdrop-filter clips to --card-radius \u2014 belt-and-suspenders with the
       existing overflow:clip. */
    backdrop-filter: var(--card-backdrop-filter);
    -webkit-backdrop-filter: var(--card-backdrop-filter);
    isolation: isolate;
    box-shadow: var(--shadow-card-rest);
    --card-px: var(--space-5);
    --card-py: var(--space-4);
    --card-gap: var(--space-3);
    padding: var(--card-py) var(--card-px);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  /* variant=outline: background:transparent is a raw CSS keyword, not a
     token \u2014 belongs in <style>, not the sx vocabulary. */
  .sui-card[variant='outline'] { background: transparent; }

  /* Tone-bound border. Default tone resolves --tone-border to --border (the
     same as the host baseline), so the override is a no-op for it. */
  .sui-card[tone]:not([tone='default']) { border-color: var(--tone-border); }

  /* Accent bar. hover fades in on hover; static stays at full opacity
   * (hero cards, print exports, touch contexts where hover never fires). */
  .sui-card[accent-bar='hover']::before,
  .sui-card[accent-bar='static']::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    opacity: 0;
    border-radius: var(--card-radius) var(--card-radius) 0 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    pointer-events: none;
  }
  .sui-card[accent-bar='hover']:hover::before { opacity: 1; }
  .sui-card[accent-bar='static']::before { opacity: 1; }

  .sui-card[interactive] { cursor: pointer; -webkit-tap-highlight-color: transparent; }
  .sui-card[interactive]:hover {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-hover);
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    .sui-card[interactive]:hover { transform: none; }
  }
  .sui-card[interactive]:active {
    box-shadow: var(--shadow-card-rest);
    transform: translateY(0);
    transition-duration: 60ms;
  }
  .sui-card[interactive]:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }

  /* Tone-rail. Only meaningful once a tone is set \u2014 otherwise --tone-fg
   * resolves to --foreground and the rail would read as body-text-colored
   * decoration with the weight of an emphasis cue. */
  .sui-card[status-rail] {
    border-left: 3px solid var(--tone-fg);
    padding-inline-start: calc(var(--card-px) - var(--space-1));
  }
  .sui-card[status-rail]:not([tone]),
  .sui-card[status-rail][tone='default'] {
    border-left-color: var(--border-strong);
  }

  .sui-card[fill] { background: var(--tone-bg); border-color: var(--tone-border); }

  /* Per-padding local-var recompute + fallback padding. */
  .sui-card[padding='none'] { --card-px: 0px; --card-py: 0px; --card-gap: 0px; padding: 0; }
  .sui-card[padding='sm']   { --card-px: var(--space-3); --card-py: var(--space-2); --card-gap: var(--space-2); padding: var(--space-2) var(--space-3); }
  .sui-card[padding='md']   { --card-px: var(--space-5); --card-py: var(--space-4); padding: var(--space-4) var(--space-5); }
  .sui-card[padding='lg']   { --card-px: var(--space-6); --card-py: var(--space-5); --card-gap: var(--space-4); padding: var(--space-5) var(--space-6); }

  /* Header/footer \u2014 presence-selected direct children (see the structural
     divergence note above), edge-to-edge via negative margins that cancel
     the host's own padding, separated by a hairline. */
  .sui-card > [slot='header'] {
    display: block;
    margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
    padding: var(--card-py) var(--card-px);
    border-bottom: 1px solid var(--border-subtle);
    font-family: var(--heading-font);
    font-weight: var(--heading-weight);
  }
  .sui-card > [slot='footer'] {
    display: block;
    margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
    padding: var(--card-py) var(--card-px);
    border-top: 1px solid var(--border-subtle);
  }
  .sui-card[padding='none'] > [slot='header'] {
    margin: 0 0 var(--space-3);
    padding: 0 0 var(--space-3);
  }
  .sui-card[padding='none'] > [slot='footer'] {
    margin: var(--space-3) 0 0;
    padding: var(--space-3) 0 0;
  }

  @media (forced-colors: active) {
    .sui-card[interactive]:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  .sui-alert {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: var(--space-3);
    position: relative;
    overflow: hidden;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-1);
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    box-shadow: var(--shadow-xs);
  }
  .sui-alert[hidden] { display: none; }

  /* Tone: muted default (specificity 0,2,0) so the GLOBAL [tone='X'] channel
     fills every branded tone. --tone drives the leading stripe; --tone-bg/-fg
     the icon chip. */
  .sui-alert:not([tone]),
  .sui-alert[tone='default'] {
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-bg: var(--tone-muted-bg);
  }

  /* Leading tone stripe (anchored to the relative root). */
  .sui-alert::before {
    content: '';
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: var(--space-1);
    background: var(--tone);
  }

  .sui-alert .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--control-h-xs);
    height: var(--control-h-xs);
    border-radius: var(--radius-md);
    background: var(--tone-bg);
    color: var(--tone-fg);
  }
  .sui-alert .ico:empty { display: none; }
  .sui-alert .ico svg,
  .sui-alert .ico * { width: var(--icon-size-sm); height: var(--icon-size-sm); }

  .sui-alert .content {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .sui-alert .title { font-weight: var(--font-weight-semibold); }
  .sui-alert .title:empty { display: none; }
  .sui-alert .body { color: var(--muted-foreground); }
  .sui-alert .actions {
    display: flex;
    flex-direction: row;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
  .sui-alert .actions:empty { display: none; }

  /* Self-contained dismiss control \u2014 hidden until [dismissible]. */
  .sui-alert .close {
    display: none;
    align-items: center;
    justify-content: center;
    align-self: start;
    width: var(--control-h-xs);
    height: var(--control-h-xs);
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--muted-foreground);
    cursor: pointer;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .sui-alert[dismissible] .close { display: inline-flex; }
  .sui-alert .close:hover { background: var(--hover-bg); color: var(--foreground); }
  .sui-alert .close:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }
  .sui-alert .close svg { width: var(--icon-size-sm); height: var(--icon-size-sm); }

  @media (forced-colors: active) {
    .sui-alert { border-color: CanvasText; }
    .sui-alert::before { background: CanvasText; }
  }
.sui-badge { display: inline-flex; }


  .sui-badge {
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
    /* Size: md base; sm/lg override below. */
    --bdg-h: 22px;
    --bdg-px: 8px;
    --bdg-fs: 10.5px;
    --bdg-gap: 4px;
    --bdg-radius: var(--radius-sm);
    gap: var(--bdg-gap);
    min-height: var(--bdg-h);
    padding: 0 var(--bdg-px);
    font-family: var(--font-body);
    font-size: var(--bdg-fs);
    font-weight: var(--font-weight-semibold);
    line-height: 1;
    white-space: nowrap;
    border-radius: var(--bdg-radius);
    -webkit-font-smoothing: antialiased;
    font-variant-numeric: tabular-nums;
    /* soft is the default variant \u2014 base .sui-badge renders soft. */
    background: var(--tone-bg);
    border: 1px solid var(--tone-border);
    color: var(--tone-fg);
  }
  .sui-badge[hidden] { display: none; }

  /* Tone: muted is the default. Scoped to no-tone/default (specificity 0,2,0)
     so the GLOBAL [tone='X'] channel (contracts.css, 0,1,0) fills every branded
     tone without a per-tone rule here and without an order-dependent clash. */
  .sui-badge:not([tone]),
  .sui-badge[tone='default'] {
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-contrast: var(--background);
    --tone-bg: var(--tone-muted-bg);
    --tone-border: var(--tone-muted-border);
  }

  .sui-badge[size='sm'] { --bdg-h: 18px; --bdg-px: 6px;  --bdg-fs: 9.5px;  --bdg-gap: 3px; }
  .sui-badge[size='lg'] { --bdg-h: 26px; --bdg-px: 10px; --bdg-fs: 11.5px; --bdg-gap: 5px; }

  /* Shape \u2014 square (radius-sm) default; pill opts into the full radius. */
  .sui-badge[pill] { --bdg-radius: var(--radius-full); }

  /* Variant matrix (soft is the base above). */
  .sui-badge[variant='solid'] {
    background: var(--tone);
    border-color: var(--tone);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 var(--hover-bg);
  }
  .sui-badge[variant='outline'] {
    background: transparent;
    border-color: var(--tone-border);
    color: var(--tone-fg);
  }

  @media (forced-colors: active) {
    .sui-badge { border-color: CanvasText; color: CanvasText; }
    .sui-badge[variant='solid'] { background: Highlight; color: HighlightText; border-color: Highlight; }
  }
.sui-chip { display: inline-flex; }


  .sui-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--chip-gap);
    vertical-align: middle;
    min-height: var(--chip-h);
    padding: 0 var(--chip-px);
    font-family: var(--font-body);
    font-size: var(--chip-fs);
    font-weight: var(--font-weight-medium);
    line-height: 1;
    color: var(--tone-fg);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--chip-radius);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
    -webkit-tap-highlight-color: transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
    /* size: md base; sm/lg override below */
    --chip-h: var(--control-h-xs);
    --chip-px: var(--space-2_5);
    --chip-fs: var(--text-sm);
    --chip-gap: var(--space-1_5);
    --chip-icon: var(--icon-size-xs);
    --chip-remove: var(--icon-size-md);
    --chip-radius: var(--radius-full);
    /* tone: default channels; per-tone overrides below */
    --tone-fg: var(--foreground);
    --tone-bg: var(--tone-default-bg);
    --tone-border: var(--tone-default-border);
    --tone-bg-hover: var(--tone-default-bg);
    --tone-border-hover: var(--tone-default-border);
    --tone-bg-solid: var(--foreground);
    --tone-outline-border: var(--border-strong);
    --tone-outline-bg-hover: var(--hover-bg);
    --tone-ghost-bg-hover: var(--hover-bg);
    --tone-contrast: var(--background);
  }
  .sui-chip[hidden] { display: none; }

  .sui-chip[size='sm'] {
    --chip-h: var(--control-h-2xs); --chip-px: var(--space-2); --chip-fs: var(--text-xs);
    --chip-gap: var(--space-1); --chip-icon: var(--icon-size-2xs); --chip-remove: var(--icon-size-sm);
  }
  .sui-chip[size='lg'] {
    --chip-h: var(--control-h-md); --chip-px: var(--space-3); --chip-fs: var(--text-base);
    --chip-gap: var(--space-1_5); --chip-icon: var(--icon-size-sm); --chip-remove: var(--icon-size-lg);
  }

  /* Per-tone channel overrides (default is the base above). */
  .sui-chip[tone='primary'] {
    --tone-fg: var(--primary); --tone-bg: var(--tone-primary-bg); --tone-border: var(--tone-primary-border);
    --tone-bg-hover: var(--tone-primary-border); --tone-border-hover: var(--tone-primary-border);
    --tone-bg-solid: var(--primary); --tone-outline-border: var(--tone-primary-border);
    --tone-outline-bg-hover: var(--tone-primary-bg); --tone-ghost-bg-hover: var(--tone-primary-bg);
    --tone-contrast: var(--primary-foreground);
  }
  .sui-chip[tone='info'] {
    --tone-fg: var(--info); --tone-bg: var(--tone-info-bg); --tone-border: var(--tone-info-border);
    --tone-bg-hover: var(--tone-info-border); --tone-border-hover: var(--tone-info-border);
    --tone-bg-solid: var(--info); --tone-outline-border: var(--tone-info-border);
    --tone-outline-bg-hover: var(--tone-info-bg); --tone-ghost-bg-hover: var(--tone-info-bg);
    --tone-contrast: var(--info-foreground);
  }
  .sui-chip[tone='success'] {
    --tone-fg: var(--success); --tone-bg: var(--tone-success-bg); --tone-border: var(--tone-success-border);
    --tone-bg-hover: var(--tone-success-border); --tone-border-hover: var(--tone-success-border);
    --tone-bg-solid: var(--success); --tone-outline-border: var(--tone-success-border);
    --tone-outline-bg-hover: var(--tone-success-bg); --tone-ghost-bg-hover: var(--tone-success-bg);
    --tone-contrast: var(--success-foreground);
  }
  .sui-chip[tone='warning'] {
    --tone-fg: var(--warning); --tone-bg: var(--tone-warning-bg); --tone-border: var(--tone-warning-border);
    --tone-bg-hover: var(--tone-warning-border); --tone-border-hover: var(--tone-warning-border);
    --tone-bg-solid: var(--warning); --tone-outline-border: var(--tone-warning-border);
    --tone-outline-bg-hover: var(--tone-warning-bg); --tone-ghost-bg-hover: var(--tone-warning-bg);
    --tone-contrast: var(--warning-foreground);
  }
  .sui-chip[tone='destructive'] {
    --tone-fg: var(--destructive); --tone-bg: var(--tone-destructive-bg); --tone-border: var(--tone-destructive-border);
    --tone-bg-hover: var(--tone-destructive-border); --tone-border-hover: var(--tone-destructive-border);
    --tone-bg-solid: var(--destructive); --tone-outline-border: var(--tone-destructive-border);
    --tone-outline-bg-hover: var(--tone-destructive-bg); --tone-ghost-bg-hover: var(--tone-destructive-bg);
    --tone-contrast: var(--destructive-foreground);
  }
  .sui-chip[tone='accent'] {
    --tone-fg: var(--accent); --tone-bg: var(--tone-accent-bg); --tone-border: var(--tone-accent-border);
    --tone-bg-hover: var(--tone-accent-border); --tone-border-hover: var(--tone-accent-border);
    --tone-bg-solid: var(--accent); --tone-outline-border: var(--tone-accent-border);
    --tone-outline-bg-hover: var(--tone-accent-bg); --tone-ghost-bg-hover: var(--tone-accent-bg);
    --tone-contrast: var(--accent-foreground);
  }
  .sui-chip[tone='muted'] {
    --tone-fg: var(--muted-foreground); --tone-bg: var(--tone-muted-bg); --tone-border: var(--tone-muted-border);
    --tone-bg-hover: var(--tone-muted-bg); --tone-border-hover: var(--tone-muted-border);
    --tone-bg-solid: var(--muted); --tone-outline-border: var(--border-strong);
    --tone-outline-bg-hover: var(--tone-muted-bg); --tone-ghost-bg-hover: var(--tone-muted-bg);
    --tone-contrast: var(--background);
  }

  /* Variant matrix (soft is the default). */
  .sui-chip:not([variant]),
  .sui-chip[variant='soft'] { background: var(--tone-bg); border-color: var(--tone-border); }
  .sui-chip[variant='soft']:not([disabled]):hover { background: var(--tone-bg-hover); border-color: var(--tone-border-hover); }
  .sui-chip[variant='outline'] { background: transparent; border-color: var(--tone-outline-border); }
  .sui-chip[variant='outline']:not([disabled]):hover { background: var(--tone-outline-bg-hover); }
  .sui-chip[variant='solid'] {
    background: var(--tone-bg-solid); border-color: var(--tone-border); color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 var(--hover-bg);
  }
  .sui-chip[variant='ghost'] { background: transparent; border-color: transparent; }
  .sui-chip[variant='ghost']:not([disabled]):hover { background: var(--tone-ghost-bg-hover); }

  /* Selected. */
  .sui-chip[selected] {
    background: var(--tone-bg-solid); border-color: var(--tone-border); color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 var(--hover-bg); font-weight: var(--font-weight-semibold);
  }
  .sui-chip[selected]:not([disabled]):hover { background: var(--tone-bg-hover); }
  .sui-chip[selected]:not([tone]),
  .sui-chip[selected][tone='muted'] { background: var(--primary); border-color: var(--primary); color: var(--primary-foreground); }
  .sui-chip[selected][tone='default'] { background: var(--foreground); border-color: var(--foreground); color: var(--background); }

  .sui-chip:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
    border-radius: var(--chip-radius);
  }
  .sui-chip[disabled]:focus-visible { outline: none; }
  .sui-chip[disabled] {
    cursor: not-allowed;
    background: var(--muted);
    border-color: var(--border-subtle);
    color: var(--muted-foreground);
    filter: saturate(0.65);
    box-shadow: none;
  }

  .sui-chip .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--chip-icon);
    height: var(--chip-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .sui-chip .leading:not(:empty) { display: inline-flex; }
  .sui-chip .leading svg { width: 100%; height: 100%; }

  .sui-chip .remove {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--chip-remove);
    height: var(--chip-remove);
    margin-inline: var(--space-1) calc(var(--space-1) * -1);
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
    border-radius: var(--radius-full);
    flex-shrink: 0;
    transition: background-color var(--duration-fast) var(--ease-out);
  }
  .sui-chip[removable] .remove { display: inline-flex; }
  .sui-chip .remove svg { width: 70%; height: 70%; display: block; stroke: currentColor; fill: none; stroke-width: 1.75; stroke-linecap: round; }
  .sui-chip .remove:hover:not(:disabled) { background: var(--hover-bg); }
  .sui-chip[disabled] .remove { cursor: not-allowed; }

  @media (forced-colors: active) {
    .sui-chip { border-color: CanvasText; color: CanvasText; }
    .sui-chip[selected] { background: Highlight; color: HighlightText; border-color: Highlight; }
    .sui-chip:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
.sui-switch { display: inline-flex; }


  .sui-switch {
    appearance: none;
    -webkit-appearance: none;
    position: relative;
    display: inline-block;
    flex-shrink: 0;
    vertical-align: middle;
    box-sizing: border-box;
    margin: 0;
    cursor: pointer;
    background: var(--muted);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-full);
    -webkit-tap-highlight-color: transparent;
    /* Tone: default reads as a brand toggle (primary). */
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
    /* Size: md base; sm/lg override. */
    --sw-w: var(--control-h-sm);
    --sw-h: var(--icon-size-md);
    --sw-thumb: var(--space-3);
    width: var(--sw-w);
    height: var(--sw-h);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .sui-switch[size='sm'] { --sw-w: var(--control-h-xs); --sw-h: var(--icon-size-sm); --sw-thumb: var(--space-2_5); }
  .sui-switch[size='lg'] { --sw-w: var(--space-7);      --sw-h: var(--icon-size-lg); --sw-thumb: var(--icon-size-sm); }

  .sui-switch[tone='primary']     { --tone: var(--primary);          --tone-on-fg: var(--primary-foreground); }
  .sui-switch[tone='info']        { --tone: var(--info);             --tone-on-fg: var(--info-foreground); }
  .sui-switch[tone='success']     { --tone: var(--success);          --tone-on-fg: var(--success-foreground); }
  .sui-switch[tone='warning']     { --tone: var(--warning);          --tone-on-fg: var(--warning-foreground); }
  .sui-switch[tone='destructive'] { --tone: var(--destructive);      --tone-on-fg: var(--destructive-foreground); }
  .sui-switch[tone='accent']      { --tone: var(--accent);           --tone-on-fg: var(--accent-foreground); }
  .sui-switch[tone='muted']       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }

  /* Thumb \u2014 a pseudo-element (works because appearance:none un-replaces the box). */
  .sui-switch::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 1px;
    width: var(--sw-thumb);
    height: var(--sw-thumb);
    background: var(--surface-1);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translateY(-50%);
    transition:
      left var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out);
  }

  .sui-switch:not(:disabled):not(:checked):hover { background: var(--hover-bg); }

  .sui-switch:checked {
    background: var(--tone);
    border-color: var(--selected-border);
    box-shadow: var(--shadow-xs);
  }
  .sui-switch:checked::before {
    left: calc(100% - var(--sw-thumb) - 1px);
    background: var(--tone-on-fg);
  }

  .sui-switch:focus-visible {
    --ring-color: var(--tone);
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }

  .sui-switch:disabled {
    cursor: not-allowed;
    background: var(--muted);
    border-color: var(--border);
    filter: saturate(0.6);
    box-shadow: none;
  }
  .sui-switch:disabled::before { box-shadow: var(--shadow-xs); }

  @media (forced-colors: active) {
    .sui-switch { border-color: CanvasText; }
    .sui-switch:checked { background: Highlight; }
    .sui-switch:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .sui-switch, .sui-switch::before { transition: none; }
  }

  .sui-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-family: var(--font-body);
  }
  .sui-field[hidden] { display: none; }

  .sui-field .msg { margin-top: var(--space-1); }
  .sui-field .hint,
  .sui-field .error {
    font-size: var(--text-xs);
    line-height: var(--leading-snug);
  }
  .sui-field .hint { color: var(--muted-foreground); }
  .sui-field .hint:empty { display: none; }
  .sui-field .error {
    display: flex;
    align-items: flex-start;
    gap: var(--space-1);
    color: var(--destructive);
    font-weight: var(--font-weight-medium);
  }
  .sui-field .error svg { width: var(--icon-size-sm); height: var(--icon-size-sm); flex-shrink: 0; }

  /* hint shows when valid, error shows when [invalid] \u2014 pure CSS swap. */
  .sui-field:not([invalid]) .error { display: none; }
  .sui-field[invalid] .hint { display: none; }

  .sui-dropdown {
    position: fixed;
    inset: auto;
    margin: 0;
    min-width: 12rem;
    padding: var(--space-1);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: var(--z-popover, 1000);
  }
  .sui-dropdown::backdrop { background: transparent; }
.sui-menu-item { display: inline-flex; }


  .sui-menu-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-2_5);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    line-height: 1.2;
    color: var(--foreground);
    background: transparent;
    border: 0;
    border-radius: var(--radius-sm);
    text-align: start;
    cursor: pointer;
    white-space: nowrap;
  }
  .sui-menu-item:hover:not([disabled]) { background: var(--muted); }
  .sui-menu-item:focus-visible { background: var(--muted); outline: var(--ring-width) solid var(--ring); outline-offset: -1px; }
  .sui-menu-item[disabled] { opacity: 0.5; cursor: not-allowed; }

  .sui-tabs {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    /* size vocabulary (md base) \u2014 read by <tab> */
    --tab-fs: var(--text-sm);
    --tab-py: var(--space-2);
    --tab-px: var(--space-3);
  }
  .sui-tabs[size='sm'] { --tab-fs: var(--text-xs); --tab-py: var(--space-1_5); --tab-px: var(--space-2_5); }
  .sui-tabs[size='lg'] { --tab-fs: var(--text-base); --tab-py: var(--space-2_5); --tab-px: var(--space-4); }

  .sui-tabs[orientation='vertical'] { flex-direction: row; align-items: flex-start; }

  .sui-tabs[sticky] .sui-tab-list {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 100);
    background: var(--background);
  }

  /* pill: the strip is a muted track, the active tab a raised chip */
  .sui-tabs[variant='pill'] .sui-tab-list {
    gap: var(--space-1);
    padding: var(--space-1);
    border: 0;
    border-radius: var(--radius-md);
    background: var(--muted);
  }
  .sui-tabs[variant='pill'] .sui-tab {
    border-bottom: 0;
    border-radius: var(--radius-sm);
  }
  .sui-tabs[variant='pill'] .sui-tab[aria-selected='true'] {
    color: var(--foreground);
    background: var(--background);
    box-shadow: var(--shadow-xs);
  }

  /* enclosed: bordered folder tabs joined to the panel */
  .sui-tabs[variant='enclosed'] .sui-tab-list { gap: 0; }
  .sui-tabs[variant='enclosed'] .sui-tab {
    border: 1px solid transparent;
    border-bottom: 0;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }
  .sui-tabs[variant='enclosed'] .sui-tab[aria-selected='true'] {
    color: var(--foreground);
    border-color: var(--border);
    background: var(--background);
    margin-bottom: -1px;
  }

  .sui-tab-list {
    display: flex;
    align-items: stretch;
    gap: var(--space-1);
    border-bottom: 1px solid var(--border);
  }
  .sui-tabs[orientation='vertical'] .sui-tab-list {
    flex-direction: column;
    align-items: stretch;
    border-bottom: 0;
    border-right: 1px solid var(--border);
  }
.sui-tab { display: inline-flex; }


  .sui-tab {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--tab-py, var(--space-2)) var(--tab-px, var(--space-3));
    font-family: var(--font-body);
    font-size: var(--tab-fs, var(--text-sm));
    font-weight: var(--font-weight-medium);
    line-height: 1;
    color: var(--muted-foreground);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    -webkit-tap-highlight-color: transparent;
    transition:
      color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .sui-tab:hover:not([disabled]) { color: var(--foreground); }
  .sui-tab[aria-selected='true'] {
    color: var(--foreground);
    font-weight: var(--font-weight-semibold);
    border-bottom-color: var(--tone, var(--primary));
  }
  .sui-tab[disabled] { opacity: 0.5; cursor: not-allowed; }
  .sui-tab:focus-visible {
    outline: var(--ring-width) solid var(--tone-ring, var(--ring));
    outline-offset: calc(-1 * var(--ring-width));
  }

  .sui-tab-panel { padding: var(--space-4) 0 0; min-width: 0; }
  .sui-tabs[orientation='vertical'] .sui-tab-panel { padding: 0 0 0 var(--space-4); }`;{let m=null,y=i.firstChild;for(;y;){if(y.nodeType===1&&y.tagName==="STYLE"&&y.getAttribute("data-sui-style")===d){m=y;break}y=y.nextSibling}if(!m){let R=document.createElement("style");R.setAttribute("data-sui-style",d),R.textContent=a,i.insertBefore(R,i.firstChild)}}let c=tn(),p=c.firstChild.firstChild;S(p,r.alert,void 0);let v=p.nextSibling,x=v.firstChild.nextSibling.nextSibling;S(x,r.chip,void 0);let f=v.nextSibling;S(f,r.field,void 0);let g=f.nextSibling.nextSibling;S(g,j,void 0),S(g,r.menu,void 0);let h=g.nextSibling.nextSibling.nextSibling;return S(h,r.tabs,void 0),l.appendChild(c),()=>{for(let m of s)try{m()}catch{}}}})}),{_compilerVersion:{cargo:"0.1.0",git:"4739e844",dirty:!0,builtAt:"2026-07-12T11:09:18Z"},clients:[]});
//! The one `MutableState` proxy. Both mount paths build their writable state
//! through `createMountState` — there is no second implementation, and there
//! is no per-mount mode flag.
//!
//! Reads of declared signal keys return the UNWRAPPED value (the ergonomic
//! `state.count++`, `state.label = 'hi'`). Writes route through `signal.set`
//! via the set-trap. The raw signal is reached via `state.$signals.x` for the
//! cases the assignment-style can't express (handing a `Signal<T>` to a
//! callback, calling `.set(updater)` with a function).
//! Non-signal scope members (methods, plain refs, `derived` getters that
//! arrived as callable views) fall through unchanged — `state.togglePanel`
//! is the bound method, `state.foo` is whatever the factory returned.
//! The unwrap is INTENTIONAL sugar for IMPERATIVE contexts (`on:` handlers,
//! `mount`, `unmount`) where reactive tracking doesn't apply — those are
//! one-shot reads, not subscriptions. Reactive contexts (`derived`, `effect`,
//! templates) receive the raw scope, not this proxy, so they always work
//! against callable signals.
//! History: the proxy used to take an `unwrap: boolean` option — component-
//! mount passed `unwrap: false`, leaving `state.x` as the raw signal. That
//! created a type/runtime lie (`MutableState<Scope>` unwraps in TS, but the
//! component runtime didn't), which silently broke casts like
//! `state.canvasRef as HTMLCanvasElement`. The fork retired in favor of one
//! shape; the 58 component sites that called `state.x()` became `state.x`.
/*! _compilerVersion {"cargo":"0.1.0","git":"4739e844","dirty":true,"builtAt":"2026-07-12T11:09:18Z"} */
