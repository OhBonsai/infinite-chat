var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
import { e as W, _ as Y, __tla as __tla_0 } from "./boot-B4nH8qNC.js";
import { F as J } from "./director-CgSS3eI1.js";
Promise.all([
  (() => {
    try {
      return __tla_0;
    } catch {
    }
  })()
]).then(async () => {
  const V = {
    title: {
      eyebrow: "RUST \xB7 WASM \xB7 WEBGPU \xB7 SDF",
      title: "INFINITE CHAT",
      note: "\u6587\u5B57\u5373\u56FE\u5143\u7684\u5BF9\u8BDD\u6E32\u67D3\u5F15\u64CE"
    },
    claim: {
      eyebrow: "\u662F\u4EC0\u4E48 \xB7 WHAT"
    },
    table: {
      eyebrow: "MARKDOWN \u5899 \xB7 \u5C40\u90E8",
      link: {
        href: "components/",
        text: "\u5168\u90E8\u7EC4\u4EF6 \u2192"
      }
    },
    card: {
      eyebrow: "AGENT \u5361\u5899 \xB7 \u5C40\u90E8",
      link: {
        href: "components/",
        text: "\u5168\u90E8\u7EC4\u4EF6 \u2192"
      }
    },
    bloom: {
      eyebrow: "\u4E0A\u9650\u79C0",
      title: "\u6BCF\u4E2A\u5B57\u90FD\u662F\u4E00\u7B49\u56FE\u5143"
    },
    door: {}
  }, N = [
    {
      id: "title",
      title: "\u6807\u9898",
      durationMs: 6e3,
      enter: (e) => {
        e.call("set_effect_preset", "expressive"), e.call("set_reveal_preset", "reader");
      }
    },
    {
      id: "claim",
      title: "\u4E3B\u5F20",
      durationMs: 6e3,
      enter: (e) => {
        e.call("set_effect_preset", "subtle"), e.call("set_reveal_preset", "typewriter"), e.call("set_reveal_cps", 38);
      }
    },
    {
      id: "table",
      title: "\u8868\u683C",
      durationMs: 7e3,
      enter: (e) => {
        e.call("set_effect_preset", "subtle"), e.call("set_reveal_preset", "typewriter"), e.call("set_reveal_cps", 44);
      }
    },
    {
      id: "card",
      title: "\u5361\u7247",
      durationMs: 7e3,
      enter: (e) => {
        e.call("set_effect_preset", "subtle"), e.call("set_reveal_preset", "typewriter"), e.call("set_reveal_cps", 60);
      },
      cues: [
        {
          at: 2e3,
          fn: (e) => e.call("tap_fold_first")
        },
        {
          at: 4500,
          fn: (e) => e.call("tap_fold_first")
        }
      ]
    },
    {
      id: "bloom",
      title: "\u7EFD\u653E",
      durationMs: 1e4,
      enter: (e) => {
        e.call("set_effect_preset", "expressive");
      }
    },
    {
      id: "door",
      title: "\u5165\u53E3",
      durationMs: 6e3,
      enter: (e) => {
        e.call("set_effect_preset", "subtle");
      }
    }
  ], z = {
    title: 1,
    claim: 1.45,
    table: 1.18,
    card: 1.12,
    bloom: 1,
    door: 1
  }, p = "home", g = "home-slide", d = (e) => ({
    kind: "text",
    text: e
  }), X = {
    table: {
      row_h: 132,
      tiles: [
        {
          id: "h-head",
          span: [
            2,
            1
          ],
          title: "HEADINGS",
          content: [
            d(`# \u5148\u9AA8\u67B6
## \u518D\u6587\u5B57`)
          ]
        },
        {
          id: "h-emph",
          span: [
            2,
            1
          ],
          title: "RICH",
          content: [
            d("**\u7C97** \xB7 *\u659C* \xB7 `\u7801` \xB7 [\u94FE\u63A5](markdown/)")
          ]
        },
        {
          id: "h-table",
          span: [
            4,
            2
          ],
          title: "TABLE \xB7 \u6D41\u5F0F",
          content: [
            d(`| \u80FD\u529B | \u624B\u6CD5 | \u89C4\u6A21 |
| --- | --- | --- |
| \u6587\u5B57 | SDF / MSDF | \u4EFB\u610F\u7F29\u653E\u9510\u5229 |
| \u6D41\u5F0F | \u9010\u5B57 reveal | \u5168\u7A0B\u65E0\u8DF3\u53D8 |
| \u5386\u53F2 | settled \u6D3E\u7ED8 | 100+ \u8F6E\u4E1D\u6ED1 |`)
          ]
        }
      ]
    },
    card: {
      row_h: 132,
      tiles: [
        {
          id: "h-user",
          span: [
            2,
            1
          ],
          title: "USER",
          content: [
            d("\u628A diff \u5361\u505A\u6210\u53EF\u70B9\u5F00\u7684 tile\u3002")
          ]
        },
        {
          id: "h-asst",
          span: [
            2,
            1
          ],
          title: "ASSISTANT",
          content: [
            d("\u5DF2\u63A5\u65E2\u6709\u6298\u53E0\u8DEF,**\u70B9\u6807\u9898**\u5C55\u5F00\u4E0A\u4E0B\u6587\u3002")
          ]
        },
        {
          id: "h-diff",
          span: [
            4,
            2
          ],
          title: "TOOL \xB7 EDIT",
          content: [
            {
              kind: "tool",
              tool: "edit",
              state: {
                status: "completed",
                input: {
                  path: "src/auth/login.ts"
                },
                metadata: {
                  filediff: `@@ -12,3 +12,4 @@
 ctx const TIMEOUT = 1000;
-for (let i = 0; i < 5; i++) { await fetchToken({ timeout: TIMEOUT }); }
+for (let i = 0; i < 3; i++) {
+  await fetchToken({ timeout: TIMEOUT * 2 ** i }); // \u6307\u6570\u9000\u907F
+}
`
                }
              }
            }
          ]
        }
      ]
    }
  }, j = {
    title: [
      {
        kind: "text",
        text: "# \u6587\u5B57,\u662F\u6D3B\u7684\u56FE\u5143\u3002"
      }
    ],
    claim: [
      {
        kind: "text",
        text: `# \u6BCF\u4E2A\u5B57\u90FD\u662F\u4E00\u7B49\u56FE\u5143

## GPU \u4E0A\u7684 SDF \u5B9E\u4F8B \u2014\u2014 \u53EF\u63ED\u793A \xB7 \u53EF\u8C03\u5EA6 \xB7 \u53EF\u98DE\u884C

## fps \u4E0E\u5185\u5B58,\u53EA\u968F\u53EF\u89C1\u7684\u4E00\u5C4F`
      }
    ],
    table: [
      {
        kind: "text",
        text: `## \u5148\u9AA8\u67B6,\u518D\u6587\u5B57

| \u80FD\u529B | \u624B\u6CD5 | \u89C4\u6A21 |
| --- | --- | --- |
| \u6587\u5B57 | SDF / MSDF | \u4EFB\u610F\u7F29\u653E\u9510\u5229 |
| \u6D41\u5F0F | \u9010\u5B57 reveal | \u5168\u7A0B\u65E0\u8DF3\u53D8 |
| \u5386\u53F2 | settled \u6D3E\u7ED8 | 100+ \u8F6E\u4E1D\u6ED1 |

\u8D28\u80FD\u65B9\u7A0B $E = mc^2$`
      }
    ],
    card: [
      {
        kind: "tool",
        tool: "edit",
        state: {
          status: "completed",
          input: {
            path: "src/auth/login.ts"
          },
          metadata: {
            filediff: `@@ -12,3 +12,4 @@
 ctx const TIMEOUT = 1000;
-for (let i = 0; i < 5; i++) { await fetchToken({ timeout: TIMEOUT }); }
+for (let i = 0; i < 3; i++) {
+  await fetchToken({ timeout: TIMEOUT * 2 ** i }); // \u6307\u6570\u9000\u907F
+}
`
          }
        }
      }
    ],
    bloom: [
      {
        kind: "text",
        text: `# \u6BCF\u4E2A\u5B57\u90FD\u662F\u4E00\u7B49\u56FE\u5143

## GPU \u4E0A\u7684 SDF \u5B9E\u4F8B \u2014\u2014 \u53EF\u63ED\u793A \xB7 \u53EF\u8C03\u5EA6 \xB7 \u53EF\u98DE\u884C

## fps \u4E0E\u5185\u5B58,\u53EA\u968F\u53EF\u89C1\u7684\u4E00\u5C4F`
      }
    ],
    door: []
  };
  class K {
    constructor(s) {
      __publicField(this, "live", []);
      __publicField(this, "tileLive", /* @__PURE__ */ new Map());
      __publicField(this, "seq", 0);
      __publicField(this, "built", false);
      var _a, _b;
      this.chat = s, (_b = (_a = this.chat).set_focus_valign) == null ? void 0 : _b.call(_a, 0.42);
    }
    pushPart(s, t, i) {
      var _a, _b;
      const n = i.kind === "text" ? {
        type: "text",
        id: t,
        messageID: s,
        sessionID: p,
        text: i.text
      } : {
        type: "tool",
        id: t,
        messageID: s,
        sessionID: p,
        tool: i.tool,
        state: i.state
      };
      (_b = (_a = this.chat).push_event) == null ? void 0 : _b.call(_a, JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: n,
          time: 1
        }
      }));
    }
    removePart(s, t) {
      var _a, _b;
      (_b = (_a = this.chat).push_event) == null ? void 0 : _b.call(_a, JSON.stringify({
        type: "message.part.removed",
        properties: {
          sessionID: p,
          messageID: s,
          partID: t
        }
      }));
    }
    clearTiles() {
      for (const [s, t] of this.tileLive) for (const i of t) this.removePart(s, i);
      this.tileLive.clear();
    }
    build(s) {
      var _a, _b, _c, _d, _e, _f, _g, _h, _i, _j, _k, _l, _m, _n;
      this.built || ((_b = (_a = this.chat).push_event) == null ? void 0 : _b.call(_a, JSON.stringify({
        type: "message.updated",
        properties: {
          info: {
            id: g,
            role: "assistant",
            sessionID: p
          }
        }
      })), this.built = true);
      const t = X[s];
      if (t) {
        for (const o of this.live) this.removePart(g, o);
        this.live = [], this.clearTiles();
        for (const o of t.tiles) {
          (_d = (_c = this.chat).push_event) == null ? void 0 : _d.call(_c, JSON.stringify({
            type: "message.updated",
            properties: {
              info: {
                id: o.id,
                role: "assistant",
                sessionID: p
              }
            }
          }));
          const l = [];
          for (const c of [
            d(`**${o.title}**`),
            ...o.content
          ]) {
            const h = `${o.id}-${this.seq++}`;
            this.pushPart(o.id, h, c), l.push(h);
          }
          this.tileLive.set(o.id, l);
        }
        const r = {
          cols: 4,
          gap: 16,
          row_h: t.row_h,
          pad: 12,
          title_h: 24,
          tiles: t.tiles.map((o) => ({
            id: o.id,
            span: o.span,
            title: o.title
          }))
        };
        (_f = (_e = this.chat).set_tile_spec) == null ? void 0 : _f.call(_e, JSON.stringify(r)), (_h = (_g = this.chat).set_focus_zoom) == null ? void 0 : _h.call(_g, 1), (_j = (_i = this.chat).set_reveal_cps) == null ? void 0 : _j.call(_i, 1e9), s === "card" && window.setTimeout(() => {
          var _a2, _b2;
          return (_b2 = (_a2 = this.chat).tap_fold_first) == null ? void 0 : _b2.call(_a2);
        }, 1400);
        return;
      }
      (_l = (_k = this.chat).set_tile_spec) == null ? void 0 : _l.call(_k, ""), this.clearTiles();
      const i = this.live, n = j[s] ?? [], a = [];
      for (const r of n) {
        const o = `s${this.seq++}`;
        this.pushPart(g, o, r), a.push(o);
      }
      for (const r of i) this.removePart(g, r);
      this.live = a, (_n = (_m = this.chat).set_focus_zoom) == null ? void 0 : _n.call(_m, z[s] ?? 1);
    }
  }
  class Z {
    constructor(s, t = {}) {
      __publicField(this, "takeover", false);
      __publicField(this, "paused", false);
      __publicField(this, "idleTimer", 0);
      __publicField(this, "wheelLock", 0);
      __publicField(this, "last", 0);
      __publicField(this, "raf", 0);
      __publicField(this, "idleMs");
      __publicField(this, "auto");
      __publicField(this, "onState");
      __publicField(this, "loop", (s) => {
        this.last || (this.last = s);
        const t = s - this.last;
        this.last = s, !this.paused && !this.takeover && this.dir.playing && (this.dir.tick(t), this.dir.clock >= this.dir.totalMs && (this.dir.seek(0), this.dir.play())), this.raf = requestAnimationFrame(this.loop);
      });
      this.dir = s, this.idleMs = t.idleMs ?? 3e4, this.auto = t.auto ?? true;
    }
    start() {
      this.dir.seek(0), this.auto ? this.dir.play() : this.takeover = true, this.raf = requestAnimationFrame(this.loop), this.bindInput();
    }
    stop() {
      cancelAnimationFrame(this.raf), window.clearTimeout(this.idleTimer);
    }
    currentIdx() {
      const s = this.dir.marks;
      let t = 0;
      for (let i = 0; i < s.length; i++) this.dir.clock >= s[i].startMs && (t = i);
      return t;
    }
    gotoScene(s) {
      const t = this.dir.marks.length, i = Math.max(0, Math.min(s, t - 1));
      this.takeover = true, this.dir.seek(this.dir.marks[i].startMs), this.paused || this.dir.play(), this.resetIdle(), this.emitState();
    }
    next() {
      this.gotoScene(this.currentIdx() + 1);
    }
    prev() {
      this.gotoScene(this.currentIdx() - 1);
    }
    togglePause() {
      this.paused = !this.paused, this.paused ? (this.dir.pause(), window.clearTimeout(this.idleTimer)) : (this.dir.play(), this.takeover && this.resetIdle()), this.emitState();
    }
    isPaused() {
      return this.paused;
    }
    resetIdle() {
      window.clearTimeout(this.idleTimer), this.auto && (this.idleTimer = window.setTimeout(() => {
        this.takeover = false, this.emitState();
      }, this.idleMs));
    }
    emitState() {
      var _a;
      (_a = this.onState) == null ? void 0 : _a.call(this, {
        paused: this.paused,
        takeover: this.takeover
      });
    }
    bindInput() {
      window.addEventListener("keydown", (i) => {
        i.key === "ArrowRight" ? (i.preventDefault(), this.next()) : i.key === "ArrowLeft" ? (i.preventDefault(), this.prev()) : (i.key === " " || i.key === "Spacebar") && (i.preventDefault(), this.togglePause());
      }), window.addEventListener("wheel", (i) => {
        const n = i.timeStamp;
        n - this.wheelLock < 700 || Math.abs(i.deltaY) < 12 || (this.wheelLock = n, i.deltaY > 0 ? this.next() : this.prev());
      }, {
        passive: true
      });
      let s = 0, t = 0;
      window.addEventListener("touchstart", (i) => {
        s = i.changedTouches[0].clientX, t = i.changedTouches[0].clientY;
      }, {
        passive: true
      }), window.addEventListener("touchend", (i) => {
        const n = i.changedTouches[0].clientX - s, a = i.changedTouches[0].clientY - t;
        Math.abs(n) > 48 && Math.abs(n) > Math.abs(a) && (n < 0 ? this.next() : this.prev());
      }, {
        passive: true
      });
    }
  }
  function Q(e, s, t) {
    const i = document.createElement("div");
    i.className = "chrome-inner";
    const n = document.createElement("button");
    n.className = "pp", n.setAttribute("aria-label", "\u64AD\u653E/\u6682\u505C"), n.textContent = "\u23F8", n.addEventListener("click", () => t.togglePause());
    const a = document.createElement("div");
    a.className = "dots";
    const r = [];
    return s.forEach((o, l) => {
      const c = document.createElement("button");
      c.className = "dot", c.setAttribute("aria-label", `\u7B2C ${l + 1} \u5E55:${o.title}`), c.title = o.title, c.addEventListener("click", () => t.gotoScene(l)), r.push(c), a.appendChild(c);
    }), i.append(n, a), e.appendChild(i), {
      update: (o) => {
        r.forEach((l, c) => l.classList.toggle("on", c === o));
      },
      setPaused: (o) => {
        n.textContent = o ? "\u25B6" : "\u23F8";
      }
    };
  }
  const tt = `#version 300 es
in vec2 p; void main(){ gl_Position = vec4(p, 0.0, 1.0); }`, et = `#version 300 es
precision highp float;
uniform vec2 uRes; uniform float uTime; uniform float uAmp;
out vec4 o;
float hash(vec2 p){ p=fract(p*vec2(123.34,456.21)); p+=dot(p,p+45.32); return fract(p.x*p.y); }
float noise(vec2 p){ vec2 i=floor(p),f=fract(p); f=f*f*(3.-2.*f);
  float a=hash(i),b=hash(i+vec2(1,0)),c=hash(i+vec2(0,1)),d=hash(i+vec2(1,1));
  return mix(mix(a,b,f.x),mix(c,d,f.x),f.y); }
float fbm(vec2 p){ float v=0.,a=.5; for(int i=0;i<4;i++){ v+=a*noise(p); p=p*2.0+7.0; a*=.5; } return v; }
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  float t = uTime * 0.08;
  // \u7AD6\u5411\u57DF\u626D\u66F2(\u6CBF y \u4E0A\u5347\u6D41\u52A8)\u2192 \u4E1D\u7EF8/\u5149\u5E18\u7684\u98D8\u52A8
  float warp = fbm(vec2(uv.x * 1.6, uv.y * 1.0 - t * 1.5));
  float warp2 = fbm(vec2(uv.x * 3.2 + 5.0, uv.y * 1.6 - t * 2.2));
  // \u4E24\u5C42\u4E0D\u540C\u9891\u7684**\u7EC6\u4EAE\u7AD6\u7EBF**(pow \u6536\u7A84\u6210\u7EBF;\u76F8\u4F4D\u968F\u65F6\u95F4/warp \u98D8)
  float ph1 = (uv.x * 9.0 + warp * 3.5) * 6.2831 + t * 2.5;
  float ph2 = (uv.x * 15.0 + warp2 * 4.0) * 6.2831 - t * 1.7;
  float lines = pow(0.5 + 0.5 * sin(ph1), 7.0) * 0.7
              + pow(0.5 + 0.5 * sin(ph2), 9.0) * 0.5;
  // \u6CBF\u5149\u5E18\u7684\u4EAE\u5EA6\u6D41\u52A8(\u660E\u6697\u968F y \u4E0A\u6D8C)
  float flow = smoothstep(0.25, 0.95, fbm(vec2(uv.x * 2.2, uv.y * 1.3 - t * 2.6)));
  float streak = lines * (0.25 + 0.9 * flow);
  // \u5DE6\u53F3\u8FB9\u7F18 mask:\u4E2D\u95F4 (|x-.5|<~0.30) \u900F\u660E,\u5411\u4E24\u4FA7\u5347\u8D77
  float d = abs(uv.x - 0.5) * 2.0;
  float mask = smoothstep(0.40, 0.92, d);
  // \u9876/\u5E95(nav/chrome \u5904)\u6DE1\u51FA
  mask *= smoothstep(0.0, 0.16, uv.y) * smoothstep(1.0, 0.80, uv.y);
  float a = streak * mask * uAmp;
  // \u5E72\u51C0\u938F\u91D1:\u6697\u91D1 \u2192 \u4EAE\u91D1(\u968F\u6D41\u52A8),\u5076\u63BA\u4E00\u4E1D hextech \u51B7\u8C03
  vec3 col = mix(vec3(0.66, 0.54, 0.32), vec3(0.95, 0.84, 0.58), flow);
  col = mix(col, vec3(0.10, 0.62, 0.70), smoothstep(0.7, 1.0, flow) * 0.25);
  o = vec4(col * a, a); // \u9884\u4E58,\u914D ONE/1-src alpha \u6DF7\u5408
}`;
  function F(e, s, t) {
    const i = e.createShader(s);
    return i ? (e.shaderSource(i, t), e.compileShader(i), e.getShaderParameter(i, e.COMPILE_STATUS) ? i : (console.warn("[home-bg] shader \u7F16\u8BD1\u5931\u8D25", e.getShaderInfoLog(i)), null)) : null;
  }
  function it(e, s = {}) {
    const t = e.getContext("webgl2", {
      alpha: true,
      premultipliedAlpha: true,
      antialias: false
    });
    if (!t) return {
      destroy: () => {
      }
    };
    const i = F(t, t.VERTEX_SHADER, tt), n = F(t, t.FRAGMENT_SHADER, et);
    if (!i || !n) return {
      destroy: () => {
      }
    };
    const a = t.createProgram();
    t.attachShader(a, i), t.attachShader(a, n), t.linkProgram(a), t.useProgram(a);
    const r = t.createBuffer();
    t.bindBuffer(t.ARRAY_BUFFER, r), t.bufferData(t.ARRAY_BUFFER, new Float32Array([
      -1,
      -1,
      3,
      -1,
      -1,
      3
    ]), t.STATIC_DRAW);
    const o = t.getAttribLocation(a, "p");
    t.enableVertexAttribArray(o), t.vertexAttribPointer(o, 2, t.FLOAT, false, 0, 0), t.enable(t.BLEND), t.blendFunc(t.ONE, t.ONE_MINUS_SRC_ALPHA);
    const l = t.getUniformLocation(a, "uRes"), c = t.getUniformLocation(a, "uTime"), h = t.getUniformLocation(a, "uAmp");
    t.uniform1f(h, s.amp ?? 0.5);
    const A = 0.5;
    function L() {
      const m = Math.max(1, Math.round(e.clientWidth * A)), b = Math.max(1, Math.round(e.clientHeight * A));
      (e.width !== m || e.height !== b) && (e.width = m, e.height = b, t.viewport(0, 0, m, b)), t.uniform2f(l, e.width, e.height);
    }
    const I = new ResizeObserver(L);
    I.observe(e), L();
    let f = 0, u = true;
    const G = performance.now(), y = (m) => {
      u && (t.uniform1f(c, (m - G) / 1e3), t.drawArrays(t.TRIANGLES, 0, 3), f = requestAnimationFrame(y));
    };
    f = requestAnimationFrame(y);
    const M = () => {
      document.hidden ? (u = false, cancelAnimationFrame(f)) : u || (u = true, f = requestAnimationFrame(y));
    };
    return document.addEventListener("visibilitychange", M), {
      destroy: () => {
        u = false, cancelAnimationFrame(f), I.disconnect(), document.removeEventListener("visibilitychange", M);
      }
    };
  }
  const v = 2600, k = 4400, P = 2600;
  function D(e) {
    return e < 0.5 ? 4 * e * e * e : 1 - Math.pow(-2 * e + 2, 3) / 2;
  }
  class st {
    constructor(s, t) {
      __publicField(this, "active", false);
      __publicField(this, "start", 0);
      __publicField(this, "raf", 0);
      __publicField(this, "lastPetal", 0);
      __publicField(this, "dpr", window.devicePixelRatio || 1);
      __publicField(this, "tick", (s) => {
        if (!this.active) return;
        const t = s - this.start;
        let i;
        t < v ? i = D(t / v) : t < v + k ? (i = 1, s - this.lastPetal > 650 && (this.engine.formation_petals(18), this.lastPetal = s)) : t < v + k + P ? i = 1 - D((t - v - k) / P) : i = 0, this.engine.formation_progress(i), this.raf = requestAnimationFrame(this.tick);
      });
      this.engine = s, this.canvas = t, this.onMove = this.onMove.bind(this), this.onLeave = this.onLeave.bind(this), this.onClick = this.onClick.bind(this);
    }
    onScene(s) {
      s === "bloom" ? this.begin() : this.end();
    }
    begin() {
      this.active || (this.active = true, this.start = performance.now(), this.lastPetal = 0, this.engine.formation_begin(JSON.stringify({
        shape: "rose",
        a: 300,
        k: 5,
        seed: 2
      })), this.canvas.addEventListener("pointermove", this.onMove), this.canvas.addEventListener("pointerleave", this.onLeave), this.canvas.addEventListener("pointerdown", this.onClick), this.raf = requestAnimationFrame(this.tick));
    }
    end() {
      this.active && (this.active = false, cancelAnimationFrame(this.raf), this.engine.set_wind(0, 0, 0, 0), this.engine.formation_progress(0), this.engine.formation_end(), this.canvas.removeEventListener("pointermove", this.onMove), this.canvas.removeEventListener("pointerleave", this.onLeave), this.canvas.removeEventListener("pointerdown", this.onClick));
    }
    windAt(s) {
      const t = this.canvas.getBoundingClientRect();
      return [
        (s.clientX - t.left) * this.dpr,
        (s.clientY - t.top) * this.dpr
      ];
    }
    onMove(s) {
      const [t, i] = this.windAt(s);
      this.engine.set_wind(t, i, 220 * this.dpr, 110 * this.dpr);
    }
    onLeave() {
      this.engine.set_wind(0, 0, 0, 0);
    }
    onClick(s) {
      this.engine.formation_petals(40);
      const [t, i] = this.windAt(s), n = performance.now(), a = (r) => {
        if (!this.active) return;
        const o = 1 - Math.min((r - n) / 400, 1);
        o <= 0 || (this.engine.set_wind(t, i, 300 * this.dpr, 260 * this.dpr * o), requestAnimationFrame(a));
      };
      requestAnimationFrame(a);
    }
    dispose() {
      this.end();
    }
  }
  const x = "/infinite-chat/", C = window.matchMedia("(prefers-reduced-motion: reduce)").matches, _ = (e, s) => {
    const t = document.getElementById(e);
    t && (t.href = x + s);
  };
  _("nav-brand", "");
  _("o-chat", "chat/");
  _("o-md", "markdown/");
  _("o-gallery", "gallery.html");
  const R = document.getElementById("home-foot-ver");
  R && (R.textContent = "infinite-chat \xB7 pages");
  const E = document.getElementById("home-subtitle"), nt = document.getElementById("home-outro"), T = document.getElementById("home-player"), S = document.getElementById("home-transition");
  function ot() {
    S.classList.remove("flash"), S.offsetWidth, S.classList.add("flash"), T.classList.add("switching"), window.setTimeout(() => T.classList.remove("switching"), 240);
  }
  C || it(document.getElementById("home-bg"), {
    amp: 0.55
  });
  function at(e) {
    const s = [];
    return e.eyebrow && s.push(`<div class="sub-eyebrow">${e.eyebrow}</div>`), e.title && s.push(`<h2 class="sub-title">${e.title}</h2>`), e.note && s.push(`<p class="sub-note">${e.note}</p>`), e.link && s.push(`<p class="sub-note"><a class="sub-link" href="${x + e.link.href}">${e.link.text}</a></p>`), s.join("");
  }
  let w = -1, U = null, B = null, H = null;
  function $(e) {
    if (e === w) return;
    const s = w === -1;
    w = e, s || ot();
    const t = N[e], i = V[t.id] ?? {}, n = t.id === "door";
    U == null ? void 0 : U.show(t.id), H == null ? void 0 : H.build(t.id), B == null ? void 0 : B.onScene(t.id), T.classList.toggle("dolly", t.id === "title"), nt.classList.toggle("show", n), n || !i.eyebrow && !i.title && !i.note ? E.classList.remove("show") : (E.innerHTML = at(i), E.classList.add("show"));
  }
  $(0);
  const rt = {
    show: () => {
    },
    hide: () => {
    }
  };
  function q(e, s) {
    const t = new J(N, e, rt), i = new Z(t, {
      auto: !C,
      idleMs: 3e4
    }), n = Q(document.getElementById("home-chrome"), t.marks, i);
    t.onUpdate = (a) => {
      $(a.sceneIdx), n.update(a.sceneIdx);
    }, i.onState = (a) => n.setPaused(a.paused), document.addEventListener("visibilitychange", () => s == null ? void 0 : s(document.hidden || i.isPaused())), i.start();
  }
  async function O(e) {
    e.style.display = "none";
    const { mountFallbackSlides: s } = await Y(async () => {
      const { mountFallbackSlides: t } = await import("./home-fallback-gsMpAjBk.js");
      return {
        mountFallbackSlides: t
      };
    }, []);
    U = s(document.getElementById("home-fallback"), x), w = -1, q({});
  }
  async function lt() {
    const e = document.getElementById("home-canvas");
    if (!e) return;
    if (new URLSearchParams(location.search).has("slides")) {
      await O(e);
      return;
    }
    const { chat: t, ok: i } = await W({
      canvasId: "home-canvas",
      rhythmPreset: "reader",
      glyphMode: 1
    });
    if (!i || !t) {
      await O(e);
      return;
    }
    const n = (a = 0) => {
      const r = t.effect_preset_name;
      if (!r || r.call(t) === "") {
        a < 600 && requestAnimationFrame(() => n(a + 1));
        return;
      }
      H = new K(t), B = new st(t, e);
      const o = new Proxy(t, {
        get(l, c) {
          const h = l[c];
          return typeof h == "function" ? h.bind(l) : h;
        }
      });
      w = -1, q(o, (l) => {
        var _a;
        return (_a = t.set_paused) == null ? void 0 : _a.call(t, l);
      });
    };
    n();
  }
  lt();
});
