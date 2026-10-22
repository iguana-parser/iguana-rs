import e from "cytoscape";
import t from "cytoscape-tidytree";
//#region ../web-ui/src/graph-controls.ts
var n = "width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"", r = {
	zoomIn: `<svg ${n}><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35M11 8v6M8 11h6"/></svg>`,
	zoomOut: `<svg ${n}><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35M8 11h6"/></svg>`,
	fit: `<svg ${n}><path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2"/><rect x="7" y="8" width="10" height="8" rx="1"/></svg>`,
	exportPng: `<svg ${n}><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>`,
	popOut: `<svg ${n}><path d="M15 3h6v6M10 14 21 3M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>`
};
function i(e, t) {
	let n = document.createElement("div");
	n.className = "graph-controls";
	function i(e, t, r) {
		let i = document.createElement("button");
		i.type = "button", i.title = e, i.setAttribute("aria-label", e), i.innerHTML = t, i.addEventListener("click", r), n.appendChild(i);
	}
	return i("Zoom in", r.zoomIn, t.zoomIn), i("Zoom out", r.zoomOut, t.zoomOut), i("Fit view", r.fit, t.fit), t.exportPng && i("Export as PNG", r.exportPng, t.exportPng), t.popOut && i("Pop out", r.popOut, t.popOut), e.appendChild(n), () => n.remove();
}
//#endregion
//#region ../web-ui/src/theme.ts
function a(e, t) {
	return getComputedStyle(document.documentElement).getPropertyValue(e).trim() || t;
}
var o = "\"Fira Code\", ui-monospace, \"SF Mono\", Menlo, Consolas, \"Liberation Mono\", monospace";
function s(e) {
	let t = new MutationObserver(e);
	t.observe(document.documentElement, {
		attributes: !0,
		attributeFilter: [
			"data-theme",
			"class",
			"style"
		]
	});
	let n = window.matchMedia("(prefers-color-scheme: dark)");
	return n.addEventListener("change", e), () => {
		t.disconnect(), n.removeEventListener("change", e);
	};
}
var c;
function l(e) {
	if (c ??= document.createElement("canvas").getContext("2d"), !c || !e) return null;
	let t = (t) => (c.fillStyle = t, c.fillStyle = e, String(c.fillStyle)), n = t("#000000");
	if (n !== t("#ffffff")) return null;
	if (n.startsWith("#")) return n;
	let r = n.match(/^rgba\((\d+), (\d+), (\d+), ([\d.]+)\)$/);
	if (!r) return null;
	let i = (e) => Math.round(e).toString(16).padStart(2, "0");
	return "#" + [
		1,
		2,
		3
	].map((e) => i(Number(r[e]))).join("") + i(Number(r[4]) * 255);
}
function u(e) {
	return l(a(e, "")) ?? void 0;
}
function d(e, t, n) {
	let r = l(e)?.slice(0, 7);
	if (!r) return e;
	e = r;
	let i = (e, t) => parseInt(e.slice(1 + 2 * t, 3 + 2 * t), 16), a = "#";
	for (let r = 0; r < 3; r++) {
		let o = Math.round(i(t, r) + (i(e, r) - i(t, r)) * n);
		a += o.toString(16).padStart(2, "0");
	}
	return a;
}
function f(e) {
	let t = (e) => {
		let t = l(a(e, ""));
		if (!t) return;
		if (t.length === 7) return t;
		let [n, r, i, o] = [
			1,
			3,
			5,
			7
		].map((e) => parseInt(t.slice(e, e + 2), 16));
		return `rgba(${n}, ${r}, ${i}, ${Math.round(o / 255 * 1e3) / 1e3})`;
	}, n = u("--iguana-bg"), r = n?.length === 7 ? n : "#1e1e1e", i = {}, o = (e, t) => {
		i[e] = {
			...i[e],
			...t
		};
	}, s = (e) => ({
		"line-color": e,
		"target-arrow-color": e
	}), c = t("--iguana-text");
	c && o("node", { color: c });
	let f = t("--iguana-bg-raised");
	f && o("node", { "background-color": f });
	for (let e of [
		{
			name: "--iguana-syntax-nonterminal",
			fallback: "#4ec9b0",
			rule: "node.nonterminal, node[kind='Nonterminal']",
			selected: "node.nonterminal.selected, node[kind='Nonterminal'].selected",
			edge: "edge.edge-selected-nonterminal"
		},
		{
			name: "--iguana-syntax-token",
			fallback: "#ce9178",
			rule: "node.token, node[kind='Token']",
			selected: "node.token.selected, node[kind='Token'].selected",
			edge: "edge.edge-selected-terminal"
		},
		{
			name: "--iguana-syntax-ambiguity",
			fallback: "#e05050",
			rule: "node.ambiguous, node.amb",
			selected: "node.ambiguous.selected, node.amb.selected",
			edge: "edge.edge-selected-ambiguous"
		}
	]) {
		let i = t(e.name);
		if (!i && !n) continue;
		let a = i ?? e.fallback;
		o(e.rule, {
			"background-color": d(a, r, .18),
			"border-color": a
		}), o(e.selected, {
			"background-color": d(a, r, .35),
			"border-color": a
		}), i && o(e.edge, s(i));
	}
	let p = t("--iguana-syntax-ambiguity");
	p && o("edge.edge-ambiguous", s(p));
	let m = t("--iguana-border-strong");
	m && o("edge", s(m));
	let h = t("--iguana-syntax-reference");
	h && o("edge.shared", s(h));
	let g = t("--iguana-text-muted");
	return g && o("edge.edge-clicked", s(g)), e.map((e) => i[e.selector] ? {
		...e,
		style: {
			...e.style,
			...i[e.selector]
		}
	} : e);
}
function p(e, t = 20) {
	return e.length <= t ? e : e.substring(0, t - 3) + "...";
}
function m(e, t) {
	let n = document.createElement("div");
	n.className = "graph-tooltip", n.style.cssText = `
    position: fixed;
    background: var(--iguana-bg-raised, #252526);
    border: 1px solid var(--iguana-border, #454545);
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 11px;
    font-family: var(--iguana-font-mono, ${o});
    color: var(--iguana-text, #d4d4d4);
    pointer-events: none;
    z-index: 10000;
    display: none;
    max-width: 400px;
    word-wrap: break-word;
    white-space: pre-wrap;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  `, document.body.appendChild(n);
	let r = (e) => {
		let r = e.target, i = r.data("fullLabel"), a = r.data("label");
		if (!i || i === a) return;
		n.textContent = i, n.style.display = "block";
		let o = e.renderedPosition || e.position, s = t.getBoundingClientRect();
		n.style.left = `${s.left + o.x + 15}px`, n.style.top = `${s.top + o.y + 15}px`;
	}, i = () => {
		n.style.display = "none";
	}, a = (e) => {
		if (n.style.display === "none") return;
		let r = e.renderedPosition || e.position, i = t.getBoundingClientRect();
		n.style.left = `${i.left + r.x + 15}px`, n.style.top = `${i.top + r.y + 15}px`;
	};
	return e.on("mouseover", "node", r), e.on("mouseout", "node", i), e.on("mousemove", "node", a), () => {
		e.off("mouseover", "node", r), e.off("mouseout", "node", i), e.off("mousemove", "node", a), n.remove();
	};
}
var h = {
	label: "data(label)",
	"text-valign": "center",
	"text-halign": "center",
	"font-size": "9px",
	color: "#d4d4d4",
	"border-width": 1,
	width: "label",
	shape: "round-rectangle"
}, g = {
	width: 1,
	"line-color": "#555",
	"target-arrow-color": "#555",
	"target-arrow-shape": "triangle",
	"curve-style": "bezier",
	"arrow-scale": .8
}, _ = {
	nonterminal: {
		bg: "#2d4a3d",
		border: "#4ec9b0",
		selectedBg: "#3a5f50",
		selectedBorder: "#7fffaa"
	},
	intermediate: {
		bg: "#2d3a4d",
		border: "#569cd6",
		selectedBg: "#3a4d60",
		selectedBorder: "#7eb8ff"
	},
	terminal: {
		bg: "#4d3a2d",
		border: "#ce9178",
		selectedBg: "#5f4a3a",
		selectedBorder: "#ffb07a"
	},
	packed: {
		bg: "#666",
		selectedBg: "#888",
		selectedBorder: "#aaa"
	},
	ambiguous: {
		bg: "#4d2d2d",
		border: "#e05050",
		selectedBg: "#5f3a3a",
		selectedBorder: "#ff7a7a"
	}
}, v = [
	{
		selector: "node:active",
		style: { "overlay-opacity": 0 }
	},
	{
		selector: "edge:active",
		style: { "overlay-opacity": 0 }
	},
	{
		selector: "core",
		style: { "active-bg-opacity": 0 }
	}
], y = [
	...v,
	{
		selector: "node",
		style: {
			...h,
			"text-wrap": "wrap",
			"background-color": "#3c3c3c",
			"border-color": "#555",
			height: "label",
			"padding-left": "9px",
			"padding-right": "9px",
			"padding-top": "5px",
			"padding-bottom": "5px"
		}
	},
	{
		selector: "node.nonterminal, node[kind='Nonterminal']",
		style: {
			"background-color": _.nonterminal.bg,
			"border-color": _.nonterminal.border
		}
	},
	{
		selector: "node.intermediate, node[kind='Intermediate']",
		style: {
			"background-color": _.intermediate.bg,
			"border-color": _.intermediate.border,
			shape: "rectangle"
		}
	},
	{
		selector: "node.terminal, node[kind='Terminal']",
		style: {
			"background-color": _.terminal.bg,
			"border-color": _.terminal.border
		}
	},
	{
		selector: "node.token, node[kind='Token']",
		style: {
			"background-color": _.terminal.bg,
			"border-color": _.terminal.border
		}
	},
	{
		selector: "node.packed",
		style: {
			width: 12,
			height: 12,
			"background-color": "#666",
			"border-width": 0,
			label: ""
		}
	},
	{
		selector: "node.collapsed",
		style: { "border-style": "dashed" }
	},
	{
		selector: "node.selected",
		style: {
			"border-width": 3,
			"border-style": "double"
		}
	},
	{
		selector: "node.nonterminal.selected, node[kind='Nonterminal'].selected",
		style: {
			"background-color": _.nonterminal.selectedBg,
			"border-color": _.nonterminal.selectedBorder
		}
	},
	{
		selector: "node.intermediate.selected, node[kind='Intermediate'].selected",
		style: {
			"background-color": _.intermediate.selectedBg,
			"border-color": _.intermediate.selectedBorder
		}
	},
	{
		selector: "node.terminal.selected, node[kind='Terminal'].selected",
		style: {
			"background-color": _.terminal.selectedBg,
			"border-color": _.terminal.selectedBorder
		}
	},
	{
		selector: "node.token.selected, node[kind='Token'].selected",
		style: {
			"background-color": _.terminal.selectedBg,
			"border-color": _.terminal.selectedBorder
		}
	},
	{
		selector: "node.packed.selected, node[kind='Packed'].selected",
		style: {
			"background-color": _.packed.selectedBg,
			"border-width": 2,
			"border-color": _.packed.selectedBorder
		}
	},
	{
		selector: "node.ambiguous, node.amb",
		style: {
			"background-color": _.ambiguous.bg,
			"border-color": _.ambiguous.border
		}
	},
	{
		selector: "node.ambiguous.selected, node.amb.selected",
		style: {
			"background-color": _.ambiguous.selectedBg,
			"border-color": _.ambiguous.selectedBorder
		}
	},
	{
		selector: "node.shared-span",
		style: {
			"border-color": "#e07030",
			"border-width": 2
		}
	},
	{
		selector: "node.shared-span.selected",
		style: { "border-color": "#ff9050" }
	}
];
[
	...v,
	({ ...h }, _.nonterminal.bg, _.nonterminal.border),
	_.nonterminal.border
];
var b = [
	{
		selector: "edge",
		style: { ...g }
	},
	{
		selector: "edge.shared",
		style: {
			"line-color": "#c586c0",
			"target-arrow-color": "#c586c0"
		}
	},
	{
		selector: "edge.edge-selected-nonterminal",
		style: {
			"line-color": _.nonterminal.selectedBorder,
			"target-arrow-color": _.nonterminal.selectedBorder
		}
	},
	{
		selector: "edge.edge-selected-intermediate",
		style: {
			"line-color": _.intermediate.selectedBorder,
			"target-arrow-color": _.intermediate.selectedBorder
		}
	},
	{
		selector: "edge.edge-selected-terminal",
		style: {
			"line-color": _.terminal.selectedBorder,
			"target-arrow-color": _.terminal.selectedBorder
		}
	},
	{
		selector: "edge.edge-selected-packed",
		style: {
			"line-color": _.packed.selectedBorder,
			"target-arrow-color": _.packed.selectedBorder
		}
	},
	{
		selector: "edge.edge-clicked",
		style: {
			"line-color": "#999",
			"target-arrow-color": "#999"
		}
	},
	{
		selector: "edge.edge-selected-ambiguous",
		style: {
			"line-color": _.ambiguous.selectedBorder,
			"target-arrow-color": _.ambiguous.selectedBorder
		}
	},
	{
		selector: "edge.edge-ambiguous",
		style: {
			"line-color": _.ambiguous.border,
			"target-arrow-color": _.ambiguous.border
		}
	}
];
({ ...g });
var x = {
	name: "tidytree",
	direction: "TB",
	horizontalSpacing: 16,
	verticalSpacing: 30,
	nodeDimensionsIncludeLabels: !0,
	fit: !0,
	padding: 30
}, S = 1.6, C = .5, w = 12;
function T(e) {
	let t = e.zoom();
	if (t > 1.6) e.zoom(S), e.center();
	else if (t < .5) {
		e.zoom(C);
		let t = e.elements().boundingBox();
		e.pan({
			x: (e.width() - t.w * C) / 2 - t.x1 * C,
			y: w - t.y1 * C
		});
	}
}
function E(e, t) {
	e && e.zoom(e.zoom() * t);
}
var D = 1 / 1.2;
function O(e) {
	if (!e) return;
	e.fit(e.elements(), w);
	let t = e.zoom();
	if (t > 1.6) e.zoom(S), e.center();
	else if (t < .8333333333333334) {
		e.zoom(D);
		let t = e.elements().boundingBox(), n = e.nodes().roots().first(), r = t.w * .8333333333333334 > e.width() && n.nonempty() ? n.position("x") : (t.x1 + t.x2) / 2;
		e.pan({
			x: e.width() / 2 - r * D,
			y: w - t.y1 * D
		});
	}
}
function k(e) {
	e && (e.fit(e.elements(), w), e.zoom() > 1.6 && (e.zoom(S), e.center()));
}
function A(e) {
	let t = e.elements().boundingBox(), n = (e.width() - 24) / t.w, r = Math.max(D, Math.min(S, n));
	return Math.ceil(t.h * r + 24);
}
function j(t) {
	let { container: n, elements: r, styles: i, layout: a = "sppf", viewport: o } = t, s = r.filter((e) => e.data.source === void 0).length, c = a === "tree" ? x : {
		name: "dagre",
		rankDir: a === "gss" ? "BT" : "TB",
		nodeSep: a === "gss" ? 50 : 30,
		rankSep: a === "gss" ? 60 : 50
	}, l = a !== "tree" || s >= 1e3, u = e({
		container: n,
		elements: r,
		style: i,
		layout: c,
		userZoomingEnabled: !1,
		userPanningEnabled: !0,
		boxSelectionEnabled: !1,
		renderer: {
			name: "canvas",
			webgl: l
		}
	});
	if (u.scratch("_renderer", { webgl: l }), n) {
		let e = (e) => {
			if (e.preventDefault(), e.ctrlKey) {
				let t = 1 - e.deltaY * .01, n = u.zoom() * t;
				u.zoom({
					level: n,
					renderedPosition: {
						x: e.offsetX,
						y: e.offsetY
					}
				});
			} else {
				let t = u.pan();
				u.pan({
					x: t.x - e.deltaX,
					y: t.y - e.deltaY
				});
			}
		};
		n.addEventListener("wheel", e, { passive: !1 }), u.scratch("_disposeWheel", () => n.removeEventListener("wheel", e));
	}
	return o ? (u.zoom(o.zoom), u.pan(o.pan)) : T(u), u;
}
var M = [
	"edge-selected-nonterminal",
	"edge-selected-intermediate",
	"edge-selected-terminal",
	"edge-selected-packed",
	"edge-selected-ambiguous",
	"edge-clicked"
];
function N(e) {
	if (e.data("ambiguous") || e.hasClass("ambiguous")) return "edge-selected-ambiguous";
	let t = e.data("kind");
	return t === "Packed" || e.hasClass("packed") ? "edge-selected-packed" : t === "Nonterminal" || e.hasClass("nonterminal") ? "edge-selected-nonterminal" : t === "Intermediate" || e.hasClass("intermediate") ? "edge-selected-intermediate" : t === "Terminal" || t === "Token" || e.hasClass("terminal") || e.hasClass("token") ? "edge-selected-terminal" : "edge-selected-nonterminal";
}
function P(e, t) {
	let n = e.getElementById(t);
	if (n.empty()) return;
	let r = N(n);
	n.outgoers("edge").addClass(r);
}
function F(e) {
	e.edges().removeClass(M);
}
function I(e, t) {
	F(e), e.getElementById(t).addClass("edge-clicked");
}
//#endregion
//#region ../web-ui/src/parse-tree-graph.ts
var L = class {
	cy = null;
	collapsedNodes = /* @__PURE__ */ new Set();
	focusedNodeId = null;
	setCy(e) {
		this.cy = e;
	}
	reset() {
		this.collapsedNodes = /* @__PURE__ */ new Set(), this.focusedNodeId = null;
	}
	isFocused() {
		return this.focusedNodeId !== null;
	}
	getFocusedNodeId() {
		return this.focusedNodeId;
	}
	isCollapsed(e) {
		return this.collapsedNodes.has(e);
	}
	findRoot() {
		if (!this.cy) return null;
		let e = this.cy.nodes().filter((e) => e.incomers("edge").length === 0);
		return e.length > 0 ? e.first().id() : this.cy.nodes().first().id() ?? null;
	}
	getReachableFromNode(e, t) {
		if (!this.cy) return /* @__PURE__ */ new Set();
		let n = /* @__PURE__ */ new Set(), r = [e];
		for (; r.length > 0;) {
			let i = r.shift();
			n.has(i) || (n.add(i), !(this.collapsedNodes.has(i) && !(t && i === e)) && this.cy.getElementById(i).outgoers("node").forEach((e) => {
				n.has(e.id()) || r.push(e.id());
			}));
		}
		return n;
	}
	getReachableNodes() {
		if (!this.cy) return /* @__PURE__ */ new Set();
		if (this.focusedNodeId !== null) return this.getReachableFromNode(this.focusedNodeId, !0);
		let e = this.findRoot();
		return e ? this.getReachableFromNode(e, !1) : /* @__PURE__ */ new Set();
	}
	updateVisibility() {
		if (!this.cy) return;
		let e = this.getReachableNodes();
		this.cy.nodes().forEach((t) => {
			e.has(t.id()) ? t.style("display", "element") : t.style("display", "none");
		}), this.cy.edges().forEach((t) => {
			let n = t.source().id(), r = t.target().id();
			e.has(n) && e.has(r) && !this.collapsedNodes.has(n) ? t.style("display", "element") : t.style("display", "none");
		});
	}
	toggleCollapse(e) {
		if (!this.cy) return;
		let t = this.cy.getElementById(e);
		t.outgoers("node").length !== 0 && (this.collapsedNodes.has(e) ? (this.collapsedNodes.delete(e), t.removeClass("collapsed")) : (this.collapsedNodes.add(e), t.addClass("collapsed")), this.updateVisibility());
	}
	expandAll() {
		this.cy && (this.collapsedNodes.forEach((e) => {
			this.cy.getElementById(e).removeClass("collapsed");
		}), this.collapsedNodes = /* @__PURE__ */ new Set(), this.updateVisibility());
	}
	expandAncestors(e) {
		if (!this.cy) return;
		let t = !1, n = /* @__PURE__ */ new Set(), r = e;
		for (; r !== null && !n.has(r);) {
			n.add(r);
			let e = this.cy.getElementById(r);
			if (e.length === 0) break;
			let i = e.incomers("node");
			if (i.length === 0) break;
			let a = i.first(), o = a.id();
			this.collapsedNodes.has(o) && (this.collapsedNodes.delete(o), a.removeClass("collapsed"), t = !0), r = o;
		}
		t && this.updateVisibility();
	}
	focusOnSubtree(e) {
		if (!this.cy) return;
		this.focusedNodeId = e, this.updateVisibility();
		let t = this.cy.nodes().filter((e) => e.style("display") !== "none");
		t.length > 0 && this.cy.fit(t, 50);
	}
	clearFocus() {
		if (!this.cy) return;
		this.focusedNodeId = null, this.updateVisibility();
		let e = this.cy.nodes().filter((e) => e.style("display") !== "none");
		e.length > 0 && this.cy.fit(e, 50);
	}
};
function R(e, t) {
	let n = e.nodes.map((e) => {
		let n = `(${e.start}, ${e.end})`, r = t ? `${p(e.label, 20)}\n${n}` : p(e.label, 20), i = t ? `${e.label}\n${n}` : e.label;
		return {
			data: {
				id: `n${e.id}`,
				label: r,
				fullLabel: i,
				start: e.start,
				end: e.end
			},
			classes: e.kind.toLowerCase()
		};
	}), r = /* @__PURE__ */ new Map();
	for (let t of e.edges) r.set(t.dest, (r.get(t.dest) ?? 0) + 1);
	let i = e.edges.map((e, t) => ({
		data: {
			id: `e${t}`,
			source: `n${e.src}`,
			target: `n${e.dest}`
		},
		...r.get(e.dest) > 1 ? { classes: "shared" } : {}
	}));
	return [...n, ...i];
}
//#endregion
//#region ../web-ui/src/sexpr-parse.ts
function z(e) {
	let t = [], n = [], r = /* @__PURE__ */ new Map(), i = 0, a = 0;
	function o(e) {
		throw Error(`s-expression parse error at offset ${i}: ${e}`);
	}
	function s() {
		for (; i < e.length && /\s/.test(e[i]);) i++;
	}
	function c() {
		let t = i;
		for (i++; i < e.length;) {
			let n = e[i];
			if (n === "\\") i += 2;
			else if (n === "\"") return i++, e.slice(t, i);
			else i++;
		}
		o("unterminated string");
	}
	function l(t, n) {
		let r = i, a = 0;
		for (; i < e.length;) {
			let o = e[i];
			if (o === "\"") {
				c();
				continue;
			}
			if (o === t) a++;
			else if (o === n && (a--, a === 0)) return i++, e.slice(r, i);
			i++;
		}
		o(`unbalanced ${t}${n} group`);
	}
	function u(t) {
		let n = i;
		for (t && e[i] === "(" && l("(", ")"); i < e.length;) {
			let t = e[i];
			if (/\s/.test(t) || t === ")" || t === "(") break;
			if (t === "{") {
				l("{", "}");
				continue;
			}
			if (t === "\"") {
				c();
				continue;
			}
			i++;
		}
		return i === n && o("expected an atom"), e.slice(n, i);
	}
	function d(e, n) {
		let r = a++;
		return t.push({
			id: r,
			kind: n,
			label: e,
			start: 0,
			end: 0
		}), r;
	}
	function f() {
		let t = /^#(\d+)([=#])/.exec(e.slice(i));
		return t ? (i += t[0].length, {
			n: Number(t[1]),
			kind: t[2] === "=" ? "def" : "ref"
		}) : null;
	}
	function p() {
		let t = i;
		l("(", ")");
		let n = i < e.length && /[?*+]/.test(e[i]);
		return i = t, n;
	}
	function m() {
		s(), i >= e.length && o("unexpected end of input");
		let t = f();
		if (t?.kind === "ref") {
			let e = r.get(t.n);
			return e === void 0 && o(`reference #${t.n}# before its definition`), e;
		}
		let a;
		if (e[i] === "(" && !p()) {
			i++, s();
			let c = u(!0);
			for (a = d(c, c === "Amb" ? "Amb" : "Nonterminal"), t && r.set(t.n, a), s(); i < e.length && e[i] !== ")";) {
				let e = m();
				n.push({
					src: a,
					dest: e
				}), s();
			}
			e[i] !== ")" && o("expected )"), i++;
		} else e[i] === "\"" ? (a = d(c(), "Token"), t && r.set(t.n, a)) : (a = d(u(!0), "Nonterminal"), t && r.set(t.n, a));
		return a;
	}
	return m(), s(), i < e.length && o("trailing content after the tree"), {
		layout_name: null,
		nodes: t,
		edges: n
	};
}
//#endregion
//#region ../web-ui/src/png.ts
async function B(e, t) {
	if (!e) return;
	let n = await e.png({
		output: "blob",
		bg: "#1e1e1e",
		scale: 2
	}), r = URL.createObjectURL(n), i = document.createElement("a");
	i.href = r, i.download = `${t}.png`, i.click(), URL.revokeObjectURL(r);
}
//#endregion
//#region src/main.ts
e.use(t);
function V(e, t, n = {}) {
	let r = z(t), a = () => f([...y, ...b]), o = j({
		container: e,
		elements: R(r, !1),
		styles: a(),
		layout: "tree"
	});
	O(o);
	let c = s(() => o.style(a())), l = new L();
	l.setCy(o);
	let u = m(o, e), d = null;
	function p() {
		d &&= (o.getElementById(d).removeClass("selected"), null), F(o);
	}
	o.on("dbltap", "node", (e) => {
		l.toggleCollapse(e.target.id());
	}), o.on("tap", "node", (e) => {
		p(), d = e.target.id(), e.target.addClass("selected"), P(o, d);
	}), o.on("tap", "edge", (e) => {
		p(), I(o, e.target.id());
	}), o.on("tap", (e) => {
		e.target === o && p();
	});
	let h = i(e, {
		zoomIn: () => E(o, 1.2),
		zoomOut: () => E(o, 1 / 1.2),
		fit: () => {
			l.expandAll(), k(o);
		},
		exportPng: n.controls?.exportPng ? () => B(o, "parse-tree") : void 0
	});
	return {
		zoomIn: () => E(o, 1.2),
		zoomOut: () => E(o, 1 / 1.2),
		resetView: () => O(o),
		expandAll: () => l.expandAll(),
		exportPng: (e = "parse-tree") => B(o, e),
		resize: () => o.resize(),
		fitHeight: () => A(o),
		destroy: () => {
			o.scratch("_disposeWheel")?.(), h(), u(), c(), o.destroy();
		}
	};
}
//#endregion
export { V as mountParseTreeGraph };
