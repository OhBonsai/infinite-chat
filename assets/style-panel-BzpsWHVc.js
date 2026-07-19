import { g as i, T as $, M as D, R as B, f as q, h as b, i as z, j as Q, s as U, d as V, k as X, n as Z } from "./boot-B4nH8qNC.js";
const F = ["auto", "bitmap", "tinysdf", "msdf"], H = "infinite-chat.stylePanelCollapsed";
function oe(t, n = document.body) {
  const a = () => t.refresh_fonts(), c = () => t.set_table_style({ ...i().tableRender }), l = () => t.set_theme(JSON.stringify(i().theme)), o = () => t.set_motion(JSON.stringify(i().motion)), p = () => {
    const e = i();
    e.rhythmPreset && t.set_reveal_preset(e.rhythmPreset), Object.keys(e.rhythm).length && t.set_rhythm(JSON.stringify(e.rhythm)), t.restart_reveal();
  };
  c(), Object.keys(i().theme).length && l(), Object.keys(i().motion).length && o(), (i().rhythmPreset || Object.keys(i().rhythm).length) && p();
  const r = m("div", ["font:11px/1.6 ui-monospace,Menlo,Consolas,monospace", "color:#cdd6f4", "background:rgba(17,20,28,.86)", "border:1px solid #313244", "border-radius:6px", "padding:8px 10px", "min-width:230px", "backdrop-filter:blur(4px)", "user-select:none"].join(";")), f = m("div", "");
  let d = localStorage.getItem(H) !== "0";
  const M = m("span", "color:#7f849c"), v = m("div", "display:flex;justify-content:space-between;align-items:center;cursor:pointer;font-weight:bold;color:#89b4fa;letter-spacing:.5px"), R = m("span", "");
  R.textContent = "style", v.append(R, M);
  const N = () => {
    f.style.display = d ? "none" : "", M.textContent = d ? "\u25B8" : "\u25BE";
  };
  v.onclick = () => {
    d = !d, localStorage.setItem(H, d ? "1" : "0"), N();
  };
  const P = (e) => {
    const s = i();
    b({ ...s, table: { ...s.table, ...e } }), a();
  }, y = (e) => {
    const s = i();
    b({ ...s, tableRender: { ...s.tableRender, ...e } }), c();
  }, x = () => i().tableRender, L = (e, s) => {
    const u = i();
    b({ ...u, theme: { ...u.theme, [e]: s } }), l();
  }, G = (e, s) => i().theme[e] ?? s, I = (e) => {
    U(e) && t.refresh_fonts();
  }, K = (e) => e === "auto" || e === "msdf";
  let j = "auto";
  const W = (e) => {
    j = e, V(e), K(e) && !X() && Z(t).then(() => t.refresh_fonts()).catch((s) => console.error("[msdf] load failed", s)), t.set_glyph_mode(F.indexOf(e)), t.refresh_fonts();
  }, Y = (e, s) => {
    const u = i();
    b({ ...u, motion: { ...u.motion, [e]: s } }), o();
  }, J = (e, s) => i().motion[e] ?? s;
  f.append(g("Theme", $.map(([e, s, u]) => T(s, () => G(e, u), (S) => L(e, S)))), g("Motion", D.map(([e, s, u, S, w, C]) => _(s, S, w, C, () => J(e, u), (O) => Y(e, O)))), g("Rhythm", [h("preset", z.map((e) => [e, e]), () => i().rhythmPreset || "reader", (e) => {
    const s = i();
    b({ ...s, rhythmPreset: e, rhythm: {} }), p();
  }), ...B.map(([e, s, u, S, w, C]) => _(s, S, w, C, () => i().rhythm[e] ?? u, (O) => {
    const A = i();
    b({ ...A, rhythm: { ...A.rhythm, [e]: O } }), p();
  }))]), g("Effects", [h("preset", [["subtle", "subtle(\u9ED8\u8BA4)"], ["off", "off(\u5168\u6052\u7B49)"], ["expressive", "expressive"]], () => "subtle", (e) => t.set_effect_preset(e)), h("exit dissolve", [["off", "off"], ["300", "300ms"], ["600", "600ms"]], () => "off", (e) => t.set_exit_dissolve_ms(e === "off" ? 0 : Number(e))), h("post fx", [["off", "off"], ["vignette", "vignette .35"], ["grain", "grain .12"], ["chroma", "chroma 2px"], ["all", "\u4E09\u4EF6\u5168\u5F00"]], () => "off", (e) => t.set_post_params(e === "vignette" || e === "all" ? 0.35 : 0, e === "grain" || e === "all" ? 0.12 : 0, e === "chroma" || e === "all" ? 2 : 0)), h("feedback trail", [["off", "off"], ["0.7", "decay 0.7"], ["0.85", "decay 0.85"]], () => "off", (e) => t.set_feedback_decay(e === "off" ? 0 : Number(e))), h("enter curve", [["default", "default"], ["spring", "spring(k6 w12)"]], () => "default", (e) => t.set_spring_enter(e === "spring"))], "\u8DDD\u79BB\u5E26\u4E09\u4EF6/\u566A\u58F0\u56DB\u8C61\u9650\u89C1 ?gallery \u6F14\u793A\u6761"), g("Table \xB7 layout", [h("text align \u2195", [["top", "top"], ["center", "center"], ["bottom", "bottom"]], () => i().table.vAlign, (e) => P({ vAlign: e })), h("text align \u2194", [["auto", "auto (\u5217\u5BF9\u9F50)"], ["left", "left"], ["center", "center"], ["right", "right"]], () => i().table.hAlign, (e) => P({ hAlign: e }))]), g("Table \xB7 render", [T("line color", () => x().lineColor, (e) => y({ lineColor: e })), T("header fill", () => x().headerFill, (e) => y({ headerFill: e })), T("AO color", () => [...x().aoColor, 1], (e) => y({ aoColor: [e[0], e[1], e[2]] })), _("line width", 0, 4, 0.5, () => x().lineW, (e) => y({ lineW: e })), _("AO strength", 0, 0.6, 0.02, () => x().ao, (e) => y({ ao: e })), _("AO width", 0, 30, 1, () => x().aoWidth, (e) => y({ aoWidth: e })), _("corner radius", 0, 16, 1, () => x().radius, (e) => y({ radius: e }))]), g("Render \xB7 font", [h("font", q().map((e) => [e, e]), () => Q(), I), h("glyph", F.map((e) => [e, e]), () => j, W)]), g("List", [], "\u2014\u2014 \u5F85\u63A5(\u6807\u8BB0/\u7F29\u8FDB/\u677E\u7D27)"), g("Div", [], "\u2014\u2014 \u5F85\u63A5(\u5BB9\u5668\u5185\u8FB9\u8DDD/\u5E95\u8272)")), r.append(v, f), n.appendChild(r), N();
}
function m(t, n) {
  const a = document.createElement(t);
  return a.style.cssText = n, a;
}
function k(t, n, a) {
  const c = m(t, n);
  return c.textContent = a, c;
}
function g(t, n, a) {
  const c = m("div", "margin-top:8px;border-top:1px solid #262b38;padding-top:6px");
  let l = true;
  const o = m("span", "color:#7f849c;margin-right:4px");
  o.textContent = "\u25BE";
  const p = m("div", "cursor:pointer;color:#a6adc8;font-weight:bold");
  p.append(o, document.createTextNode(t));
  const r = m("div", "margin-top:4px");
  for (const f of n) r.append(f);
  return a && r.append(k("div", "color:#7f849c;font-style:italic", a)), p.onclick = () => {
    l = !l, r.style.display = l ? "" : "none", o.textContent = l ? "\u25BE" : "\u25B8";
  }, c.append(p, r), c;
}
function E(t) {
  const n = m("div", "display:flex;justify-content:space-between;align-items:center;margin:3px 0;gap:8px");
  return n.append(k("span", "color:#7f849c;flex:1", t)), n;
}
function h(t, n, a, c) {
  const l = E(t), o = document.createElement("select");
  o.style.cssText = "font:11px ui-monospace,monospace;color:#cdd6f4;background:#313244;border:0;border-radius:4px;padding:2px 4px;cursor:pointer";
  const p = a();
  for (const [r, f] of n) {
    const d = document.createElement("option");
    d.value = r, d.textContent = f, d.selected = r === p, o.append(d);
  }
  return o.onchange = () => c(o.value), l.append(o), l;
}
function _(t, n, a, c, l, o) {
  const p = E(t), r = document.createElement("input");
  r.type = "range", r.min = String(n), r.max = String(a), r.step = String(c), r.value = String(l()), r.style.cssText = "width:90px";
  const f = k("span", "color:#cdd6f4;width:30px;text-align:right", String(l()));
  return r.oninput = () => {
    const d = Number(r.value);
    f.textContent = String(d), o(d);
  }, p.append(r, f), p;
}
function T(t, n, a) {
  const c = E(t), l = document.createElement("input");
  l.type = "color", l.value = ee(n()), l.style.cssText = "width:28px;height:18px;padding:0;border:0;background:none;cursor:pointer";
  const o = document.createElement("input");
  o.type = "range", o.min = "0", o.max = "1", o.step = "0.05", o.value = String(n()[3]), o.title = "opacity", o.style.cssText = "width:64px";
  const p = () => {
    const [r, f, d] = te(l.value);
    a([r, f, d, Number(o.value)]);
  };
  return l.oninput = p, o.oninput = p, c.append(l, o), c;
}
function ee(t) {
  const n = (a) => Math.max(0, Math.min(255, Math.round(a * 255))).toString(16).padStart(2, "0");
  return `#${n(t[0])}${n(t[1])}${n(t[2])}`;
}
function te(t) {
  const n = parseInt(t.slice(1), 16);
  return [(n >> 16 & 255) / 255, (n >> 8 & 255) / 255, (n & 255) / 255];
}
export {
  oe as mountStylePanel
};
