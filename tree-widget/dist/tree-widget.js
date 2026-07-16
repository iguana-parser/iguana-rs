import e from "cytoscape";
import t from "cytoscape-tidytree";
//#region ../web-ui/src/graph-controls.ts
var n = "width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"", r = {
	zoomIn: `<svg ${n}><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35M11 8v6M8 11h6"/></svg>`,
	zoomOut: `<svg ${n}><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35M8 11h6"/></svg>`,
	fit: `<svg ${n}><path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2"/></svg>`,
	expandAll: `<svg ${n}><path d="m15 15 6 6m0 0v-4.8m0 4.8h-4.8M9 9 3 3m0 0v4.8M3 3h4.8M15 9l6-6m0 0v4.8M21 3h-4.8M9 15l-6 6m0 0v-4.8M3 21h4.8"/></svg>`,
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
	return i("Zoom in", r.zoomIn, t.zoomIn), i("Zoom out", r.zoomOut, t.zoomOut), i("Fit to view", r.fit, t.fit), t.expandAll && i("Expand all (double-click a node to collapse)", r.expandAll, t.expandAll), t.exportPng && i("Export as PNG", r.exportPng, t.exportPng), t.popOut && i("Pop out", r.popOut, t.popOut), e.appendChild(n), () => n.remove();
}
function a(e, t = 20) {
	return e.length <= t ? e : e.substring(0, t - 3) + "...";
}
function o(e, t) {
	let n = document.createElement("div");
	n.className = "graph-tooltip", n.style.cssText = "\n    position: fixed;\n    background: #252526;\n    border: 1px solid #454545;\n    border-radius: 4px;\n    padding: 6px 10px;\n    font-size: 11px;\n    font-family: ui-monospace, SFMono-Regular, \"SF Mono\", Menlo, monospace;\n    color: #d4d4d4;\n    pointer-events: none;\n    z-index: 10000;\n    display: none;\n    max-width: 400px;\n    word-wrap: break-word;\n    white-space: pre-wrap;\n    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);\n  ", document.body.appendChild(n);
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
var s = {
	label: "data(label)",
	"text-valign": "center",
	"text-halign": "center",
	"font-size": "9px",
	color: "#d4d4d4",
	"border-width": 1,
	width: "label",
	shape: "round-rectangle"
}, c = {
	width: 1,
	"line-color": "#555",
	"target-arrow-color": "#555",
	"target-arrow-shape": "triangle",
	"curve-style": "bezier",
	"arrow-scale": .8
}, l = {
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
}, u = [
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
], d = [
	...u,
	{
		selector: "node",
		style: {
			...s,
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
			"background-color": l.nonterminal.bg,
			"border-color": l.nonterminal.border
		}
	},
	{
		selector: "node.intermediate, node[kind='Intermediate']",
		style: {
			"background-color": l.intermediate.bg,
			"border-color": l.intermediate.border,
			shape: "rectangle"
		}
	},
	{
		selector: "node.terminal, node[kind='Terminal']",
		style: {
			"background-color": l.terminal.bg,
			"border-color": l.terminal.border
		}
	},
	{
		selector: "node.token, node[kind='Token']",
		style: {
			"background-color": l.terminal.bg,
			"border-color": l.terminal.border
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
			"background-color": l.nonterminal.selectedBg,
			"border-color": l.nonterminal.selectedBorder
		}
	},
	{
		selector: "node.intermediate.selected, node[kind='Intermediate'].selected",
		style: {
			"background-color": l.intermediate.selectedBg,
			"border-color": l.intermediate.selectedBorder
		}
	},
	{
		selector: "node.terminal.selected, node[kind='Terminal'].selected",
		style: {
			"background-color": l.terminal.selectedBg,
			"border-color": l.terminal.selectedBorder
		}
	},
	{
		selector: "node.token.selected, node[kind='Token'].selected",
		style: {
			"background-color": l.terminal.selectedBg,
			"border-color": l.terminal.selectedBorder
		}
	},
	{
		selector: "node.packed.selected, node[kind='Packed'].selected",
		style: {
			"background-color": l.packed.selectedBg,
			"border-width": 2,
			"border-color": l.packed.selectedBorder
		}
	},
	{
		selector: "node.ambiguous, node.amb",
		style: {
			"background-color": l.ambiguous.bg,
			"border-color": l.ambiguous.border
		}
	},
	{
		selector: "node.ambiguous.selected, node.amb.selected",
		style: {
			"background-color": l.ambiguous.selectedBg,
			"border-color": l.ambiguous.selectedBorder
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
	...u,
	({ ...s }, l.nonterminal.bg, l.nonterminal.border),
	l.nonterminal.border
];
var f = [
	{
		selector: "edge",
		style: { ...c }
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
			"line-color": l.nonterminal.selectedBorder,
			"target-arrow-color": l.nonterminal.selectedBorder
		}
	},
	{
		selector: "edge.edge-selected-intermediate",
		style: {
			"line-color": l.intermediate.selectedBorder,
			"target-arrow-color": l.intermediate.selectedBorder
		}
	},
	{
		selector: "edge.edge-selected-terminal",
		style: {
			"line-color": l.terminal.selectedBorder,
			"target-arrow-color": l.terminal.selectedBorder
		}
	},
	{
		selector: "edge.edge-selected-packed",
		style: {
			"line-color": l.packed.selectedBorder,
			"target-arrow-color": l.packed.selectedBorder
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
			"line-color": l.ambiguous.selectedBorder,
			"target-arrow-color": l.ambiguous.selectedBorder
		}
	},
	{
		selector: "edge.edge-ambiguous",
		style: {
			"line-color": l.ambiguous.border,
			"target-arrow-color": l.ambiguous.border
		}
	}
];
({ ...c });
var p = {
	name: "tidytree",
	direction: "TB",
	horizontalSpacing: 16,
	verticalSpacing: 30,
	nodeDimensionsIncludeLabels: !0,
	fit: !0,
	padding: 30
};
function m(e) {
	e.zoom() > 1 && (e.zoom(1), e.center());
}
function h(e, t) {
	e && e.zoom(e.zoom() * t);
}
function g(e) {
	e && (e.fit(), m(e));
}
function _(t) {
	let { container: n, elements: r, styles: i, layout: a = "sppf", viewport: o } = t, s = r.filter((e) => e.data.source === void 0).length, c = a === "tree" ? p : {
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
	return o ? (u.zoom(o.zoom), u.pan(o.pan)) : m(u), u;
}
var v = [
	"edge-selected-nonterminal",
	"edge-selected-intermediate",
	"edge-selected-terminal",
	"edge-selected-packed",
	"edge-selected-ambiguous",
	"edge-clicked"
];
function y(e) {
	if (e.data("ambiguous") || e.hasClass("ambiguous")) return "edge-selected-ambiguous";
	let t = e.data("kind");
	return t === "Packed" || e.hasClass("packed") ? "edge-selected-packed" : t === "Nonterminal" || e.hasClass("nonterminal") ? "edge-selected-nonterminal" : t === "Intermediate" || e.hasClass("intermediate") ? "edge-selected-intermediate" : t === "Terminal" || t === "Token" || e.hasClass("terminal") || e.hasClass("token") ? "edge-selected-terminal" : "edge-selected-nonterminal";
}
function b(e, t) {
	let n = e.getElementById(t);
	if (n.empty()) return;
	let r = y(n);
	n.outgoers("edge").addClass(r);
}
function x(e) {
	e.edges().removeClass(v);
}
function S(e, t) {
	x(e), e.getElementById(t).addClass("edge-clicked");
}
//#endregion
//#region ../web-ui/src/parse-tree-graph.ts
var C = class {
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
		return e.length > 0 ? e.first().id() : null;
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
		let t = !1, n = e;
		for (; n !== null;) {
			let e = this.cy.getElementById(n);
			if (e.length === 0) break;
			let r = e.incomers("node");
			if (r.length === 0) break;
			let i = r.first(), a = i.id();
			this.collapsedNodes.has(a) && (this.collapsedNodes.delete(a), i.removeClass("collapsed"), t = !0), n = a;
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
function w(e, t) {
	let n = e.nodes.map((e) => {
		let n = `(${e.start}, ${e.end})`, r = t ? `${a(e.label, 20)}\n${n}` : a(e.label, 20), i = t ? `${e.label}\n${n}` : e.label;
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
function T(e) {
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
async function E(e, t) {
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
function D(e, t, n = {}) {
	let r = _({
		container: e,
		elements: w(T(t), !1),
		styles: [...d, ...f],
		layout: "tree"
	}), a = new C();
	a.setCy(r);
	let s = o(r, e), c = null;
	function l() {
		c &&= (r.getElementById(c).removeClass("selected"), null), x(r);
	}
	r.on("dbltap", "node", (e) => {
		a.toggleCollapse(e.target.id());
	}), r.on("tap", "node", (e) => {
		l(), c = e.target.id(), e.target.addClass("selected"), b(r, c);
	}), r.on("tap", "edge", (e) => {
		l(), S(r, e.target.id());
	}), r.on("tap", (e) => {
		e.target === r && l();
	});
	let u = i(e, {
		zoomIn: () => h(r, 1.2),
		zoomOut: () => h(r, 1 / 1.2),
		fit: () => g(r),
		expandAll: n.controls?.expandAll ? () => a.expandAll() : void 0,
		exportPng: n.controls?.exportPng ? () => E(r, "parse-tree") : void 0
	});
	return {
		zoomIn: () => h(r, 1.2),
		zoomOut: () => h(r, 1 / 1.2),
		resetView: () => g(r),
		expandAll: () => a.expandAll(),
		exportPng: (e = "parse-tree") => E(r, e),
		resize: () => r.resize(),
		destroy: () => {
			r.scratch("_disposeWheel")?.(), u(), s(), r.destroy();
		}
	};
}
//#endregion
export { D as mountParseTreeGraph };
