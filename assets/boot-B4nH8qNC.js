const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/embed-overlay-C3Xossmv.js","assets/canvas-rect-1-4iAlsh.js","assets/copy-button-Cw-KNp8G.js","assets/text-layer-knFZa3Wj.js","assets/ask-overlay-DIIdeubT.js"])))=>i.map(i=>d[i]);
let fn, gn, bn, B, wn, kn, pn, yn, An, xn, qt, ln, dn, hn, Wt, vn, Sn, zt, Ut, mn;
let __tla = (async () => {
  (function() {
    const e = document.createElement("link").relList;
    if (e && e.supports && e.supports("modulepreload")) return;
    for (const o of document.querySelectorAll('link[rel="modulepreload"]')) _(o);
    new MutationObserver((o) => {
      for (const s of o) if (s.type === "childList") for (const c of s.addedNodes) c.tagName === "LINK" && c.rel === "modulepreload" && _(c);
    }).observe(document, {
      childList: true,
      subtree: true
    });
    function t(o) {
      const s = {};
      return o.integrity && (s.integrity = o.integrity), o.referrerPolicy && (s.referrerPolicy = o.referrerPolicy), o.crossOrigin === "use-credentials" ? s.credentials = "include" : o.crossOrigin === "anonymous" ? s.credentials = "omit" : s.credentials = "same-origin", s;
    }
    function _(o) {
      if (o.ep) return;
      o.ep = true;
      const s = t(o);
      fetch(o.href, s);
    }
  })();
  let bt, ft, De;
  bt = "modulepreload";
  ft = function(n) {
    return "/infinite-chat/" + n;
  };
  De = {};
  B = function(e, t, _) {
    let o = Promise.resolve();
    if (t && t.length > 0) {
      document.getElementsByTagName("link");
      const c = document.querySelector("meta[property=csp-nonce]"), i = (c == null ? void 0 : c.nonce) || (c == null ? void 0 : c.getAttribute("nonce"));
      o = Promise.allSettled(t.map((u) => {
        if (u = ft(u), u in De) return;
        De[u] = true;
        const g = u.endsWith(".css"), f = g ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${u}"]${f}`)) return;
        const m = document.createElement("link");
        if (m.rel = g ? "stylesheet" : bt, g || (m.as = "script"), m.crossOrigin = "", m.href = u, i && m.setAttribute("nonce", i), document.head.appendChild(m), g) return new Promise((d, y) => {
          m.addEventListener("load", d), m.addEventListener("error", () => y(new Error(`Unable to preload CSS for ${u}`)));
        });
      }));
    }
    function s(c) {
      const i = new Event("vite:preloadError", {
        cancelable: true
      });
      if (i.payload = c, window.dispatchEvent(i), !i.defaultPrevented) throw c;
    }
    return o.then((c) => {
      for (const i of c || []) i.status === "rejected" && s(i.reason);
      return e().catch(s);
    });
  };
  let a;
  const G = new Array(128).fill(void 0);
  G.push(void 0, null, true, false);
  function r(n) {
    return G[n];
  }
  let Z = G.length;
  function b(n) {
    Z === G.length && G.push(G.length + 1);
    const e = Z;
    return Z = G[e], G[e] = n, e;
  }
  const Ke = typeof TextDecoder < "u" ? new TextDecoder("utf-8", {
    ignoreBOM: true,
    fatal: true
  }) : {
    decode: () => {
      throw Error("TextDecoder not available");
    }
  };
  typeof TextDecoder < "u" && Ke.decode();
  let K = null;
  function Q() {
    return (K === null || K.byteLength === 0) && (K = new Uint8Array(a.memory.buffer)), K;
  }
  function w(n, e) {
    return n = n >>> 0, Ke.decode(Q().subarray(n, n + e));
  }
  function C(n, e) {
    try {
      return n.apply(this, e);
    } catch (t) {
      a.__wbindgen_export_0(b(t));
    }
  }
  function R(n) {
    return n == null;
  }
  let I = 0;
  const ue = typeof TextEncoder < "u" ? new TextEncoder("utf-8") : {
    encode: () => {
      throw Error("TextEncoder not available");
    }
  }, gt = typeof ue.encodeInto == "function" ? function(n, e) {
    return ue.encodeInto(n, e);
  } : function(n, e) {
    const t = ue.encode(n);
    return e.set(t), {
      read: n.length,
      written: t.length
    };
  };
  function P(n, e, t) {
    if (t === void 0) {
      const i = ue.encode(n), u = e(i.length, 1) >>> 0;
      return Q().subarray(u, u + i.length).set(i), I = i.length, u;
    }
    let _ = n.length, o = e(_, 1) >>> 0;
    const s = Q();
    let c = 0;
    for (; c < _; c++) {
      const i = n.charCodeAt(c);
      if (i > 127) break;
      s[o + c] = i;
    }
    if (c !== _) {
      c !== 0 && (n = n.slice(c)), o = t(o, _, _ = c + n.length * 3, 1) >>> 0;
      const i = Q().subarray(o + c, o + _), u = gt(n, i);
      c += u.written, o = t(o, _, c, 1) >>> 0;
    }
    return I = c, o;
  }
  let N = null;
  function l() {
    return (N === null || N.buffer.detached === true || N.buffer.detached === void 0 && N.buffer !== a.memory.buffer) && (N = new DataView(a.memory.buffer)), N;
  }
  let Y = null;
  function Ye() {
    return (Y === null || Y.byteLength === 0) && (Y = new Uint32Array(a.memory.buffer)), Y;
  }
  function dt(n, e) {
    return n = n >>> 0, Ye().subarray(n / 4, n / 4 + e);
  }
  function lt(n) {
    n < 132 || (G[n] = Z, Z = n);
  }
  function ee(n) {
    const e = r(n);
    return lt(n), e;
  }
  const We = typeof FinalizationRegistry > "u" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((n) => {
    a.__wbindgen_export_4.get(n.dtor)(n.a, n.b);
  });
  function ze(n, e, t, _) {
    const o = {
      a: n,
      b: e,
      cnt: 1,
      dtor: t
    }, s = (...c) => {
      o.cnt++;
      const i = o.a;
      o.a = 0;
      try {
        return _(i, o.b, ...c);
      } finally {
        --o.cnt === 0 ? (a.__wbindgen_export_4.get(o.dtor)(i, o.b), We.unregister(o)) : o.a = i;
      }
    };
    return s.original = o, We.register(s, o, o), s;
  }
  function Se(n) {
    const e = typeof n;
    if (e == "number" || e == "boolean" || n == null) return `${n}`;
    if (e == "string") return `"${n}"`;
    if (e == "symbol") {
      const o = n.description;
      return o == null ? "Symbol" : `Symbol(${o})`;
    }
    if (e == "function") {
      const o = n.name;
      return typeof o == "string" && o.length > 0 ? `Function(${o})` : "Function";
    }
    if (Array.isArray(n)) {
      const o = n.length;
      let s = "[";
      o > 0 && (s += Se(n[0]));
      for (let c = 1; c < o; c++) s += ", " + Se(n[c]);
      return s += "]", s;
    }
    const t = /\[object ([^\]]+)\]/.exec(toString.call(n));
    let _;
    if (t && t.length > 1) _ = t[1];
    else return toString.call(n);
    if (_ == "Object") try {
      return "Object(" + JSON.stringify(n) + ")";
    } catch {
      return "Object";
    }
    return n instanceof Error ? `${n.name}: ${n.message}
${n.stack}` : _;
  }
  function wt(n, e) {
    const t = e(n.length * 4, 4) >>> 0;
    return Ye().set(n, t / 4), I = n.length, t;
  }
  function pt(n, e) {
    const t = e(n.length * 1, 1) >>> 0;
    return Q().set(n, t / 1), I = n.length, t;
  }
  function mt(n, e) {
    a.__wbindgen_export_5(n, e);
  }
  function ht(n, e, t) {
    a.__wbindgen_export_6(n, e, b(t));
  }
  const de = [
    "clamp-to-edge",
    "repeat",
    "mirror-repeat"
  ], Ge = [
    "zero",
    "one",
    "src",
    "one-minus-src",
    "src-alpha",
    "one-minus-src-alpha",
    "dst",
    "one-minus-dst",
    "dst-alpha",
    "one-minus-dst-alpha",
    "src-alpha-saturated",
    "constant",
    "one-minus-constant",
    "src1",
    "one-minus-src1",
    "src1-alpha",
    "one-minus-src1-alpha"
  ], yt = [
    "add",
    "subtract",
    "reverse-subtract",
    "min",
    "max"
  ], xt = [
    "uniform",
    "storage",
    "read-only-storage"
  ], vt = [
    "opaque",
    "premultiplied"
  ], le = [
    "never",
    "less",
    "equal",
    "less-equal",
    "greater",
    "not-equal",
    "greater-equal",
    "always"
  ], St = [
    "none",
    "front",
    "back"
  ], Ue = [
    "nearest",
    "linear"
  ], kt = [
    "ccw",
    "cw"
  ], At = [
    "uint16",
    "uint32"
  ], we = [
    "load",
    "clear"
  ], Mt = [
    "nearest",
    "linear"
  ], Ct = [
    "low-power",
    "high-performance"
  ], It = [
    "point-list",
    "line-list",
    "line-strip",
    "triangle-list",
    "triangle-strip"
  ], Pt = [
    "filtering",
    "non-filtering",
    "comparison"
  ], pe = [
    "keep",
    "zero",
    "replace",
    "invert",
    "increment-clamp",
    "decrement-clamp",
    "increment-wrap",
    "decrement-wrap"
  ], Tt = [
    "write-only",
    "read-only",
    "read-write"
  ], me = [
    "store",
    "discard"
  ], Et = [
    "all",
    "stencil-only",
    "depth-only"
  ], Lt = [
    "1d",
    "2d",
    "3d"
  ], q = [
    "r8unorm",
    "r8snorm",
    "r8uint",
    "r8sint",
    "r16uint",
    "r16sint",
    "r16float",
    "rg8unorm",
    "rg8snorm",
    "rg8uint",
    "rg8sint",
    "r32uint",
    "r32sint",
    "r32float",
    "rg16uint",
    "rg16sint",
    "rg16float",
    "rgba8unorm",
    "rgba8unorm-srgb",
    "rgba8snorm",
    "rgba8uint",
    "rgba8sint",
    "bgra8unorm",
    "bgra8unorm-srgb",
    "rgb9e5ufloat",
    "rgb10a2uint",
    "rgb10a2unorm",
    "rg11b10ufloat",
    "rg32uint",
    "rg32sint",
    "rg32float",
    "rgba16uint",
    "rgba16sint",
    "rgba16float",
    "rgba32uint",
    "rgba32sint",
    "rgba32float",
    "stencil8",
    "depth16unorm",
    "depth24plus",
    "depth24plus-stencil8",
    "depth32float",
    "depth32float-stencil8",
    "bc1-rgba-unorm",
    "bc1-rgba-unorm-srgb",
    "bc2-rgba-unorm",
    "bc2-rgba-unorm-srgb",
    "bc3-rgba-unorm",
    "bc3-rgba-unorm-srgb",
    "bc4-r-unorm",
    "bc4-r-snorm",
    "bc5-rg-unorm",
    "bc5-rg-snorm",
    "bc6h-rgb-ufloat",
    "bc6h-rgb-float",
    "bc7-rgba-unorm",
    "bc7-rgba-unorm-srgb",
    "etc2-rgb8unorm",
    "etc2-rgb8unorm-srgb",
    "etc2-rgb8a1unorm",
    "etc2-rgb8a1unorm-srgb",
    "etc2-rgba8unorm",
    "etc2-rgba8unorm-srgb",
    "eac-r11unorm",
    "eac-r11snorm",
    "eac-rg11unorm",
    "eac-rg11snorm",
    "astc-4x4-unorm",
    "astc-4x4-unorm-srgb",
    "astc-5x4-unorm",
    "astc-5x4-unorm-srgb",
    "astc-5x5-unorm",
    "astc-5x5-unorm-srgb",
    "astc-6x5-unorm",
    "astc-6x5-unorm-srgb",
    "astc-6x6-unorm",
    "astc-6x6-unorm-srgb",
    "astc-8x5-unorm",
    "astc-8x5-unorm-srgb",
    "astc-8x6-unorm",
    "astc-8x6-unorm-srgb",
    "astc-8x8-unorm",
    "astc-8x8-unorm-srgb",
    "astc-10x5-unorm",
    "astc-10x5-unorm-srgb",
    "astc-10x6-unorm",
    "astc-10x6-unorm-srgb",
    "astc-10x8-unorm",
    "astc-10x8-unorm-srgb",
    "astc-10x10-unorm",
    "astc-10x10-unorm-srgb",
    "astc-12x10-unorm",
    "astc-12x10-unorm-srgb",
    "astc-12x12-unorm",
    "astc-12x12-unorm-srgb"
  ], Ft = [
    "float",
    "unfilterable-float",
    "depth",
    "sint",
    "uint"
  ], he = [
    "1d",
    "2d",
    "2d-array",
    "cube",
    "cube-array",
    "3d"
  ], Ot = [
    "uint8",
    "uint8x2",
    "uint8x4",
    "sint8",
    "sint8x2",
    "sint8x4",
    "unorm8",
    "unorm8x2",
    "unorm8x4",
    "snorm8",
    "snorm8x2",
    "snorm8x4",
    "uint16",
    "uint16x2",
    "uint16x4",
    "sint16",
    "sint16x2",
    "sint16x4",
    "unorm16",
    "unorm16x2",
    "unorm16x4",
    "snorm16",
    "snorm16x2",
    "snorm16x4",
    "float16",
    "float16x2",
    "float16x4",
    "float32",
    "float32x2",
    "float32x3",
    "float32x4",
    "uint32",
    "uint32x2",
    "uint32x3",
    "uint32x4",
    "sint32",
    "sint32x2",
    "sint32x3",
    "sint32x4",
    "unorm10-10-10-2",
    "unorm8x4-bgra"
  ], Rt = [
    "vertex",
    "instance"
  ], He = typeof FinalizationRegistry > "u" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((n) => a.__wbg_chatcanvas_free(n >>> 0, 1));
  class Je {
    __destroy_into_raw() {
      const e = this.__wbg_ptr;
      return this.__wbg_ptr = 0, He.unregister(this), e;
    }
    free() {
      const e = this.__destroy_into_raw();
      a.__wbg_chatcanvas_free(e, 0);
    }
    ask_hit_at(e, t) {
      let _, o;
      try {
        const i = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_ask_hit_at(i, this.__wbg_ptr, e, t);
        var s = l().getInt32(i + 4 * 0, true), c = l().getInt32(i + 4 * 1, true);
        return _ = s, o = c, w(s, c);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(_, o, 1);
      }
    }
    push_event(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_push_event(this.__wbg_ptr, t, _);
    }
    set_motion(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_set_motion(this.__wbg_ptr, t, _);
    }
    set_paused(e) {
      a.chatcanvas_set_paused(this.__wbg_ptr, e);
    }
    set_rhythm(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_set_rhythm(this.__wbg_ptr, t, _);
    }
    ask_release() {
      a.chatcanvas_ask_release(this.__wbg_ptr);
    }
    link_hit_at(e, t) {
      let _, o;
      try {
        const i = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_link_hit_at(i, this.__wbg_ptr, e, t);
        var s = l().getInt32(i + 4 * 0, true), c = l().getInt32(i + 4 * 1, true);
        return _ = s, o = c, w(s, c);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(_, o, 1);
      }
    }
    seek_reveal(e) {
      a.chatcanvas_seek_reveal(this.__wbg_ptr, e);
    }
    set_math_em(e) {
      a.chatcanvas_set_math_em(this.__wbg_ptr, e);
    }
    set_pointer(e, t) {
      a.chatcanvas_set_pointer(this.__wbg_ptr, e, t);
    }
    follow_state() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_follow_state(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    frame_embeds() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_frame_embeds(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    image_failed(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_image_failed(this.__wbg_ptr, t, _);
    }
    formation_end() {
      a.chatcanvas_formation_end(this.__wbg_ptr);
    }
    refresh_fonts() {
      a.chatcanvas_refresh_fonts(this.__wbg_ptr);
    }
    render_flavor() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_render_flavor(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    set_selection(e) {
      const t = wt(e, a.__wbindgen_export_2), _ = I;
      a.chatcanvas_set_selection(this.__wbg_ptr, t, _);
    }
    set_tile_spec(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      return a.chatcanvas_set_tile_spec(this.__wbg_ptr, t, _) !== 0;
    }
    visible_turns() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_visible_turns(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    restart_reveal() {
      a.chatcanvas_restart_reveal(this.__wbg_ptr);
    }
    session_status() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_session_status(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    set_ask_height(e) {
      a.chatcanvas_set_ask_height(this.__wbg_ptr, e);
    }
    set_focus_zoom(e) {
      a.chatcanvas_set_focus_zoom(this.__wbg_ptr, e);
    }
    set_glyph_mode(e) {
      a.chatcanvas_set_glyph_mode(this.__wbg_ptr, e);
    }
    set_reveal_cps(e) {
      a.chatcanvas_set_reveal_cps(this.__wbg_ptr, e);
    }
    set_url_policy(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_set_url_policy(this.__wbg_ptr, t, _);
    }
    set_virtualize(e) {
      a.chatcanvas_set_virtualize(this.__wbg_ptr, e);
    }
    tap_fold_first() {
      return a.chatcanvas_tap_fold_first(this.__wbg_ptr) !== 0;
    }
    ask_hit_targets() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_ask_hit_targets(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    formation_begin(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      return a.chatcanvas_formation_begin(this.__wbg_ptr, t, _) !== 0;
    }
    set_post_params(e, t, _) {
      a.chatcanvas_set_post_params(this.__wbg_ptr, e, t, _);
    }
    set_reveal_slow(e) {
      a.chatcanvas_set_reveal_slow(this.__wbg_ptr, e);
    }
    set_stream_rate(e) {
      a.chatcanvas_set_stream_rate(this.__wbg_ptr, e);
    }
    set_table_style(e) {
      a.chatcanvas_set_table_style(this.__wbg_ptr, b(e));
    }
    fold_hit_targets() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_fold_hit_targets(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    formation_petals(e) {
      a.chatcanvas_formation_petals(this.__wbg_ptr, e);
    }
    link_hit_targets() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_link_hit_targets(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    reply_permission(e) {
      return a.chatcanvas_reply_permission(this.__wbg_ptr, e) !== 0;
    }
    scroll_to_latest() {
      a.chatcanvas_scroll_to_latest(this.__wbg_ptr);
    }
    set_cell_metrics(e, t) {
      a.chatcanvas_set_cell_metrics(this.__wbg_ptr, e, t);
    }
    set_focus_valign(e) {
      a.chatcanvas_set_focus_valign(this.__wbg_ptr, e);
    }
    set_pointer_down(e) {
      a.chatcanvas_set_pointer_down(this.__wbg_ptr, e);
    }
    set_spring_enter(e) {
      a.chatcanvas_set_spring_enter(this.__wbg_ptr, e);
    }
    scroll_code_block(e, t, _) {
      const o = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
      a.chatcanvas_scroll_code_block(this.__wbg_ptr, o, s, t, _);
    }
    set_effect_preset(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      return a.chatcanvas_set_effect_preset(this.__wbg_ptr, t, _) !== 0;
    }
    set_render_flavor(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      return a.chatcanvas_set_render_flavor(this.__wbg_ptr, t, _) !== 0;
    }
    set_reveal_preset(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_set_reveal_preset(this.__wbg_ptr, t, _);
    }
    upload_image_rgba(e, t, _, o, s) {
      const c = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), i = I, u = pt(t, a.__wbindgen_export_2), g = I;
      a.chatcanvas_upload_image_rgba(this.__wbg_ptr, c, i, u, g, _, o, s);
    }
    visible_text_runs() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_visible_text_runs(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    effect_preset_name() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_effect_preset_name(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    formation_progress(e) {
      a.chatcanvas_formation_progress(this.__wbg_ptr, e);
    }
    set_debug_geometry(e) {
      a.chatcanvas_set_debug_geometry(this.__wbg_ptr, e);
    }
    set_feedback_decay(e) {
      a.chatcanvas_set_feedback_decay(this.__wbg_ptr, e);
    }
    reply_question_with(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      return a.chatcanvas_reply_question_with(this.__wbg_ptr, t, _) !== 0;
    }
    take_pending_images() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_take_pending_images(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    code_block_at_screen(e, t) {
      let _, o;
      try {
        const i = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_code_block_at_screen(i, this.__wbg_ptr, e, t);
        var s = l().getInt32(i + 4 * 0, true), c = l().getInt32(i + 4 * 1, true);
        return _ = s, o = c, w(s, c);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(_, o, 1);
      }
    }
    set_bench_fold_width(e) {
      a.chatcanvas_set_bench_fold_width(this.__wbg_ptr, e);
    }
    set_exit_dissolve_ms(e) {
      a.chatcanvas_set_exit_dissolve_ms(this.__wbg_ptr, e);
    }
    set_open_url_handler(e) {
      a.chatcanvas_set_open_url_handler(this.__wbg_ptr, b(e));
    }
    set_shaderbox_gallery(e) {
      a.chatcanvas_set_shaderbox_gallery(this.__wbg_ptr, e);
    }
    set_effect_preset_json(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      return a.chatcanvas_set_effect_preset_json(this.__wbg_ptr, t, _) !== 0;
    }
    set_table_reveal_style(e) {
      a.chatcanvas_set_table_reveal_style(this.__wbg_ptr, e);
    }
    take_settle_announcements() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_take_settle_announcements(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    constructor(e, t) {
      try {
        const c = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_new(c, b(e), b(t));
        var _ = l().getInt32(c + 4 * 0, true), o = l().getInt32(c + 4 * 1, true), s = l().getInt32(c + 4 * 2, true);
        if (s) throw ee(o);
        return this.__wbg_ptr = _ >>> 0, He.register(this, this.__wbg_ptr, this), this;
      } finally {
        a.__wbindgen_add_to_stack_pointer(16);
      }
    }
    tap(e, t) {
      return a.chatcanvas_tap(this.__wbg_ptr, e, t) !== 0;
    }
    find(e) {
      let t, _;
      try {
        const c = a.__wbindgen_add_to_stack_pointer(-16), i = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), u = I;
        a.chatcanvas_find(c, this.__wbg_ptr, i, u);
        var o = l().getInt32(c + 4 * 0, true), s = l().getInt32(c + 4 * 1, true);
        return t = o, _ = s, w(o, s);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(t, _, 1);
      }
    }
    step() {
      a.chatcanvas_step(this.__wbg_ptr);
    }
    tick(e) {
      a.chatcanvas_tick(this.__wbg_ptr, e);
    }
    start() {
      a.chatcanvas_start(this.__wbg_ptr);
    }
    stats() {
      const e = a.chatcanvas_stats(this.__wbg_ptr);
      return ee(e);
    }
    pan_by(e, t) {
      a.chatcanvas_pan_by(this.__wbg_ptr, e, t);
    }
    zoom_at(e, t, _) {
      a.chatcanvas_zoom_at(this.__wbg_ptr, e, t, _);
    }
    ask_rect() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_ask_rect(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    set_wind(e, t, _, o) {
      a.chatcanvas_set_wind(this.__wbg_ptr, e, t, _, o);
    }
    tile_hit(e, t) {
      let _, o;
      try {
        const i = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_tile_hit(i, this.__wbg_ptr, e, t);
        var s = l().getInt32(i + 4 * 0, true), c = l().getInt32(i + 4 * 1, true);
        return _ = s, o = c, w(s, c);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(_, o, 1);
      }
    }
    ask_press(e, t) {
      return a.chatcanvas_ask_press(this.__wbg_ptr, e, t) !== 0;
    }
    ask_state() {
      let e, t;
      try {
        const s = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_ask_state(s, this.__wbg_ptr);
        var _ = l().getInt32(s + 4 * 0, true), o = l().getInt32(s + 4 * 1, true);
        return e = _, t = o, w(_, o);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16), a.__wbindgen_export_1(e, t, 1);
      }
    }
    load_msdf(e) {
      try {
        const o = a.__wbindgen_add_to_stack_pointer(-16);
        a.chatcanvas_load_msdf(o, this.__wbg_ptr, b(e));
        var t = l().getInt32(o + 4 * 0, true), _ = l().getInt32(o + 4 * 1, true);
        if (_) throw ee(t);
      } finally {
        a.__wbindgen_add_to_stack_pointer(16);
      }
    }
    note_send() {
      a.chatcanvas_note_send(this.__wbg_ptr);
    }
    scroll_to(e) {
      a.chatcanvas_scroll_to(this.__wbg_ptr, e);
    }
    set_theme(e) {
      const t = P(e, a.__wbindgen_export_2, a.__wbindgen_export_3), _ = I;
      a.chatcanvas_set_theme(this.__wbg_ptr, t, _);
    }
    stop_turn() {
      a.chatcanvas_stop_turn(this.__wbg_ptr);
    }
  }
  async function Bt(n, e) {
    if (typeof Response == "function" && n instanceof Response) {
      if (typeof WebAssembly.instantiateStreaming == "function") try {
        return await WebAssembly.instantiateStreaming(n, e);
      } catch (_) {
        if (n.headers.get("Content-Type") != "application/wasm") console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", _);
        else throw _;
      }
      const t = await n.arrayBuffer();
      return await WebAssembly.instantiate(t, e);
    } else {
      const t = await WebAssembly.instantiate(n, e);
      return t instanceof WebAssembly.Instance ? {
        instance: t,
        module: n
      } : t;
    }
  }
  function $t() {
    const n = {};
    return n.wbg = {}, n.wbg.__wbg_Window_9e7ea8667e28eb00 = function(e) {
      const t = r(e).Window;
      return b(t);
    }, n.wbg.__wbg_WorkerGlobalScope_0169ffb9adb5f5ef = function(e) {
      const t = r(e).WorkerGlobalScope;
      return b(t);
    }, n.wbg.__wbg_addEventListener_90e553fdce254421 = function() {
      return C(function(e, t, _, o) {
        r(e).addEventListener(w(t, _), r(o));
      }, arguments);
    }, n.wbg.__wbg_apply_36be6a55257c99bf = function() {
      return C(function(e, t, _) {
        const o = r(e).apply(r(t), r(_));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_beginRenderPass_aefd0d9681a1f010 = function() {
      return C(function(e, t) {
        const _ = r(e).beginRenderPass(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_buffer_09165b52af8c5237 = function(e) {
      const t = r(e).buffer;
      return b(t);
    }, n.wbg.__wbg_buffer_609cc3eee51ed158 = function(e) {
      const t = r(e).buffer;
      return b(t);
    }, n.wbg.__wbg_call_672a4d21634d4a24 = function() {
      return C(function(e, t) {
        const _ = r(e).call(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_call_7cccdd69e0791ae2 = function() {
      return C(function(e, t, _) {
        const o = r(e).call(r(t), r(_));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_call_b8adc8b1d0a0d8eb = function() {
      return C(function(e, t, _, o, s) {
        const c = r(e).call(r(t), r(_), r(o), r(s));
        return b(c);
      }, arguments);
    }, n.wbg.__wbg_clientHeight_216178c194000db4 = function(e) {
      return r(e).clientHeight;
    }, n.wbg.__wbg_clientWidth_ce67a04dc15fce39 = function(e) {
      return r(e).clientWidth;
    }, n.wbg.__wbg_configure_86dd92dde48d105a = function() {
      return C(function(e, t) {
        r(e).configure(r(t));
      }, arguments);
    }, n.wbg.__wbg_createBindGroupLayout_f0635625a1a82bea = function() {
      return C(function(e, t) {
        const _ = r(e).createBindGroupLayout(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_createBindGroup_043b06d20124f91e = function(e, t) {
      const _ = r(e).createBindGroup(r(t));
      return b(_);
    }, n.wbg.__wbg_createBuffer_086a8bb05ced884a = function() {
      return C(function(e, t) {
        const _ = r(e).createBuffer(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_createCommandEncoder_aa9ae9d445bb7abf = function(e, t) {
      const _ = r(e).createCommandEncoder(r(t));
      return b(_);
    }, n.wbg.__wbg_createPipelineLayout_5cc7e994e46201c7 = function(e, t) {
      const _ = r(e).createPipelineLayout(r(t));
      return b(_);
    }, n.wbg.__wbg_createRenderPipeline_47152f2f57b11194 = function() {
      return C(function(e, t) {
        const _ = r(e).createRenderPipeline(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_createSampler_f3970b77a6f36963 = function(e, t) {
      const _ = r(e).createSampler(r(t));
      return b(_);
    }, n.wbg.__wbg_createShaderModule_9ec201507fe4949e = function(e, t) {
      const _ = r(e).createShaderModule(r(t));
      return b(_);
    }, n.wbg.__wbg_createTexture_09f18232c5ad6e69 = function() {
      return C(function(e, t) {
        const _ = r(e).createTexture(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_createView_f7cd0a0356a46f3b = function() {
      return C(function(e, t) {
        const _ = r(e).createView(r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_devicePixelRatio_68c391265f05d093 = function(e) {
      return r(e).devicePixelRatio;
    }, n.wbg.__wbg_document_d249400bd7bd996d = function(e) {
      const t = r(e).document;
      return R(t) ? 0 : b(t);
    }, n.wbg.__wbg_draw_d38c9207eb049f56 = function(e, t, _, o, s) {
      r(e).draw(t >>> 0, _ >>> 0, o >>> 0, s >>> 0);
    }, n.wbg.__wbg_end_d54348baf0bf3b70 = function(e) {
      r(e).end();
    }, n.wbg.__wbg_error_7534b8e9a36f1ab4 = function(e, t) {
      let _, o;
      try {
        _ = e, o = t, console.error(w(e, t));
      } finally {
        a.__wbindgen_export_1(_, o, 1);
      }
    }, n.wbg.__wbg_fetch_a9bc66c159c18e19 = function(e) {
      const t = fetch(r(e));
      return b(t);
    }, n.wbg.__wbg_finish_db34a19c90c07af7 = function(e) {
      const t = r(e).finish();
      return b(t);
    }, n.wbg.__wbg_finish_e2d3808af76b422a = function(e, t) {
      const _ = r(e).finish(r(t));
      return b(_);
    }, n.wbg.__wbg_from_2a5d3e218e67aa85 = function(e) {
      const t = Array.from(r(e));
      return b(t);
    }, n.wbg.__wbg_getContext_e9cf379449413580 = function() {
      return C(function(e, t, _) {
        const o = r(e).getContext(w(t, _));
        return R(o) ? 0 : b(o);
      }, arguments);
    }, n.wbg.__wbg_getContext_f65a0debd1e8f8e8 = function() {
      return C(function(e, t, _) {
        const o = r(e).getContext(w(t, _));
        return R(o) ? 0 : b(o);
      }, arguments);
    }, n.wbg.__wbg_getCurrentTexture_6ee19b05d6ba43ba = function() {
      return C(function(e) {
        const t = r(e).getCurrentTexture();
        return b(t);
      }, arguments);
    }, n.wbg.__wbg_getPreferredCanvasFormat_c56b5a9a243fe942 = function(e) {
      const t = r(e).getPreferredCanvasFormat();
      return (q.indexOf(t) + 1 || 96) - 1;
    }, n.wbg.__wbg_get_67b2ba62fc30de12 = function() {
      return C(function(e, t) {
        const _ = Reflect.get(r(e), r(t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_get_b9b93047fe3cf45b = function(e, t) {
      const _ = r(e)[t >>> 0];
      return b(_);
    }, n.wbg.__wbg_get_e27dfaeb6f46bd45 = function(e, t) {
      const _ = r(e)[t >>> 0];
      return R(_) ? 0 : b(_);
    }, n.wbg.__wbg_gpu_1b22165b67dd5a59 = function(e) {
      const t = r(e).gpu;
      return b(t);
    }, n.wbg.__wbg_height_838cee19ba8597db = function(e) {
      return r(e).height;
    }, n.wbg.__wbg_instanceof_Error_4d54113b22d20306 = function(e) {
      let t;
      try {
        t = r(e) instanceof Error;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_Float32Array_01dd91be3195315d = function(e) {
      let t;
      try {
        t = r(e) instanceof Float32Array;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_GpuAdapter_331cc7dcda68de8c = function(e) {
      let t;
      try {
        t = r(e) instanceof GPUAdapter;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_GpuCanvasContext_4ea475a10f693c29 = function(e) {
      let t;
      try {
        t = r(e) instanceof GPUCanvasContext;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_Response_f2cc20d9f7dfd644 = function(e) {
      let t;
      try {
        t = r(e) instanceof Response;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_Uint32Array_b8b88c093c0d7ff4 = function(e) {
      let t;
      try {
        t = r(e) instanceof Uint32Array;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_Uint8Array_17156bcf118086a9 = function(e) {
      let t;
      try {
        t = r(e) instanceof Uint8Array;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_instanceof_Window_def73ea0955fc569 = function(e) {
      let t;
      try {
        t = r(e) instanceof Window;
      } catch {
        t = false;
      }
      return t;
    }, n.wbg.__wbg_isArray_a1eab7e0d067391b = function(e) {
      return Array.isArray(r(e));
    }, n.wbg.__wbg_label_7045a786095b1bab = function(e, t) {
      const _ = r(t).label, o = P(_, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
      l().setInt32(e + 4 * 1, s, true), l().setInt32(e + 4 * 0, o, true);
    }, n.wbg.__wbg_length_3b4f022188ae8db6 = function(e) {
      return r(e).length;
    }, n.wbg.__wbg_length_6ca527665d89694d = function(e) {
      return r(e).length;
    }, n.wbg.__wbg_length_a446193dc22c12f8 = function(e) {
      return r(e).length;
    }, n.wbg.__wbg_length_e2d2a49132c1b256 = function(e) {
      return r(e).length;
    }, n.wbg.__wbg_limits_563f98195b4aab75 = function(e) {
      const t = r(e).limits;
      return b(t);
    }, n.wbg.__wbg_location_350d99456c2f3693 = function(e) {
      const t = r(e).location;
      return b(t);
    }, n.wbg.__wbg_log_0cc1b7768397bcfe = function(e, t, _, o, s, c, i, u) {
      let g, f;
      try {
        g = e, f = t, console.log(w(e, t), w(_, o), w(s, c), w(i, u));
      } finally {
        a.__wbindgen_export_1(g, f, 1);
      }
    }, n.wbg.__wbg_log_cb9e190acc5753fb = function(e, t) {
      let _, o;
      try {
        _ = e, o = t, console.log(w(e, t));
      } finally {
        a.__wbindgen_export_1(_, o, 1);
      }
    }, n.wbg.__wbg_mark_7438147ce31e9d4b = function(e, t) {
      performance.mark(w(e, t));
    }, n.wbg.__wbg_maxBindGroups_30d01da76ad53580 = function(e) {
      return r(e).maxBindGroups;
    }, n.wbg.__wbg_maxBindingsPerBindGroup_3dcdeb4a7de67a4a = function(e) {
      return r(e).maxBindingsPerBindGroup;
    }, n.wbg.__wbg_maxBufferSize_a3c3e79851bb49a7 = function(e) {
      return r(e).maxBufferSize;
    }, n.wbg.__wbg_maxColorAttachmentBytesPerSample_61daf47ae1b88dc2 = function(e) {
      return r(e).maxColorAttachmentBytesPerSample;
    }, n.wbg.__wbg_maxColorAttachments_f8f65390ed7c3dcd = function(e) {
      return r(e).maxColorAttachments;
    }, n.wbg.__wbg_maxComputeInvocationsPerWorkgroup_dbfa932a2c3d9ca0 = function(e) {
      return r(e).maxComputeInvocationsPerWorkgroup;
    }, n.wbg.__wbg_maxComputeWorkgroupSizeX_2a7fdde2d850eb69 = function(e) {
      return r(e).maxComputeWorkgroupSizeX;
    }, n.wbg.__wbg_maxComputeWorkgroupSizeY_ae6eb3af592e045d = function(e) {
      return r(e).maxComputeWorkgroupSizeY;
    }, n.wbg.__wbg_maxComputeWorkgroupSizeZ_df6389c6ad61aa20 = function(e) {
      return r(e).maxComputeWorkgroupSizeZ;
    }, n.wbg.__wbg_maxComputeWorkgroupStorageSize_d090d78935189091 = function(e) {
      return r(e).maxComputeWorkgroupStorageSize;
    }, n.wbg.__wbg_maxComputeWorkgroupsPerDimension_5d5d832c21854769 = function(e) {
      return r(e).maxComputeWorkgroupsPerDimension;
    }, n.wbg.__wbg_maxDynamicStorageBuffersPerPipelineLayout_0d5102fd812fe086 = function(e) {
      return r(e).maxDynamicStorageBuffersPerPipelineLayout;
    }, n.wbg.__wbg_maxDynamicUniformBuffersPerPipelineLayout_fd6efab6fa18099a = function(e) {
      return r(e).maxDynamicUniformBuffersPerPipelineLayout;
    }, n.wbg.__wbg_maxSampledTexturesPerShaderStage_4ffa7a7339d366d7 = function(e) {
      return r(e).maxSampledTexturesPerShaderStage;
    }, n.wbg.__wbg_maxSamplersPerShaderStage_776dbf5a1fdc58b1 = function(e) {
      return r(e).maxSamplersPerShaderStage;
    }, n.wbg.__wbg_maxStorageBufferBindingSize_4a81009504bfcacd = function(e) {
      return r(e).maxStorageBufferBindingSize;
    }, n.wbg.__wbg_maxStorageBuffersPerShaderStage_772149c39281f13c = function(e) {
      return r(e).maxStorageBuffersPerShaderStage;
    }, n.wbg.__wbg_maxStorageTexturesPerShaderStage_181856fa7bd31bd2 = function(e) {
      return r(e).maxStorageTexturesPerShaderStage;
    }, n.wbg.__wbg_maxTextureArrayLayers_c50110b7591a08e7 = function(e) {
      return r(e).maxTextureArrayLayers;
    }, n.wbg.__wbg_maxTextureDimension1D_8886fff72f64818a = function(e) {
      return r(e).maxTextureDimension1D;
    }, n.wbg.__wbg_maxTextureDimension2D_0e30b1b618696302 = function(e) {
      return r(e).maxTextureDimension2D;
    }, n.wbg.__wbg_maxTextureDimension3D_2f567b561a18a953 = function(e) {
      return r(e).maxTextureDimension3D;
    }, n.wbg.__wbg_maxUniformBufferBindingSize_50a7723e932bbd63 = function(e) {
      return r(e).maxUniformBufferBindingSize;
    }, n.wbg.__wbg_maxUniformBuffersPerShaderStage_cfac0560ee2b33a2 = function(e) {
      return r(e).maxUniformBuffersPerShaderStage;
    }, n.wbg.__wbg_maxVertexAttributes_6bd060b2025920cc = function(e) {
      return r(e).maxVertexAttributes;
    }, n.wbg.__wbg_maxVertexBufferArrayStride_b3c77c1ff836be9f = function(e) {
      return r(e).maxVertexBufferArrayStride;
    }, n.wbg.__wbg_maxVertexBuffers_b4635256105b2915 = function(e) {
      return r(e).maxVertexBuffers;
    }, n.wbg.__wbg_measure_fb7825c11612c823 = function() {
      return C(function(e, t, _, o) {
        let s, c, i, u;
        try {
          s = e, c = t, i = _, u = o, performance.measure(w(e, t), w(_, o));
        } finally {
          a.__wbindgen_export_1(s, c, 1), a.__wbindgen_export_1(i, u, 1);
        }
      }, arguments);
    }, n.wbg.__wbg_message_97a2af9b89d693a3 = function(e) {
      const t = r(e).message;
      return b(t);
    }, n.wbg.__wbg_minStorageBufferOffsetAlignment_989812b5a6a4b5e7 = function(e) {
      return r(e).minStorageBufferOffsetAlignment;
    }, n.wbg.__wbg_minUniformBufferOffsetAlignment_ff7899c34a8303e7 = function(e) {
      return r(e).minUniformBufferOffsetAlignment;
    }, n.wbg.__wbg_name_0b327d569f00ebee = function(e) {
      const t = r(e).name;
      return b(t);
    }, n.wbg.__wbg_navigator_0a9bf1120e24fec2 = function(e) {
      const t = r(e).navigator;
      return b(t);
    }, n.wbg.__wbg_navigator_1577371c070c8947 = function(e) {
      const t = r(e).navigator;
      return b(t);
    }, n.wbg.__wbg_new_018dcc2d6c8c2f6a = function() {
      return C(function() {
        const e = new Headers();
        return b(e);
      }, arguments);
    }, n.wbg.__wbg_new_405e22f390576ce2 = function() {
      const e = new Object();
      return b(e);
    }, n.wbg.__wbg_new_780abee5c1739fd7 = function(e) {
      const t = new Float32Array(r(e));
      return b(t);
    }, n.wbg.__wbg_new_78feb108b6472713 = function() {
      const e = new Array();
      return b(e);
    }, n.wbg.__wbg_new_80bf4ee74f41ff92 = function() {
      return C(function() {
        const e = new URLSearchParams();
        return b(e);
      }, arguments);
    }, n.wbg.__wbg_new_8a6f238a6ece86ea = function() {
      const e = new Error();
      return b(e);
    }, n.wbg.__wbg_new_9ffbe0a71eff35e3 = function() {
      return C(function(e, t) {
        const _ = new URL(w(e, t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_new_a12002a7f91c75be = function(e) {
      const t = new Uint8Array(r(e));
      return b(t);
    }, n.wbg.__wbg_new_e3b321dcfef89fc7 = function(e) {
      const t = new Uint32Array(r(e));
      return b(t);
    }, n.wbg.__wbg_newnoargs_105ed471475aaf50 = function(e, t) {
      const _ = new Function(w(e, t));
      return b(_);
    }, n.wbg.__wbg_newwithbyteoffsetandlength_d97e637ebe145a9a = function(e, t, _) {
      const o = new Uint8Array(r(e), t >>> 0, _ >>> 0);
      return b(o);
    }, n.wbg.__wbg_newwithbyteoffsetandlength_f1dead44d1fc7212 = function(e, t, _) {
      const o = new Uint32Array(r(e), t >>> 0, _ >>> 0);
      return b(o);
    }, n.wbg.__wbg_newwithlength_bd3de93688d68fbc = function(e) {
      const t = new Uint32Array(e >>> 0);
      return b(t);
    }, n.wbg.__wbg_newwithstr_78e86e03c4ae814e = function() {
      return C(function(e, t) {
        const _ = new Request(w(e, t));
        return b(_);
      }, arguments);
    }, n.wbg.__wbg_newwithstrandinit_06c535e0a867c635 = function() {
      return C(function(e, t, _) {
        const o = new Request(w(e, t), r(_));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_now_2c95c9de01293173 = function(e) {
      return r(e).now();
    }, n.wbg.__wbg_now_d18023d54d4e5500 = function(e) {
      return r(e).now();
    }, n.wbg.__wbg_ok_3aaf32d069979723 = function(e) {
      return r(e).ok;
    }, n.wbg.__wbg_performance_7a3ffd0b17f663ad = function(e) {
      const t = r(e).performance;
      return b(t);
    }, n.wbg.__wbg_performance_c185c0cdc2766575 = function(e) {
      const t = r(e).performance;
      return R(t) ? 0 : b(t);
    }, n.wbg.__wbg_push_737cfc8c1432c2c6 = function(e, t) {
      return r(e).push(r(t));
    }, n.wbg.__wbg_querySelectorAll_40998fd748f057ef = function() {
      return C(function(e, t, _) {
        const o = r(e).querySelectorAll(w(t, _));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_queueMicrotask_97d92b4fcc8a61c5 = function(e) {
      queueMicrotask(r(e));
    }, n.wbg.__wbg_queueMicrotask_d3219def82552485 = function(e) {
      const t = r(e).queueMicrotask;
      return b(t);
    }, n.wbg.__wbg_queue_0ffbb97537a0c4ed = function(e) {
      const t = r(e).queue;
      return b(t);
    }, n.wbg.__wbg_requestAdapter_f09d28b3f37de26c = function(e, t) {
      const _ = r(e).requestAdapter(r(t));
      return b(_);
    }, n.wbg.__wbg_requestAnimationFrame_d7fd890aaefc3246 = function() {
      return C(function(e, t) {
        return r(e).requestAnimationFrame(r(t));
      }, arguments);
    }, n.wbg.__wbg_requestDevice_51509dadc50b2e9d = function(e, t) {
      const _ = r(e).requestDevice(r(t));
      return b(_);
    }, n.wbg.__wbg_resolve_4851785c9c5f573d = function(e) {
      const t = Promise.resolve(r(e));
      return b(t);
    }, n.wbg.__wbg_search_c1c3bfbeadd96c47 = function() {
      return C(function(e, t) {
        const _ = r(t).search, o = P(_, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
        l().setInt32(e + 4 * 1, s, true), l().setInt32(e + 4 * 0, o, true);
      }, arguments);
    }, n.wbg.__wbg_search_e0e79cfe010c5c23 = function(e, t) {
      const _ = r(t).search, o = P(_, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
      l().setInt32(e + 4 * 1, s, true), l().setInt32(e + 4 * 0, o, true);
    }, n.wbg.__wbg_setBindGroup_a81ce7b3934585bf = function(e, t, _) {
      r(e).setBindGroup(t >>> 0, r(_));
    }, n.wbg.__wbg_setBindGroup_bb0c2c05b7c49401 = function() {
      return C(function(e, t, _, o, s, c, i) {
        r(e).setBindGroup(t >>> 0, r(_), dt(o, s), c, i >>> 0);
      }, arguments);
    }, n.wbg.__wbg_setPipeline_78f8f6d440dddd25 = function(e, t) {
      r(e).setPipeline(r(t));
    }, n.wbg.__wbg_setVertexBuffer_b0d3128a04bfd766 = function(e, t, _, o, s) {
      r(e).setVertexBuffer(t >>> 0, r(_), o, s);
    }, n.wbg.__wbg_setVertexBuffer_edbff6ddb5055174 = function(e, t, _, o) {
      r(e).setVertexBuffer(t >>> 0, r(_), o);
    }, n.wbg.__wbg_set_10bad9bee0e9c58b = function(e, t, _) {
      r(e).set(r(t), _ >>> 0);
    }, n.wbg.__wbg_set_65595bdd868b3009 = function(e, t, _) {
      r(e).set(r(t), _ >>> 0);
    }, n.wbg.__wbg_set_bb8cecf6a62b9f46 = function() {
      return C(function(e, t, _) {
        return Reflect.set(r(e), r(t), r(_));
      }, arguments);
    }, n.wbg.__wbg_set_d23661d19148b229 = function(e, t, _) {
      r(e).set(r(t), _ >>> 0);
    }, n.wbg.__wbg_seta_721deab95e136b71 = function(e, t) {
      r(e).a = t;
    }, n.wbg.__wbg_setaccess_b20bfa3ec6b65d05 = function(e, t) {
      r(e).access = Tt[t];
    }, n.wbg.__wbg_setaddressmodeu_9c0b2104a94d10f3 = function(e, t) {
      r(e).addressModeU = de[t];
    }, n.wbg.__wbg_setaddressmodev_a9bedc188ff29608 = function(e, t) {
      r(e).addressModeV = de[t];
    }, n.wbg.__wbg_setaddressmodew_5774889145ce3789 = function(e, t) {
      r(e).addressModeW = de[t];
    }, n.wbg.__wbg_setalpha_2c7bdc9da833b6c2 = function(e, t) {
      r(e).alpha = r(t);
    }, n.wbg.__wbg_setalphamode_fc3528d234b1fefa = function(e, t) {
      r(e).alphaMode = vt[t];
    }, n.wbg.__wbg_setalphatocoverageenabled_314ce1ca1759b395 = function(e, t) {
      r(e).alphaToCoverageEnabled = t !== 0;
    }, n.wbg.__wbg_setarraylayercount_3c7942d623042874 = function(e, t) {
      r(e).arrayLayerCount = t >>> 0;
    }, n.wbg.__wbg_setarraystride_4b36d0822dea74a8 = function(e, t) {
      r(e).arrayStride = t;
    }, n.wbg.__wbg_setaspect_f06e234d0aacd1a6 = function(e, t) {
      r(e).aspect = Et[t];
    }, n.wbg.__wbg_setattributes_382cc084e6792c33 = function(e, t) {
      r(e).attributes = r(t);
    }, n.wbg.__wbg_setb_f53c2f10173c804f = function(e, t) {
      r(e).b = t;
    }, n.wbg.__wbg_setbasearraylayer_a5b968338c5c56b6 = function(e, t) {
      r(e).baseArrayLayer = t >>> 0;
    }, n.wbg.__wbg_setbasemiplevel_e3288c2d851da708 = function(e, t) {
      r(e).baseMipLevel = t >>> 0;
    }, n.wbg.__wbg_setbeginningofpasswriteindex_35dcbf135e4f9d61 = function(e, t) {
      r(e).beginningOfPassWriteIndex = t >>> 0;
    }, n.wbg.__wbg_setbindgrouplayouts_8de6e109dd34a448 = function(e, t) {
      r(e).bindGroupLayouts = r(t);
    }, n.wbg.__wbg_setbinding_5276d6202fceba46 = function(e, t) {
      r(e).binding = t >>> 0;
    }, n.wbg.__wbg_setbinding_9e9ed8b6e1418176 = function(e, t) {
      r(e).binding = t >>> 0;
    }, n.wbg.__wbg_setblend_6828ff186670f414 = function(e, t) {
      r(e).blend = r(t);
    }, n.wbg.__wbg_setbuffer_1acdac44d9638973 = function(e, t) {
      r(e).buffer = r(t);
    }, n.wbg.__wbg_setbuffer_74b7b0adf855cf1a = function(e, t) {
      r(e).buffer = r(t);
    }, n.wbg.__wbg_setbuffers_53e83b7c7a5c95aa = function(e, t) {
      r(e).buffers = r(t);
    }, n.wbg.__wbg_setbytesperrow_9249690c44f83135 = function(e, t) {
      r(e).bytesPerRow = t >>> 0;
    }, n.wbg.__wbg_setclearvalue_f82fff01ed0b5c35 = function(e, t) {
      r(e).clearValue = r(t);
    }, n.wbg.__wbg_setcode_6b6ad02fc1705aa2 = function(e, t, _) {
      r(e).code = w(t, _);
    }, n.wbg.__wbg_setcolor_0df2c5f47a951ac1 = function(e, t) {
      r(e).color = r(t);
    }, n.wbg.__wbg_setcolorattachments_de625dd9a4850a13 = function(e, t) {
      r(e).colorAttachments = r(t);
    }, n.wbg.__wbg_setcompare_1b67d8112d05628e = function(e, t) {
      r(e).compare = le[t];
    }, n.wbg.__wbg_setcompare_8fbddcdd4781f49a = function(e, t) {
      r(e).compare = le[t];
    }, n.wbg.__wbg_setcount_e8b681b1185cf5da = function(e, t) {
      r(e).count = t >>> 0;
    }, n.wbg.__wbg_setcullmode_74bc6eaab528c94b = function(e, t) {
      r(e).cullMode = St[t];
    }, n.wbg.__wbg_setdepthbias_cdcc35c6971d19cd = function(e, t) {
      r(e).depthBias = t;
    }, n.wbg.__wbg_setdepthbiasclamp_57801e26f66496d9 = function(e, t) {
      r(e).depthBiasClamp = t;
    }, n.wbg.__wbg_setdepthbiasslopescale_81699f807bd5a647 = function(e, t) {
      r(e).depthBiasSlopeScale = t;
    }, n.wbg.__wbg_setdepthclearvalue_9801aa9eff7645df = function(e, t) {
      r(e).depthClearValue = t;
    }, n.wbg.__wbg_setdepthcompare_53d249a136855bd8 = function(e, t) {
      r(e).depthCompare = le[t];
    }, n.wbg.__wbg_setdepthfailop_2e4767995acd4c0a = function(e, t) {
      r(e).depthFailOp = pe[t];
    }, n.wbg.__wbg_setdepthloadop_af0b0f05e83f6571 = function(e, t) {
      r(e).depthLoadOp = we[t];
    }, n.wbg.__wbg_setdepthorarraylayers_5d480fc05509ea0c = function(e, t) {
      r(e).depthOrArrayLayers = t >>> 0;
    }, n.wbg.__wbg_setdepthreadonly_a7b7224074e024d3 = function(e, t) {
      r(e).depthReadOnly = t !== 0;
    }, n.wbg.__wbg_setdepthstencil_2bb2fcea55783858 = function(e, t) {
      r(e).depthStencil = r(t);
    }, n.wbg.__wbg_setdepthstencilattachment_dcbd5b74e4350e16 = function(e, t) {
      r(e).depthStencilAttachment = r(t);
    }, n.wbg.__wbg_setdepthstoreop_40dfd99c7e42f894 = function(e, t) {
      r(e).depthStoreOp = me[t];
    }, n.wbg.__wbg_setdepthwriteenabled_4368a2fe5d258cb0 = function(e, t) {
      r(e).depthWriteEnabled = t !== 0;
    }, n.wbg.__wbg_setdevice_d372d6aa06f20cae = function(e, t) {
      r(e).device = r(t);
    }, n.wbg.__wbg_setdimension_268b2b7bfc3e2bb8 = function(e, t) {
      r(e).dimension = Lt[t];
    }, n.wbg.__wbg_setdimension_359b229ea1b67a77 = function(e, t) {
      r(e).dimension = he[t];
    }, n.wbg.__wbg_setdstfactor_96e73b9eaedeb23e = function(e, t) {
      r(e).dstFactor = Ge[t];
    }, n.wbg.__wbg_setendofpasswriteindex_71e7659a9d2a9d60 = function(e, t) {
      r(e).endOfPassWriteIndex = t >>> 0;
    }, n.wbg.__wbg_setentries_5941f16619f54d42 = function(e, t) {
      r(e).entries = r(t);
    }, n.wbg.__wbg_setentries_97a6ad10aa7fa4d1 = function(e, t) {
      r(e).entries = r(t);
    }, n.wbg.__wbg_setentrypoint_a858879f63ec2236 = function(e, t, _) {
      r(e).entryPoint = w(t, _);
    }, n.wbg.__wbg_setentrypoint_a8ce0b22c20548b0 = function(e, t, _) {
      r(e).entryPoint = w(t, _);
    }, n.wbg.__wbg_setfailop_d55bda42958efa98 = function(e, t) {
      r(e).failOp = pe[t];
    }, n.wbg.__wbg_setformat_69ba449c0e080708 = function(e, t) {
      r(e).format = q[t];
    }, n.wbg.__wbg_setformat_713b9e90b13df6aa = function(e, t) {
      r(e).format = Ot[t];
    }, n.wbg.__wbg_setformat_76bcf93126fcdc9d = function(e, t) {
      r(e).format = q[t];
    }, n.wbg.__wbg_setformat_970299d3f84a8f20 = function(e, t) {
      r(e).format = q[t];
    }, n.wbg.__wbg_setformat_a8a60feb127f0971 = function(e, t) {
      r(e).format = q[t];
    }, n.wbg.__wbg_setformat_beb33029aea4cf8e = function(e, t) {
      r(e).format = q[t];
    }, n.wbg.__wbg_setformat_f6ec428901712514 = function(e, t) {
      r(e).format = q[t];
    }, n.wbg.__wbg_setfragment_0f23dfb67b3e84ab = function(e, t) {
      r(e).fragment = r(t);
    }, n.wbg.__wbg_setfrontface_c80337acd997f8c6 = function(e, t) {
      r(e).frontFace = kt[t];
    }, n.wbg.__wbg_setg_7eb6b5e67456a09e = function(e, t) {
      r(e).g = t;
    }, n.wbg.__wbg_sethasdynamicoffset_b34dfdba692a7959 = function(e, t) {
      r(e).hasDynamicOffset = t !== 0;
    }, n.wbg.__wbg_setheaders_834c0bdb6a8949ad = function(e, t) {
      r(e).headers = r(t);
    }, n.wbg.__wbg_setheight_433680330c9420c3 = function(e, t) {
      r(e).height = t >>> 0;
    }, n.wbg.__wbg_setheight_a7439239ff109215 = function(e, t) {
      r(e).height = t >>> 0;
    }, n.wbg.__wbg_setheight_da683a33fa99843c = function(e, t) {
      r(e).height = t >>> 0;
    }, n.wbg.__wbg_setlabel_1df8805b2aad72d7 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_460a52030d604dd7 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_57008c2e11276b5e = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_66db708c47a585b2 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_68cd87490e02e1de = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_76b058f0224eb49e = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_89c327fa94d8076b = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_969d6f8279c74456 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_a0c41069e355431e = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_c14214ffbf6e5c4a = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_ca2c132e2b646244 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_e6fab993e10f1dd3 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlabel_f9a45e9ef445b781 = function(e, t, _) {
      r(e).label = w(t, _);
    }, n.wbg.__wbg_setlayout_67a29edc6247c437 = function(e, t) {
      r(e).layout = r(t);
    }, n.wbg.__wbg_setlayout_758d30edbd6ea91c = function(e, t) {
      r(e).layout = r(t);
    }, n.wbg.__wbg_setloadop_5644a3bf70f4f76c = function(e, t) {
      r(e).loadOp = we[t];
    }, n.wbg.__wbg_setlodmaxclamp_d80060a9922f9fe3 = function(e, t) {
      r(e).lodMaxClamp = t;
    }, n.wbg.__wbg_setlodminclamp_bee469ae69d038f0 = function(e, t) {
      r(e).lodMinClamp = t;
    }, n.wbg.__wbg_setmagfilter_f50646cfdc01700d = function(e, t) {
      r(e).magFilter = Ue[t];
    }, n.wbg.__wbg_setmappedatcreation_0dc5796d4e90ab4b = function(e, t) {
      r(e).mappedAtCreation = t !== 0;
    }, n.wbg.__wbg_setmask_800b15ad78613be8 = function(e, t) {
      r(e).mask = t >>> 0;
    }, n.wbg.__wbg_setmaxanisotropy_83ac2a8bef9f9ec8 = function(e, t) {
      r(e).maxAnisotropy = t;
    }, n.wbg.__wbg_setmethod_3c5280fe5d890842 = function(e, t, _) {
      r(e).method = w(t, _);
    }, n.wbg.__wbg_setminbindingsize_20ca594cd6d93818 = function(e, t) {
      r(e).minBindingSize = t;
    }, n.wbg.__wbg_setminfilter_8ffc9e1ac6b4149f = function(e, t) {
      r(e).minFilter = Ue[t];
    }, n.wbg.__wbg_setmiplevel_6f507098915add77 = function(e, t) {
      r(e).mipLevel = t >>> 0;
    }, n.wbg.__wbg_setmiplevelcount_5e59806cbcf116e9 = function(e, t) {
      r(e).mipLevelCount = t >>> 0;
    }, n.wbg.__wbg_setmiplevelcount_f896fe8cbb669df2 = function(e, t) {
      r(e).mipLevelCount = t >>> 0;
    }, n.wbg.__wbg_setmipmapfilter_037575f2e647f024 = function(e, t) {
      r(e).mipmapFilter = Mt[t];
    }, n.wbg.__wbg_setmodule_4c73bb35cb0beb0b = function(e, t) {
      r(e).module = r(t);
    }, n.wbg.__wbg_setmodule_ca21130b3f66ea5d = function(e, t) {
      r(e).module = r(t);
    }, n.wbg.__wbg_setmultisample_4f57dcaa4144a62f = function(e, t) {
      r(e).multisample = r(t);
    }, n.wbg.__wbg_setmultisampled_0bb9fc1b577bf11a = function(e, t) {
      r(e).multisampled = t !== 0;
    }, n.wbg.__wbg_setoffset_67ee100819c122f2 = function(e, t) {
      r(e).offset = t;
    }, n.wbg.__wbg_setoffset_a8194a4fcfff8910 = function(e, t) {
      r(e).offset = t;
    }, n.wbg.__wbg_setoffset_d37e5fa34e9ded2e = function(e, t) {
      r(e).offset = t;
    }, n.wbg.__wbg_setoperation_173958551af7f4f2 = function(e, t) {
      r(e).operation = yt[t];
    }, n.wbg.__wbg_setorigin_e26b73e77b3e275d = function(e, t) {
      r(e).origin = r(t);
    }, n.wbg.__wbg_setpassop_070547fd6160a00d = function(e, t) {
      r(e).passOp = pe[t];
    }, n.wbg.__wbg_setpowerpreference_1f3351e5d2acf765 = function(e, t) {
      r(e).powerPreference = Ct[t];
    }, n.wbg.__wbg_setprimitive_ee18492ab93953bc = function(e, t) {
      r(e).primitive = r(t);
    }, n.wbg.__wbg_setqueryset_3b14f95f9bd114db = function(e, t) {
      r(e).querySet = r(t);
    }, n.wbg.__wbg_setr_a4e2f60e3466da86 = function(e, t) {
      r(e).r = t;
    }, n.wbg.__wbg_setrequiredfeatures_fc44bc3433300ee3 = function(e, t) {
      r(e).requiredFeatures = r(t);
    }, n.wbg.__wbg_setresolvetarget_c4b519cab7eb42b7 = function(e, t) {
      r(e).resolveTarget = r(t);
    }, n.wbg.__wbg_setresource_1659f5a29a2e0541 = function(e, t) {
      r(e).resource = r(t);
    }, n.wbg.__wbg_setrowsperimage_53ed2c633b1adfcc = function(e, t) {
      r(e).rowsPerImage = t >>> 0;
    }, n.wbg.__wbg_setsamplecount_e88d044f067a2241 = function(e, t) {
      r(e).sampleCount = t >>> 0;
    }, n.wbg.__wbg_setsampler_a778272f31d31ce5 = function(e, t) {
      r(e).sampler = r(t);
    }, n.wbg.__wbg_setsampletype_c0e25b966db74174 = function(e, t) {
      r(e).sampleType = Ft[t];
    }, n.wbg.__wbg_setsearch_609451e9e712f3c6 = function(e, t, _) {
      r(e).search = w(t, _);
    }, n.wbg.__wbg_setshaderlocation_985046f48e76573f = function(e, t) {
      r(e).shaderLocation = t >>> 0;
    }, n.wbg.__wbg_setsize_23676383c9c0732f = function(e, t) {
      r(e).size = t;
    }, n.wbg.__wbg_setsize_51616eaf8209c58b = function(e, t) {
      r(e).size = r(t);
    }, n.wbg.__wbg_setsize_5878aadcd23673cf = function(e, t) {
      r(e).size = t;
    }, n.wbg.__wbg_setsrcfactor_04ce8874f1bff5a8 = function(e, t) {
      r(e).srcFactor = Ge[t];
    }, n.wbg.__wbg_setstencilback_4b20ecfcd4c4816a = function(e, t) {
      r(e).stencilBack = r(t);
    }, n.wbg.__wbg_setstencilclearvalue_7ba82e1993788f37 = function(e, t) {
      r(e).stencilClearValue = t >>> 0;
    }, n.wbg.__wbg_setstencilfront_1ca3b695f7c42f6a = function(e, t) {
      r(e).stencilFront = r(t);
    }, n.wbg.__wbg_setstencilloadop_b65c60a0077315cd = function(e, t) {
      r(e).stencilLoadOp = we[t];
    }, n.wbg.__wbg_setstencilreadmask_4f5b98747141e796 = function(e, t) {
      r(e).stencilReadMask = t >>> 0;
    }, n.wbg.__wbg_setstencilreadonly_9006a99a91d198e9 = function(e, t) {
      r(e).stencilReadOnly = t !== 0;
    }, n.wbg.__wbg_setstencilstoreop_4f00c5eca345c145 = function(e, t) {
      r(e).stencilStoreOp = me[t];
    }, n.wbg.__wbg_setstencilwritemask_e37a7214d84ace99 = function(e, t) {
      r(e).stencilWriteMask = t >>> 0;
    }, n.wbg.__wbg_setstepmode_7d58d75e6547a7a6 = function(e, t) {
      r(e).stepMode = Rt[t];
    }, n.wbg.__wbg_setstoragetexture_2987339fec972d54 = function(e, t) {
      r(e).storageTexture = r(t);
    }, n.wbg.__wbg_setstoreop_c62dd050b5806095 = function(e, t) {
      r(e).storeOp = me[t];
    }, n.wbg.__wbg_setstripindexformat_3e4893749b3f00b0 = function(e, t) {
      r(e).stripIndexFormat = At[t];
    }, n.wbg.__wbg_settargets_0ef1de33af7253a6 = function(e, t) {
      r(e).targets = r(t);
    }, n.wbg.__wbg_settexture_2553e9c3ae6f7687 = function(e, t) {
      r(e).texture = r(t);
    }, n.wbg.__wbg_settexture_f62859f817324dd1 = function(e, t) {
      r(e).texture = r(t);
    }, n.wbg.__wbg_settimestampwrites_1995524c3a31cb8f = function(e, t) {
      r(e).timestampWrites = r(t);
    }, n.wbg.__wbg_settopology_3d9b2f0ffe2e350c = function(e, t) {
      r(e).topology = It[t];
    }, n.wbg.__wbg_settype_0b59dd5f4721c490 = function(e, t) {
      r(e).type = Pt[t];
    }, n.wbg.__wbg_settype_8c8bbfab4cf7e32e = function(e, t) {
      r(e).type = xt[t];
    }, n.wbg.__wbg_setusage_44ebc3b496e60ff4 = function(e, t) {
      r(e).usage = t >>> 0;
    }, n.wbg.__wbg_setusage_4cf7b16df5617a46 = function(e, t) {
      r(e).usage = t >>> 0;
    }, n.wbg.__wbg_setusage_c45cca4a5b9f8376 = function(e, t) {
      r(e).usage = t >>> 0;
    }, n.wbg.__wbg_setusage_e58b3c3ce83fbbda = function(e, t) {
      r(e).usage = t >>> 0;
    }, n.wbg.__wbg_setvertex_6144c56d98e2314a = function(e, t) {
      r(e).vertex = r(t);
    }, n.wbg.__wbg_setview_4bc3dfcbfc8a58ba = function(e, t) {
      r(e).view = r(t);
    }, n.wbg.__wbg_setview_8d0b0055b6ef07e3 = function(e, t) {
      r(e).view = r(t);
    }, n.wbg.__wbg_setviewdimension_afac48443b8fb565 = function(e, t) {
      r(e).viewDimension = he[t];
    }, n.wbg.__wbg_setviewdimension_f5d4b5336a27d302 = function(e, t) {
      r(e).viewDimension = he[t];
    }, n.wbg.__wbg_setviewformats_0cfe174ac882efaf = function(e, t) {
      r(e).viewFormats = r(t);
    }, n.wbg.__wbg_setviewformats_c566feb1da7b1925 = function(e, t) {
      r(e).viewFormats = r(t);
    }, n.wbg.__wbg_setvisibility_7245f1acbedb4ff4 = function(e, t) {
      r(e).visibility = t >>> 0;
    }, n.wbg.__wbg_setwidth_056381a7176ba440 = function(e, t) {
      r(e).width = t >>> 0;
    }, n.wbg.__wbg_setwidth_660ca581e3fbe279 = function(e, t) {
      r(e).width = t >>> 0;
    }, n.wbg.__wbg_setwidth_c5fed9f5e7f0b406 = function(e, t) {
      r(e).width = t >>> 0;
    }, n.wbg.__wbg_setwritemask_c381ff702509999c = function(e, t) {
      r(e).writeMask = t >>> 0;
    }, n.wbg.__wbg_setx_6e550cba86f408f0 = function(e, t) {
      r(e).x = t >>> 0;
    }, n.wbg.__wbg_sety_16ff3ff771600f8c = function(e, t) {
      r(e).y = t >>> 0;
    }, n.wbg.__wbg_setz_b2c09b24c996ee06 = function(e, t) {
      r(e).z = t >>> 0;
    }, n.wbg.__wbg_stack_0ed75d68575b0f3c = function(e, t) {
      const _ = r(t).stack, o = P(_, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
      l().setInt32(e + 4 * 1, s, true), l().setInt32(e + 4 * 0, o, true);
    }, n.wbg.__wbg_static_accessor_GLOBAL_88a902d13a557d07 = function() {
      const e = typeof global > "u" ? null : global;
      return R(e) ? 0 : b(e);
    }, n.wbg.__wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0 = function() {
      const e = typeof globalThis > "u" ? null : globalThis;
      return R(e) ? 0 : b(e);
    }, n.wbg.__wbg_static_accessor_SELF_37c5d418e4bf5819 = function() {
      const e = typeof self > "u" ? null : self;
      return R(e) ? 0 : b(e);
    }, n.wbg.__wbg_static_accessor_WINDOW_5de37043a91a9c40 = function() {
      const e = typeof window > "u" ? null : window;
      return R(e) ? 0 : b(e);
    }, n.wbg.__wbg_status_f6360336ca686bf0 = function(e) {
      return r(e).status;
    }, n.wbg.__wbg_submit_252766c4e0945cee = function(e, t) {
      r(e).submit(r(t));
    }, n.wbg.__wbg_text_7805bea50de2af49 = function() {
      return C(function(e) {
        const t = r(e).text();
        return b(t);
      }, arguments);
    }, n.wbg.__wbg_then_44b73946d2fb3e7d = function(e, t) {
      const _ = r(e).then(r(t));
      return b(_);
    }, n.wbg.__wbg_then_48b406749878a531 = function(e, t, _) {
      const o = r(e).then(r(t), r(_));
      return b(o);
    }, n.wbg.__wbg_toString_5285597960676b7b = function(e) {
      const t = r(e).toString();
      return b(t);
    }, n.wbg.__wbg_toString_c813bbd34d063839 = function(e) {
      const t = r(e).toString();
      return b(t);
    }, n.wbg.__wbg_url_8f9653b899456042 = function(e, t) {
      const _ = r(t).url, o = P(_, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
      l().setInt32(e + 4 * 1, s, true), l().setInt32(e + 4 * 0, o, true);
    }, n.wbg.__wbg_width_5dde457d606ba683 = function(e) {
      return r(e).width;
    }, n.wbg.__wbg_writeBuffer_3193eaacefdcf39a = function() {
      return C(function(e, t, _, o, s, c) {
        r(e).writeBuffer(r(t), _, r(o), s, c);
      }, arguments);
    }, n.wbg.__wbg_writeTexture_cd7877c213ee5704 = function() {
      return C(function(e, t, _, o, s) {
        r(e).writeTexture(r(t), r(_), r(o), r(s));
      }, arguments);
    }, n.wbg.__wbindgen_cb_drop = function(e) {
      const t = ee(e).original;
      return t.cnt-- == 1 ? (t.a = 0, true) : false;
    }, n.wbg.__wbindgen_closure_wrapper2757 = function(e, t, _) {
      const o = ze(e, t, 221, mt);
      return b(o);
    }, n.wbg.__wbindgen_closure_wrapper2815 = function(e, t, _) {
      const o = ze(e, t, 232, ht);
      return b(o);
    }, n.wbg.__wbindgen_debug_string = function(e, t) {
      const _ = Se(r(t)), o = P(_, a.__wbindgen_export_2, a.__wbindgen_export_3), s = I;
      l().setInt32(e + 4 * 1, s, true), l().setInt32(e + 4 * 0, o, true);
    }, n.wbg.__wbindgen_is_function = function(e) {
      return typeof r(e) == "function";
    }, n.wbg.__wbindgen_is_null = function(e) {
      return r(e) === null;
    }, n.wbg.__wbindgen_is_undefined = function(e) {
      return r(e) === void 0;
    }, n.wbg.__wbindgen_memory = function() {
      const e = a.memory;
      return b(e);
    }, n.wbg.__wbindgen_number_get = function(e, t) {
      const _ = r(t), o = typeof _ == "number" ? _ : void 0;
      l().setFloat64(e + 8 * 1, R(o) ? 0 : o, true), l().setInt32(e + 4 * 0, !R(o), true);
    }, n.wbg.__wbindgen_number_new = function(e) {
      return b(e);
    }, n.wbg.__wbindgen_object_clone_ref = function(e) {
      const t = r(e);
      return b(t);
    }, n.wbg.__wbindgen_object_drop_ref = function(e) {
      ee(e);
    }, n.wbg.__wbindgen_string_get = function(e, t) {
      const _ = r(t), o = typeof _ == "string" ? _ : void 0;
      var s = R(o) ? 0 : P(o, a.__wbindgen_export_2, a.__wbindgen_export_3), c = I;
      l().setInt32(e + 4 * 1, c, true), l().setInt32(e + 4 * 0, s, true);
    }, n.wbg.__wbindgen_string_new = function(e, t) {
      const _ = w(e, t);
      return b(_);
    }, n.wbg.__wbindgen_throw = function(e, t) {
      throw new Error(w(e, t));
    }, n;
  }
  function Dt(n, e) {
    return a = n.exports, Pe.__wbindgen_wasm_module = e, N = null, Y = null, K = null, a;
  }
  async function Pe(n) {
    if (a !== void 0) return a;
    typeof n < "u" && (Object.getPrototypeOf(n) === Object.prototype ? { module_or_path: n } = n : console.warn("using deprecated parameters for the initialization function; pass a single object instead")), typeof n > "u" && (n = new URL("/infinite-chat/assets/infinite_chat_wasm_bg-77lnhlaY.wasm", import.meta.url));
    const e = $t();
    (typeof n == "string" || typeof Request == "function" && n instanceof Request || typeof URL == "function" && n instanceof URL) && (n = fetch(n));
    const { instance: t, module: _ } = await Bt(await n, e);
    return Dt(t, _);
  }
  let be = false, ae = null;
  const Ze = /* @__PURE__ */ new Map();
  let ke = 0;
  Wt = function() {
    return be;
  };
  function Te(n, e) {
    if (!be || ke === 0) return null;
    const t = Ze.get(n);
    return t == null ? null : t * e / ke;
  }
  zt = function(n, e = "/infinite-chat/fonts/lxgw-msdf") {
    if (be) return Promise.resolve();
    if (ae) return ae;
    const t = e.replace(/[^/]+$/, "");
    return ae = (async () => {
      const _ = await fetch(`${e}.json`).then((u) => u.json()), o = _.chars;
      ke = _.info.size;
      const s = new Uint32Array(o.length), c = new Float32Array(o.length * 7);
      o.forEach((u, g) => {
        s[g] = u.id, c.set([
          u.x,
          u.y,
          u.width,
          u.height,
          u.xoffset,
          u.yoffset,
          u.page
        ], g * 7), Ze.set(u.id, u.xadvance);
      });
      const i = [];
      for (const u of _.pages) {
        const g = await fetch(`${t}${u}`).then((A) => A.blob()), f = await createImageBitmap(g), d = new OffscreenCanvas(f.width, f.height).getContext("2d");
        if (!d) throw new Error("MSDF \u89E3\u7801:\u65E0 2D \u4E0A\u4E0B\u6587");
        d.drawImage(f, 0, 0);
        const y = d.getImageData(0, 0, f.width, f.height).data;
        i.push(new Uint8Array(y.buffer.slice(0)));
      }
      n.load_msdf({
        atlasW: _.common.scaleW,
        atlasH: _.common.scaleH,
        fontSize: _.info.size,
        ids: s,
        cells: c,
        pixels: i
      }), be = true, console.info(`[msdf] loaded ${o.length} glyphs / ${_.pages.length} pages`);
    })(), ae;
  };
  let se = null, qe = false;
  function Gt(n, e = "/infinite-chat/fonts/katex-msdf") {
    if (qe) return Promise.resolve();
    if (se) return se;
    const t = e.replace(/[^/]+$/, "");
    return se = (async () => {
      const _ = await fetch(`${e}.json`).then((u) => u.json()), o = _.chars, s = new Uint32Array(o.length), c = new Float32Array(o.length * 7);
      o.forEach((u, g) => {
        s[g] = u.id, c.set([
          u.x,
          u.y,
          u.width,
          u.height,
          u.xoffset,
          u.yoffset,
          u.page
        ], g * 7);
      });
      const i = [];
      for (const u of _.pages) {
        const g = await fetch(`${t}${u}`).then((A) => A.blob()), f = await createImageBitmap(g), d = new OffscreenCanvas(f.width, f.height).getContext("2d");
        if (!d) throw new Error("MSDF \u89E3\u7801:\u65E0 2D \u4E0A\u4E0B\u6587");
        d.drawImage(f, 0, 0);
        const y = d.getImageData(0, 0, f.width, f.height).data;
        i.push(new Uint8Array(y.buffer.slice(0)));
      }
      n.load_msdf({
        atlasW: _.common.scaleW,
        atlasH: _.common.scaleH,
        fontSize: _.info.size,
        ids: s,
        cells: c,
        pixels: i
      }), qe = true, console.info(`[math-msdf] loaded ${o.length} glyphs`);
    })(), se;
  }
  let ce, Qe;
  Ut = Object.freeze(Object.defineProperty({
    __proto__: null,
    loadMathMsdf: Gt,
    loadMsdf: zt,
    msdfAdvancePx: Te,
    msdfLoaded: Wt
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  bn = [
    [
      "code_bg",
      "code bg",
      [
        0.1,
        0.11,
        0.16,
        0.75
      ]
    ],
    [
      "code_chip",
      "code chip",
      [
        0.18,
        0.19,
        0.26,
        0.7
      ]
    ],
    [
      "code_border",
      "code border",
      [
        0.32,
        0.36,
        0.46,
        0.85
      ]
    ],
    [
      "quote_bar",
      "quote bar",
      [
        0.42,
        0.46,
        0.56,
        0.9
      ]
    ],
    [
      "head_rule",
      "head rule",
      [
        0.24,
        0.27,
        0.33,
        0.9
      ]
    ],
    [
      "hr_rule",
      "hr rule",
      [
        0.82,
        0.86,
        0.94,
        1
      ]
    ],
    [
      "selection",
      "selection",
      [
        0.26,
        0.45,
        0.92,
        0.4
      ]
    ],
    [
      "card_bg",
      "card bg",
      [
        0.14,
        0.16,
        0.21,
        0.55
      ]
    ],
    [
      "card_border",
      "card border",
      [
        0.3,
        0.34,
        0.44,
        0.7
      ]
    ],
    [
      "diff_add_bg",
      "diff add",
      [
        0.22,
        0.45,
        0.27,
        0.35
      ]
    ],
    [
      "diff_del_bg",
      "diff del",
      [
        0.5,
        0.22,
        0.24,
        0.35
      ]
    ]
  ];
  fn = [
    [
      "dur_glyph",
      "dur glyph ms",
      200,
      0,
      600,
      10
    ],
    [
      "dur_element",
      "dur element ms",
      240,
      0,
      800,
      10
    ],
    [
      "dur_block",
      "dur block ms",
      320,
      0,
      1e3,
      10
    ],
    [
      "dur_panel",
      "dur panel ms",
      420,
      0,
      1200,
      10
    ],
    [
      "exit_factor",
      "exit factor",
      0.8,
      0,
      1,
      0.05
    ],
    [
      "stagger_glyph",
      "stagger glyph ms",
      25,
      0,
      100,
      5
    ],
    [
      "stagger_item",
      "stagger item ms",
      50,
      0,
      200,
      5
    ],
    [
      "stagger_stage",
      "stagger stage ms",
      60,
      0,
      200,
      5
    ],
    [
      "space_turn",
      "space turn px",
      24,
      0,
      96,
      2
    ],
    [
      "space_part",
      "space part px",
      12,
      0,
      48,
      2
    ],
    [
      "space_inset",
      "space inset px",
      16,
      0,
      32,
      1
    ]
  ];
  gn = [
    [
      "tempo_ms",
      "tempo ms/\u62CD",
      180,
      20,
      400,
      5
    ],
    [
      "rest",
      "rest \u8FB9\u754C\u505C\u987F\xD7",
      1,
      0,
      2,
      0.1
    ],
    [
      "final_lengthening",
      "\u672B\u5EF6\u957F\xD7",
      1.4,
      1,
      2,
      0.05
    ],
    [
      "accent",
      "accent \u91CD\u97F3",
      1,
      0,
      2,
      0.1
    ],
    [
      "microtiming",
      "microtiming \u03B5",
      1,
      0,
      1,
      0.05
    ],
    [
      "accelerando",
      "accelerando",
      0,
      0,
      1,
      0.05
    ]
  ];
  dn = [
    "reader",
    "typewriter",
    "flow"
  ];
  ce = {
    table: {
      vAlign: "center",
      hAlign: "auto"
    },
    tableRender: {
      lineColor: [
        0.1569,
        0.1569,
        0.1569,
        1
      ],
      headerFill: [
        0,
        0,
        0,
        0
      ],
      aoColor: [
        1,
        1,
        1
      ],
      lineW: 1,
      ao: 0,
      aoWidth: 10,
      radius: 0
    },
    theme: {},
    motion: {},
    rhythmPreset: "",
    rhythm: {}
  };
  Qe = "infinite-chat.styleConfig.v2";
  function Ne(n) {
    return JSON.parse(JSON.stringify(n));
  }
  let Ae = Ht();
  function Ht() {
    try {
      const n = localStorage.getItem(Qe);
      if (!n) return Ne(ce);
      const e = JSON.parse(n);
      return {
        table: {
          ...ce.table,
          ...e.table ?? {}
        },
        tableRender: {
          ...ce.tableRender,
          ...e.tableRender ?? {}
        },
        theme: {
          ...e.theme ?? {}
        },
        motion: {
          ...e.motion ?? {}
        },
        rhythmPreset: e.rhythmPreset ?? "",
        rhythm: {
          ...e.rhythm ?? {}
        }
      };
    } catch {
      return Ne(ce);
    }
  }
  qt = function() {
    return Ae;
  };
  ln = function(n) {
    Ae = n;
    try {
      localStorage.setItem(Qe, JSON.stringify(Ae));
    } catch {
    }
  };
  const X = typeof window < "u" && window.devicePixelRatio || 1, Nt = 14, S = Math.round(Nt * X);
  let O = Math.ceil(S * 1.6);
  const jt = 1.6, Xt = 1.25;
  wn = function(n) {
    O = Math.ceil(S * (n === "tui" ? Xt : jt));
  };
  pn = function() {
    const n = Te("M".codePointAt(0) ?? 0, S);
    let e;
    if (n != null && n > 0) e = n;
    else {
      const t = tt();
      t.font = `${S}px ${j}`, e = Math.max(1, t.measureText("M").width);
    }
    return {
      cellW: e,
      cellH: O
    };
  };
  const L = 128, te = 8, Vt = L - 2 * te, fe = {
    system: {
      sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif',
      mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", "Noto Sans Mono CJK SC", monospace'
    },
    serif: {
      sans: 'ui-serif, Georgia, Cambria, "Times New Roman", "Songti SC", "SimSun", "Noto Serif CJK SC", serif',
      mono: 'ui-monospace, SFMono-Regular, Menlo, "Noto Sans Mono CJK SC", monospace'
    },
    rounded: {
      sans: '"SF Pro Rounded", ui-rounded, "Hiragino Maru Gothic ProN", "Yuanti SC", system-ui, sans-serif',
      mono: 'ui-monospace, SFMono-Regular, Menlo, "Noto Sans Mono CJK SC", monospace'
    },
    mono: {
      sans: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Noto Sans Mono CJK SC", monospace',
      mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Noto Sans Mono CJK SC", monospace'
    }
  };
  let Me = "system", U = fe.system.sans, j = fe.system.mono;
  mn = function(n) {
    const e = fe[n];
    return !e || n === Me ? false : (Me = n, U = e.sans, j = e.mono, true);
  };
  hn = function() {
    return Me;
  };
  let Ce = "auto";
  yn = function(n) {
    Ce = n;
  };
  function Kt() {
    return Ce === "auto" || Ce === "msdf";
  }
  xn = function() {
    return Object.keys(fe);
  };
  function et(n) {
    switch (n) {
      case 1:
        return `bold ${S}px ${U}`;
      case 2:
        return `italic ${S}px ${U}`;
      case 3:
        return `bold italic ${S}px ${U}`;
      case 4:
      case 5:
      case 43:
      case 44:
      case 45:
      case 46:
      case 47:
      case 48:
      case 49:
      case 50:
        return `${S}px ${j}`;
      case 6:
      case 10:
      case 11:
      case 12:
      case 13:
      case 14:
        return `bold ${S}px ${U}`;
      case 8:
        return `italic ${S}px ${U}`;
      case 16:
        return `bold ${S}px ${U}`;
      case 17:
      case 18:
        return `${S}px ${j}`;
      case 19:
        return `bold ${S}px ${j}`;
      case 20:
        return `italic ${S}px ${j}`;
      case 21:
        return `${S}px ${j}`;
      case 26:
        return `${S}px KaTeX_Main`;
      case 27:
        return `bold ${S}px KaTeX_Main`;
      case 28:
        return `italic ${S}px KaTeX_Main`;
      case 29:
        return `bold italic ${S}px KaTeX_Main`;
      case 30:
        return `italic ${S}px KaTeX_Math`;
      case 31:
        return `${S}px KaTeX_AMS`;
      case 32:
        return `${S}px KaTeX_Size1`;
      case 33:
        return `${S}px KaTeX_Size2`;
      case 34:
        return `${S}px KaTeX_Size3`;
      case 35:
        return `${S}px KaTeX_Size4`;
      case 36:
        return `${S}px KaTeX_Caligraphic`;
      case 37:
        return `${S}px KaTeX_Fraktur`;
      case 38:
        return `${S}px KaTeX_SansSerif`;
      case 39:
        return `${S}px KaTeX_Script`;
      case 40:
        return `${S}px KaTeX_Typewriter`;
      default:
        return `${S}px ${U}`;
    }
  }
  function je(n) {
    return n === 5 || n >= 43 && n <= 50;
  }
  function Ie(n) {
    switch (n) {
      case 6:
      case 10:
      case 11:
      case 12:
      case 13:
        return 1;
      case 14:
        return 1;
      case 4:
      case 5:
      case 43:
      case 44:
      case 45:
      case 46:
      case 47:
      case 48:
      case 49:
      case 50:
        return 0.93;
      case 24:
        return 0.7;
      case 25:
        return 0.85;
      default:
        return 1;
    }
  }
  const Yt = new Intl.Segmenter(void 0, {
    granularity: "grapheme"
  });
  let ye = null;
  function tt() {
    if (!ye) {
      const n = new OffscreenCanvas(8, 8).getContext("2d");
      if (!n) throw new Error("\u65E0\u6CD5\u521B\u5EFA 2D \u6D4B\u91CF\u4E0A\u4E0B\u6587");
      ye = n;
    }
    return ye;
  }
  function Jt(n, e) {
    if (Kt()) {
      const _ = [
        ...n
      ];
      if (_.length === 1) {
        const o = Te(_[0].codePointAt(0) ?? 0, S);
        if (o != null) return o * Ie(e);
      }
    }
    const t = tt();
    return t.font = et(e), Math.max(1, t.measureText(n).width) * Ie(e);
  }
  function Zt(n) {
    const e = n.codePointAt(0) ?? 0;
    return e >= 19968 && e <= 40959 || e >= 12352 && e <= 12543 || e >= 44032 && e <= 55203 || e >= 65280 && e <= 65519 || e >= 12288 && e <= 12351;
  }
  function Qt(n) {
    return /\s/.test(n);
  }
  const en = "\u3002\uFF0C\u3001\uFF1B\uFF1A\uFF1F\uFF01\uFF09\u300D\u300F\u3011\u300B\u3009\u201D\u2019%\u2026\xB7.,;:?!)]}";
  function tn(n) {
    return en.includes(n);
  }
  function nn(n) {
    return n >= 17 && n <= 21;
  }
  const ie = Math.round(8 * X), xe = Math.max(1, Math.round(X));
  function rn(n, e, t, _) {
    const o = [];
    let s = e, c = 0, i = e;
    for (; i < t; ) {
      if (n[i].nl) {
        o.push([
          s,
          i
        ]), i++, s = i, c = 0;
        continue;
      }
      let u = i, g = 0;
      if (n[i].cjk) u = i + 1, g = n[i].adv;
      else for (; u < t && !n[u].cjk && !n[u].nl && (g += n[u].adv, u++, !n[u - 1].space); ) ;
      c + g > _ && c > 0 && (o.push([
        s,
        i
      ]), s = i, c = 0), c += g, i = u;
    }
    return (s < t || o.length === 0) && o.push([
      s,
      t
    ]), o;
  }
  function _n(n, e, t, _, o, s) {
    const c = t.rows.length, i = Math.max(t.aligns.length, ...t.rows.map((v) => v.length)), u = (v, k) => {
      const p = t.rows[v][k];
      return p ? [
        _[p[0]],
        _[p[1]]
      ] : [
        0,
        0
      ];
    }, g = (v, k) => {
      let p = 0;
      for (let h = v; h < k; h++) n[h].nl || (p += n[h].adv);
      return p;
    }, f = new Array(i).fill(0);
    for (let v = 0; v < c; v++) for (let k = 0; k < t.rows[v].length; k++) {
      const [p, h] = u(v, k);
      f[k] = Math.max(f[k], g(p, h));
    }
    const m = i * ie * 2 + Math.max(0, i - 1) * xe, d = s - m, y = f.reduce((v, k) => v + k, 0);
    if (y > d && d > 0) {
      const v = Math.round(4 * S), k = f.map((M) => M / y * d);
      let p = 0, h = 0;
      for (const M of k) M < v ? p += v - M : h += M - v;
      for (let M = 0; M < i; M++) k[M] < v ? f[M] = v : f[M] = p <= h && h > 0 ? k[M] - (k[M] - v) / h * p : k[M];
    }
    const A = new Array(i).fill(0);
    for (let v = 1; v < i; v++) A[v] = A[v - 1] + f[v - 1] + ie * 2 + xe;
    let T = o;
    const D = [];
    for (let v = 0; v < c; v++) {
      D.push(T);
      const k = [];
      let p = 1;
      for (let x = 0; x < t.rows[v].length; x++) {
        const [E, F] = u(v, x), V = rn(n, E, F, f[x]);
        k.push(V), V.length > p && (p = V.length);
      }
      const h = qt().table, M = h.hAlign === "auto" ? null : h.hAlign === "right" ? 2 : h.hAlign === "center" ? 1 : 0;
      for (let x = 0; x < t.rows[v].length; x++) {
        const E = M ?? t.aligns[x] ?? 0, F = k[x], V = (F.length - 1) * O + S, Fe = Math.max(0, p * O - V), ct = h.vAlign === "top" ? 0 : h.vAlign === "bottom" ? Fe : Fe / 2;
        for (let oe = 0; oe < F.length; oe++) {
          const [Oe, Re] = F[oe], Be = f[x] - g(Oe, Re), it = E === 2 ? Be : E === 1 ? Be / 2 : 0;
          let $e = A[x] + ie + Math.max(0, it);
          const ut = T + ct + oe * O;
          for (let H = Oe; H < Re; H++) {
            const z = n[H];
            e[H * 4] = $e - z.off, e[H * 4 + 1] = ut + (O - z.cell) * 0.5 - z.off, e[H * 4 + 2] = z.nl ? 0 : z.cell, e[H * 4 + 3] = z.nl ? 0 : z.cell, z.nl || ($e += z.adv);
          }
        }
      }
      T += p * O;
    }
    const W = [];
    for (let v = 1; v < i; v++) W.push(A[v] - xe * 0.5);
    const $ = D.slice(1), re = A[i - 1] + f[i - 1] + 2 * ie, _e = c >= 2 ? D[1] : o, ge = {
      x: 0,
      y: o,
      w: re,
      h: T - o,
      headerBottom: _e,
      cols: W,
      rows: $
    };
    return {
      height: T - o,
      panel: ge
    };
  }
  function on(n) {
    const e = [];
    for (const t of n) e.push(t.x, t.y, t.w, t.h, t.headerBottom, t.cols.length, t.rows.length, ...t.cols, ...t.rows);
    return new Float32Array(e);
  }
  let nt = 0, rt = 0;
  vn = function() {
    return {
      layout: nt,
      measure: rt
    };
  };
  function Ee(n, e, t, _) {
    nt++;
    const o = S / Vt, s = [], c = [];
    for (let p = 0; p < n.length; p++) {
      c.push(s.length);
      const h = e[p] ?? 0, M = Ie(h);
      for (const { segment: x } of Yt.segment(n[p])) {
        const E = x === `
`;
        s.push({
          cluster: x,
          role: h,
          adv: E ? 0 : Jt(x, h),
          cell: L * o * M,
          off: te * o * M,
          lineH: Math.ceil(O * M),
          nl: E,
          cjk: !E && Zt(x),
          space: !E && Qt(x),
          inTable: false
        });
      }
    }
    c.push(s.length);
    const i = /* @__PURE__ */ new Map();
    if (_) for (const p of _) {
      if (!p.rows.length || !p.rows[0].length) continue;
      const h = c[p.rows[0][0][0]];
      let M = h;
      for (const x of p.rows) for (const [, E] of x) M = Math.max(M, c[E]);
      i.set(h, {
        region: p,
        gEnd: M
      });
    }
    {
      let p = 0;
      for (let h = 0; h <= s.length; h++) if (h === s.length || s[h].nl) {
        let M = false;
        for (let x = p; x < h; x++) if (nn(s[x].role)) {
          M = true;
          break;
        }
        if (M) for (let x = p; x < h; x++) s[x].inTable = true;
        p = h + 1;
      }
    }
    const u = new Float32Array(s.length * 4);
    let g = 0, f = 0, m = O;
    const d = Math.round(12 * X), y = Math.round(24 * X), A = Math.round(8 * X), T = (p) => p === 6 || p >= 10 && p <= 14;
    let D = false, W = false, $ = -1, re = false, _e = false;
    const ge = (p, h) => {
      u[h * 4] = g - p.off, u[h * 4 + 1] = f + (p.lineH - p.cell) * 0.5 - p.off, u[h * 4 + 2] = p.cell, u[h * 4 + 3] = p.cell, g += p.adv, p.lineH > m && (m = p.lineH), $ < 0 && !p.space && ($ = p.role), T(p.role) && (D = true), je(p.role) && (W = true);
    }, v = [];
    let k = 0;
    for (; k < s.length; ) {
      const p = i.get(k);
      if (p) {
        const F = _n(s, u, p.region, c, f, t);
        f += F.height, v.push(F.panel), g = 0, m = O, k = p.gEnd;
        continue;
      }
      const h = s[k];
      if (h.nl) {
        u[k * 4] = g - h.off, u[k * 4 + 1] = f - h.off, u[k * 4 + 2] = 0, u[k * 4 + 3] = 0, g === 0 ? f += _e ? m : re ? y : d : (re = D, _e = W, f += m, $ === 9 && (f += A)), g = 0, m = O, D = false, W = false, $ = -1, k++;
        continue;
      }
      const M = h.inTable || je(h.role);
      let x = k, E = 0;
      if (M) for (; x < s.length && !s[x].nl; ) E += s[x].adv, x++;
      else if (h.cjk) x = k + 1, E = h.adv;
      else for (; x < s.length && !s[x].nl && !s[x].cjk && (E += s[x].adv, x++, !s[x - 1].space); ) ;
      !M && g + E > t && g > 0 && !tn(h.cluster) && (g = 0, f += m, m = O);
      for (let F = k; F < x; F++) ge(s[F], F);
      k = x;
    }
    return v.length > 0 ? {
      positions: u,
      tables: on(v)
    } : u;
  }
  const J = /* @__PURE__ */ new Map(), an = 4096;
  let _t = 0, ot = 0;
  function at(n, e, t) {
    if (rt++, n.length === 0) return new Float32Array([
      0,
      0
    ]);
    const _ = `${n.join("")}${Array.from(e).join(",")}@${Math.round(t)}`, o = J.get(_);
    if (o) return _t++, o;
    ot++;
    const s = Ee(n, e, t), c = s instanceof Float32Array ? s : s.positions;
    let i = 0, u = 0;
    for (let f = 0; f + 3 < c.length; f += 4) {
      const m = c[f + 2];
      m > 0 && (i = Math.max(i, c[f] + m)), u = Math.max(u, c[f + 1] + c[f + 3]);
    }
    const g = new Float32Array([
      Math.min(i, t),
      u
    ]);
    return J.size >= an && J.clear(), J.set(_, g), g;
  }
  Sn = function() {
    return {
      hits: _t,
      misses: ot,
      size: J.size
    };
  };
  const sn = 8, ne = 1e20;
  let ve = null;
  function cn() {
    if (!ve) {
      const n = new OffscreenCanvas(L, L).getContext("2d", {
        willReadFrequently: true
      });
      if (!n) throw new Error("\u65E0\u6CD5\u521B\u5EFA SDF \u5149\u6805\u4E0A\u4E0B\u6587");
      ve = n;
    }
    return ve;
  }
  function Xe(n, e, t, _) {
    const o = new Float64Array(_), s = new Int32Array(_), c = new Float64Array(_ + 1);
    for (let u = 0; u < _; u++) o[u] = n[e + u * t];
    s[0] = 0, c[0] = -ne, c[1] = ne;
    let i = 0;
    for (let u = 1; u < _; u++) {
      let g = (o[u] + u * u - (o[s[i]] + s[i] * s[i])) / (2 * u - 2 * s[i]);
      for (; g <= c[i]; ) i--, g = (o[u] + u * u - (o[s[i]] + s[i] * s[i])) / (2 * u - 2 * s[i]);
      i++, s[i] = u, c[i] = g, c[i + 1] = ne;
    }
    i = 0;
    for (let u = 0; u < _; u++) {
      for (; c[i + 1] < u; ) i++;
      const g = u - s[i];
      n[e + u * t] = o[s[i]] + g * g;
    }
  }
  function Ve(n, e, t) {
    for (let _ = 0; _ < e; _++) Xe(n, _, e, t);
    for (let _ = 0; _ < t; _++) Xe(n, _ * e, 1, e);
  }
  function st(n, e, t = 1) {
    const _ = cn();
    _.clearRect(0, 0, L, L);
    const o = L - 2 * te;
    _.font = et(e).replace(/^\s*(bold |italic )*\d+px/, `$1${o}px`), _.textBaseline = "top", _.fillStyle = "#ffffff", _.fillText(n, te, te);
    const s = _.getImageData(0, 0, L, L).data, c = L * L;
    if (t === 3) return new Uint8Array(s);
    if (t === 0) {
      const f = new Uint8Array(c * 4);
      for (let m = 0; m < c; m++) {
        const d = s[m * 4 + 3];
        f[m * 4] = d, f[m * 4 + 1] = d, f[m * 4 + 2] = d, f[m * 4 + 3] = 255;
      }
      return f;
    }
    const i = new Float64Array(c), u = new Float64Array(c);
    for (let f = 0; f < c; f++) {
      const m = s[f * 4 + 3] / 255;
      if (m === 1) i[f] = 0, u[f] = ne;
      else if (m === 0) i[f] = ne, u[f] = 0;
      else {
        const d = Math.max(0, 0.5 - m), y = Math.max(0, m - 0.5);
        i[f] = d * d, u[f] = y * y;
      }
    }
    Ve(i, L, L), Ve(u, L, L);
    const g = new Uint8Array(c * 4);
    for (let f = 0; f < c; f++) {
      const m = Math.sqrt(i[f]) - Math.sqrt(u[f]), d = Math.max(0, Math.min(255, Math.round(255 * (0.5 - m / (2 * sn)))));
      g[f * 4] = d, g[f * 4 + 1] = d, g[f * 4 + 2] = d, g[f * 4 + 3] = 255;
    }
    return g;
  }
  function un(n, e) {
    const t = () => window.devicePixelRatio || 1;
    let _ = 0;
    const o = (d) => {
      d.preventDefault();
      const y = t();
      if (d.ctrlKey) {
        const A = Math.exp(-d.deltaY * 0.01), T = n.getBoundingClientRect();
        e.zoom_at(A, (d.clientX - T.left) * y, (d.clientY - T.top) * y);
      } else {
        const A = n.getBoundingClientRect(), T = (d.clientX - A.left) * y, D = (d.clientY - A.top) * y, W = e.code_block_at_screen(T, D);
        if (W) {
          _ += d.deltaY / O;
          const $ = Math.trunc(_);
          _ -= $, ($ !== 0 || d.deltaX !== 0) && e.scroll_code_block(W, d.deltaX * y, $);
        } else e.pan_by(d.deltaX * y, d.deltaY * y);
      }
    };
    n.addEventListener("wheel", o, {
      passive: false
    });
    const s = (d) => {
      const y = n.getBoundingClientRect(), A = t();
      return [
        (d.clientX - y.left) * A,
        (d.clientY - y.top) * A
      ];
    };
    let c = false, i = null;
    const u = (d) => {
      if (d.button !== 0) return;
      c = true, i = [
        d.clientX,
        d.clientY
      ];
      const [y, A] = s(d);
      e.ask_press(y, A), e.set_pointer_down(true), n.setPointerCapture(d.pointerId), n.style.cursor = "grabbing";
    }, g = (d) => {
      const [y, A] = s(d);
      if (e.set_pointer(y, A), !c) {
        n.style.cursor = e.ask_hit_at(y, A) || e.link_hit_at(y, A) ? "pointer" : "";
        return;
      }
      const T = t();
      e.pan_by(-d.movementX * T, -d.movementY * T);
    }, f = () => e.set_pointer(-1, -1), m = (d) => {
      if (e.set_pointer_down(false), !!c && (c = false, n.hasPointerCapture(d.pointerId) && n.releasePointerCapture(d.pointerId), n.style.cursor = "", e.ask_release(), i)) {
        const y = Math.hypot(d.clientX - i[0], d.clientY - i[1]);
        if (i = null, y <= 4) {
          const [A, T] = s(d);
          e.tap(A, T);
        }
      }
    };
    return n.addEventListener("pointerdown", u), n.addEventListener("pointermove", g), n.addEventListener("pointerup", m), n.addEventListener("pointercancel", m), n.addEventListener("pointerleave", f), () => {
      n.removeEventListener("wheel", o), n.removeEventListener("pointerdown", u), n.removeEventListener("pointermove", g), n.removeEventListener("pointerup", m), n.removeEventListener("pointercancel", m), n.removeEventListener("pointerleave", f);
    };
  }
  function Le(n, e, t = 0) {
    if (n.effect_preset_name() !== "") {
      n.set_glyph_mode(e);
      return;
    }
    t < 600 && requestAnimationFrame(() => Le(n, e, t + 1));
  }
  kn = async function(n) {
    const e = document.getElementById("chat"), t = window.devicePixelRatio || 1, _ = e.clientWidth || window.innerWidth, o = e.clientHeight || window.innerHeight;
    e.width = Math.round(_ * t), e.height = Math.round(o * t);
    {
      const { watchCanvasRect: i } = await B(async () => {
        const { watchCanvasRect: u } = await import("./canvas-rect-1-4iAlsh.js");
        return {
          watchCanvasRect: u
        };
      }, []);
      i(e);
    }
    const s = await Pe(), c = new Je(e, {
      layout: Ee,
      measure: at,
      rasterize: st,
      serverUrl: n.serverUrl,
      sessionId: n.sessionId,
      replay: n.replay
    });
    c.set_math_em(S), n.rhythmPreset && c.set_reveal_preset(n.rhythmPreset), c.start(), n.glyphMode != null && Le(c, n.glyphMode), c.set_open_url_handler((i) => {
      window.open(i, "_blank", "noopener,noreferrer");
    }), un(e, c);
    {
      const { pumpImageLoads: i } = await B(async () => {
        const { pumpImageLoads: u } = await import("./image-loader-DEJYb4Ur.js");
        return {
          pumpImageLoads: u
        };
      }, []);
      setInterval(() => i(c), 120);
    }
    e.setAttribute("role", "main"), e.setAttribute("aria-label", "\u5BF9\u8BDD\u753B\u5E03"), e.setAttribute("tabindex", "0");
    {
      const { mountAnnouncer: i } = await B(async () => {
        const { mountAnnouncer: u } = await import("./announcer-C1vfQ71U.js");
        return {
          mountAnnouncer: u
        };
      }, []);
      i(c);
    }
    {
      const i = document.createElement("button");
      i.className = "jump-latest", i.textContent = "\u2193 \u8DF3\u5230\u6700\u65B0", i.setAttribute("aria-label", "\u8DF3\u5230\u6700\u65B0\u6D88\u606F"), i.style.cssText = "position:fixed;right:24px;bottom:96px;z-index:60;padding:6px 12px;border-radius:16px;border:1px solid #333;background:#1d1d20;color:#ededed;cursor:pointer;font-size:13px;display:none", i.addEventListener("click", () => c.scroll_to_latest()), document.body.appendChild(i);
      const u = () => {
        const g = c.follow_state() === "released";
        i.style.display = g ? "" : "none", i.toggleAttribute("inert", !g), i.tabIndex = g ? 0 : -1, requestAnimationFrame(u);
      };
      requestAnimationFrame(u);
    }
    {
      const { pumpEmbedOverlay: i } = await B(async () => {
        const { pumpEmbedOverlay: y } = await import("./embed-overlay-C3Xossmv.js");
        return {
          pumpEmbedOverlay: y
        };
      }, __vite__mapDeps([0,1])), { pumpCopyButtons: u } = await B(async () => {
        const { pumpCopyButtons: y } = await import("./copy-button-Cw-KNp8G.js");
        return {
          pumpCopyButtons: y
        };
      }, __vite__mapDeps([2,1])), { pumpTextLayer: g, attachSelection: f } = await B(async () => {
        const { pumpTextLayer: y, attachSelection: A } = await import("./text-layer-knFZa3Wj.js");
        return {
          pumpTextLayer: y,
          attachSelection: A
        };
      }, __vite__mapDeps([3,1])), { pumpAskOverlay: m } = await B(async () => {
        const { pumpAskOverlay: y } = await import("./ask-overlay-DIIdeubT.js");
        return {
          pumpAskOverlay: y
        };
      }, __vite__mapDeps([4,1]));
      if (f(c), n.findBar !== false) {
        const { mountFindBar: y } = await B(async () => {
          const { mountFindBar: A } = await import("./find-bar-BQX65CJo.js");
          return {
            mountFindBar: A
          };
        }, []);
        y(c);
      }
      const d = () => {
        i(c), u(c), g(c, e), m(c), requestAnimationFrame(d);
      };
      requestAnimationFrame(d);
    }
    window.__chat = c;
    {
      const { loadMathMsdf: i } = await B(async () => {
        const { loadMathMsdf: g } = await Promise.resolve().then(() => Ut);
        return {
          loadMathMsdf: g
        };
      }, void 0), { loadMathFonts: u } = await B(async () => {
        const { loadMathFonts: g } = await import("./math-fonts-CSpAQE3e.js");
        return {
          loadMathFonts: g
        };
      }, []);
      Promise.all([
        i(c).catch((g) => console.error("[math-msdf] load failed", g)),
        u().catch((g) => console.error("[math-fonts] load failed", g))
      ]).then(() => c.refresh_fonts());
    }
    return {
      chat: c,
      canvas: e,
      wasmModule: s
    };
  };
  An = async function(n) {
    const e = document.getElementById(n.canvasId);
    if (!e) return {
      chat: null,
      ok: false
    };
    const t = window.devicePixelRatio || 1, _ = e.clientWidth || window.innerWidth, o = e.clientHeight || 480;
    e.width = Math.round(_ * t), e.height = Math.round(o * t);
    try {
      const { watchCanvasRect: s } = await B(async () => {
        const { watchCanvasRect: i } = await import("./canvas-rect-1-4iAlsh.js");
        return {
          watchCanvasRect: i
        };
      }, []);
      s(e), await Pe();
      const c = new Je(e, {
        layout: Ee,
        measure: at,
        rasterize: st,
        replay: []
      });
      return c.set_math_em(S), n.rhythmPreset && c.set_reveal_preset(n.rhythmPreset), c.start(), n.glyphMode != null && Le(c, n.glyphMode), window.__hero = c, {
        chat: c,
        ok: true
      };
    } catch (s) {
      return console.warn("[hero] \u5F15\u64CE\u8D77\u4E0D\u6765(\u65E0 WebGPU/WebGL2?)\u2192 \u9759\u5E27\u964D\u7EA7", s), {
        chat: null,
        ok: false
      };
    }
  };
})();
export {
  fn as M,
  gn as R,
  bn as T,
  B as _,
  __tla,
  wn as a,
  kn as b,
  pn as c,
  yn as d,
  An as e,
  xn as f,
  qt as g,
  ln as h,
  dn as i,
  hn as j,
  Wt as k,
  vn as l,
  Sn as m,
  zt as n,
  Ut as o,
  mn as s
};
