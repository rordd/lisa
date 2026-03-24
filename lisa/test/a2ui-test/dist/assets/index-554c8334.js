(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const r of i)if(r.type==="childList")for(const o of r.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&s(o)}).observe(document,{childList:!0,subtree:!0});function n(i){const r={};return i.integrity&&(r.integrity=i.integrity),i.referrerPolicy&&(r.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?r.credentials="include":i.crossOrigin==="anonymous"?r.credentials="omit":r.credentials="same-origin",r}function s(i){if(i.ep)return;i.ep=!0;const r=n(i);fetch(i.href,r)}})();/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const B=globalThis,Q=B.ShadowRoot&&(B.ShadyCSS===void 0||B.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,X=Symbol(),ae=new WeakMap;let ye=class{constructor(e,n,s){if(this._$cssResult$=!0,s!==X)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=n}get styleSheet(){let e=this.o;const n=this.t;if(Q&&e===void 0){const s=n!==void 0&&n.length===1;s&&(e=ae.get(n)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&ae.set(n,e))}return e}toString(){return this.cssText}};const ke=t=>new ye(typeof t=="string"?t:t+"",void 0,X),Pe=(t,...e)=>{const n=t.length===1?t[0]:e.reduce((s,i,r)=>s+(o=>{if(o._$cssResult$===!0)return o.cssText;if(typeof o=="number")return o;throw Error("Value passed to 'css' function must be a 'css' function result: "+o+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(i)+t[r+1],t[0]);return new ye(n,t,X)},Ie=(t,e)=>{if(Q)t.adoptedStyleSheets=e.map(n=>n instanceof CSSStyleSheet?n:n.styleSheet);else for(const n of e){const s=document.createElement("style"),i=B.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=n.cssText,t.appendChild(s)}},le=Q?t=>t:t=>t instanceof CSSStyleSheet?(e=>{let n="";for(const s of e.cssRules)n+=s.cssText;return ke(n)})(t):t;/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const{is:Te,defineProperty:Oe,getOwnPropertyDescriptor:Ne,getOwnPropertyNames:Ue,getOwnPropertySymbols:je,getPrototypeOf:ze}=Object,G=globalThis,ce=G.trustedTypes,He=ce?ce.emptyScript:"",Le=G.reactiveElementPolyfillSupport,N=(t,e)=>t,W={toAttribute(t,e){switch(e){case Boolean:t=t?He:null;break;case Object:case Array:t=t==null?t:JSON.stringify(t)}return t},fromAttribute(t,e){let n=t;switch(e){case Boolean:n=t!==null;break;case Number:n=t===null?null:Number(t);break;case Object:case Array:try{n=JSON.parse(t)}catch{n=null}}return n}},ee=(t,e)=>!Te(t,e),de={attribute:!0,type:String,converter:W,reflect:!1,useDefault:!1,hasChanged:ee};Symbol.metadata??=Symbol("metadata"),G.litPropertyMetadata??=new WeakMap;let P=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,n=de){if(n.state&&(n.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((n=Object.create(n)).wrapped=!0),this.elementProperties.set(e,n),!n.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,n);i!==void 0&&Oe(this.prototype,e,i)}}static getPropertyDescriptor(e,n,s){const{get:i,set:r}=Ne(this.prototype,e)??{get(){return this[n]},set(o){this[n]=o}};return{get:i,set(o){const a=i?.call(this);r?.call(this,o),this.requestUpdate(e,a,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??de}static _$Ei(){if(this.hasOwnProperty(N("elementProperties")))return;const e=ze(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(N("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(N("properties"))){const n=this.properties,s=[...Ue(n),...je(n)];for(const i of s)this.createProperty(i,n[i])}const e=this[Symbol.metadata];if(e!==null){const n=litPropertyMetadata.get(e);if(n!==void 0)for(const[s,i]of n)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[n,s]of this.elementProperties){const i=this._$Eu(n,s);i!==void 0&&this._$Eh.set(i,n)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const n=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)n.unshift(le(i))}else e!==void 0&&n.push(le(e));return n}static _$Eu(e,n){const s=n.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,n=this.constructor.elementProperties;for(const s of n.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Ie(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,n,s){this._$AK(e,s)}_$ET(e,n){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const r=(s.converter?.toAttribute!==void 0?s.converter:W).toAttribute(n,s.type);this._$Em=e,r==null?this.removeAttribute(i):this.setAttribute(i,r),this._$Em=null}}_$AK(e,n){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const r=s.getPropertyOptions(i),o=typeof r.converter=="function"?{fromAttribute:r.converter}:r.converter?.fromAttribute!==void 0?r.converter:W;this._$Em=i;const a=o.fromAttribute(n,r.type);this[i]=a??this._$Ej?.get(i)??a,this._$Em=null}}requestUpdate(e,n,s,i=!1,r){if(e!==void 0){const o=this.constructor;if(i===!1&&(r=this[e]),s??=o.getPropertyOptions(e),!((s.hasChanged??ee)(r,n)||s.useDefault&&s.reflect&&r===this._$Ej?.get(e)&&!this.hasAttribute(o._$Eu(e,s))))return;this.C(e,n,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,n,{useDefault:s,reflect:i,wrapped:r},o){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,o??n??this[e]),r!==!0||o!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(n=void 0),this._$AL.set(e,n)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(n){Promise.reject(n)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,r]of this._$Ep)this[i]=r;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,r]of s){const{wrapped:o}=r,a=this[i];o!==!0||this._$AL.has(i)||a===void 0||this.C(i,void 0,r,a)}}let e=!1;const n=this._$AL;try{e=this.shouldUpdate(n),e?(this.willUpdate(n),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(n)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(n)}willUpdate(e){}_$AE(e){this._$EO?.forEach(n=>n.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(n=>this._$ET(n,this[n])),this._$EM()}updated(e){}firstUpdated(e){}};P.elementStyles=[],P.shadowRootOptions={mode:"open"},P[N("elementProperties")]=new Map,P[N("finalized")]=new Map,Le?.({ReactiveElement:P}),(G.reactiveElementVersions??=[]).push("2.1.2");/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const te=globalThis,ue=t=>t,F=te.trustedTypes,pe=F?F.createPolicy("lit-html",{createHTML:t=>t}):void 0,_e="$lit$",w=`lit$${Math.random().toFixed(9).slice(2)}$`,xe="?"+w,Re=`<${xe}>`,k=document,j=()=>k.createComment(""),z=t=>t===null||typeof t!="object"&&typeof t!="function",ne=Array.isArray,De=t=>ne(t)||typeof t?.[Symbol.iterator]=="function",Y=`[ 	
\f\r]`,O=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,he=/-->/g,fe=/>/g,E=RegExp(`>|${Y}(?:([^\\s"'>=/]+)(${Y}*=${Y}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),me=/'/g,ge=/"/g,ve=/^(?:script|style|textarea|title)$/i,Be=t=>(e,...n)=>({_$litType$:t,strings:e,values:n}),p=Be(1),I=Symbol.for("lit-noChange"),u=Symbol.for("lit-nothing"),$e=new WeakMap,M=k.createTreeWalker(k,129);function we(t,e){if(!ne(t)||!t.hasOwnProperty("raw"))throw Error("invalid template strings array");return pe!==void 0?pe.createHTML(e):e}const Ve=(t,e)=>{const n=t.length-1,s=[];let i,r=e===2?"<svg>":e===3?"<math>":"",o=O;for(let a=0;a<n;a++){const l=t[a];let d,h,c=-1,m=0;for(;m<l.length&&(o.lastIndex=m,h=o.exec(l),h!==null);)m=o.lastIndex,o===O?h[1]==="!--"?o=he:h[1]!==void 0?o=fe:h[2]!==void 0?(ve.test(h[2])&&(i=RegExp("</"+h[2],"g")),o=E):h[3]!==void 0&&(o=E):o===E?h[0]===">"?(o=i??O,c=-1):h[1]===void 0?c=-2:(c=o.lastIndex-h[2].length,d=h[1],o=h[3]===void 0?E:h[3]==='"'?ge:me):o===ge||o===me?o=E:o===he||o===fe?o=O:(o=E,i=void 0);const y=o===E&&t[a+1].startsWith("/>")?" ":"";r+=o===O?l+Re:c>=0?(s.push(d),l.slice(0,c)+_e+l.slice(c)+w+y):l+w+(c===-2?a:y)}return[we(t,r+(t[n]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class H{constructor({strings:e,_$litType$:n},s){let i;this.parts=[];let r=0,o=0;const a=e.length-1,l=this.parts,[d,h]=Ve(e,n);if(this.el=H.createElement(d,s),M.currentNode=this.el.content,n===2||n===3){const c=this.el.content.firstChild;c.replaceWith(...c.childNodes)}for(;(i=M.nextNode())!==null&&l.length<a;){if(i.nodeType===1){if(i.hasAttributes())for(const c of i.getAttributeNames())if(c.endsWith(_e)){const m=h[o++],y=i.getAttribute(c).split(w),v=/([.?@])?(.*)/.exec(m);l.push({type:1,index:r,name:v[2],strings:y,ctor:v[1]==="."?Fe:v[1]==="?"?qe:v[1]==="@"?Je:K}),i.removeAttribute(c)}else c.startsWith(w)&&(l.push({type:6,index:r}),i.removeAttribute(c));if(ve.test(i.tagName)){const c=i.textContent.split(w),m=c.length-1;if(m>0){i.textContent=F?F.emptyScript:"";for(let y=0;y<m;y++)i.append(c[y],j()),M.nextNode(),l.push({type:2,index:++r});i.append(c[m],j())}}}else if(i.nodeType===8)if(i.data===xe)l.push({type:2,index:r});else{let c=-1;for(;(c=i.data.indexOf(w,c+1))!==-1;)l.push({type:7,index:r}),c+=w.length-1}r++}}static createElement(e,n){const s=k.createElement("template");return s.innerHTML=e,s}}function T(t,e,n=t,s){if(e===I)return e;let i=s!==void 0?n._$Co?.[s]:n._$Cl;const r=z(e)?void 0:e._$litDirective$;return i?.constructor!==r&&(i?._$AO?.(!1),r===void 0?i=void 0:(i=new r(t),i._$AT(t,n,s)),s!==void 0?(n._$Co??=[])[s]=i:n._$Cl=i),i!==void 0&&(e=T(t,i._$AS(t,e.values),i,s)),e}class We{constructor(e,n){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=n}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:n},parts:s}=this._$AD,i=(e?.creationScope??k).importNode(n,!0);M.currentNode=i;let r=M.nextNode(),o=0,a=0,l=s[0];for(;l!==void 0;){if(o===l.index){let d;l.type===2?d=new R(r,r.nextSibling,this,e):l.type===1?d=new l.ctor(r,l.name,l.strings,this,e):l.type===6&&(d=new Ge(r,this,e)),this._$AV.push(d),l=s[++a]}o!==l?.index&&(r=M.nextNode(),o++)}return M.currentNode=k,i}p(e){let n=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,n),n+=s.strings.length-2):s._$AI(e[n])),n++}}class R{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,n,s,i){this.type=2,this._$AH=u,this._$AN=void 0,this._$AA=e,this._$AB=n,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const n=this._$AM;return n!==void 0&&e?.nodeType===11&&(e=n.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,n=this){e=T(this,e,n),z(e)?e===u||e==null||e===""?(this._$AH!==u&&this._$AR(),this._$AH=u):e!==this._$AH&&e!==I&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):De(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==u&&z(this._$AH)?this._$AA.nextSibling.data=e:this.T(k.createTextNode(e)),this._$AH=e}$(e){const{values:n,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=H.createElement(we(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(n);else{const r=new We(i,this),o=r.u(this.options);r.p(n),this.T(o),this._$AH=r}}_$AC(e){let n=$e.get(e.strings);return n===void 0&&$e.set(e.strings,n=new H(e)),n}k(e){ne(this._$AH)||(this._$AH=[],this._$AR());const n=this._$AH;let s,i=0;for(const r of e)i===n.length?n.push(s=new R(this.O(j()),this.O(j()),this,this.options)):s=n[i],s._$AI(r),i++;i<n.length&&(this._$AR(s&&s._$AB.nextSibling,i),n.length=i)}_$AR(e=this._$AA.nextSibling,n){for(this._$AP?.(!1,!0,n);e!==this._$AB;){const s=ue(e).nextSibling;ue(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class K{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,n,s,i,r){this.type=1,this._$AH=u,this._$AN=void 0,this.element=e,this.name=n,this._$AM=i,this.options=r,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=u}_$AI(e,n=this,s,i){const r=this.strings;let o=!1;if(r===void 0)e=T(this,e,n,0),o=!z(e)||e!==this._$AH&&e!==I,o&&(this._$AH=e);else{const a=e;let l,d;for(e=r[0],l=0;l<r.length-1;l++)d=T(this,a[s+l],n,l),d===I&&(d=this._$AH[l]),o||=!z(d)||d!==this._$AH[l],d===u?e=u:e!==u&&(e+=(d??"")+r[l+1]),this._$AH[l]=d}o&&!i&&this.j(e)}j(e){e===u?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Fe extends K{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===u?void 0:e}}class qe extends K{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==u)}}class Je extends K{constructor(e,n,s,i,r){super(e,n,s,i,r),this.type=5}_$AI(e,n=this){if((e=T(this,e,n,0)??u)===I)return;const s=this._$AH,i=e===u&&s!==u||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,r=e!==u&&(s===u||i);i&&this.element.removeEventListener(this.name,this,s),r&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Ge{constructor(e,n,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=n,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){T(this,e)}}const Ke=te.litHtmlPolyfillSupport;Ke?.(H,R),(te.litHtmlVersions??=[]).push("3.3.2");const Ze=(t,e,n)=>{const s=n?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const r=n?.renderBefore??null;s._$litPart$=i=new R(e.insertBefore(j(),r),r,void 0,n??{})}return i._$AI(t),i};/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const se=globalThis;class U extends P{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const n=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=Ze(n,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return I}}U._$litElement$=!0,U.finalized=!0,se.litElementHydrateSupport?.({LitElement:U});const Ye=se.litElementPolyfillSupport;Ye?.({LitElement:U});(se.litElementVersions??=[]).push("4.2.2");/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const Qe=t=>(e,n)=>{n!==void 0?n.addInitializer(()=>{customElements.define(t,e)}):customElements.define(t,e)};/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const Xe={attribute:!0,type:String,converter:W,reflect:!1,hasChanged:ee},et=(t=Xe,e,n)=>{const{kind:s,metadata:i}=n;let r=globalThis.litPropertyMetadata.get(i);if(r===void 0&&globalThis.litPropertyMetadata.set(i,r=new Map),s==="setter"&&((t=Object.create(t)).wrapped=!0),r.set(n.name,t),s==="accessor"){const{name:o}=n;return{set(a){const l=e.get.call(this);e.set.call(this,a),this.requestUpdate(o,l,t,!0,a)},init(a){return a!==void 0&&this.C(o,void 0,t,a),a}}}if(s==="setter"){const{name:o}=n;return function(a){const l=this[o];e.call(this,a),this.requestUpdate(o,l,t,!0,a)}}throw Error("Unsupported decorator location: "+s)};function tt(t){return(e,n)=>typeof n=="object"?et(t,e,n):((s,i,r)=>{const o=i.hasOwnProperty(r);return i.constructor.createProperty(r,s),o?Object.getOwnPropertyDescriptor(i,r):void 0})(t,e,n)}var nt=Object.defineProperty,st=Object.getOwnPropertyDescriptor,Ae=(t,e,n,s)=>{for(var i=s>1?void 0:s?st(e,n):e,r=t.length-1,o;r>=0;r--)(o=t[r])&&(i=(s?o(e,n,i):o(i))||i);return s&&i&&nt(e,n,i),i};function it(t){let e="@default";const n=new Map;let s={},i="root",r=!1;for(const o of t){const a=o.createSurface||o.beginRendering;a&&(e=a.surfaceId||"@default",a.root&&(i=a.root),a.sendDataModel&&(r=!0));const l=o.updateComponents||o.surfaceUpdate;if(l)for(const h of l.components){const c={id:h.id};if(h.component&&typeof h.component=="object"){const m=Object.keys(h.component);if(m.length===1){const y=m[0];c.component=y;const v=h.component[y];if(v&&typeof v=="object")for(const[S,f]of Object.entries(v))S==="children"&&f&&typeof f=="object"&&f.explicitList?c.children=f.explicitList:S==="text"&&f&&typeof f=="object"&&f.literalString!=null?c.text=f.literalString:S==="label"&&f&&typeof f=="object"&&f.literalString!=null?c.label=f.literalString:S==="name"&&f&&typeof f=="object"&&f.literalString!=null?c.name=f.literalString:S==="description"&&f&&typeof f=="object"&&f.literalString!=null?c.description=f.literalString:S==="url"&&f&&typeof f=="object"&&f.literalString!=null?c.url=f.literalString:c[S]=f}}else Object.assign(c,h);n.set(c.id,c),c.id==="root"&&(i="root")}const d=o.updateDataModel||o.dataModelUpdate;d&&(!d.path||d.path==="/")&&(s={...s,...d.value})}return n.size===0?null:{surfaceId:e,components:n,dataModel:s,rootId:i,sendDataModel:r}}function q(t,e){const n=t.replace(/^\//,"").split("/");let s=e;for(const i of n){if(s==null||typeof s!="object")return;let r=i;if(/^\{.+\}$/.test(i)){const o=i.slice(1,-1),a=e["_"+o]??e[o];a!=null&&(r=String(a))}s=s[r]}return s}function $(t,e){if(t==null)return"";if(typeof t=="string")return t.includes("${/")?t.replace(/\$\{(\/[^}]+)\}/g,(n,s)=>{const i=q(s,e);return i!=null?String(i):""}):t;if(t.path){const n=q(t.path,e);return n!=null?String(n):""}return""}function D(t,e){if(t!=null){if(typeof t=="object"&&t!==null&&"path"in t)return q(t.path,e);if(typeof t=="string"&&t.includes("${/")){const n=t.match(/^\$\{(\/[^}]+)\}$/);if(n)return q(n[1],e)}return t}}let J=class extends U{constructor(){super(...arguments),this.surface=null,this._inputValues=new Map}updated(t){t.has("surface")&&this._inputValues.clear()}_fireAction(t,e,n){const s={},i=this.surface?.dataModel??{};for(const[o,a]of Object.entries(n))s[o]=D(a,i);const r={...s};for(const[o,a]of this._inputValues)o in r||(r[o]=a);if(this._inputValues.size>0){const o={};for(const[a,l]of this._inputValues)o[a]=l;r._formData=o}this.dispatchEvent(new CustomEvent("a2ui-action",{bubbles:!0,composed:!0,detail:{name:t,sourceComponentId:e,context:r}}))}render(){return this.surface?this._renderComponent(this.surface.rootId):u}_renderComponent(t){const n=this.surface.components.get(t);if(!n)return u;switch(n.component||n.type||""){case"Card":return this._renderCard(n);case"Column":return this._renderColumn(n);case"Row":return this._renderRow(n);case"Text":return this._renderText(n);case"Button":return this._renderButton(n);case"Divider":return p`<hr class="divider ${n.axis==="vertical"?"vertical":""}" />`;case"CheckBox":return this._renderCheckBox(n);case"Slider":return this._renderSlider(n);case"TextField":return this._renderTextField(n);case"Image":return this._renderImage(n);case"Icon":return this._renderIcon(n);case"Tabs":return this._renderTabs(n);case"List":return this._renderList(n);case"Grid":return this._renderGrid(n);case"Modal":return this._renderModal(n);case"ChoicePicker":case"MultipleChoice":return this._renderChoicePicker(n);case"DateTimeInput":return this._renderDateTimeInput(n);case"Video":return this._renderVideo(n);case"AudioPlayer":return this._renderAudioPlayer(n);default:return n.children?p`<div>${n.children.map(i=>this._renderComponent(i))}</div>`:n.child?this._renderComponent(n.child):u}}_renderCard(t){return p`
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
    `}_renderText(t){const e=$(t.text,this.surface.dataModel),n=t.variant||"body";return p`<span class="text-${n}">${e}</span>`}_renderButton(t){let e="";if(t.child){const o=this.surface.components.get(t.child);o&&(e=$(o.text,this.surface.dataModel))}e||(e=t.label||t.text||t.id);const n=t.variant||"",i=t.primary===!0||n==="filled"?"primary":n||"";return p`
      <button class="btn ${i}" @click=${()=>{const o=t.action?.functionCall;if(o){this._handleFunctionCall(o);return}const a=t.action?.event;if(a){const l=this._extractUrlFromEvent(a);if(l){window.open(l,"_blank","noopener");return}this._fireAction(a.name||"unknown",t.id,a.context||{})}}}>
        ${e}
      </button>
    `}_handleFunctionCall(t){switch(t.call){case"openUrl":{const e=t.args?.url;e&&window.open(e,"_blank","noopener");break}default:console.warn(`[A2UI] Unhandled client functionCall: ${t.call}`,t.args)}}_extractUrlFromEvent(t){const e=t.context;if(!e)return null;for(const n of Object.values(e))if(typeof n=="string"&&/^https?:\/\/.+/i.test(n))return n;return null}_renderCheckBox(t){const e=$(t.label,this.surface.dataModel),n=D(t.value,this.surface.dataModel);return p`
      <label class="checkbox-wrapper">
        <input type="checkbox" .checked=${!!n} @change=${i=>{const r=i.target;this._inputValues.set(t.id,r.checked)}} />
        ${e}
      </label>
    `}_renderSlider(t){const e=$(t.label,this.surface.dataModel),n=D(t.value,this.surface.dataModel),s=t.min??0,i=t.max??100,r=o=>{const a=o.target;this._inputValues.set(t.id,Number(a.value))};return p`
      <div class="slider-wrapper">
        ${e?p`<label>${e}</label>`:u}
        <input type="range" min=${s} max=${i} .value=${String(n??s)} @input=${r} />
        <span class="slider-value">${n??s} / ${i}</span>
      </div>
    `}_renderTextField(t){const e=$(t.label,this.surface.dataModel),n=$(t.text??t.value,this.surface.dataModel),s=t.textFieldType||"shortText",i=o=>{const a=o.target;this._inputValues.set(t.id,a.value)};if(s==="longText"||s==="multiline")return p`
        <div class="textfield-wrapper">
          ${e?p`<label>${e}</label>`:u}
          <textarea rows="3" .value=${n} placeholder=${e} @input=${i}></textarea>
        </div>
      `;const r=s==="obscured"||s==="password"?"password":s==="number"?"number":s==="date"?"date":s==="email"?"email":"text";return p`
      <div class="textfield-wrapper">
        ${e?p`<label>${e}</label>`:u}
        <input type=${r} .value=${n} placeholder=${e} @input=${i} />
      </div>
    `}_renderImage(t){const e=$(t.url,this.surface.dataModel),n=t.variant||t.usageHint||"",s=t.fit?`object-fit: ${t.fit}`:"";return p`<img class="a2ui-image ${n}" src=${e} style=${s} alt="" />`}_renderIcon(t){const n=$(t.name,this.surface.dataModel).replace(/([A-Z])/g,"_$1").toLowerCase().replace(/^_/,""),i={cloud:"☁️",sunny:"☀️",clear:"☀️",sun:"☀️",umbrella:"☂️",rain:"🌧️",rainy:"🌧️",snow:"❄️",thunderstorm:"⛈️",fog:"🌫️",wind:"💨",partly_cloudy:"⛅",partly_cloudy_day:"⛅",partly_cloudy_night:"⛅",check:"✅",close:"❌",star:"⭐",favorite:"❤️",home:"🏠",settings:"⚙️",search:"🔍",info:"ℹ️",warning:"⚠️",error:"❗",calendar:"📅",schedule:"📅",location:"📍",place:"📍",restaurant:"🍽️",music:"🎵",play:"▶️",pause:"⏸️",stop:"⏹️"}[n];return i?p`<span class="a2ui-icon-emoji">${i}</span>`:p`<span class="material-symbols-outlined a2ui-icon">${n}</span>`}_renderTabs(t){const e=t.tabItems||t.tabs||[];if(e.length===0)return u;const n=0;return p`
      <div>
        <div class="tabs-header">
          ${e.map((s,i)=>{const r=$(s.title||s.label,this.surface.dataModel);return p`<button class="tab-btn ${i===n?"active":""}">${r}</button>`})}
        </div>
        <div class="tab-content">
          ${this._renderComponent(e[n].child)}
        </div>
      </div>
    `}_renderList(t){const e=t.direction==="horizontal"?"horizontal":"vertical",n=t.children||[];if(!Array.isArray(n)&&typeof n=="object"){const s=n;if(s.componentId&&s.path){const i=this.surface.components.get(s.componentId),r=D({path:s.path},this.surface.dataModel);if(i&&Array.isArray(r))return p`
            <div class="list-${e}">
              ${r.map((o,a)=>this._renderTemplateInstance(i,o,a))}
            </div>
          `}return u}return Array.isArray(n)?p`
        <div class="list-${e}">
          ${n.map(s=>this._renderComponent(s))}
        </div>
      `:u}_renderTemplateInstance(t,e,n){const s=this.surface,i=s.dataModel;s.dataModel={...i,...e,current:e,_index:n};try{return this._renderComponent(t.id)}finally{s.dataModel=i}}_renderGrid(t){const e=t.columns||2,n=Array.isArray(t.children)?t.children:[];return p`
      <div class="grid" style="grid-template-columns: repeat(${e}, 1fr)">
        ${n.map(s=>this._renderComponent(s))}
      </div>
    `}_renderModal(t){const e=t.entryPointChild||t.trigger||"";return t.contentChild||t.content,p`
      <div>
        ${e?this._renderComponent(e):u}
      </div>
    `}_renderChoicePicker(t){const e=$(t.label,this.surface.dataModel),n=t.options||[],s=t.variant||"radio",i=s==="multipleSelection"||s==="chip"||t.component==="MultipleChoice",r=i?"checkbox":"radio",o=`choice-${t.id}`,a=l=>{const d=l.target;if(i){const h=this._inputValues.get(t.id)||[];d.checked?this._inputValues.set(t.id,[...h,d.value]):this._inputValues.set(t.id,h.filter(c=>c!==d.value))}else this._inputValues.set(t.id,d.value)};return p`
      <div class="choice-picker">
        ${e?p`<label class="group-label">${e}</label>`:u}
        ${n.map(l=>{const d=$(l.label,this.surface.dataModel);return p`
            <label class="choice-option">
              <input type=${r} name=${o} value=${l.value} @change=${a} />
              ${d}
            </label>
          `})}
      </div>
    `}_renderDateTimeInput(t){const e=$(t.label,this.surface.dataModel),n=$(t.value,this.surface.dataModel),s=t.enableDate!==!1,i=t.enableTime===!0,r=s&&i?"datetime-local":i?"time":"date";return p`
      <div class="datetime-wrapper">
        ${e?p`<label>${e}</label>`:u}
        <input type=${r} .value=${n} />
      </div>
    `}_renderVideo(t){const e=$(t.url,this.surface.dataModel);return p`<video class="a2ui-video" src=${e} controls></video>`}_renderAudioPlayer(t){const e=$(t.url,this.surface.dataModel),n=$(t.description,this.surface.dataModel);return p`
      <div class="audio-wrapper">
        ${n?p`<span class="audio-desc">${n}</span>`:u}
        <audio src=${e} controls></audio>
      </div>
    `}_getConsumedChildIds(){const t=new Set;for(const e of this.surface.components.values())(e.component||e.type||"")==="Button"&&e.child&&t.add(e.child);return t}};J.styles=Pe`
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

    /* Grid */
    .grid { display: grid; gap: 8px; }

    /* AudioPlayer */
    .audio-wrapper { display: flex; flex-direction: column; gap: 4px; }
    .audio-wrapper .audio-desc { font-size: 12px; color: #5f6368; }
    .audio-wrapper audio { width: 100%; }

    /* Video */
    .a2ui-video { width: 100%; max-height: 360px; border-radius: 8px; }
  `;Ae([tt({type:Object})],J.prototype,"surface",2);J=Ae([Qe("a2ui-surface-v09")],J);let g=null,C=null,V=null,x=null;const Ce="lisa-sessions";function ie(){const t=localStorage.getItem(Ce);return t?JSON.parse(t):["lisa-test"]}function Se(t){localStorage.setItem(Ce,JSON.stringify(t))}function re(){const t=document.getElementById("session-select"),e=ie(),n=t.value||e[0];t.innerHTML=e.map(s=>`<option value="${s}"${s===n?" selected":""}>${s}</option>`).join("")}function oe(){return document.getElementById("session-select")?.value||"lisa-test"}window.newSession=function(){const t=prompt("세션 이름:");if(!t||!t.trim())return;const e=ie();e.includes(t.trim())||(e.push(t.trim()),Se(e)),re(),document.getElementById("session-select").value=t.trim()};window.deleteSession=function(){const t=oe();if(t==="lisa-test"){alert("기본 세션은 삭제할 수 없습니다");return}if(!confirm(`"${t}" 세션을 삭제하시겠습니까?`))return;const e=ie().filter(n=>n!==t);Se(e),re()};window.toggleMenu=function(){const t=document.getElementById("session-menu");t.style.display=t.style.display==="none"?"block":"none"};window.switchSession=function(){document.getElementById("session-menu").style.display="none";const t=document.getElementById("current-session-label");t&&(t.textContent=oe()),g&&g.readyState===WebSocket.OPEN&&g.close(),setTimeout(()=>window.toggleConnection(),300)};document.addEventListener("click",t=>{const e=document.getElementById("session-menu"),n=document.getElementById("menu-btn");e&&n&&!e.contains(t.target)&&!n.contains(t.target)&&(e.style.display="none")});let _=[];const b=t=>document.getElementById(t);function be(t){const e=b("status"),n=b("status-text");e.className=t?"connected":"",n.textContent=t?"Connected":"Disconnected",b("chat-input").disabled=!t,b("send-btn").disabled=!t,b("connect-btn").textContent=t?"Disconnect":"Connect"}function Z(){const t=b("messages");t.scrollTop=t.scrollHeight}function A(t,e,n){const s=b("messages"),i=document.createElement("div");if(i.className=`msg ${t}`,i.textContent=e,n!=null&&t==="assistant"){const r=document.createElement("span");r.className="elapsed",r.textContent=`${n.toFixed(1)}s`,i.appendChild(r)}s.appendChild(i),Z()}function Ee(){L(),C=performance.now();const t=b("messages");x=document.createElement("div"),x.className="thinking",x.innerHTML=`
    <div class="spinner"></div>
    <span>Thinking...</span>
    <span class="timer">0.0s</span>
  `,t.appendChild(x),Z(),V=window.setInterval(()=>{if(!x||!C)return;const e=((performance.now()-C)/1e3).toFixed(1),n=x.querySelector(".timer");n&&(n.textContent=`${e}s`)},100)}function L(){V&&(clearInterval(V),V=null),x&&(x.remove(),x=null)}function Me(){return C?(performance.now()-C)/1e3:null}function rt(t){const e=b("messages"),n=document.createElement("div");n.className="a2ui-surface-container";const s=it(_);if(s){const r=document.createElement("a2ui-surface-v09");r.surface=s,r.addEventListener("a2ui-action",o=>{lt(o.detail,s)}),n.appendChild(r)}const i=s?.surfaceId;if(i&&(n.dataset.surfaceId=i),_.length>0){const r=document.createElement("div");r.className="inspector-wrap";const o=JSON.stringify(_,null,2),a=document.createElement("button");a.className="copy-btn",a.textContent="📋 Copy",a.addEventListener("click",()=>{(()=>{if(navigator.clipboard?.writeText)return navigator.clipboard.writeText(o);const m=document.createElement("textarea");m.value=o,m.style.cssText="position:fixed;left:-9999px;top:0",document.body.appendChild(m),m.select();const y=document.execCommand("copy");return document.body.removeChild(m),y?Promise.resolve():Promise.reject()})().then(()=>{a.textContent="✅ Copied!",setTimeout(()=>{a.textContent="📋 Copy"},1500)}).catch(()=>{a.textContent="❌ Failed",setTimeout(()=>{a.textContent="📋 Copy"},1500)})}),r.appendChild(a);const l=document.createElement("details");l.className="inspector";const d=document.createElement("summary");d.textContent=`Raw A2UI JSON (${_.length} messages)`;const h=document.createElement("pre");h.textContent=o,l.appendChild(d),l.appendChild(h),r.appendChild(l),n.appendChild(r)}if(t!=null){const r=document.createElement("div");r.style.cssText="text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px",r.textContent=`${t.toFixed(1)}s`,n.appendChild(r)}e.appendChild(n),Z()}function ot(t){try{const e=new URL(t);return(e.hostname==="127.0.0.1"||e.hostname==="localhost")&&(e.hostname=location.hostname),e.toString()}catch{return t}}function at(t){const e=Me();L();const n=b("messages"),s=document.createElement("div");s.className="a2web-container";const i=ot(t.url||""),r=document.createElement("div");r.className="a2web-header",r.innerHTML=`
    <span class="a2web-label">a2web</span>
    <span class="a2web-title">${t.title||"Web Page"}</span>
    <a href="${i}" target="_blank" rel="noopener" class="a2web-open">새 탭에서 열기 ↗</a>
  `,s.appendChild(r);const o=document.createElement("iframe");if(o.src=i,o.className="a2web-iframe",o.setAttribute("sandbox","allow-scripts allow-same-origin allow-forms allow-popups"),s.appendChild(o),e!=null){const a=document.createElement("div");a.style.cssText="text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px",a.textContent=`${e.toFixed(1)}s`,s.appendChild(a)}n.appendChild(s),Z(),C=null}function lt(t,e){if(console.log("A2UI action:",t),g&&g.readyState===WebSocket.OPEN){_=[];const n={surfaceId:e.surfaceId,name:t?.name||"unknown",sourceComponentId:t?.sourceComponentId||"unknown",context:t?.context||{}};e.sendDataModel&&e.dataModel&&(n.dataModel=e.dataModel),g.send(JSON.stringify({type:"a2ui_action",payload:n})),Ee()}}function ct(t){switch(console.log("[WS]",t.type,t),t.type){case"history":t.messages?.length&&A("system",`History: ${t.messages.length} messages`);break;case"a2ui":if(t.messages){console.log("[A2UI] received",t.messages.length,"messages");for(const e of t.messages)if(e.deleteSurface?.surfaceId){const n=e.deleteSurface.surfaceId,s=document.querySelector(`[data-surface-id="${n}"]`);s&&s.remove(),console.log("[A2UI] deleteSurface:",n)}_=t.messages}break;case"a2web":at(t);break;case"done":{const e=Me();L(),console.log("[DONE] a2ui msgs:",_.length,"full_response:",!!t.full_response),_.length>0&&(rt(e),_=[]),t.full_response&&A("assistant",t.full_response,_.length>0?null:e),C=null;break}case"chunk":break;case"error":L(),A("system",`Error: ${t.message}`),C=null;break;default:console.log("Unknown WS message:",t)}}window.toggleConnection=function(){if(g&&g.readyState===WebSocket.OPEN){g.close();return}let t=b("ws-url").value.trim();if(!t)return;const e=t.includes("?")?"&":"?";t.includes("session_id=")||(t+=`${e}session_id=${encodeURIComponent(oe())}`),A("system",`Connecting to ${t}...`),g=new WebSocket(t),g.onopen=()=>{be(!0),A("system","Connected")},g.onclose=()=>{be(!1),L(),A("system","Disconnected")},g.onerror=()=>{A("system","Connection error")},g.onmessage=n=>{try{ct(JSON.parse(n.data))}catch(s){console.error("Parse error:",s)}}};window.sendMessage=function(){const t=b("chat-input"),e=t.value.trim();!e||!g||g.readyState!==WebSocket.OPEN||(A("user",e),_=[],Ee(),g.send(JSON.stringify({type:"message",content:e})),t.value="",t.focus())};window.addEventListener("load",()=>{re(),b("chat-input").focus();const t=b("ws-url"),e=location.protocol==="https:"?"wss:":"ws:",n=location.hostname+":42617";t.value=`${e}//${n}/app`,window.toggleConnection()});
