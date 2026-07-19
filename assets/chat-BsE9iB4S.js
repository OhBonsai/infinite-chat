var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
import { b as I, l as J, _ as O, c as U, s as T, a as M, d as A, __tla as __tla_0 } from "./boot-B4nH8qNC.js";
import { m as R } from "./pages-nav-BiSBOohm.js";
import { mountScriptedInput as H } from "./chat-input-DZ_x24Jy.js";
Promise.all([
  (() => {
    try {
      return __tla_0;
    } catch {
    }
  })()
]).then(async () => {
  function x(s) {
    return typeof s == "object" && s !== null && !Array.isArray(s);
  }
  function G(s) {
    if (!x(s)) return {
      ok: false,
      error: "\u5267\u672C\u5FC5\u987B\u662F\u5BF9\u8C61",
      index: -1
    };
    const t = x(s.meta) ? s.meta : {}, d = s.track;
    if (!Array.isArray(d)) return {
      ok: false,
      error: "\u7F3A track \u6570\u7EC4",
      index: -1
    };
    const l = [];
    for (let o = 0; o < d.length; o++) {
      const a = d[o];
      if (!x(a)) return {
        ok: false,
        error: "\u6307\u4EE4\u5FC5\u987B\u662F\u5BF9\u8C61",
        index: o
      };
      if (typeof a.dt != "number" || !Number.isFinite(a.dt) || a.dt < 0) return {
        ok: false,
        error: "dt \u5FC5\u987B\u662F \u22650 \u7684\u6709\u9650\u6570",
        index: o
      };
      const y = [
        "user",
        "event",
        "ask"
      ].filter((i) => a[i] !== void 0);
      if (y.length !== 1) return {
        ok: false,
        error: `\u6307\u4EE4\u987B\u6070\u542B user|event|ask \u4E4B\u4E00(\u73B0 ${y.length} \u4E2A)`,
        index: o
      };
      if (a.user !== void 0) {
        const i = a.user;
        if (!x(i) || typeof i.text != "string") return {
          ok: false,
          error: "user.text \u5FC5\u987B\u662F\u5B57\u7B26\u4E32",
          index: o
        };
        if (i.cps !== void 0 && (typeof i.cps != "number" || i.cps <= 0)) return {
          ok: false,
          error: "user.cps \u5FC5\u987B\u662F\u6B63\u6570",
          index: o
        };
      }
      if (a.event !== void 0) {
        const i = a.event;
        if (!x(i) || typeof i.type != "string" || i.type.length === 0) return {
          ok: false,
          error: "event.type \u5FC5\u987B\u662F\u975E\u7A7A\u5B57\u7B26\u4E32",
          index: o
        };
        if (i.properties !== void 0 && !x(i.properties)) return {
          ok: false,
          error: "event.properties \u5FC5\u987B\u662F\u5BF9\u8C61",
          index: o
        };
      }
      if (a.ask !== void 0) {
        const i = a.ask;
        if (!x(i)) return {
          ok: false,
          error: "ask \u5FC5\u987B\u662F\u5BF9\u8C61",
          index: o
        };
        const m = typeof i.allow == "boolean", f = Array.isArray(i.options) || typeof i.text == "string";
        if (!m && !f) return {
          ok: false,
          error: "ask \u987B\u542B allow(permission)\u6216 options/text(question)",
          index: o
        };
        if (i.options !== void 0 && (!Array.isArray(i.options) || i.options.some((r) => typeof r != "number"))) return {
          ok: false,
          error: "ask.options \u5FC5\u987B\u662F\u6570\u5B57\u6570\u7EC4",
          index: o
        };
      }
      l.push(a);
    }
    return {
      ok: true,
      script: {
        meta: t,
        track: l
      }
    };
  }
  function W(s, t = 1) {
    const d = t > 0 ? t : 1;
    let l = 0;
    return s.map((o) => (l += o.dt / d, {
      ...o,
      at: l
    }));
  }
  function X(s) {
    return s.length ? s[s.length - 1].at : 0;
  }
  class z {
    constructor(t, d, l = {}) {
      __publicField(this, "timeline");
      __publicField(this, "total");
      __publicField(this, "driver");
      __publicField(this, "virtual", 0);
      __publicField(this, "cursor", 0);
      __publicField(this, "playing", false);
      __publicField(this, "speed", 1);
      __publicField(this, "lastNow", null);
      __publicField(this, "gated", false);
      this.timeline = W(t.track, 1), this.total = X(this.timeline), this.driver = d;
    }
    duration() {
      return this.total;
    }
    position() {
      return this.virtual;
    }
    isPlaying() {
      return this.playing;
    }
    play() {
      this.playing = true, this.lastNow = null;
    }
    pause() {
      this.playing = false;
    }
    setSpeed(t) {
      t > 0 && (this.speed = t);
    }
    tick(t) {
      var _a, _b, _c, _d;
      if (!this.playing || this.gated) {
        this.lastNow = t;
        return;
      }
      this.lastNow === null && (this.lastNow = t), this.virtual += (t - this.lastNow) * this.speed, this.lastNow = t, this.fireDue(false), (_b = (_a = this.driver).onProgress) == null ? void 0 : _b.call(_a, Math.min(this.virtual, this.total), this.total), this.cursor >= this.timeline.length && (this.playing = false, (_d = (_c = this.driver).onDone) == null ? void 0 : _d.call(_c));
    }
    seekForward(t) {
      var _a, _b;
      t < this.virtual || (this.virtual = t, this.fireDue(true), (_b = (_a = this.driver).onProgress) == null ? void 0 : _b.call(_a, Math.min(this.virtual, this.total), this.total));
    }
    fireDue(t) {
      for (; this.cursor < this.timeline.length && this.timeline[this.cursor].at <= this.virtual; ) this.fire(this.timeline[this.cursor], t), this.cursor += 1;
    }
    fire(t, d) {
      if (t.user) {
        const l = this.driver.typeUser(t.user, d);
        l && !d && (this.gated = true, l.finally(() => {
          this.gated = false, this.lastNow = null;
        }));
      } else t.event ? this.driver.pushEvent(JSON.stringify({
        type: t.event.type,
        properties: t.event.properties ?? {}
      })) : t.ask && this.driver.ask(t.ask);
    }
  }
  const C = (s) => {
    const t = Math.floor(s / 1e3);
    return `${Math.floor(t / 60)}:${String(t % 60).padStart(2, "0")}`;
  }, Q = [
    0.5,
    1,
    2,
    4
  ];
  function V(s, t = document.body) {
    const d = t !== document.body, l = document.createElement("div");
    l.className = "chat-player", l.style.cssText = d ? "display:flex;flex-direction:column;gap:10px;font:12px/1 'JetBrains Mono',monospace;color:#a09b8c;user-select:none;" : "position:fixed;left:0;right:0;bottom:var(--input-h);z-index:9997;display:flex;align-items:center;gap:14px;padding:10px 18px;background:linear-gradient(#0d0f1700,#0d0f17cc);font:12px/1 'JetBrains Mono',monospace;color:#a09b8c;user-select:none;";
    const o = document.createElement("button");
    o.className = "chat-player-toggle", o.style.cssText = "all:unset;cursor:pointer;width:30px;height:30px;border-radius:50%;border:1px solid #c8aa6e66;color:#c8aa6e;text-align:center;line-height:30px;flex:0 0 auto;";
    const a = document.createElement("div");
    a.style.cssText = "position:relative;flex:1;height:18px;cursor:pointer;display:flex;align-items:center;";
    const y = document.createElement("div");
    y.style.cssText = "position:absolute;left:0;right:0;height:3px;background:#463714;border-radius:2px;";
    const i = document.createElement("div");
    i.style.cssText = "position:absolute;left:0;height:3px;width:0;background:#c8aa6e;border-radius:2px;";
    const m = document.createElement("div");
    m.style.cssText = "position:absolute;width:11px;height:11px;border-radius:50%;background:#c8aa6e;left:0;transform:translateX(-50%);", a.append(y, i, m);
    const f = document.createElement("span");
    f.style.cssText = "flex:0 0 auto;opacity:.7;min-width:74px;text-align:right;";
    const r = document.createElement("div");
    r.style.cssText = "display:flex;gap:6px;flex:0 0 auto;";
    let w = 1;
    const b = /* @__PURE__ */ new Map();
    for (const e of Q) {
      const n = document.createElement("button");
      n.style.cssText = "all:unset;cursor:pointer;padding:3px 7px;border-radius:5px;font-size:11px;color:#5b5a56;border:1px solid #ffffff14;", n.textContent = `${e}\xD7`, n.onclick = () => {
        w = e, s.setSpeed(e);
      }, b.set(e, n), r.appendChild(n);
    }
    if (d) {
      const e = document.createElement("div");
      e.style.cssText = "display:flex;align-items:center;gap:10px;flex-wrap:wrap;", f.style.minWidth = "0", e.append(o, f, r), l.append(e, a);
    } else l.append(o, a, f, r);
    t.appendChild(l), o.onclick = () => s.isPlaying() ? s.pause() : s.play();
    const E = (e) => {
      if (e >= s.position()) {
        s.seekForward(e);
        return;
      }
      const n = new URL(location.href);
      n.searchParams.set("at", String(Math.round(e))), location.assign(n.toString());
    }, S = (e) => {
      const n = a.getBoundingClientRect(), u = Math.max(0, Math.min(1, (e.clientX - n.left) / n.width));
      E(u * s.duration());
    };
    let h = false;
    a.addEventListener("mousedown", (e) => {
      h = true, S(e);
    }), window.addEventListener("mousemove", (e) => {
      h && S(e);
    }), window.addEventListener("mouseup", () => {
      h = false;
    });
    const k = () => {
      const e = s.duration(), n = e ? Math.min(1, s.position() / e) : 0;
      i.style.width = `${n * 100}%`, m.style.left = `${n * 100}%`, o.textContent = s.isPlaying() ? "\u23F8" : "\u25B6", f.textContent = `${C(Math.min(s.position(), e))} / ${C(e)}`;
      for (const [u, c] of b) {
        const p = w === u;
        c.style.color = p ? "#c8aa6e" : "#5b5a56", c.style.borderColor = p ? "#c8aa6e66" : "#ffffff14";
      }
      requestAnimationFrame(k);
    };
    requestAnimationFrame(k);
  }
  const K = 14, Y = 400;
  function P(s) {
    const t = document.createElement("div");
    t.style.cssText = "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;color:#ffb4b4;font:14px/1.6 system-ui;white-space:pre-wrap;padding:24px;", t.textContent = s, document.body.appendChild(t);
  }
  async function Z() {
    var _a;
    R("chat");
    const s = new URLSearchParams(location.search), t = s.get("script") ?? "showcase-full", d = Number(s.get("speed") ?? "") || 1, l = Number(s.get("at") ?? "") || 0, o = document.getElementById("stage"), a = Number(s.get("w") ?? "");
    a > 0 && o && (o.style.width = `${Math.round(a)}px`);
    {
      const e = document.getElementById("wslider"), n = document.getElementById("wval");
      if (e && n && o) {
        const u = Math.round(o.getBoundingClientRect().width) || 600;
        e.value = String(u), n.textContent = `${u}px`, e.addEventListener("input", () => {
          o.style.width = `${e.value}px`, n.textContent = `${e.value}px`;
        });
      }
    }
    const y = "/infinite-chat/";
    let i;
    try {
      const e = await fetch(`${y}chats/${t}.json`);
      if (!e.ok) throw new Error(`HTTP ${e.status}`);
      i = await e.json();
    } catch (e) {
      P(`\u5267\u672C\u8F7D\u5165\u5931\u8D25: chats/${t}.json
${String(e)}`);
      return;
    }
    const m = G(i);
    if (!m.ok) {
      P(`\u5267\u672C\u975E\u6CD5(\u6761\u76EE ${m.index}): ${m.error}`);
      return;
    }
    const f = m.script;
    f.meta.title && (document.title = `${f.meta.title} \xB7 infinite-chat`);
    const { chat: r } = await I({
      replay: [],
      rhythmPreset: "reader",
      glyphMode: 1
    });
    if (window.__layoutCounts = J, f.meta.theme) try {
      const e = f.meta.theme, n = typeof e == "string" ? await (await fetch(`${y}themes/${e}.json`)).text() : JSON.stringify(e);
      r.set_theme(n);
    } catch (e) {
      console.warn("[chat] \u4E3B\u9898\u8F7D\u5165\u5931\u8D25,\u7528\u9ED8\u8BA4", e);
    }
    {
      const e = document.getElementById("rhythm-sel");
      e && (e.value = "reader", e.addEventListener("change", () => r.set_reveal_preset(e.value)));
      const n = document.getElementById("effect-sel");
      n && (n.value = ((_a = r.effect_preset_name) == null ? void 0 : _a.call(r)) || "subtle", n.addEventListener("change", () => r.set_effect_preset(n.value)));
      const u = r;
      let c = 0;
      const p = async ($) => {
        var _a2, _b, _c, _d;
        const L = ++c;
        if ((_a2 = u.set_render_flavor) == null ? void 0 : _a2.call(u, $), $ === "tui") {
          try {
            const g = await (await fetch(`${y}themes/tui.json`)).text();
            r.set_theme(g);
          } catch (g) {
            console.warn("[chat] tui.json \u8F7D\u5165\u5931\u8D25", g);
          }
          T("mono"), M("tui"), A("msdf");
          const { loadMsdf: N, msdfLoaded: F } = await O(async () => {
            const { loadMsdf: g, msdfLoaded: j } = await import("./boot-B4nH8qNC.js").then(async (m2) => {
              await m2.__tla;
              return m2;
            }).then((D) => D.o);
            return {
              loadMsdf: g,
              msdfLoaded: j
            };
          }, []);
          if (F() || await N(r).catch((g) => console.warn("[chat] LXGW MSDF \u8F7D\u5165\u5931\u8D25,\u9000\u7CFB\u7EDF\u7B49\u5BBD", g)), L !== c) return;
          r.set_glyph_mode(3);
          const { cellW: q, cellH: B } = U();
          (_b = u.set_cell_metrics) == null ? void 0 : _b.call(u, q, B), r.set_effect_preset("off"), r.set_reveal_preset("typewriter");
        } else r.set_theme("{}"), T("system"), M("rich"), A("bitmap"), r.set_glyph_mode(1), (_c = u.set_cell_metrics) == null ? void 0 : _c.call(u, 0, 0), r.set_effect_preset("subtle"), r.set_reveal_preset("reader");
        (_d = u.refresh_fonts) == null ? void 0 : _d.call(u), e && (e.value = $ === "tui" ? "typewriter" : "reader"), n && (n.value = $ === "tui" ? "off" : "subtle");
      }, v = document.getElementById("flavor-sel"), _ = s.has("tui");
      v && (v.value = _ ? "tui" : "rich", v.addEventListener("change", () => void p(v.value))), _ && p("tui");
    }
    const w = H(o ?? document.body), b = (e, n = 0) => {
      try {
        const c = JSON.parse(r.ask_hit_targets()).find((p) => p.id === (e ? "allow" : "deny"));
        if (c) {
          r.tap(c.x + c.w / 2, c.y + c.h / 2);
          return;
        }
      } catch {
      }
      n < 120 ? requestAnimationFrame(() => b(e, n + 1)) : console.warn("[chat] permission \u6309\u94AE\u672A\u51FA\u73B0");
    }, E = (e, n, u = 0) => {
      var _a2;
      const c = document.querySelector(".ask-form");
      if (c) {
        for (const v of e) {
          const _ = c.querySelector(`.ask-option[value="${v}"]`);
          _ && (_.checked = true);
        }
        const p = c.querySelector(".ask-text");
        p && (p.value = n), (_a2 = c.querySelector(".ask-submit")) == null ? void 0 : _a2.click();
        return;
      }
      u < 120 ? requestAnimationFrame(() => E(e, n, u + 1)) : console.warn("[chat] question \u8868\u5355\u672A\u51FA\u73B0");
    }, S = {
      typeUser: async (e, n) => {
        const u = e.cps ?? K, c = e.holdMs ?? Y;
        await w.typeText(e.text, u * d, n), n || await new Promise((p) => setTimeout(p, c / d)), w.flashSend(), r.note_send();
      },
      pushEvent: (e) => r.push_event(e),
      ask: (e) => {
        typeof e.allow == "boolean" ? b(e.allow) : E(e.options ?? [], e.text ?? "");
      }
    }, h = new z(f, S);
    h.setSpeed(d), V(h, document.getElementById("panel-player") ?? document.body), window.__player = h, l > 0 && (r.set_stream_rate(1e9), h.seekForward(l), setTimeout(() => r.set_stream_rate(200), 500)), h.play();
    const k = (e) => {
      h.tick(e), requestAnimationFrame(k);
    };
    requestAnimationFrame(k), console.info(`[chat] \u5267\u672C "${t}" \xB7 ${f.track.length} \u6761\u6307\u4EE4 \xB7 ${Math.round(h.duration() / 1e3)}s`);
  }
  Z();
});
