(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const r of i)if(r.type==="childList")for(const o of r.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&s(o)}).observe(document,{childList:!0,subtree:!0});function n(i){const r={};return i.integrity&&(r.integrity=i.integrity),i.referrerPolicy&&(r.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?r.credentials="include":i.crossOrigin==="anonymous"?r.credentials="omit":r.credentials="same-origin",r}function s(i){if(i.ep)return;i.ep=!0;const r=n(i);fetch(i.href,r)}})();/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const D=globalThis,Q=D.ShadowRoot&&(D.ShadyCSS===void 0||D.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,X=Symbol(),se=new WeakMap;let me=class{constructor(e,n,s){if(this._$cssResult$=!0,s!==X)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=n}get styleSheet(){let e=this.o;const n=this.t;if(Q&&e===void 0){const s=n!==void 0&&n.length===1;s&&(e=se.get(n)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&se.set(n,e))}return e}toString(){return this.cssText}};const we=t=>new me(typeof t=="string"?t:t+"",void 0,X),Ae=(t,...e)=>{const n=t.length===1?t[0]:e.reduce((s,i,r)=>s+(o=>{if(o._$cssResult$===!0)return o.cssText;if(typeof o=="number")return o;throw Error("Value passed to 'css' function must be a 'css' function result: "+o+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(i)+t[r+1],t[0]);return new me(n,t,X)},Ce=(t,e)=>{if(Q)t.adoptedStyleSheets=e.map(n=>n instanceof CSSStyleSheet?n:n.styleSheet);else for(const n of e){const s=document.createElement("style"),i=D.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=n.cssText,t.appendChild(s)}},ie=Q?t=>t:t=>t instanceof CSSStyleSheet?(e=>{let n="";for(const s of e.cssRules)n+=s.cssText;return we(n)})(t):t;/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const{is:Se,defineProperty:Ee,getOwnPropertyDescriptor:Pe,getOwnPropertyNames:ke,getOwnPropertySymbols:Me,getPrototypeOf:Te}=Object,J=globalThis,re=J.trustedTypes,Oe=re?re.emptyScript:"",Ne=J.reactiveElementPolyfillSupport,N=(t,e)=>t,V={toAttribute(t,e){switch(e){case Boolean:t=t?Oe:null;break;case Object:case Array:t=t==null?t:JSON.stringify(t)}return t},fromAttribute(t,e){let n=t;switch(e){case Boolean:n=t!==null;break;case Number:n=t===null?null:Number(t);break;case Object:case Array:try{n=JSON.parse(t)}catch{n=null}}return n}},Y=(t,e)=>!Se(t,e),oe={attribute:!0,type:String,converter:V,reflect:!1,useDefault:!1,hasChanged:Y};Symbol.metadata??=Symbol("metadata"),J.litPropertyMetadata??=new WeakMap;let k=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,n=oe){if(n.state&&(n.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((n=Object.create(n)).wrapped=!0),this.elementProperties.set(e,n),!n.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,n);i!==void 0&&Ee(this.prototype,e,i)}}static getPropertyDescriptor(e,n,s){const{get:i,set:r}=Pe(this.prototype,e)??{get(){return this[n]},set(o){this[n]=o}};return{get:i,set(o){const a=i?.call(this);r?.call(this,o),this.requestUpdate(e,a,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??oe}static _$Ei(){if(this.hasOwnProperty(N("elementProperties")))return;const e=Te(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(N("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(N("properties"))){const n=this.properties,s=[...ke(n),...Me(n)];for(const i of s)this.createProperty(i,n[i])}const e=this[Symbol.metadata];if(e!==null){const n=litPropertyMetadata.get(e);if(n!==void 0)for(const[s,i]of n)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[n,s]of this.elementProperties){const i=this._$Eu(n,s);i!==void 0&&this._$Eh.set(i,n)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const n=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)n.unshift(ie(i))}else e!==void 0&&n.push(ie(e));return n}static _$Eu(e,n){const s=n.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,n=this.constructor.elementProperties;for(const s of n.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Ce(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,n,s){this._$AK(e,s)}_$ET(e,n){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const r=(s.converter?.toAttribute!==void 0?s.converter:V).toAttribute(n,s.type);this._$Em=e,r==null?this.removeAttribute(i):this.setAttribute(i,r),this._$Em=null}}_$AK(e,n){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const r=s.getPropertyOptions(i),o=typeof r.converter=="function"?{fromAttribute:r.converter}:r.converter?.fromAttribute!==void 0?r.converter:V;this._$Em=i;const a=o.fromAttribute(n,r.type);this[i]=a??this._$Ej?.get(i)??a,this._$Em=null}}requestUpdate(e,n,s,i=!1,r){if(e!==void 0){const o=this.constructor;if(i===!1&&(r=this[e]),s??=o.getPropertyOptions(e),!((s.hasChanged??Y)(r,n)||s.useDefault&&s.reflect&&r===this._$Ej?.get(e)&&!this.hasAttribute(o._$Eu(e,s))))return;this.C(e,n,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,n,{useDefault:s,reflect:i,wrapped:r},o){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,o??n??this[e]),r!==!0||o!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(n=void 0),this._$AL.set(e,n)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(n){Promise.reject(n)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,r]of this._$Ep)this[i]=r;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,r]of s){const{wrapped:o}=r,a=this[i];o!==!0||this._$AL.has(i)||a===void 0||this.C(i,void 0,r,a)}}let e=!1;const n=this._$AL;try{e=this.shouldUpdate(n),e?(this.willUpdate(n),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(n)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(n)}willUpdate(e){}_$AE(e){this._$EO?.forEach(n=>n.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(n=>this._$ET(n,this[n])),this._$EM()}updated(e){}firstUpdated(e){}};k.elementStyles=[],k.shadowRootOptions={mode:"open"},k[N("elementProperties")]=new Map,k[N("finalized")]=new Map,Ne?.({ReactiveElement:k}),(J.reactiveElementVersions??=[]).push("2.1.2");/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const ee=globalThis,ae=t=>t,F=ee.trustedTypes,le=F?F.createPolicy("lit-html",{createHTML:t=>t}):void 0,ge="$lit$",w=`lit$${Math.random().toFixed(9).slice(2)}$`,$e="?"+w,Ue=`<${$e}>`,P=document,j=()=>P.createComment(""),I=t=>t===null||typeof t!="object"&&typeof t!="function",te=Array.isArray,je=t=>te(t)||typeof t?.[Symbol.iterator]=="function",G=`[ 	
\f\r]`,O=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,ce=/-->/g,de=/>/g,S=RegExp(`>|${G}(?:([^\\s"'>=/]+)(${G}*=${G}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),he=/'/g,ue=/"/g,be=/^(?:script|style|textarea|title)$/i,Ie=t=>(e,...n)=>({_$litType$:t,strings:e,values:n}),p=Ie(1),M=Symbol.for("lit-noChange"),u=Symbol.for("lit-nothing"),pe=new WeakMap,E=P.createTreeWalker(P,129);function ye(t,e){if(!te(t)||!t.hasOwnProperty("raw"))throw Error("invalid template strings array");return le!==void 0?le.createHTML(e):e}const ze=(t,e)=>{const n=t.length-1,s=[];let i,r=e===2?"<svg>":e===3?"<math>":"",o=O;for(let a=0;a<n;a++){const l=t[a];let c,h,d=-1,g=0;for(;g<l.length&&(o.lastIndex=g,h=o.exec(l),h!==null);)g=o.lastIndex,o===O?h[1]==="!--"?o=ce:h[1]!==void 0?o=de:h[2]!==void 0?(be.test(h[2])&&(i=RegExp("</"+h[2],"g")),o=S):h[3]!==void 0&&(o=S):o===S?h[0]===">"?(o=i??O,d=-1):h[1]===void 0?d=-2:(d=o.lastIndex-h[2].length,c=h[1],o=h[3]===void 0?S:h[3]==='"'?ue:he):o===ue||o===he?o=S:o===ce||o===de?o=O:(o=S,i=void 0);const y=o===S&&t[a+1].startsWith("/>")?" ":"";r+=o===O?l+Ue:d>=0?(s.push(c),l.slice(0,d)+ge+l.slice(d)+w+y):l+w+(d===-2?a:y)}return[ye(t,r+(t[n]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class z{constructor({strings:e,_$litType$:n},s){let i;this.parts=[];let r=0,o=0;const a=e.length-1,l=this.parts,[c,h]=ze(e,n);if(this.el=z.createElement(c,s),E.currentNode=this.el.content,n===2||n===3){const d=this.el.content.firstChild;d.replaceWith(...d.childNodes)}for(;(i=E.nextNode())!==null&&l.length<a;){if(i.nodeType===1){if(i.hasAttributes())for(const d of i.getAttributeNames())if(d.endsWith(ge)){const g=h[o++],y=i.getAttribute(d).split(w),_=/([.?@])?(.*)/.exec(g);l.push({type:1,index:r,name:_[2],strings:y,ctor:_[1]==="."?Re:_[1]==="?"?Le:_[1]==="@"?De:Z}),i.removeAttribute(d)}else d.startsWith(w)&&(l.push({type:6,index:r}),i.removeAttribute(d));if(be.test(i.tagName)){const d=i.textContent.split(w),g=d.length-1;if(g>0){i.textContent=F?F.emptyScript:"";for(let y=0;y<g;y++)i.append(d[y],j()),E.nextNode(),l.push({type:2,index:++r});i.append(d[g],j())}}}else if(i.nodeType===8)if(i.data===$e)l.push({type:2,index:r});else{let d=-1;for(;(d=i.data.indexOf(w,d+1))!==-1;)l.push({type:7,index:r}),d+=w.length-1}r++}}static createElement(e,n){const s=P.createElement("template");return s.innerHTML=e,s}}function T(t,e,n=t,s){if(e===M)return e;let i=s!==void 0?n._$Co?.[s]:n._$Cl;const r=I(e)?void 0:e._$litDirective$;return i?.constructor!==r&&(i?._$AO?.(!1),r===void 0?i=void 0:(i=new r(t),i._$AT(t,n,s)),s!==void 0?(n._$Co??=[])[s]=i:n._$Cl=i),i!==void 0&&(e=T(t,i._$AS(t,e.values),i,s)),e}class He{constructor(e,n){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=n}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:n},parts:s}=this._$AD,i=(e?.creationScope??P).importNode(n,!0);E.currentNode=i;let r=E.nextNode(),o=0,a=0,l=s[0];for(;l!==void 0;){if(o===l.index){let c;l.type===2?c=new R(r,r.nextSibling,this,e):l.type===1?c=new l.ctor(r,l.name,l.strings,this,e):l.type===6&&(c=new Be(r,this,e)),this._$AV.push(c),l=s[++a]}o!==l?.index&&(r=E.nextNode(),o++)}return E.currentNode=P,i}p(e){let n=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,n),n+=s.strings.length-2):s._$AI(e[n])),n++}}class R{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,n,s,i){this.type=2,this._$AH=u,this._$AN=void 0,this._$AA=e,this._$AB=n,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const n=this._$AM;return n!==void 0&&e?.nodeType===11&&(e=n.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,n=this){e=T(this,e,n),I(e)?e===u||e==null||e===""?(this._$AH!==u&&this._$AR(),this._$AH=u):e!==this._$AH&&e!==M&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):je(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==u&&I(this._$AH)?this._$AA.nextSibling.data=e:this.T(P.createTextNode(e)),this._$AH=e}$(e){const{values:n,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=z.createElement(ye(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(n);else{const r=new He(i,this),o=r.u(this.options);r.p(n),this.T(o),this._$AH=r}}_$AC(e){let n=pe.get(e.strings);return n===void 0&&pe.set(e.strings,n=new z(e)),n}k(e){te(this._$AH)||(this._$AH=[],this._$AR());const n=this._$AH;let s,i=0;for(const r of e)i===n.length?n.push(s=new R(this.O(j()),this.O(j()),this,this.options)):s=n[i],s._$AI(r),i++;i<n.length&&(this._$AR(s&&s._$AB.nextSibling,i),n.length=i)}_$AR(e=this._$AA.nextSibling,n){for(this._$AP?.(!1,!0,n);e!==this._$AB;){const s=ae(e).nextSibling;ae(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class Z{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,n,s,i,r){this.type=1,this._$AH=u,this._$AN=void 0,this.element=e,this.name=n,this._$AM=i,this.options=r,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=u}_$AI(e,n=this,s,i){const r=this.strings;let o=!1;if(r===void 0)e=T(this,e,n,0),o=!I(e)||e!==this._$AH&&e!==M,o&&(this._$AH=e);else{const a=e;let l,c;for(e=r[0],l=0;l<r.length-1;l++)c=T(this,a[s+l],n,l),c===M&&(c=this._$AH[l]),o||=!I(c)||c!==this._$AH[l],c===u?e=u:e!==u&&(e+=(c??"")+r[l+1]),this._$AH[l]=c}o&&!i&&this.j(e)}j(e){e===u?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Re extends Z{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===u?void 0:e}}class Le extends Z{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==u)}}class De extends Z{constructor(e,n,s,i,r){super(e,n,s,i,r),this.type=5}_$AI(e,n=this){if((e=T(this,e,n,0)??u)===M)return;const s=this._$AH,i=e===u&&s!==u||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,r=e!==u&&(s===u||i);i&&this.element.removeEventListener(this.name,this,s),r&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Be{constructor(e,n,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=n,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){T(this,e)}}const Ve=ee.litHtmlPolyfillSupport;Ve?.(z,R),(ee.litHtmlVersions??=[]).push("3.3.2");const Fe=(t,e,n)=>{const s=n?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const r=n?.renderBefore??null;s._$litPart$=i=new R(e.insertBefore(j(),r),r,void 0,n??{})}return i._$AI(t),i};/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const ne=globalThis;class U extends k{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const n=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=Fe(n,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return M}}U._$litElement$=!0,U.finalized=!0,ne.litElementHydrateSupport?.({LitElement:U});const We=ne.litElementPolyfillSupport;We?.({LitElement:U});(ne.litElementVersions??=[]).push("4.2.2");/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const qe=t=>(e,n)=>{n!==void 0?n.addInitializer(()=>{customElements.define(t,e)}):customElements.define(t,e)};/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const Je={attribute:!0,type:String,converter:V,reflect:!1,hasChanged:Y},Ze=(t=Je,e,n)=>{const{kind:s,metadata:i}=n;let r=globalThis.litPropertyMetadata.get(i);if(r===void 0&&globalThis.litPropertyMetadata.set(i,r=new Map),s==="setter"&&((t=Object.create(t)).wrapped=!0),r.set(n.name,t),s==="accessor"){const{name:o}=n;return{set(a){const l=e.get.call(this);e.set.call(this,a),this.requestUpdate(o,l,t,!0,a)},init(a){return a!==void 0&&this.C(o,void 0,t,a),a}}}if(s==="setter"){const{name:o}=n;return function(a){const l=this[o];e.call(this,a),this.requestUpdate(o,l,t,!0,a)}}throw Error("Unsupported decorator location: "+s)};function Ke(t){return(e,n)=>typeof n=="object"?Ze(t,e,n):((s,i,r)=>{const o=i.hasOwnProperty(r);return i.constructor.createProperty(r,s),o?Object.getOwnPropertyDescriptor(i,r):void 0})(t,e,n)}var Ge=Object.defineProperty,Qe=Object.getOwnPropertyDescriptor,_e=(t,e,n,s)=>{for(var i=s>1?void 0:s?Qe(e,n):e,r=t.length-1,o;r>=0;r--)(o=t[r])&&(i=(s?o(e,n,i):o(i))||i);return s&&i&&Ge(e,n,i),i};function Xe(t){let e="@default";const n=new Map;let s={},i="root";for(const r of t){const o=r.createSurface||r.beginRendering;o&&(e=o.surfaceId||"@default",o.root&&(i=o.root));const a=r.updateComponents||r.surfaceUpdate;if(a)for(const c of a.components){const h={id:c.id};if(c.component&&typeof c.component=="object"){const d=Object.keys(c.component);if(d.length===1){const g=d[0];h.component=g;const y=c.component[g];if(y&&typeof y=="object")for(const[_,f]of Object.entries(y))_==="children"&&f&&typeof f=="object"&&f.explicitList?h.children=f.explicitList:_==="text"&&f&&typeof f=="object"&&f.literalString!=null?h.text=f.literalString:_==="label"&&f&&typeof f=="object"&&f.literalString!=null?h.label=f.literalString:_==="name"&&f&&typeof f=="object"&&f.literalString!=null?h.name=f.literalString:_==="description"&&f&&typeof f=="object"&&f.literalString!=null?h.description=f.literalString:_==="url"&&f&&typeof f=="object"&&f.literalString!=null?h.url=f.literalString:h[_]=f}}else Object.assign(h,c);n.set(h.id,h),h.id==="root"&&(i="root")}const l=r.updateDataModel||r.dataModelUpdate;l&&(!l.path||l.path==="/")&&(s={...s,...l.value})}return n.size===0?null:{surfaceId:e,components:n,dataModel:s,rootId:i}}function W(t,e){const n=t.replace(/^\//,"").split("/");let s=e;for(const i of n){if(s==null||typeof s!="object")return;let r=i;if(/^\{.+\}$/.test(i)){const o=i.slice(1,-1),a=e["_"+o]??e[o];a!=null&&(r=String(a))}s=s[r]}return s}function m(t,e){if(t==null)return"";if(typeof t=="string")return t.includes("${/")?t.replace(/\$\{(\/[^}]+)\}/g,(n,s)=>{const i=W(s,e);return i!=null?String(i):""}):t;if(t.path){const n=W(t.path,e);return n!=null?String(n):""}return""}function L(t,e){if(t!=null){if(typeof t=="object"&&t!==null&&"path"in t)return W(t.path,e);if(typeof t=="string"&&t.includes("${/")){const n=t.match(/^\$\{(\/[^}]+)\}$/);if(n)return W(n[1],e)}return t}}let q=class extends U{constructor(){super(...arguments),this.surface=null,this._inputValues=new Map}updated(t){t.has("surface")&&this._inputValues.clear()}_fireAction(t,e,n){const s={},i=this.surface?.dataModel??{};for(const[o,a]of Object.entries(n))s[o]=L(a,i);const r={...s};for(const[o,a]of this._inputValues)o in r||(r[o]=a);if(this._inputValues.size>0){const o={};for(const[a,l]of this._inputValues)o[a]=l;r._formData=o}this.dispatchEvent(new CustomEvent("a2ui-action",{bubbles:!0,composed:!0,detail:{name:t,sourceComponentId:e,context:r}}))}render(){return this.surface?this._renderComponent(this.surface.rootId):u}_renderComponent(t){const n=this.surface.components.get(t);if(!n)return u;switch(n.component||n.type||""){case"Card":return this._renderCard(n);case"Column":return this._renderColumn(n);case"Row":return this._renderRow(n);case"Text":return this._renderText(n);case"Button":return this._renderButton(n);case"Divider":return p`<hr class="divider ${n.axis==="vertical"?"vertical":""}" />`;case"CheckBox":return this._renderCheckBox(n);case"Slider":return this._renderSlider(n);case"TextField":return this._renderTextField(n);case"Image":return this._renderImage(n);case"Icon":return this._renderIcon(n);case"Tabs":return this._renderTabs(n);case"List":return this._renderList(n);case"Modal":return this._renderModal(n);case"ChoicePicker":case"MultipleChoice":return this._renderChoicePicker(n);case"DateTimeInput":return this._renderDateTimeInput(n);case"Video":return this._renderVideo(n);case"AudioPlayer":return this._renderAudioPlayer(n);default:return n.children?p`<div>${n.children.map(i=>this._renderComponent(i))}</div>`:n.child?this._renderComponent(n.child):u}}_renderCard(t){return p`
      <div class="card">
        ${t.child?this._renderComponent(t.child):u}
      </div>
    `}_renderColumn(t){const e=this._getConsumedChildIds(),n=(t.children||[]).filter(s=>!e.has(s));return p`
      <div class="column" data-align="${t.align||""}">
        ${n.map(s=>this._renderComponent(s))}
      </div>
    `}_renderRow(t){const e=this._getConsumedChildIds(),n=(t.children||[]).filter(s=>!e.has(s));return p`
      <div class="row" data-justify="${t.justify||""}">
        ${n.map(s=>this._renderComponent(s))}
      </div>
    `}_renderText(t){const e=m(t.text,this.surface.dataModel),n=t.variant||"body";return p`<span class="text-${n}">${e}</span>`}_renderButton(t){let e="";if(t.child){const o=this.surface.components.get(t.child);o&&(e=m(o.text,this.surface.dataModel))}e||(e=t.label||t.text||t.id);const n=t.variant||"",i=t.primary===!0||n==="filled"?"primary":n||"";return p`
      <button class="btn ${i}" @click=${()=>{const o=t.action?.functionCall;if(o){this._handleFunctionCall(o);return}const a=t.action?.event;if(a){const l=this._extractUrlFromEvent(a);if(l){window.open(l,"_blank","noopener");return}this._fireAction(a.name||"unknown",t.id,a.context||{})}}}>
        ${e}
      </button>
    `}_handleFunctionCall(t){switch(t.call){case"openUrl":{const e=t.args?.url;e&&window.open(e,"_blank","noopener");break}default:console.warn(`[A2UI] Unhandled client functionCall: ${t.call}`,t.args)}}_extractUrlFromEvent(t){const e=t.context;if(!e)return null;for(const n of Object.values(e))if(typeof n=="string"&&/^https?:\/\/.+/i.test(n))return n;return null}_renderCheckBox(t){const e=m(t.label,this.surface.dataModel),n=L(t.value,this.surface.dataModel);return p`
      <label class="checkbox-wrapper">
        <input type="checkbox" .checked=${!!n} @change=${i=>{const r=i.target;this._inputValues.set(t.id,r.checked)}} />
        ${e}
      </label>
    `}_renderSlider(t){const e=m(t.label,this.surface.dataModel),n=L(t.value,this.surface.dataModel),s=t.min??0,i=t.max??100,r=o=>{const a=o.target;this._inputValues.set(t.id,Number(a.value))};return p`
      <div class="slider-wrapper">
        ${e?p`<label>${e}</label>`:u}
        <input type="range" min=${s} max=${i} .value=${String(n??s)} @input=${r} />
        <span class="slider-value">${n??s} / ${i}</span>
      </div>
    `}_renderTextField(t){const e=m(t.label,this.surface.dataModel),n=m(t.text??t.value,this.surface.dataModel),s=t.textFieldType||"shortText",i=o=>{const a=o.target;this._inputValues.set(t.id,a.value)};if(s==="longText"||s==="multiline")return p`
        <div class="textfield-wrapper">
          ${e?p`<label>${e}</label>`:u}
          <textarea rows="3" .value=${n} placeholder=${e} @input=${i}></textarea>
        </div>
      `;const r=s==="obscured"||s==="password"?"password":s==="number"?"number":s==="date"?"date":s==="email"?"email":"text";return p`
      <div class="textfield-wrapper">
        ${e?p`<label>${e}</label>`:u}
        <input type=${r} .value=${n} placeholder=${e} @input=${i} />
      </div>
    `}_renderImage(t){const e=m(t.url,this.surface.dataModel),n=t.variant||t.usageHint||"",s=t.fit?`object-fit: ${t.fit}`:"";return p`<img class="a2ui-image ${n}" src=${e} style=${s} alt="" />`}_renderIcon(t){const n=m(t.name,this.surface.dataModel).replace(/([A-Z])/g,"_$1").toLowerCase().replace(/^_/,""),i={cloud:"☁️",sunny:"☀️",clear:"☀️",sun:"☀️",umbrella:"☂️",rain:"🌧️",rainy:"🌧️",snow:"❄️",thunderstorm:"⛈️",fog:"🌫️",wind:"💨",partly_cloudy:"⛅",partly_cloudy_day:"⛅",partly_cloudy_night:"⛅",check:"✅",close:"❌",star:"⭐",favorite:"❤️",home:"🏠",settings:"⚙️",search:"🔍",info:"ℹ️",warning:"⚠️",error:"❗",calendar:"📅",schedule:"📅",location:"📍",place:"📍",restaurant:"🍽️",music:"🎵",play:"▶️",pause:"⏸️",stop:"⏹️"}[n];return i?p`<span class="a2ui-icon-emoji">${i}</span>`:p`<span class="material-symbols-outlined a2ui-icon">${n}</span>`}_renderTabs(t){const e=t.tabItems||t.tabs||[];if(e.length===0)return u;const n=0;return p`
      <div>
        <div class="tabs-header">
          ${e.map((s,i)=>{const r=m(s.title||s.label,this.surface.dataModel);return p`<button class="tab-btn ${i===n?"active":""}">${r}</button>`})}
        </div>
        <div class="tab-content">
          ${this._renderComponent(e[n].child)}
        </div>
      </div>
    `}_renderList(t){const e=t.direction==="horizontal"?"horizontal":"vertical",n=t.children||[];if(!Array.isArray(n)&&typeof n=="object"){const s=n;if(s.componentId&&s.path){const i=this.surface.components.get(s.componentId),r=L({path:s.path},this.surface.dataModel);if(i&&Array.isArray(r))return p`
            <div class="list-${e}">
              ${r.map((o,a)=>this._renderTemplateInstance(i,o,a))}
            </div>
          `}return u}return Array.isArray(n)?p`
        <div class="list-${e}">
          ${n.map(s=>this._renderComponent(s))}
        </div>
      `:u}_renderTemplateInstance(t,e,n){const s=this.surface,i=s.dataModel;s.dataModel={...i,...e,current:e,_index:n};try{return this._renderComponent(t.id)}finally{s.dataModel=i}}_renderModal(t){const e=t.entryPointChild||t.trigger||"";return t.contentChild||t.content,p`
      <div>
        ${e?this._renderComponent(e):u}
      </div>
    `}_renderChoicePicker(t){const e=m(t.label,this.surface.dataModel),n=t.options||[],s=t.variant||"radio",i=s==="multipleSelection"||s==="chip"||t.component==="MultipleChoice",r=i?"checkbox":"radio",o=`choice-${t.id}`,a=l=>{const c=l.target;if(i){const h=this._inputValues.get(t.id)||[];c.checked?this._inputValues.set(t.id,[...h,c.value]):this._inputValues.set(t.id,h.filter(d=>d!==c.value))}else this._inputValues.set(t.id,c.value)};return p`
      <div class="choice-picker">
        ${e?p`<label class="group-label">${e}</label>`:u}
        ${n.map(l=>{const c=m(l.label,this.surface.dataModel);return p`
            <label class="choice-option">
              <input type=${r} name=${o} value=${l.value} @change=${a} />
              ${c}
            </label>
          `})}
      </div>
    `}_renderDateTimeInput(t){const e=m(t.label,this.surface.dataModel),n=m(t.value,this.surface.dataModel),s=t.enableDate!==!1,i=t.enableTime===!0,r=s&&i?"datetime-local":i?"time":"date";return p`
      <div class="datetime-wrapper">
        ${e?p`<label>${e}</label>`:u}
        <input type=${r} .value=${n} />
      </div>
    `}_renderVideo(t){const e=m(t.url,this.surface.dataModel);return p`<video class="a2ui-video" src=${e} controls></video>`}_renderAudioPlayer(t){const e=m(t.url,this.surface.dataModel),n=m(t.description,this.surface.dataModel);return p`
      <div class="audio-wrapper">
        ${n?p`<span class="audio-desc">${n}</span>`:u}
        <audio src=${e} controls></audio>
      </div>
    `}_getConsumedChildIds(){const t=new Set;for(const e of this.surface.components.values())(e.component||e.type||"")==="Button"&&e.child&&t.add(e.child);return t}};q.styles=Ae`
    :host { display: block; }

    .card {
      background: #fff;
      border: 1px solid #dadce0;
      border-radius: 12px;
      padding: 16px;
      box-shadow: 0 1px 3px rgba(0,0,0,.08);
    }

    .column { display: flex; flex-direction: column; gap: 6px; }
    .column[data-align="center"] { align-items: center; }
    .column[data-align="start"] { align-items: flex-start; }
    .column[data-align="end"] { align-items: flex-end; }

    .row { display: flex; flex-direction: row; gap: 8px; align-items: center; flex-wrap: wrap; }
    .row[data-justify="center"] { justify-content: center; }
    .row[data-justify="spaceAround"] { justify-content: space-around; }
    .row[data-justify="spaceBetween"] { justify-content: space-between; }
    .row[data-justify="end"] { justify-content: flex-end; }

    .text-h1 { font-size: 28px; font-weight: 500; }
    .text-h2 { font-size: 20px; font-weight: 500; }
    .text-h3 { font-size: 16px; font-weight: 500; }
    .text-subtitle { font-size: 15px; color: #5f6368; }
    .text-body { font-size: 14px; color: #3c4043; }
    .text-caption { font-size: 12px; color: #9aa0a6; }

    .btn {
      display: inline-flex; align-items: center; justify-content: center;
      padding: 8px 20px;
      border: 1px solid #dadce0;
      border-radius: 20px;
      background: #fff;
      color: #1a73e8;
      font-size: 14px; font-weight: 500;
      cursor: pointer;
      transition: background .15s, box-shadow .15s;
      font-family: inherit;
    }
    .btn:hover { background: #f0f4ff; box-shadow: 0 1px 4px rgba(26,115,232,.2); }
    .btn:active { background: #e0eaff; }
    .btn.primary, .btn.filled {
      background: #1a73e8; color: #fff; border-color: #1a73e8;
    }
    .btn.primary:hover, .btn.filled:hover { background: #1669d0; }
    .btn.outlined { background: #fff; color: #1a73e8; border-color: #1a73e8; }
    .btn.text { background: transparent; border: none; color: #1a73e8; }

    .divider { border: none; border-top: 1px solid #e0e0e0; margin: 8px 0; }
    .divider.vertical { border-top: none; border-left: 1px solid #e0e0e0; height: 100%; margin: 0 8px; }

    /* CheckBox */
    .checkbox-wrapper {
      display: flex; align-items: center; gap: 8px; cursor: pointer;
      padding: 4px 0; font-size: 14px; color: #3c4043;
    }
    .checkbox-wrapper input[type="checkbox"] {
      width: 18px; height: 18px; accent-color: #1a73e8; cursor: pointer;
    }

    /* Slider */
    .slider-wrapper { display: flex; flex-direction: column; gap: 4px; width: 100%; }
    .slider-wrapper label { font-size: 12px; color: #5f6368; }
    .slider-wrapper input[type="range"] {
      width: 100%; accent-color: #1a73e8; cursor: pointer;
    }
    .slider-value { font-size: 12px; color: #9aa0a6; text-align: right; }

    /* TextField */
    .textfield-wrapper { display: flex; flex-direction: column; gap: 4px; width: 100%; }
    .textfield-wrapper label { font-size: 12px; color: #5f6368; }
    .textfield-wrapper input, .textfield-wrapper textarea {
      padding: 8px 12px; border: 1px solid #dadce0; border-radius: 8px;
      font-size: 14px; font-family: inherit; outline: none;
      transition: border-color .15s;
    }
    .textfield-wrapper input:focus, .textfield-wrapper textarea:focus {
      border-color: #1a73e8;
    }

    /* Image */
    .a2ui-image { max-width: 100%; border-radius: 8px; }
    .a2ui-image.icon { width: 24px; height: 24px; border-radius: 0; }
    .a2ui-image.avatar { width: 40px; height: 40px; border-radius: 50%; object-fit: cover; }
    .a2ui-image.thumbnail { width: 80px; height: 80px; object-fit: cover; }
    .a2ui-image.banner { width: 100%; max-height: 200px; object-fit: cover; }
    .a2ui-image.smallFeature { width: 64px; height: 64px; object-fit: contain; }
    .a2ui-image.largeFeature { width: 100%; max-height: 280px; object-fit: contain; }

    /* Icon */
    .a2ui-icon { font-family: 'Material Symbols Outlined', sans-serif; font-size: 24px; color: #5f6368; }
    .a2ui-icon-emoji { font-size: 20px; }

    /* Tabs */
    .tabs-header { display: flex; border-bottom: 2px solid #e0e0e0; gap: 0; }
    .tab-btn {
      padding: 8px 16px; border: none; background: transparent;
      font-size: 14px; font-weight: 500; color: #5f6368;
      cursor: pointer; border-bottom: 2px solid transparent;
      margin-bottom: -2px; font-family: inherit;
    }
    .tab-btn.active { color: #1a73e8; border-bottom-color: #1a73e8; }
    .tab-btn:hover { background: #f0f4ff; }
    .tab-content { padding: 12px 0; }

    /* ChoicePicker */
    .choice-picker { display: flex; flex-direction: column; gap: 6px; }
    .choice-picker label.group-label { font-size: 12px; color: #5f6368; }
    .choice-option { display: flex; align-items: center; gap: 8px; font-size: 14px; color: #3c4043; cursor: pointer; }
    .choice-option input { accent-color: #1a73e8; }

    /* DateTimeInput */
    .datetime-wrapper { display: flex; flex-direction: column; gap: 4px; }
    .datetime-wrapper label { font-size: 12px; color: #5f6368; }
    .datetime-wrapper input {
      padding: 8px 12px; border: 1px solid #dadce0; border-radius: 8px;
      font-size: 14px; font-family: inherit;
    }

    /* Modal */
    .modal-overlay {
      position: fixed; top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,.4); display: flex; align-items: center; justify-content: center;
      z-index: 1000;
    }
    .modal-content {
      background: #fff; border-radius: 12px; padding: 24px;
      max-width: 480px; width: 90%; box-shadow: 0 4px 24px rgba(0,0,0,.2);
    }

    /* List */
    .list-vertical { display: flex; flex-direction: column; gap: 4px; }
    .list-horizontal { display: flex; flex-direction: row; gap: 8px; flex-wrap: wrap; }

    /* AudioPlayer */
    .audio-wrapper { display: flex; flex-direction: column; gap: 4px; }
    .audio-wrapper .audio-desc { font-size: 12px; color: #5f6368; }
    .audio-wrapper audio { width: 100%; }

    /* Video */
    .a2ui-video { width: 100%; max-height: 360px; border-radius: 8px; }
  `;_e([Ke({type:Object})],q.prototype,"surface",2);q=_e([qe("a2ui-surface-v09")],q);let $=null,C=null,B=null,v=null,x=[];const b=t=>document.getElementById(t);function fe(t){const e=b("status"),n=b("status-text");e.className=t?"connected":"",n.textContent=t?"Connected":"Disconnected",b("chat-input").disabled=!t,b("send-btn").disabled=!t,b("connect-btn").textContent=t?"Disconnect":"Connect"}function K(){const t=b("messages");t.scrollTop=t.scrollHeight}function A(t,e,n){const s=b("messages"),i=document.createElement("div");if(i.className=`msg ${t}`,i.textContent=e,n!=null&&t==="assistant"){const r=document.createElement("span");r.className="elapsed",r.textContent=`${n.toFixed(1)}s`,i.appendChild(r)}s.appendChild(i),K()}function xe(){H(),C=performance.now();const t=b("messages");v=document.createElement("div"),v.className="thinking",v.innerHTML=`
    <div class="spinner"></div>
    <span>Thinking...</span>
    <span class="timer">0.0s</span>
  `,t.appendChild(v),K(),B=window.setInterval(()=>{if(!v||!C)return;const e=((performance.now()-C)/1e3).toFixed(1),n=v.querySelector(".timer");n&&(n.textContent=`${e}s`)},100)}function H(){B&&(clearInterval(B),B=null),v&&(v.remove(),v=null)}function ve(){return C?(performance.now()-C)/1e3:null}function Ye(t){const e=b("messages"),n=document.createElement("div");n.className="a2ui-surface-container";const s=Xe(x);if(s){const i=document.createElement("a2ui-surface-v09");i.surface=s,i.addEventListener("a2ui-action",r=>{nt(r.detail,s.surfaceId)}),n.appendChild(i)}if(x.length>0){const i=document.createElement("div");i.className="inspector-wrap";const r=JSON.stringify(x,null,2),o=document.createElement("button");o.className="copy-btn",o.textContent="📋 Copy",o.addEventListener("click",()=>{(()=>{if(navigator.clipboard?.writeText)return navigator.clipboard.writeText(r);const d=document.createElement("textarea");d.value=r,d.style.cssText="position:fixed;left:-9999px;top:0",document.body.appendChild(d),d.select();const g=document.execCommand("copy");return document.body.removeChild(d),g?Promise.resolve():Promise.reject()})().then(()=>{o.textContent="✅ Copied!",setTimeout(()=>{o.textContent="📋 Copy"},1500)}).catch(()=>{o.textContent="❌ Failed",setTimeout(()=>{o.textContent="📋 Copy"},1500)})}),i.appendChild(o);const a=document.createElement("details");a.className="inspector";const l=document.createElement("summary");l.textContent=`Raw A2UI JSON (${x.length} messages)`;const c=document.createElement("pre");c.textContent=r,a.appendChild(l),a.appendChild(c),i.appendChild(a),n.appendChild(i)}if(t!=null){const i=document.createElement("div");i.style.cssText="text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px",i.textContent=`${t.toFixed(1)}s`,n.appendChild(i)}e.appendChild(n),K()}function et(t){try{const e=new URL(t);return(e.hostname==="127.0.0.1"||e.hostname==="localhost")&&(e.hostname=location.hostname),e.toString()}catch{return t}}function tt(t){const e=ve();H();const n=b("messages"),s=document.createElement("div");s.className="a2web-container";const i=et(t.url||""),r=document.createElement("div");r.className="a2web-header",r.innerHTML=`
    <span class="a2web-label">a2web</span>
    <span class="a2web-title">${t.title||"Web Page"}</span>
    <a href="${i}" target="_blank" rel="noopener" class="a2web-open">새 탭에서 열기 ↗</a>
  `,s.appendChild(r);const o=document.createElement("iframe");if(o.src=i,o.className="a2web-iframe",o.setAttribute("sandbox","allow-scripts allow-same-origin allow-forms allow-popups"),s.appendChild(o),e!=null){const a=document.createElement("div");a.style.cssText="text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px",a.textContent=`${e.toFixed(1)}s`,s.appendChild(a)}n.appendChild(s),K(),C=null}function nt(t,e){console.log("A2UI action:",t),$&&$.readyState===WebSocket.OPEN&&(x=[],$.send(JSON.stringify({type:"a2ui_action",payload:{surfaceId:e,name:t?.name||"unknown",sourceComponentId:t?.sourceComponentId||"unknown",context:t?.context||{}}})),xe())}function st(t){switch(console.log("[WS]",t.type,t),t.type){case"history":t.messages?.length&&A("system",`History: ${t.messages.length} messages`);break;case"a2ui":t.messages&&(console.log("[A2UI] received",t.messages.length,"messages"),x=t.messages);break;case"a2web":tt(t);break;case"done":{const e=ve();H(),console.log("[DONE] a2ui msgs:",x.length,"full_response:",!!t.full_response),x.length>0&&(Ye(e),x=[]),t.full_response&&A("assistant",t.full_response,x.length>0?null:e),C=null;break}case"chunk":break;case"error":H(),A("system",`Error: ${t.message}`),C=null;break;default:console.log("Unknown WS message:",t)}}window.toggleConnection=function(){if($&&$.readyState===WebSocket.OPEN){$.close();return}const t=b("ws-url").value.trim();t&&(A("system",`Connecting to ${t}...`),$=new WebSocket(t),$.onopen=()=>{fe(!0),A("system","Connected")},$.onclose=()=>{fe(!1),H(),A("system","Disconnected")},$.onerror=()=>{A("system","Connection error")},$.onmessage=e=>{try{st(JSON.parse(e.data))}catch(n){console.error("Parse error:",n)}})};window.sendMessage=function(){const t=b("chat-input"),e=t.value.trim();!e||!$||$.readyState!==WebSocket.OPEN||(A("user",e),x=[],xe(),$.send(JSON.stringify({type:"message",content:e})),t.value="",t.focus())};window.addEventListener("load",()=>{b("chat-input").focus();const t=b("ws-url"),e=location.protocol==="https:"?"wss:":"ws:",n=location.hostname+":42617";t.value=`${e}//${n}/ws/chat`,window.toggleConnection()});
