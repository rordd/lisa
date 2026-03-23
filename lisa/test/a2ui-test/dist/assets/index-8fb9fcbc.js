(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const r of i)if(r.type==="childList")for(const o of r.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&s(o)}).observe(document,{childList:!0,subtree:!0});function t(i){const r={};return i.integrity&&(r.integrity=i.integrity),i.referrerPolicy&&(r.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?r.credentials="include":i.crossOrigin==="anonymous"?r.credentials="omit":r.credentials="same-origin",r}function s(i){if(i.ep)return;i.ep=!0;const r=t(i);fetch(i.href,r)}})();/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const B=globalThis,X=B.ShadowRoot&&(B.ShadyCSS===void 0||B.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,Y=Symbol(),ie=new WeakMap;let ge=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==Y)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(X&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=ie.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&ie.set(t,e))}return e}toString(){return this.cssText}};const Ae=n=>new ge(typeof n=="string"?n:n+"",void 0,Y),Ce=(n,...e)=>{const t=n.length===1?n[0]:e.reduce((s,i,r)=>s+(o=>{if(o._$cssResult$===!0)return o.cssText;if(typeof o=="number")return o;throw Error("Value passed to 'css' function must be a 'css' function result: "+o+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(i)+n[r+1],n[0]);return new ge(t,n,Y)},Se=(n,e)=>{if(X)n.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),i=B.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=t.cssText,n.appendChild(s)}},re=X?n=>n:n=>n instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return Ae(t)})(n):n;/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const{is:Ee,defineProperty:Me,getOwnPropertyDescriptor:Pe,getOwnPropertyNames:ke,getOwnPropertySymbols:Te,getPrototypeOf:Ie}=Object,Z=globalThis,oe=Z.trustedTypes,Oe=oe?oe.emptyScript:"",Ne=Z.reactiveElementPolyfillSupport,N=(n,e)=>n,F={toAttribute(n,e){switch(e){case Boolean:n=n?Oe:null;break;case Object:case Array:n=n==null?n:JSON.stringify(n)}return n},fromAttribute(n,e){let t=n;switch(e){case Boolean:t=n!==null;break;case Number:t=n===null?null:Number(n);break;case Object:case Array:try{t=JSON.parse(n)}catch{t=null}}return t}},ee=(n,e)=>!Ee(n,e),ae={attribute:!0,type:String,converter:F,reflect:!1,useDefault:!1,hasChanged:ee};Symbol.metadata??=Symbol("metadata"),Z.litPropertyMetadata??=new WeakMap;let k=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=ae){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,t);i!==void 0&&Me(this.prototype,e,i)}}static getPropertyDescriptor(e,t,s){const{get:i,set:r}=Pe(this.prototype,e)??{get(){return this[t]},set(o){this[t]=o}};return{get:i,set(o){const a=i?.call(this);r?.call(this,o),this.requestUpdate(e,a,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??ae}static _$Ei(){if(this.hasOwnProperty(N("elementProperties")))return;const e=Ie(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(N("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(N("properties"))){const t=this.properties,s=[...ke(t),...Te(t)];for(const i of s)this.createProperty(i,t[i])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,i]of t)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const i=this._$Eu(t,s);i!==void 0&&this._$Eh.set(i,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)t.unshift(re(i))}else e!==void 0&&t.push(re(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Se(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const r=(s.converter?.toAttribute!==void 0?s.converter:F).toAttribute(t,s.type);this._$Em=e,r==null?this.removeAttribute(i):this.setAttribute(i,r),this._$Em=null}}_$AK(e,t){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const r=s.getPropertyOptions(i),o=typeof r.converter=="function"?{fromAttribute:r.converter}:r.converter?.fromAttribute!==void 0?r.converter:F;this._$Em=i;const a=o.fromAttribute(t,r.type);this[i]=a??this._$Ej?.get(i)??a,this._$Em=null}}requestUpdate(e,t,s,i=!1,r){if(e!==void 0){const o=this.constructor;if(i===!1&&(r=this[e]),s??=o.getPropertyOptions(e),!((s.hasChanged??ee)(r,t)||s.useDefault&&s.reflect&&r===this._$Ej?.get(e)&&!this.hasAttribute(o._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:i,wrapped:r},o){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,o??t??this[e]),r!==!0||o!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,r]of this._$Ep)this[i]=r;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,r]of s){const{wrapped:o}=r,a=this[i];o!==!0||this._$AL.has(i)||a===void 0||this.C(i,void 0,r,a)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};k.elementStyles=[],k.shadowRootOptions={mode:"open"},k[N("elementProperties")]=new Map,k[N("finalized")]=new Map,Ne?.({ReactiveElement:k}),(Z.reactiveElementVersions??=[]).push("2.1.2");/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const te=globalThis,le=n=>n,W=te.trustedTypes,ce=W?W.createPolicy("lit-html",{createHTML:n=>n}):void 0,$e="$lit$",w=`lit$${Math.random().toFixed(9).slice(2)}$`,be="?"+w,Ue=`<${be}>`,P=document,j=()=>P.createComment(""),z=n=>n===null||typeof n!="object"&&typeof n!="function",ne=Array.isArray,je=n=>ne(n)||typeof n?.[Symbol.iterator]=="function",Q=`[ 	
\f\r]`,O=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,de=/-->/g,ue=/>/g,E=RegExp(`>|${Q}(?:([^\\s"'>=/]+)(${Q}*=${Q}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),he=/'/g,pe=/"/g,ye=/^(?:script|style|textarea|title)$/i,ze=n=>(e,...t)=>({_$litType$:n,strings:e,values:t}),h=ze(1),T=Symbol.for("lit-noChange"),u=Symbol.for("lit-nothing"),fe=new WeakMap,M=P.createTreeWalker(P,129);function _e(n,e){if(!ne(n)||!n.hasOwnProperty("raw"))throw Error("invalid template strings array");return ce!==void 0?ce.createHTML(e):e}const He=(n,e)=>{const t=n.length-1,s=[];let i,r=e===2?"<svg>":e===3?"<math>":"",o=O;for(let a=0;a<t;a++){const l=n[a];let d,p,c=-1,m=0;for(;m<l.length&&(o.lastIndex=m,p=o.exec(l),p!==null);)m=o.lastIndex,o===O?p[1]==="!--"?o=de:p[1]!==void 0?o=ue:p[2]!==void 0?(ye.test(p[2])&&(i=RegExp("</"+p[2],"g")),o=E):p[3]!==void 0&&(o=E):o===E?p[0]===">"?(o=i??O,c=-1):p[1]===void 0?c=-2:(c=o.lastIndex-p[2].length,d=p[1],o=p[3]===void 0?E:p[3]==='"'?pe:he):o===pe||o===he?o=E:o===de||o===ue?o=O:(o=E,i=void 0);const y=o===E&&n[a+1].startsWith("/>")?" ":"";r+=o===O?l+Ue:c>=0?(s.push(d),l.slice(0,c)+$e+l.slice(c)+w+y):l+w+(c===-2?a:y)}return[_e(n,r+(n[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class H{constructor({strings:e,_$litType$:t},s){let i;this.parts=[];let r=0,o=0;const a=e.length-1,l=this.parts,[d,p]=He(e,t);if(this.el=H.createElement(d,s),M.currentNode=this.el.content,t===2||t===3){const c=this.el.content.firstChild;c.replaceWith(...c.childNodes)}for(;(i=M.nextNode())!==null&&l.length<a;){if(i.nodeType===1){if(i.hasAttributes())for(const c of i.getAttributeNames())if(c.endsWith($e)){const m=p[o++],y=i.getAttribute(c).split(w),v=/([.?@])?(.*)/.exec(m);l.push({type:1,index:r,name:v[2],strings:y,ctor:v[1]==="."?Re:v[1]==="?"?Le:v[1]==="@"?Be:K}),i.removeAttribute(c)}else c.startsWith(w)&&(l.push({type:6,index:r}),i.removeAttribute(c));if(ye.test(i.tagName)){const c=i.textContent.split(w),m=c.length-1;if(m>0){i.textContent=W?W.emptyScript:"";for(let y=0;y<m;y++)i.append(c[y],j()),M.nextNode(),l.push({type:2,index:++r});i.append(c[m],j())}}}else if(i.nodeType===8)if(i.data===be)l.push({type:2,index:r});else{let c=-1;for(;(c=i.data.indexOf(w,c+1))!==-1;)l.push({type:7,index:r}),c+=w.length-1}r++}}static createElement(e,t){const s=P.createElement("template");return s.innerHTML=e,s}}function I(n,e,t=n,s){if(e===T)return e;let i=s!==void 0?t._$Co?.[s]:t._$Cl;const r=z(e)?void 0:e._$litDirective$;return i?.constructor!==r&&(i?._$AO?.(!1),r===void 0?i=void 0:(i=new r(n),i._$AT(n,t,s)),s!==void 0?(t._$Co??=[])[s]=i:t._$Cl=i),i!==void 0&&(e=I(n,i._$AS(n,e.values),i,s)),e}class De{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,i=(e?.creationScope??P).importNode(t,!0);M.currentNode=i;let r=M.nextNode(),o=0,a=0,l=s[0];for(;l!==void 0;){if(o===l.index){let d;l.type===2?d=new R(r,r.nextSibling,this,e):l.type===1?d=new l.ctor(r,l.name,l.strings,this,e):l.type===6&&(d=new Ve(r,this,e)),this._$AV.push(d),l=s[++a]}o!==l?.index&&(r=M.nextNode(),o++)}return M.currentNode=P,i}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class R{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,i){this.type=2,this._$AH=u,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=I(this,e,t),z(e)?e===u||e==null||e===""?(this._$AH!==u&&this._$AR(),this._$AH=u):e!==this._$AH&&e!==T&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):je(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==u&&z(this._$AH)?this._$AA.nextSibling.data=e:this.T(P.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=H.createElement(_e(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(t);else{const r=new De(i,this),o=r.u(this.options);r.p(t),this.T(o),this._$AH=r}}_$AC(e){let t=fe.get(e.strings);return t===void 0&&fe.set(e.strings,t=new H(e)),t}k(e){ne(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,i=0;for(const r of e)i===t.length?t.push(s=new R(this.O(j()),this.O(j()),this,this.options)):s=t[i],s._$AI(r),i++;i<t.length&&(this._$AR(s&&s._$AB.nextSibling,i),t.length=i)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=le(e).nextSibling;le(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class K{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,i,r){this.type=1,this._$AH=u,this._$AN=void 0,this.element=e,this.name=t,this._$AM=i,this.options=r,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=u}_$AI(e,t=this,s,i){const r=this.strings;let o=!1;if(r===void 0)e=I(this,e,t,0),o=!z(e)||e!==this._$AH&&e!==T,o&&(this._$AH=e);else{const a=e;let l,d;for(e=r[0],l=0;l<r.length-1;l++)d=I(this,a[s+l],t,l),d===T&&(d=this._$AH[l]),o||=!z(d)||d!==this._$AH[l],d===u?e=u:e!==u&&(e+=(d??"")+r[l+1]),this._$AH[l]=d}o&&!i&&this.j(e)}j(e){e===u?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Re extends K{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===u?void 0:e}}class Le extends K{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==u)}}class Be extends K{constructor(e,t,s,i,r){super(e,t,s,i,r),this.type=5}_$AI(e,t=this){if((e=I(this,e,t,0)??u)===T)return;const s=this._$AH,i=e===u&&s!==u||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,r=e!==u&&(s===u||i);i&&this.element.removeEventListener(this.name,this,s),r&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Ve{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){I(this,e)}}const Fe=te.litHtmlPolyfillSupport;Fe?.(H,R),(te.litHtmlVersions??=[]).push("3.3.2");const We=(n,e,t)=>{const s=t?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const r=t?.renderBefore??null;s._$litPart$=i=new R(e.insertBefore(j(),r),r,void 0,t??{})}return i._$AI(n),i};/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const se=globalThis;class U extends k{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=We(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return T}}U._$litElement$=!0,U.finalized=!0,se.litElementHydrateSupport?.({LitElement:U});const qe=se.litElementPolyfillSupport;qe?.({LitElement:U});(se.litElementVersions??=[]).push("4.2.2");/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const Je=n=>(e,t)=>{t!==void 0?t.addInitializer(()=>{customElements.define(n,e)}):customElements.define(n,e)};/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const Ze={attribute:!0,type:String,converter:F,reflect:!1,hasChanged:ee},Ke=(n=Ze,e,t)=>{const{kind:s,metadata:i}=t;let r=globalThis.litPropertyMetadata.get(i);if(r===void 0&&globalThis.litPropertyMetadata.set(i,r=new Map),s==="setter"&&((n=Object.create(n)).wrapped=!0),r.set(t.name,n),s==="accessor"){const{name:o}=t;return{set(a){const l=e.get.call(this);e.set.call(this,a),this.requestUpdate(o,l,n,!0,a)},init(a){return a!==void 0&&this.C(o,void 0,n,a),a}}}if(s==="setter"){const{name:o}=t;return function(a){const l=this[o];e.call(this,a),this.requestUpdate(o,l,n,!0,a)}}throw Error("Unsupported decorator location: "+s)};function Ge(n){return(e,t)=>typeof t=="object"?Ke(n,e,t):((s,i,r)=>{const o=i.hasOwnProperty(r);return i.constructor.createProperty(r,s),o?Object.getOwnPropertyDescriptor(i,r):void 0})(n,e,t)}var Qe=Object.defineProperty,Xe=Object.getOwnPropertyDescriptor,xe=(n,e,t,s)=>{for(var i=s>1?void 0:s?Xe(e,t):e,r=n.length-1,o;r>=0;r--)(o=n[r])&&(i=(s?o(e,t,i):o(i))||i);return s&&i&&Qe(e,t,i),i};function Ye(n){let e="@default";const t=new Map;let s={},i="root",r=!1;for(const o of n){const a=o.createSurface||o.beginRendering;a&&(e=a.surfaceId||"@default",a.root&&(i=a.root),a.sendDataModel&&(r=!0));const l=o.updateComponents||o.surfaceUpdate;if(l)for(const p of l.components){const c={id:p.id};if(p.component&&typeof p.component=="object"){const m=Object.keys(p.component);if(m.length===1){const y=m[0];c.component=y;const v=p.component[y];if(v&&typeof v=="object")for(const[S,f]of Object.entries(v))S==="children"&&f&&typeof f=="object"&&f.explicitList?c.children=f.explicitList:S==="text"&&f&&typeof f=="object"&&f.literalString!=null?c.text=f.literalString:S==="label"&&f&&typeof f=="object"&&f.literalString!=null?c.label=f.literalString:S==="name"&&f&&typeof f=="object"&&f.literalString!=null?c.name=f.literalString:S==="description"&&f&&typeof f=="object"&&f.literalString!=null?c.description=f.literalString:S==="url"&&f&&typeof f=="object"&&f.literalString!=null?c.url=f.literalString:c[S]=f}}else Object.assign(c,p);t.set(c.id,c),c.id==="root"&&(i="root")}const d=o.updateDataModel||o.dataModelUpdate;d&&(!d.path||d.path==="/")&&(s={...s,...d.value})}return t.size===0?null:{surfaceId:e,components:t,dataModel:s,rootId:i,sendDataModel:r}}function q(n,e){const t=n.replace(/^\//,"").split("/");let s=e;for(const i of t){if(s==null||typeof s!="object")return;let r=i;if(/^\{.+\}$/.test(i)){const o=i.slice(1,-1),a=e["_"+o]??e[o];a!=null&&(r=String(a))}s=s[r]}return s}function g(n,e){if(n==null)return"";if(typeof n=="string")return n.includes("${/")?n.replace(/\$\{(\/[^}]+)\}/g,(t,s)=>{const i=q(s,e);return i!=null?String(i):""}):n;if(n.path){const t=q(n.path,e);return t!=null?String(t):""}return""}function L(n,e){if(n!=null){if(typeof n=="object"&&n!==null&&"path"in n)return q(n.path,e);if(typeof n=="string"&&n.includes("${/")){const t=n.match(/^\$\{(\/[^}]+)\}$/);if(t)return q(t[1],e)}return n}}let J=class extends U{constructor(){super(...arguments),this.surface=null,this._inputValues=new Map}updated(n){n.has("surface")&&this._inputValues.clear()}_fireAction(n,e,t){const s={},i=this.surface?.dataModel??{};for(const[o,a]of Object.entries(t))s[o]=L(a,i);const r={...s};for(const[o,a]of this._inputValues)o in r||(r[o]=a);if(this._inputValues.size>0){const o={};for(const[a,l]of this._inputValues)o[a]=l;r._formData=o}this.dispatchEvent(new CustomEvent("a2ui-action",{bubbles:!0,composed:!0,detail:{name:n,sourceComponentId:e,context:r}}))}render(){return this.surface?this._renderComponent(this.surface.rootId):u}_renderComponent(n){const t=this.surface.components.get(n);if(!t)return u;switch(t.component||t.type||""){case"Card":return this._renderCard(t);case"Column":return this._renderColumn(t);case"Row":return this._renderRow(t);case"Text":return this._renderText(t);case"Button":return this._renderButton(t);case"Divider":return h`<hr class="divider ${t.axis==="vertical"?"vertical":""}" />`;case"CheckBox":return this._renderCheckBox(t);case"Slider":return this._renderSlider(t);case"TextField":return this._renderTextField(t);case"Image":return this._renderImage(t);case"Icon":return this._renderIcon(t);case"Tabs":return this._renderTabs(t);case"List":return this._renderList(t);case"Modal":return this._renderModal(t);case"ChoicePicker":case"MultipleChoice":return this._renderChoicePicker(t);case"DateTimeInput":return this._renderDateTimeInput(t);case"Video":return this._renderVideo(t);case"AudioPlayer":return this._renderAudioPlayer(t);default:return t.children?h`<div>${t.children.map(i=>this._renderComponent(i))}</div>`:t.child?this._renderComponent(t.child):u}}_renderCard(n){return h`
      <div class="card">
        ${n.child?this._renderComponent(n.child):u}
      </div>
    `}_renderColumn(n){const e=this._getConsumedChildIds(),t=(n.children||[]).filter(s=>!e.has(s));return h`
      <div class="column" data-align="${n.align||""}">
        ${t.map(s=>this._renderComponent(s))}
      </div>
    `}_renderRow(n){const e=this._getConsumedChildIds(),t=(n.children||[]).filter(s=>!e.has(s));return h`
      <div class="row" data-justify="${n.justify||""}">
        ${t.map(s=>this._renderComponent(s))}
      </div>
    `}_renderText(n){const e=g(n.text,this.surface.dataModel),t=n.variant||"body";return h`<span class="text-${t}">${e}</span>`}_renderButton(n){let e="";if(n.child){const o=this.surface.components.get(n.child);o&&(e=g(o.text,this.surface.dataModel))}e||(e=n.label||n.text||n.id);const t=n.variant||"",i=n.primary===!0||t==="filled"?"primary":t||"";return h`
      <button class="btn ${i}" @click=${()=>{const o=n.action?.functionCall;if(o){this._handleFunctionCall(o);return}const a=n.action?.event;if(a){const l=this._extractUrlFromEvent(a);if(l){window.open(l,"_blank","noopener");return}this._fireAction(a.name||"unknown",n.id,a.context||{})}}}>
        ${e}
      </button>
    `}_handleFunctionCall(n){switch(n.call){case"openUrl":{const e=n.args?.url;e&&window.open(e,"_blank","noopener");break}default:console.warn(`[A2UI] Unhandled client functionCall: ${n.call}`,n.args)}}_extractUrlFromEvent(n){const e=n.context;if(!e)return null;for(const t of Object.values(e))if(typeof t=="string"&&/^https?:\/\/.+/i.test(t))return t;return null}_renderCheckBox(n){const e=g(n.label,this.surface.dataModel),t=L(n.value,this.surface.dataModel);return h`
      <label class="checkbox-wrapper">
        <input type="checkbox" .checked=${!!t} @change=${i=>{const r=i.target;this._inputValues.set(n.id,r.checked)}} />
        ${e}
      </label>
    `}_renderSlider(n){const e=g(n.label,this.surface.dataModel),t=L(n.value,this.surface.dataModel),s=n.min??0,i=n.max??100,r=o=>{const a=o.target;this._inputValues.set(n.id,Number(a.value))};return h`
      <div class="slider-wrapper">
        ${e?h`<label>${e}</label>`:u}
        <input type="range" min=${s} max=${i} .value=${String(t??s)} @input=${r} />
        <span class="slider-value">${t??s} / ${i}</span>
      </div>
    `}_renderTextField(n){const e=g(n.label,this.surface.dataModel),t=g(n.text??n.value,this.surface.dataModel),s=n.textFieldType||"shortText",i=o=>{const a=o.target;this._inputValues.set(n.id,a.value)};if(s==="longText"||s==="multiline")return h`
        <div class="textfield-wrapper">
          ${e?h`<label>${e}</label>`:u}
          <textarea rows="3" .value=${t} placeholder=${e} @input=${i}></textarea>
        </div>
      `;const r=s==="obscured"||s==="password"?"password":s==="number"?"number":s==="date"?"date":s==="email"?"email":"text";return h`
      <div class="textfield-wrapper">
        ${e?h`<label>${e}</label>`:u}
        <input type=${r} .value=${t} placeholder=${e} @input=${i} />
      </div>
    `}_renderImage(n){const e=g(n.url,this.surface.dataModel),t=n.variant||n.usageHint||"",s=n.fit?`object-fit: ${n.fit}`:"";return h`<img class="a2ui-image ${t}" src=${e} style=${s} alt="" />`}_renderIcon(n){const t=g(n.name,this.surface.dataModel).replace(/([A-Z])/g,"_$1").toLowerCase().replace(/^_/,""),i={cloud:"☁️",sunny:"☀️",clear:"☀️",sun:"☀️",umbrella:"☂️",rain:"🌧️",rainy:"🌧️",snow:"❄️",thunderstorm:"⛈️",fog:"🌫️",wind:"💨",partly_cloudy:"⛅",partly_cloudy_day:"⛅",partly_cloudy_night:"⛅",check:"✅",close:"❌",star:"⭐",favorite:"❤️",home:"🏠",settings:"⚙️",search:"🔍",info:"ℹ️",warning:"⚠️",error:"❗",calendar:"📅",schedule:"📅",location:"📍",place:"📍",restaurant:"🍽️",music:"🎵",play:"▶️",pause:"⏸️",stop:"⏹️"}[t];return i?h`<span class="a2ui-icon-emoji">${i}</span>`:h`<span class="material-symbols-outlined a2ui-icon">${t}</span>`}_renderTabs(n){const e=n.tabItems||n.tabs||[];if(e.length===0)return u;const t=0;return h`
      <div>
        <div class="tabs-header">
          ${e.map((s,i)=>{const r=g(s.title||s.label,this.surface.dataModel);return h`<button class="tab-btn ${i===t?"active":""}">${r}</button>`})}
        </div>
        <div class="tab-content">
          ${this._renderComponent(e[t].child)}
        </div>
      </div>
    `}_renderList(n){const e=n.direction==="horizontal"?"horizontal":"vertical",t=n.children||[];if(!Array.isArray(t)&&typeof t=="object"){const s=t;if(s.componentId&&s.path){const i=this.surface.components.get(s.componentId),r=L({path:s.path},this.surface.dataModel);if(i&&Array.isArray(r))return h`
            <div class="list-${e}">
              ${r.map((o,a)=>this._renderTemplateInstance(i,o,a))}
            </div>
          `}return u}return Array.isArray(t)?h`
        <div class="list-${e}">
          ${t.map(s=>this._renderComponent(s))}
        </div>
      `:u}_renderTemplateInstance(n,e,t){const s=this.surface,i=s.dataModel;s.dataModel={...i,...e,current:e,_index:t};try{return this._renderComponent(n.id)}finally{s.dataModel=i}}_renderModal(n){const e=n.entryPointChild||n.trigger||"";return n.contentChild||n.content,h`
      <div>
        ${e?this._renderComponent(e):u}
      </div>
    `}_renderChoicePicker(n){const e=g(n.label,this.surface.dataModel),t=n.options||[],s=n.variant||"radio",i=s==="multipleSelection"||s==="chip"||n.component==="MultipleChoice",r=i?"checkbox":"radio",o=`choice-${n.id}`,a=l=>{const d=l.target;if(i){const p=this._inputValues.get(n.id)||[];d.checked?this._inputValues.set(n.id,[...p,d.value]):this._inputValues.set(n.id,p.filter(c=>c!==d.value))}else this._inputValues.set(n.id,d.value)};return h`
      <div class="choice-picker">
        ${e?h`<label class="group-label">${e}</label>`:u}
        ${t.map(l=>{const d=g(l.label,this.surface.dataModel);return h`
            <label class="choice-option">
              <input type=${r} name=${o} value=${l.value} @change=${a} />
              ${d}
            </label>
          `})}
      </div>
    `}_renderDateTimeInput(n){const e=g(n.label,this.surface.dataModel),t=g(n.value,this.surface.dataModel),s=n.enableDate!==!1,i=n.enableTime===!0,r=s&&i?"datetime-local":i?"time":"date";return h`
      <div class="datetime-wrapper">
        ${e?h`<label>${e}</label>`:u}
        <input type=${r} .value=${t} />
      </div>
    `}_renderVideo(n){const e=g(n.url,this.surface.dataModel);return h`<video class="a2ui-video" src=${e} controls></video>`}_renderAudioPlayer(n){const e=g(n.url,this.surface.dataModel),t=g(n.description,this.surface.dataModel);return h`
      <div class="audio-wrapper">
        ${t?h`<span class="audio-desc">${t}</span>`:u}
        <audio src=${e} controls></audio>
      </div>
    `}_getConsumedChildIds(){const n=new Set;for(const e of this.surface.components.values())(e.component||e.type||"")==="Button"&&e.child&&n.add(e.child);return n}};J.styles=Ce`
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
  `;xe([Ge({type:Object})],J.prototype,"surface",2);J=xe([Je("a2ui-surface-v09")],J);let $=null,C=null,V=null,x=null,_=[];const b=n=>document.getElementById(n);function me(n){const e=b("status"),t=b("status-text");e.className=n?"connected":"",t.textContent=n?"Connected":"Disconnected",b("chat-input").disabled=!n,b("send-btn").disabled=!n,b("connect-btn").textContent=n?"Disconnect":"Connect"}function G(){const n=b("messages");n.scrollTop=n.scrollHeight}function A(n,e,t){const s=b("messages"),i=document.createElement("div");if(i.className=`msg ${n}`,i.textContent=e,t!=null&&n==="assistant"){const r=document.createElement("span");r.className="elapsed",r.textContent=`${t.toFixed(1)}s`,i.appendChild(r)}s.appendChild(i),G()}function ve(){D(),C=performance.now();const n=b("messages");x=document.createElement("div"),x.className="thinking",x.innerHTML=`
    <div class="spinner"></div>
    <span>Thinking...</span>
    <span class="timer">0.0s</span>
  `,n.appendChild(x),G(),V=window.setInterval(()=>{if(!x||!C)return;const e=((performance.now()-C)/1e3).toFixed(1),t=x.querySelector(".timer");t&&(t.textContent=`${e}s`)},100)}function D(){V&&(clearInterval(V),V=null),x&&(x.remove(),x=null)}function we(){return C?(performance.now()-C)/1e3:null}function et(n){const e=b("messages"),t=document.createElement("div");t.className="a2ui-surface-container";const s=Ye(_);if(s){const r=document.createElement("a2ui-surface-v09");r.surface=s,r.addEventListener("a2ui-action",o=>{st(o.detail,s)}),t.appendChild(r)}const i=s?.surfaceId;if(i&&(t.dataset.surfaceId=i),_.length>0){const r=document.createElement("div");r.className="inspector-wrap";const o=JSON.stringify(_,null,2),a=document.createElement("button");a.className="copy-btn",a.textContent="📋 Copy",a.addEventListener("click",()=>{(()=>{if(navigator.clipboard?.writeText)return navigator.clipboard.writeText(o);const m=document.createElement("textarea");m.value=o,m.style.cssText="position:fixed;left:-9999px;top:0",document.body.appendChild(m),m.select();const y=document.execCommand("copy");return document.body.removeChild(m),y?Promise.resolve():Promise.reject()})().then(()=>{a.textContent="✅ Copied!",setTimeout(()=>{a.textContent="📋 Copy"},1500)}).catch(()=>{a.textContent="❌ Failed",setTimeout(()=>{a.textContent="📋 Copy"},1500)})}),r.appendChild(a);const l=document.createElement("details");l.className="inspector";const d=document.createElement("summary");d.textContent=`Raw A2UI JSON (${_.length} messages)`;const p=document.createElement("pre");p.textContent=o,l.appendChild(d),l.appendChild(p),r.appendChild(l),t.appendChild(r)}if(n!=null){const r=document.createElement("div");r.style.cssText="text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px",r.textContent=`${n.toFixed(1)}s`,t.appendChild(r)}e.appendChild(t),G()}function tt(n){try{const e=new URL(n);return(e.hostname==="127.0.0.1"||e.hostname==="localhost")&&(e.hostname=location.hostname),e.toString()}catch{return n}}function nt(n){const e=we();D();const t=b("messages"),s=document.createElement("div");s.className="a2web-container";const i=tt(n.url||""),r=document.createElement("div");r.className="a2web-header",r.innerHTML=`
    <span class="a2web-label">a2web</span>
    <span class="a2web-title">${n.title||"Web Page"}</span>
    <a href="${i}" target="_blank" rel="noopener" class="a2web-open">새 탭에서 열기 ↗</a>
  `,s.appendChild(r);const o=document.createElement("iframe");if(o.src=i,o.className="a2web-iframe",o.setAttribute("sandbox","allow-scripts allow-same-origin allow-forms allow-popups"),s.appendChild(o),e!=null){const a=document.createElement("div");a.style.cssText="text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px",a.textContent=`${e.toFixed(1)}s`,s.appendChild(a)}t.appendChild(s),G(),C=null}function st(n,e){if(console.log("A2UI action:",n),$&&$.readyState===WebSocket.OPEN){_=[];const t={surfaceId:e.surfaceId,name:n?.name||"unknown",sourceComponentId:n?.sourceComponentId||"unknown",context:n?.context||{}};e.sendDataModel&&e.dataModel&&(t.dataModel=e.dataModel),$.send(JSON.stringify({type:"a2ui_action",payload:t})),ve()}}function it(n){switch(console.log("[WS]",n.type,n),n.type){case"history":n.messages?.length&&A("system",`History: ${n.messages.length} messages`);break;case"a2ui":if(n.messages){console.log("[A2UI] received",n.messages.length,"messages");for(const e of n.messages)if(e.deleteSurface?.surfaceId){const t=e.deleteSurface.surfaceId,s=document.querySelector(`[data-surface-id="${t}"]`);s&&s.remove(),console.log("[A2UI] deleteSurface:",t)}_=n.messages}break;case"a2web":nt(n);break;case"done":{const e=we();D(),console.log("[DONE] a2ui msgs:",_.length,"full_response:",!!n.full_response),_.length>0&&(et(e),_=[]),n.full_response&&A("assistant",n.full_response,_.length>0?null:e),C=null;break}case"chunk":break;case"error":D(),A("system",`Error: ${n.message}`),C=null;break;default:console.log("Unknown WS message:",n)}}window.toggleConnection=function(){if($&&$.readyState===WebSocket.OPEN){$.close();return}let n=b("ws-url").value.trim();if(!n)return;const e=n.includes("?")?"&":"?";n.includes("session_id=")||(n+=`${e}session_id=lisa-test`),A("system",`Connecting to ${n}...`),$=new WebSocket(n),$.onopen=()=>{me(!0),A("system","Connected")},$.onclose=()=>{me(!1),D(),A("system","Disconnected")},$.onerror=()=>{A("system","Connection error")},$.onmessage=t=>{try{it(JSON.parse(t.data))}catch(s){console.error("Parse error:",s)}}};window.sendMessage=function(){const n=b("chat-input"),e=n.value.trim();!e||!$||$.readyState!==WebSocket.OPEN||(A("user",e),_=[],ve(),$.send(JSON.stringify({type:"message",content:e})),n.value="",n.focus())};window.addEventListener("load",()=>{b("chat-input").focus();const n=b("ws-url"),e=location.protocol==="https:"?"wss:":"ws:",t=location.hostname+":42617";n.value=`${e}//${t}/ws/chat`,window.toggleConnection()});
