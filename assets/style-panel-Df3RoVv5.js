import { f as k, g as x, c as P, s as _, a as L, b as O, l as N, d as W } from "./index-CS50p2dJ.js";
const E = ["auto", "bitmap", "tinysdf", "msdf"], R = "infinite-chat.stylePanelCollapsed";
function $(e, o = document.body) {
  const l = () => e.refresh_fonts(), c = () => e.set_table_style({ ...x().tableRender });
  c();
  const s = p("div", ["font:11px/1.6 ui-monospace,Menlo,Consolas,monospace", "color:#cdd6f4", "background:rgba(17,20,28,.86)", "border:1px solid #313244", "border-radius:6px", "padding:8px 10px", "min-width:230px", "backdrop-filter:blur(4px)", "user-select:none"].join(";")), n = p("div", "");
  let a = localStorage.getItem(R) !== "0";
  const r = p("span", "color:#7f849c"), i = p("div", "display:flex;justify-content:space-between;align-items:center;cursor:pointer;font-weight:bold;color:#89b4fa;letter-spacing:.5px"), d = p("span", "");
  d.textContent = "style", i.append(d, r);
  const w = () => {
    n.style.display = a ? "none" : "", r.textContent = a ? "\u25B8" : "\u25BE";
  };
  i.onclick = () => {
    a = !a, localStorage.setItem(R, a ? "1" : "0"), w();
  };
  const S = (t) => {
    const g = x();
    _({ ...g, table: { ...g.table, ...t } }), l();
  }, u = (t) => {
    const g = x();
    _({ ...g, tableRender: { ...g.tableRender, ...t } }), c();
  }, f = () => x().tableRender, A = (t) => {
    L(t) && e.refresh_fonts();
  }, F = (t) => t === "auto" || t === "msdf";
  let T = "auto";
  const M = (t) => {
    T = t, W(t), F(t) && !O() && N(e).then(() => e.refresh_fonts()).catch((g) => console.error("[msdf] load failed", g)), e.set_glyph_mode(E.indexOf(t)), e.refresh_fonts();
  };
  n.append(m("Table \xB7 layout", [h("text align \u2195", [["top", "top"], ["center", "center"], ["bottom", "bottom"]], () => x().table.vAlign, (t) => S({ vAlign: t })), h("text align \u2194", [["auto", "auto (\u5217\u5BF9\u9F50)"], ["left", "left"], ["center", "center"], ["right", "right"]], () => x().table.hAlign, (t) => S({ hAlign: t }))]), m("Table \xB7 render", [y("line color", () => f().lineColor, (t) => u({ lineColor: t })), y("header fill", () => f().headerFill, (t) => u({ headerFill: t })), y("AO color", () => [...f().aoColor, 1], (t) => u({ aoColor: [t[0], t[1], t[2]] })), b("line width", 0, 4, 0.5, () => f().lineW, (t) => u({ lineW: t })), b("AO strength", 0, 0.6, 0.02, () => f().ao, (t) => u({ ao: t })), b("AO width", 0, 30, 1, () => f().aoWidth, (t) => u({ aoWidth: t })), b("corner radius", 0, 16, 1, () => f().radius, (t) => u({ radius: t }))]), m("Render \xB7 font", [h("font", k().map((t) => [t, t]), () => P(), A), h("glyph", E.map((t) => [t, t]), () => T, M)]), m("List", [], "\u2014\u2014 \u5F85\u63A5(\u6807\u8BB0/\u7F29\u8FDB/\u677E\u7D27)"), m("Div", [], "\u2014\u2014 \u5F85\u63A5(\u5BB9\u5668\u5185\u8FB9\u8DDD/\u5E95\u8272)")), s.append(i, n), o.appendChild(s), w();
}
function p(e, o) {
  const l = document.createElement(e);
  return l.style.cssText = o, l;
}
function v(e, o, l) {
  const c = p(e, o);
  return c.textContent = l, c;
}
function m(e, o, l) {
  const c = p("div", "margin-top:8px;border-top:1px solid #262b38;padding-top:6px");
  let s = true;
  const n = p("span", "color:#7f849c;margin-right:4px");
  n.textContent = "\u25BE";
  const a = p("div", "cursor:pointer;color:#a6adc8;font-weight:bold");
  a.append(n, document.createTextNode(e));
  const r = p("div", "margin-top:4px");
  for (const i of o) r.append(i);
  return l && r.append(v("div", "color:#7f849c;font-style:italic", l)), a.onclick = () => {
    s = !s, r.style.display = s ? "" : "none", n.textContent = s ? "\u25BE" : "\u25B8";
  }, c.append(a, r), c;
}
function C(e) {
  const o = p("div", "display:flex;justify-content:space-between;align-items:center;margin:3px 0;gap:8px");
  return o.append(v("span", "color:#7f849c;flex:1", e)), o;
}
function h(e, o, l, c) {
  const s = C(e), n = document.createElement("select");
  n.style.cssText = "font:11px ui-monospace,monospace;color:#cdd6f4;background:#313244;border:0;border-radius:4px;padding:2px 4px;cursor:pointer";
  const a = l();
  for (const [r, i] of o) {
    const d = document.createElement("option");
    d.value = r, d.textContent = i, d.selected = r === a, n.append(d);
  }
  return n.onchange = () => c(n.value), s.append(n), s;
}
function b(e, o, l, c, s, n) {
  const a = C(e), r = document.createElement("input");
  r.type = "range", r.min = String(o), r.max = String(l), r.step = String(c), r.value = String(s()), r.style.cssText = "width:90px";
  const i = v("span", "color:#cdd6f4;width:30px;text-align:right", String(s()));
  return r.oninput = () => {
    const d = Number(r.value);
    i.textContent = String(d), n(d);
  }, a.append(r, i), a;
}
function y(e, o, l) {
  const c = C(e), s = document.createElement("input");
  s.type = "color", s.value = j(o()), s.style.cssText = "width:28px;height:18px;padding:0;border:0;background:none;cursor:pointer";
  const n = document.createElement("input");
  n.type = "range", n.min = "0", n.max = "1", n.step = "0.05", n.value = String(o()[3]), n.title = "opacity", n.style.cssText = "width:64px";
  const a = () => {
    const [r, i, d] = G(s.value);
    l([r, i, d, Number(n.value)]);
  };
  return s.oninput = a, n.oninput = a, c.append(s, n), c;
}
function j(e) {
  const o = (l) => Math.max(0, Math.min(255, Math.round(l * 255))).toString(16).padStart(2, "0");
  return `#${o(e[0])}${o(e[1])}${o(e[2])}`;
}
function G(e) {
  const o = parseInt(e.slice(1), 16);
  return [(o >> 16 & 255) / 255, (o >> 8 & 255) / 255, (o & 255) / 255];
}
export {
  $ as mountStylePanel
};
