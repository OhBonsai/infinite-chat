const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/debug-panel-BYWudt6Y.js","assets/replay-config-BGpffVO3.js"])))=>i.map(i=>d[i]);
let cn, Bt, un, fn, bn, $t, Lt, gn, sn;
let __tla = (async () => {
  (function() {
    const e = document.createElement("link").relList;
    if (e && e.supports && e.supports("modulepreload")) return;
    for (const a of document.querySelectorAll('link[rel="modulepreload"]')) o(a);
    new MutationObserver((a) => {
      for (const _ of a) if (_.type === "childList") for (const c of _.addedNodes) c.tagName === "LINK" && c.rel === "modulepreload" && o(c);
    }).observe(document, {
      childList: true,
      subtree: true
    });
    function t(a) {
      const _ = {};
      return a.integrity && (_.integrity = a.integrity), a.referrerPolicy && (_.referrerPolicy = a.referrerPolicy), a.crossOrigin === "use-credentials" ? _.credentials = "include" : a.crossOrigin === "anonymous" ? _.credentials = "omit" : _.credentials = "same-origin", _;
    }
    function o(a) {
      if (a.ep) return;
      a.ep = true;
      const _ = t(a);
      fetch(a.href, _);
    }
  })();
  const ot = "modulepreload", at = function(n) {
    return "/infinite-chat/" + n;
  }, Le = {}, O = function(e, t, o) {
    let a = Promise.resolve();
    if (t && t.length > 0) {
      document.getElementsByTagName("link");
      const c = document.querySelector("meta[property=csp-nonce]"), u = (c == null ? void 0 : c.nonce) || (c == null ? void 0 : c.getAttribute("nonce"));
      a = Promise.allSettled(t.map((s) => {
        if (s = at(s), s in Le) return;
        Le[s] = true;
        const g = s.endsWith(".css"), f = g ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${s}"]${f}`)) return;
        const w = document.createElement("link");
        if (w.rel = g ? "stylesheet" : ot, g || (w.as = "script"), w.crossOrigin = "", w.href = s, u && w.setAttribute("nonce", u), document.head.appendChild(w), g) return new Promise((v, F) => {
          w.addEventListener("load", v), w.addEventListener("error", () => F(new Error(`Unable to preload CSS for ${s}`)));
        });
      }));
    }
    function _(c) {
      const u = new Event("vite:preloadError", {
        cancelable: true
      });
      if (u.payload = c, window.dispatchEvent(u), !u.defaultPrevented) throw c;
    }
    return a.then((c) => {
      for (const u of c || []) u.status === "rejected" && _(u.reason);
      return e().catch(_);
    });
  };
  let i;
  const N = new Array(128).fill(void 0);
  N.push(void 0, null, true, false);
  function r(n) {
    return N[n];
  }
  let ee = N.length;
  function b(n) {
    ee === N.length && N.push(N.length + 1);
    const e = ee;
    return ee = N[e], N[e] = n, e;
  }
  const je = typeof TextDecoder < "u" ? new TextDecoder("utf-8", {
    ignoreBOM: true,
    fatal: true
  }) : {
    decode: () => {
      throw Error("TextDecoder not available");
    }
  };
  typeof TextDecoder < "u" && je.decode();
  let J = null;
  function te() {
    return (J === null || J.byteLength === 0) && (J = new Uint8Array(i.memory.buffer)), J;
  }
  function p(n, e) {
    return n = n >>> 0, je.decode(te().subarray(n, n + e));
  }
  function P(n, e) {
    try {
      return n.apply(this, e);
    } catch (t) {
      i.__wbindgen_export_0(b(t));
    }
  }
  function D(n) {
    return n == null;
  }
  let L = 0;
  const ue = typeof TextEncoder < "u" ? new TextEncoder("utf-8") : {
    encode: () => {
      throw Error("TextEncoder not available");
    }
  }, _t = typeof ue.encodeInto == "function" ? function(n, e) {
    return ue.encodeInto(n, e);
  } : function(n, e) {
    const t = ue.encode(n);
    return e.set(t), {
      read: n.length,
      written: t.length
    };
  };
  function G(n, e, t) {
    if (t === void 0) {
      const u = ue.encode(n), s = e(u.length, 1) >>> 0;
      return te().subarray(s, s + u.length).set(u), L = u.length, s;
    }
    let o = n.length, a = e(o, 1) >>> 0;
    const _ = te();
    let c = 0;
    for (; c < o; c++) {
      const u = n.charCodeAt(c);
      if (u > 127) break;
      _[a + c] = u;
    }
    if (c !== o) {
      c !== 0 && (n = n.slice(c)), a = t(a, o, o = c + n.length * 3, 1) >>> 0;
      const u = te().subarray(a + c, a + o), s = _t(n, u);
      c += s.written, a = t(a, o, c, 1) >>> 0;
    }
    return L = c, a;
  }
  let q = null;
  function S() {
    return (q === null || q.buffer.detached === true || q.buffer.detached === void 0 && q.buffer !== i.memory.buffer) && (q = new DataView(i.memory.buffer)), q;
  }
  let Z = null;
  function qe() {
    return (Z === null || Z.byteLength === 0) && (Z = new Uint32Array(i.memory.buffer)), Z;
  }
  function st(n, e) {
    return n = n >>> 0, qe().subarray(n / 4, n / 4 + e);
  }
  function ct(n) {
    n < 132 || (N[n] = ee, ee = n);
  }
  function ne(n) {
    const e = r(n);
    return ct(n), e;
  }
  const Oe = typeof FinalizationRegistry > "u" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((n) => {
    i.__wbindgen_export_4.get(n.dtor)(n.a, n.b);
  });
  function Re(n, e, t, o) {
    const a = {
      a: n,
      b: e,
      cnt: 1,
      dtor: t
    }, _ = (...c) => {
      a.cnt++;
      const u = a.a;
      a.a = 0;
      try {
        return o(u, a.b, ...c);
      } finally {
        --a.cnt === 0 ? (i.__wbindgen_export_4.get(a.dtor)(u, a.b), Oe.unregister(a)) : a.a = u;
      }
    };
    return _.original = a, Oe.register(_, a, a), _;
  }
  function ve(n) {
    const e = typeof n;
    if (e == "number" || e == "boolean" || n == null) return `${n}`;
    if (e == "string") return `"${n}"`;
    if (e == "symbol") {
      const a = n.description;
      return a == null ? "Symbol" : `Symbol(${a})`;
    }
    if (e == "function") {
      const a = n.name;
      return typeof a == "string" && a.length > 0 ? `Function(${a})` : "Function";
    }
    if (Array.isArray(n)) {
      const a = n.length;
      let _ = "[";
      a > 0 && (_ += ve(n[0]));
      for (let c = 1; c < a; c++) _ += ", " + ve(n[c]);
      return _ += "]", _;
    }
    const t = /\[object ([^\]]+)\]/.exec(toString.call(n));
    let o;
    if (t && t.length > 1) o = t[1];
    else return toString.call(n);
    if (o == "Object") try {
      return "Object(" + JSON.stringify(n) + ")";
    } catch {
      return "Object";
    }
    return n instanceof Error ? `${n.name}: ${n.message}
${n.stack}` : o;
  }
  function it(n, e) {
    const t = e(n.length * 4, 4) >>> 0;
    return qe().set(n, t / 4), L = n.length, t;
  }
  function ut(n, e) {
    const t = e(n.length * 1, 1) >>> 0;
    return te().set(n, t / 1), L = n.length, t;
  }
  function ft(n, e) {
    i.__wbindgen_export_5(n, e);
  }
  function bt(n, e, t) {
    i.__wbindgen_export_6(n, e, b(t));
  }
  const ge = [
    "clamp-to-edge",
    "repeat",
    "mirror-repeat"
  ], $e = [
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
  ], gt = [
    "add",
    "subtract",
    "reverse-subtract",
    "min",
    "max"
  ], dt = [
    "uniform",
    "storage",
    "read-only-storage"
  ], lt = [
    "opaque",
    "premultiplied"
  ], de = [
    "never",
    "less",
    "equal",
    "less-equal",
    "greater",
    "not-equal",
    "greater-equal",
    "always"
  ], wt = [
    "none",
    "front",
    "back"
  ], De = [
    "nearest",
    "linear"
  ], pt = [
    "ccw",
    "cw"
  ], mt = [
    "uint16",
    "uint32"
  ], le = [
    "load",
    "clear"
  ], ht = [
    "nearest",
    "linear"
  ], yt = [
    "low-power",
    "high-performance"
  ], xt = [
    "point-list",
    "line-list",
    "line-strip",
    "triangle-list",
    "triangle-strip"
  ], vt = [
    "filtering",
    "non-filtering",
    "comparison"
  ], we = [
    "keep",
    "zero",
    "replace",
    "invert",
    "increment-clamp",
    "decrement-clamp",
    "increment-wrap",
    "decrement-wrap"
  ], St = [
    "write-only",
    "read-only",
    "read-write"
  ], pe = [
    "store",
    "discard"
  ], At = [
    "all",
    "stencil-only",
    "depth-only"
  ], Ct = [
    "1d",
    "2d",
    "3d"
  ], j = [
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
  ], Mt = [
    "float",
    "unfilterable-float",
    "depth",
    "sint",
    "uint"
  ], me = [
    "1d",
    "2d",
    "2d-array",
    "cube",
    "cube-array",
    "3d"
  ], Pt = [
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
  ], Tt = [
    "vertex",
    "instance"
  ], Ge = typeof FinalizationRegistry > "u" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((n) => i.__wbg_chatcanvas_free(n >>> 0, 1));
  class It {
    __destroy_into_raw() {
      const e = this.__wbg_ptr;
      return this.__wbg_ptr = 0, Ge.unregister(this), e;
    }
    free() {
      const e = this.__destroy_into_raw();
      i.__wbg_chatcanvas_free(e, 0);
    }
    push_event(e) {
      const t = G(e, i.__wbindgen_export_2, i.__wbindgen_export_3), o = L;
      i.chatcanvas_push_event(this.__wbg_ptr, t, o);
    }
    set_paused(e) {
      i.chatcanvas_set_paused(this.__wbg_ptr, e);
    }
    seek_reveal(e) {
      i.chatcanvas_seek_reveal(this.__wbg_ptr, e);
    }
    set_math_em(e) {
      i.chatcanvas_set_math_em(this.__wbg_ptr, e);
    }
    frame_embeds() {
      let e, t;
      try {
        const _ = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_frame_embeds(_, this.__wbg_ptr);
        var o = S().getInt32(_ + 4 * 0, true), a = S().getInt32(_ + 4 * 1, true);
        return e = o, t = a, p(o, a);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(e, t, 1);
      }
    }
    image_failed(e) {
      const t = G(e, i.__wbindgen_export_2, i.__wbindgen_export_3), o = L;
      i.chatcanvas_image_failed(this.__wbg_ptr, t, o);
    }
    refresh_fonts() {
      i.chatcanvas_refresh_fonts(this.__wbg_ptr);
    }
    set_selection(e) {
      const t = it(e, i.__wbindgen_export_2), o = L;
      i.chatcanvas_set_selection(this.__wbg_ptr, t, o);
    }
    visible_turns() {
      let e, t;
      try {
        const _ = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_visible_turns(_, this.__wbg_ptr);
        var o = S().getInt32(_ + 4 * 0, true), a = S().getInt32(_ + 4 * 1, true);
        return e = o, t = a, p(o, a);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(e, t, 1);
      }
    }
    reply_question() {
      i.chatcanvas_reply_question(this.__wbg_ptr);
    }
    restart_reveal() {
      i.chatcanvas_restart_reveal(this.__wbg_ptr);
    }
    session_status() {
      let e, t;
      try {
        const _ = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_session_status(_, this.__wbg_ptr);
        var o = S().getInt32(_ + 4 * 0, true), a = S().getInt32(_ + 4 * 1, true);
        return e = o, t = a, p(o, a);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(e, t, 1);
      }
    }
    set_glyph_mode(e) {
      i.chatcanvas_set_glyph_mode(this.__wbg_ptr, e);
    }
    set_reveal_cps(e) {
      i.chatcanvas_set_reveal_cps(this.__wbg_ptr, e);
    }
    set_virtualize(e) {
      i.chatcanvas_set_virtualize(this.__wbg_ptr, e);
    }
    set_reveal_slow(e) {
      i.chatcanvas_set_reveal_slow(this.__wbg_ptr, e);
    }
    set_stream_rate(e) {
      i.chatcanvas_set_stream_rate(this.__wbg_ptr, e);
    }
    set_table_style(e) {
      i.chatcanvas_set_table_style(this.__wbg_ptr, b(e));
    }
    reply_permission() {
      i.chatcanvas_reply_permission(this.__wbg_ptr);
    }
    scroll_code_block(e, t, o) {
      const a = G(e, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
      i.chatcanvas_scroll_code_block(this.__wbg_ptr, a, _, t, o);
    }
    upload_image_rgba(e, t, o, a, _) {
      const c = G(e, i.__wbindgen_export_2, i.__wbindgen_export_3), u = L, s = ut(t, i.__wbindgen_export_2), g = L;
      i.chatcanvas_upload_image_rgba(this.__wbg_ptr, c, u, s, g, o, a, _);
    }
    visible_text_runs() {
      let e, t;
      try {
        const _ = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_visible_text_runs(_, this.__wbg_ptr);
        var o = S().getInt32(_ + 4 * 0, true), a = S().getInt32(_ + 4 * 1, true);
        return e = o, t = a, p(o, a);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(e, t, 1);
      }
    }
    set_debug_geometry(e) {
      i.chatcanvas_set_debug_geometry(this.__wbg_ptr, e);
    }
    take_pending_images() {
      let e, t;
      try {
        const _ = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_take_pending_images(_, this.__wbg_ptr);
        var o = S().getInt32(_ + 4 * 0, true), a = S().getInt32(_ + 4 * 1, true);
        return e = o, t = a, p(o, a);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(e, t, 1);
      }
    }
    code_block_at_screen(e, t) {
      let o, a;
      try {
        const u = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_code_block_at_screen(u, this.__wbg_ptr, e, t);
        var _ = S().getInt32(u + 4 * 0, true), c = S().getInt32(u + 4 * 1, true);
        return o = _, a = c, p(_, c);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(o, a, 1);
      }
    }
    set_bench_fold_width(e) {
      i.chatcanvas_set_bench_fold_width(this.__wbg_ptr, e);
    }
    set_shaderbox_gallery(e) {
      i.chatcanvas_set_shaderbox_gallery(this.__wbg_ptr, e);
    }
    set_table_reveal_style(e) {
      i.chatcanvas_set_table_reveal_style(this.__wbg_ptr, e);
    }
    constructor(e, t) {
      try {
        const c = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_new(c, b(e), b(t));
        var o = S().getInt32(c + 4 * 0, true), a = S().getInt32(c + 4 * 1, true), _ = S().getInt32(c + 4 * 2, true);
        if (_) throw ne(a);
        return this.__wbg_ptr = o >>> 0, Ge.register(this, this.__wbg_ptr, this), this;
      } finally {
        i.__wbindgen_add_to_stack_pointer(16);
      }
    }
    find(e) {
      let t, o;
      try {
        const c = i.__wbindgen_add_to_stack_pointer(-16), u = G(e, i.__wbindgen_export_2, i.__wbindgen_export_3), s = L;
        i.chatcanvas_find(c, this.__wbg_ptr, u, s);
        var a = S().getInt32(c + 4 * 0, true), _ = S().getInt32(c + 4 * 1, true);
        return t = a, o = _, p(a, _);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16), i.__wbindgen_export_1(t, o, 1);
      }
    }
    step() {
      i.chatcanvas_step(this.__wbg_ptr);
    }
    tick(e) {
      i.chatcanvas_tick(this.__wbg_ptr, e);
    }
    start() {
      i.chatcanvas_start(this.__wbg_ptr);
    }
    stats() {
      const e = i.chatcanvas_stats(this.__wbg_ptr);
      return ne(e);
    }
    pan_by(e, t) {
      i.chatcanvas_pan_by(this.__wbg_ptr, e, t);
    }
    zoom_at(e, t, o) {
      i.chatcanvas_zoom_at(this.__wbg_ptr, e, t, o);
    }
    load_msdf(e) {
      try {
        const a = i.__wbindgen_add_to_stack_pointer(-16);
        i.chatcanvas_load_msdf(a, this.__wbg_ptr, b(e));
        var t = S().getInt32(a + 4 * 0, true), o = S().getInt32(a + 4 * 1, true);
        if (o) throw ne(t);
      } finally {
        i.__wbindgen_add_to_stack_pointer(16);
      }
    }
    note_send() {
      i.chatcanvas_note_send(this.__wbg_ptr);
    }
    scroll_to(e) {
      i.chatcanvas_scroll_to(this.__wbg_ptr, e);
    }
    stop_turn() {
      i.chatcanvas_stop_turn(this.__wbg_ptr);
    }
  }
  async function Et(n, e) {
    if (typeof Response == "function" && n instanceof Response) {
      if (typeof WebAssembly.instantiateStreaming == "function") try {
        return await WebAssembly.instantiateStreaming(n, e);
      } catch (o) {
        if (n.headers.get("Content-Type") != "application/wasm") console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", o);
        else throw o;
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
  function Ft() {
    const n = {};
    return n.wbg = {}, n.wbg.__wbg_Window_9e7ea8667e28eb00 = function(e) {
      const t = r(e).Window;
      return b(t);
    }, n.wbg.__wbg_WorkerGlobalScope_0169ffb9adb5f5ef = function(e) {
      const t = r(e).WorkerGlobalScope;
      return b(t);
    }, n.wbg.__wbg_addEventListener_90e553fdce254421 = function() {
      return P(function(e, t, o, a) {
        r(e).addEventListener(p(t, o), r(a));
      }, arguments);
    }, n.wbg.__wbg_apply_36be6a55257c99bf = function() {
      return P(function(e, t, o) {
        const a = r(e).apply(r(t), r(o));
        return b(a);
      }, arguments);
    }, n.wbg.__wbg_beginRenderPass_aefd0d9681a1f010 = function() {
      return P(function(e, t) {
        const o = r(e).beginRenderPass(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_buffer_09165b52af8c5237 = function(e) {
      const t = r(e).buffer;
      return b(t);
    }, n.wbg.__wbg_buffer_609cc3eee51ed158 = function(e) {
      const t = r(e).buffer;
      return b(t);
    }, n.wbg.__wbg_call_672a4d21634d4a24 = function() {
      return P(function(e, t) {
        const o = r(e).call(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_call_b8adc8b1d0a0d8eb = function() {
      return P(function(e, t, o, a, _) {
        const c = r(e).call(r(t), r(o), r(a), r(_));
        return b(c);
      }, arguments);
    }, n.wbg.__wbg_clientHeight_216178c194000db4 = function(e) {
      return r(e).clientHeight;
    }, n.wbg.__wbg_clientWidth_ce67a04dc15fce39 = function(e) {
      return r(e).clientWidth;
    }, n.wbg.__wbg_configure_86dd92dde48d105a = function() {
      return P(function(e, t) {
        r(e).configure(r(t));
      }, arguments);
    }, n.wbg.__wbg_createBindGroupLayout_f0635625a1a82bea = function() {
      return P(function(e, t) {
        const o = r(e).createBindGroupLayout(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_createBindGroup_043b06d20124f91e = function(e, t) {
      const o = r(e).createBindGroup(r(t));
      return b(o);
    }, n.wbg.__wbg_createBuffer_086a8bb05ced884a = function() {
      return P(function(e, t) {
        const o = r(e).createBuffer(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_createCommandEncoder_aa9ae9d445bb7abf = function(e, t) {
      const o = r(e).createCommandEncoder(r(t));
      return b(o);
    }, n.wbg.__wbg_createPipelineLayout_5cc7e994e46201c7 = function(e, t) {
      const o = r(e).createPipelineLayout(r(t));
      return b(o);
    }, n.wbg.__wbg_createRenderPipeline_47152f2f57b11194 = function() {
      return P(function(e, t) {
        const o = r(e).createRenderPipeline(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_createSampler_f3970b77a6f36963 = function(e, t) {
      const o = r(e).createSampler(r(t));
      return b(o);
    }, n.wbg.__wbg_createShaderModule_9ec201507fe4949e = function(e, t) {
      const o = r(e).createShaderModule(r(t));
      return b(o);
    }, n.wbg.__wbg_createTexture_09f18232c5ad6e69 = function() {
      return P(function(e, t) {
        const o = r(e).createTexture(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_createView_f7cd0a0356a46f3b = function() {
      return P(function(e, t) {
        const o = r(e).createView(r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_devicePixelRatio_68c391265f05d093 = function(e) {
      return r(e).devicePixelRatio;
    }, n.wbg.__wbg_document_d249400bd7bd996d = function(e) {
      const t = r(e).document;
      return D(t) ? 0 : b(t);
    }, n.wbg.__wbg_draw_d38c9207eb049f56 = function(e, t, o, a, _) {
      r(e).draw(t >>> 0, o >>> 0, a >>> 0, _ >>> 0);
    }, n.wbg.__wbg_end_d54348baf0bf3b70 = function(e) {
      r(e).end();
    }, n.wbg.__wbg_error_7534b8e9a36f1ab4 = function(e, t) {
      let o, a;
      try {
        o = e, a = t, console.error(p(e, t));
      } finally {
        i.__wbindgen_export_1(o, a, 1);
      }
    }, n.wbg.__wbg_fetch_a9bc66c159c18e19 = function(e) {
      const t = fetch(r(e));
      return b(t);
    }, n.wbg.__wbg_finish_db34a19c90c07af7 = function(e) {
      const t = r(e).finish();
      return b(t);
    }, n.wbg.__wbg_finish_e2d3808af76b422a = function(e, t) {
      const o = r(e).finish(r(t));
      return b(o);
    }, n.wbg.__wbg_from_2a5d3e218e67aa85 = function(e) {
      const t = Array.from(r(e));
      return b(t);
    }, n.wbg.__wbg_getContext_e9cf379449413580 = function() {
      return P(function(e, t, o) {
        const a = r(e).getContext(p(t, o));
        return D(a) ? 0 : b(a);
      }, arguments);
    }, n.wbg.__wbg_getContext_f65a0debd1e8f8e8 = function() {
      return P(function(e, t, o) {
        const a = r(e).getContext(p(t, o));
        return D(a) ? 0 : b(a);
      }, arguments);
    }, n.wbg.__wbg_getCurrentTexture_6ee19b05d6ba43ba = function() {
      return P(function(e) {
        const t = r(e).getCurrentTexture();
        return b(t);
      }, arguments);
    }, n.wbg.__wbg_getPreferredCanvasFormat_c56b5a9a243fe942 = function(e) {
      const t = r(e).getPreferredCanvasFormat();
      return (j.indexOf(t) + 1 || 96) - 1;
    }, n.wbg.__wbg_get_67b2ba62fc30de12 = function() {
      return P(function(e, t) {
        const o = Reflect.get(r(e), r(t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_get_b9b93047fe3cf45b = function(e, t) {
      const o = r(e)[t >>> 0];
      return b(o);
    }, n.wbg.__wbg_get_e27dfaeb6f46bd45 = function(e, t) {
      const o = r(e)[t >>> 0];
      return D(o) ? 0 : b(o);
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
      const o = r(t).label, a = G(o, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
      S().setInt32(e + 4 * 1, _, true), S().setInt32(e + 4 * 0, a, true);
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
    }, n.wbg.__wbg_log_0cc1b7768397bcfe = function(e, t, o, a, _, c, u, s) {
      let g, f;
      try {
        g = e, f = t, console.log(p(e, t), p(o, a), p(_, c), p(u, s));
      } finally {
        i.__wbindgen_export_1(g, f, 1);
      }
    }, n.wbg.__wbg_log_cb9e190acc5753fb = function(e, t) {
      let o, a;
      try {
        o = e, a = t, console.log(p(e, t));
      } finally {
        i.__wbindgen_export_1(o, a, 1);
      }
    }, n.wbg.__wbg_mark_7438147ce31e9d4b = function(e, t) {
      performance.mark(p(e, t));
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
      return P(function(e, t, o, a) {
        let _, c, u, s;
        try {
          _ = e, c = t, u = o, s = a, performance.measure(p(e, t), p(o, a));
        } finally {
          i.__wbindgen_export_1(_, c, 1), i.__wbindgen_export_1(u, s, 1);
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
      return P(function() {
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
      return P(function() {
        const e = new URLSearchParams();
        return b(e);
      }, arguments);
    }, n.wbg.__wbg_new_8a6f238a6ece86ea = function() {
      const e = new Error();
      return b(e);
    }, n.wbg.__wbg_new_9ffbe0a71eff35e3 = function() {
      return P(function(e, t) {
        const o = new URL(p(e, t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_new_a12002a7f91c75be = function(e) {
      const t = new Uint8Array(r(e));
      return b(t);
    }, n.wbg.__wbg_new_e3b321dcfef89fc7 = function(e) {
      const t = new Uint32Array(r(e));
      return b(t);
    }, n.wbg.__wbg_newnoargs_105ed471475aaf50 = function(e, t) {
      const o = new Function(p(e, t));
      return b(o);
    }, n.wbg.__wbg_newwithbyteoffsetandlength_d97e637ebe145a9a = function(e, t, o) {
      const a = new Uint8Array(r(e), t >>> 0, o >>> 0);
      return b(a);
    }, n.wbg.__wbg_newwithbyteoffsetandlength_f1dead44d1fc7212 = function(e, t, o) {
      const a = new Uint32Array(r(e), t >>> 0, o >>> 0);
      return b(a);
    }, n.wbg.__wbg_newwithlength_bd3de93688d68fbc = function(e) {
      const t = new Uint32Array(e >>> 0);
      return b(t);
    }, n.wbg.__wbg_newwithstr_78e86e03c4ae814e = function() {
      return P(function(e, t) {
        const o = new Request(p(e, t));
        return b(o);
      }, arguments);
    }, n.wbg.__wbg_newwithstrandinit_06c535e0a867c635 = function() {
      return P(function(e, t, o) {
        const a = new Request(p(e, t), r(o));
        return b(a);
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
      return D(t) ? 0 : b(t);
    }, n.wbg.__wbg_push_737cfc8c1432c2c6 = function(e, t) {
      return r(e).push(r(t));
    }, n.wbg.__wbg_querySelectorAll_40998fd748f057ef = function() {
      return P(function(e, t, o) {
        const a = r(e).querySelectorAll(p(t, o));
        return b(a);
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
      const o = r(e).requestAdapter(r(t));
      return b(o);
    }, n.wbg.__wbg_requestAnimationFrame_d7fd890aaefc3246 = function() {
      return P(function(e, t) {
        return r(e).requestAnimationFrame(r(t));
      }, arguments);
    }, n.wbg.__wbg_requestDevice_51509dadc50b2e9d = function(e, t) {
      const o = r(e).requestDevice(r(t));
      return b(o);
    }, n.wbg.__wbg_resolve_4851785c9c5f573d = function(e) {
      const t = Promise.resolve(r(e));
      return b(t);
    }, n.wbg.__wbg_search_c1c3bfbeadd96c47 = function() {
      return P(function(e, t) {
        const o = r(t).search, a = G(o, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
        S().setInt32(e + 4 * 1, _, true), S().setInt32(e + 4 * 0, a, true);
      }, arguments);
    }, n.wbg.__wbg_search_e0e79cfe010c5c23 = function(e, t) {
      const o = r(t).search, a = G(o, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
      S().setInt32(e + 4 * 1, _, true), S().setInt32(e + 4 * 0, a, true);
    }, n.wbg.__wbg_setBindGroup_a81ce7b3934585bf = function(e, t, o) {
      r(e).setBindGroup(t >>> 0, r(o));
    }, n.wbg.__wbg_setBindGroup_bb0c2c05b7c49401 = function() {
      return P(function(e, t, o, a, _, c, u) {
        r(e).setBindGroup(t >>> 0, r(o), st(a, _), c, u >>> 0);
      }, arguments);
    }, n.wbg.__wbg_setPipeline_78f8f6d440dddd25 = function(e, t) {
      r(e).setPipeline(r(t));
    }, n.wbg.__wbg_setVertexBuffer_b0d3128a04bfd766 = function(e, t, o, a, _) {
      r(e).setVertexBuffer(t >>> 0, r(o), a, _);
    }, n.wbg.__wbg_setVertexBuffer_edbff6ddb5055174 = function(e, t, o, a) {
      r(e).setVertexBuffer(t >>> 0, r(o), a);
    }, n.wbg.__wbg_set_10bad9bee0e9c58b = function(e, t, o) {
      r(e).set(r(t), o >>> 0);
    }, n.wbg.__wbg_set_65595bdd868b3009 = function(e, t, o) {
      r(e).set(r(t), o >>> 0);
    }, n.wbg.__wbg_set_bb8cecf6a62b9f46 = function() {
      return P(function(e, t, o) {
        return Reflect.set(r(e), r(t), r(o));
      }, arguments);
    }, n.wbg.__wbg_set_d23661d19148b229 = function(e, t, o) {
      r(e).set(r(t), o >>> 0);
    }, n.wbg.__wbg_seta_721deab95e136b71 = function(e, t) {
      r(e).a = t;
    }, n.wbg.__wbg_setaccess_b20bfa3ec6b65d05 = function(e, t) {
      r(e).access = St[t];
    }, n.wbg.__wbg_setaddressmodeu_9c0b2104a94d10f3 = function(e, t) {
      r(e).addressModeU = ge[t];
    }, n.wbg.__wbg_setaddressmodev_a9bedc188ff29608 = function(e, t) {
      r(e).addressModeV = ge[t];
    }, n.wbg.__wbg_setaddressmodew_5774889145ce3789 = function(e, t) {
      r(e).addressModeW = ge[t];
    }, n.wbg.__wbg_setalpha_2c7bdc9da833b6c2 = function(e, t) {
      r(e).alpha = r(t);
    }, n.wbg.__wbg_setalphamode_fc3528d234b1fefa = function(e, t) {
      r(e).alphaMode = lt[t];
    }, n.wbg.__wbg_setalphatocoverageenabled_314ce1ca1759b395 = function(e, t) {
      r(e).alphaToCoverageEnabled = t !== 0;
    }, n.wbg.__wbg_setarraylayercount_3c7942d623042874 = function(e, t) {
      r(e).arrayLayerCount = t >>> 0;
    }, n.wbg.__wbg_setarraystride_4b36d0822dea74a8 = function(e, t) {
      r(e).arrayStride = t;
    }, n.wbg.__wbg_setaspect_f06e234d0aacd1a6 = function(e, t) {
      r(e).aspect = At[t];
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
    }, n.wbg.__wbg_setcode_6b6ad02fc1705aa2 = function(e, t, o) {
      r(e).code = p(t, o);
    }, n.wbg.__wbg_setcolor_0df2c5f47a951ac1 = function(e, t) {
      r(e).color = r(t);
    }, n.wbg.__wbg_setcolorattachments_de625dd9a4850a13 = function(e, t) {
      r(e).colorAttachments = r(t);
    }, n.wbg.__wbg_setcompare_1b67d8112d05628e = function(e, t) {
      r(e).compare = de[t];
    }, n.wbg.__wbg_setcompare_8fbddcdd4781f49a = function(e, t) {
      r(e).compare = de[t];
    }, n.wbg.__wbg_setcount_e8b681b1185cf5da = function(e, t) {
      r(e).count = t >>> 0;
    }, n.wbg.__wbg_setcullmode_74bc6eaab528c94b = function(e, t) {
      r(e).cullMode = wt[t];
    }, n.wbg.__wbg_setdepthbias_cdcc35c6971d19cd = function(e, t) {
      r(e).depthBias = t;
    }, n.wbg.__wbg_setdepthbiasclamp_57801e26f66496d9 = function(e, t) {
      r(e).depthBiasClamp = t;
    }, n.wbg.__wbg_setdepthbiasslopescale_81699f807bd5a647 = function(e, t) {
      r(e).depthBiasSlopeScale = t;
    }, n.wbg.__wbg_setdepthclearvalue_9801aa9eff7645df = function(e, t) {
      r(e).depthClearValue = t;
    }, n.wbg.__wbg_setdepthcompare_53d249a136855bd8 = function(e, t) {
      r(e).depthCompare = de[t];
    }, n.wbg.__wbg_setdepthfailop_2e4767995acd4c0a = function(e, t) {
      r(e).depthFailOp = we[t];
    }, n.wbg.__wbg_setdepthloadop_af0b0f05e83f6571 = function(e, t) {
      r(e).depthLoadOp = le[t];
    }, n.wbg.__wbg_setdepthorarraylayers_5d480fc05509ea0c = function(e, t) {
      r(e).depthOrArrayLayers = t >>> 0;
    }, n.wbg.__wbg_setdepthreadonly_a7b7224074e024d3 = function(e, t) {
      r(e).depthReadOnly = t !== 0;
    }, n.wbg.__wbg_setdepthstencil_2bb2fcea55783858 = function(e, t) {
      r(e).depthStencil = r(t);
    }, n.wbg.__wbg_setdepthstencilattachment_dcbd5b74e4350e16 = function(e, t) {
      r(e).depthStencilAttachment = r(t);
    }, n.wbg.__wbg_setdepthstoreop_40dfd99c7e42f894 = function(e, t) {
      r(e).depthStoreOp = pe[t];
    }, n.wbg.__wbg_setdepthwriteenabled_4368a2fe5d258cb0 = function(e, t) {
      r(e).depthWriteEnabled = t !== 0;
    }, n.wbg.__wbg_setdevice_d372d6aa06f20cae = function(e, t) {
      r(e).device = r(t);
    }, n.wbg.__wbg_setdimension_268b2b7bfc3e2bb8 = function(e, t) {
      r(e).dimension = Ct[t];
    }, n.wbg.__wbg_setdimension_359b229ea1b67a77 = function(e, t) {
      r(e).dimension = me[t];
    }, n.wbg.__wbg_setdstfactor_96e73b9eaedeb23e = function(e, t) {
      r(e).dstFactor = $e[t];
    }, n.wbg.__wbg_setendofpasswriteindex_71e7659a9d2a9d60 = function(e, t) {
      r(e).endOfPassWriteIndex = t >>> 0;
    }, n.wbg.__wbg_setentries_5941f16619f54d42 = function(e, t) {
      r(e).entries = r(t);
    }, n.wbg.__wbg_setentries_97a6ad10aa7fa4d1 = function(e, t) {
      r(e).entries = r(t);
    }, n.wbg.__wbg_setentrypoint_a858879f63ec2236 = function(e, t, o) {
      r(e).entryPoint = p(t, o);
    }, n.wbg.__wbg_setentrypoint_a8ce0b22c20548b0 = function(e, t, o) {
      r(e).entryPoint = p(t, o);
    }, n.wbg.__wbg_setfailop_d55bda42958efa98 = function(e, t) {
      r(e).failOp = we[t];
    }, n.wbg.__wbg_setformat_69ba449c0e080708 = function(e, t) {
      r(e).format = j[t];
    }, n.wbg.__wbg_setformat_713b9e90b13df6aa = function(e, t) {
      r(e).format = Pt[t];
    }, n.wbg.__wbg_setformat_76bcf93126fcdc9d = function(e, t) {
      r(e).format = j[t];
    }, n.wbg.__wbg_setformat_970299d3f84a8f20 = function(e, t) {
      r(e).format = j[t];
    }, n.wbg.__wbg_setformat_a8a60feb127f0971 = function(e, t) {
      r(e).format = j[t];
    }, n.wbg.__wbg_setformat_beb33029aea4cf8e = function(e, t) {
      r(e).format = j[t];
    }, n.wbg.__wbg_setformat_f6ec428901712514 = function(e, t) {
      r(e).format = j[t];
    }, n.wbg.__wbg_setfragment_0f23dfb67b3e84ab = function(e, t) {
      r(e).fragment = r(t);
    }, n.wbg.__wbg_setfrontface_c80337acd997f8c6 = function(e, t) {
      r(e).frontFace = pt[t];
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
    }, n.wbg.__wbg_setlabel_1df8805b2aad72d7 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_460a52030d604dd7 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_57008c2e11276b5e = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_66db708c47a585b2 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_68cd87490e02e1de = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_76b058f0224eb49e = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_89c327fa94d8076b = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_969d6f8279c74456 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_a0c41069e355431e = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_c14214ffbf6e5c4a = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_ca2c132e2b646244 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_e6fab993e10f1dd3 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlabel_f9a45e9ef445b781 = function(e, t, o) {
      r(e).label = p(t, o);
    }, n.wbg.__wbg_setlayout_67a29edc6247c437 = function(e, t) {
      r(e).layout = r(t);
    }, n.wbg.__wbg_setlayout_758d30edbd6ea91c = function(e, t) {
      r(e).layout = r(t);
    }, n.wbg.__wbg_setloadop_5644a3bf70f4f76c = function(e, t) {
      r(e).loadOp = le[t];
    }, n.wbg.__wbg_setlodmaxclamp_d80060a9922f9fe3 = function(e, t) {
      r(e).lodMaxClamp = t;
    }, n.wbg.__wbg_setlodminclamp_bee469ae69d038f0 = function(e, t) {
      r(e).lodMinClamp = t;
    }, n.wbg.__wbg_setmagfilter_f50646cfdc01700d = function(e, t) {
      r(e).magFilter = De[t];
    }, n.wbg.__wbg_setmappedatcreation_0dc5796d4e90ab4b = function(e, t) {
      r(e).mappedAtCreation = t !== 0;
    }, n.wbg.__wbg_setmask_800b15ad78613be8 = function(e, t) {
      r(e).mask = t >>> 0;
    }, n.wbg.__wbg_setmaxanisotropy_83ac2a8bef9f9ec8 = function(e, t) {
      r(e).maxAnisotropy = t;
    }, n.wbg.__wbg_setmethod_3c5280fe5d890842 = function(e, t, o) {
      r(e).method = p(t, o);
    }, n.wbg.__wbg_setminbindingsize_20ca594cd6d93818 = function(e, t) {
      r(e).minBindingSize = t;
    }, n.wbg.__wbg_setminfilter_8ffc9e1ac6b4149f = function(e, t) {
      r(e).minFilter = De[t];
    }, n.wbg.__wbg_setmiplevel_6f507098915add77 = function(e, t) {
      r(e).mipLevel = t >>> 0;
    }, n.wbg.__wbg_setmiplevelcount_5e59806cbcf116e9 = function(e, t) {
      r(e).mipLevelCount = t >>> 0;
    }, n.wbg.__wbg_setmiplevelcount_f896fe8cbb669df2 = function(e, t) {
      r(e).mipLevelCount = t >>> 0;
    }, n.wbg.__wbg_setmipmapfilter_037575f2e647f024 = function(e, t) {
      r(e).mipmapFilter = ht[t];
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
      r(e).operation = gt[t];
    }, n.wbg.__wbg_setorigin_e26b73e77b3e275d = function(e, t) {
      r(e).origin = r(t);
    }, n.wbg.__wbg_setpassop_070547fd6160a00d = function(e, t) {
      r(e).passOp = we[t];
    }, n.wbg.__wbg_setpowerpreference_1f3351e5d2acf765 = function(e, t) {
      r(e).powerPreference = yt[t];
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
      r(e).sampleType = Mt[t];
    }, n.wbg.__wbg_setsearch_609451e9e712f3c6 = function(e, t, o) {
      r(e).search = p(t, o);
    }, n.wbg.__wbg_setshaderlocation_985046f48e76573f = function(e, t) {
      r(e).shaderLocation = t >>> 0;
    }, n.wbg.__wbg_setsize_23676383c9c0732f = function(e, t) {
      r(e).size = t;
    }, n.wbg.__wbg_setsize_51616eaf8209c58b = function(e, t) {
      r(e).size = r(t);
    }, n.wbg.__wbg_setsize_5878aadcd23673cf = function(e, t) {
      r(e).size = t;
    }, n.wbg.__wbg_setsrcfactor_04ce8874f1bff5a8 = function(e, t) {
      r(e).srcFactor = $e[t];
    }, n.wbg.__wbg_setstencilback_4b20ecfcd4c4816a = function(e, t) {
      r(e).stencilBack = r(t);
    }, n.wbg.__wbg_setstencilclearvalue_7ba82e1993788f37 = function(e, t) {
      r(e).stencilClearValue = t >>> 0;
    }, n.wbg.__wbg_setstencilfront_1ca3b695f7c42f6a = function(e, t) {
      r(e).stencilFront = r(t);
    }, n.wbg.__wbg_setstencilloadop_b65c60a0077315cd = function(e, t) {
      r(e).stencilLoadOp = le[t];
    }, n.wbg.__wbg_setstencilreadmask_4f5b98747141e796 = function(e, t) {
      r(e).stencilReadMask = t >>> 0;
    }, n.wbg.__wbg_setstencilreadonly_9006a99a91d198e9 = function(e, t) {
      r(e).stencilReadOnly = t !== 0;
    }, n.wbg.__wbg_setstencilstoreop_4f00c5eca345c145 = function(e, t) {
      r(e).stencilStoreOp = pe[t];
    }, n.wbg.__wbg_setstencilwritemask_e37a7214d84ace99 = function(e, t) {
      r(e).stencilWriteMask = t >>> 0;
    }, n.wbg.__wbg_setstepmode_7d58d75e6547a7a6 = function(e, t) {
      r(e).stepMode = Tt[t];
    }, n.wbg.__wbg_setstoragetexture_2987339fec972d54 = function(e, t) {
      r(e).storageTexture = r(t);
    }, n.wbg.__wbg_setstoreop_c62dd050b5806095 = function(e, t) {
      r(e).storeOp = pe[t];
    }, n.wbg.__wbg_setstripindexformat_3e4893749b3f00b0 = function(e, t) {
      r(e).stripIndexFormat = mt[t];
    }, n.wbg.__wbg_settargets_0ef1de33af7253a6 = function(e, t) {
      r(e).targets = r(t);
    }, n.wbg.__wbg_settexture_2553e9c3ae6f7687 = function(e, t) {
      r(e).texture = r(t);
    }, n.wbg.__wbg_settexture_f62859f817324dd1 = function(e, t) {
      r(e).texture = r(t);
    }, n.wbg.__wbg_settimestampwrites_1995524c3a31cb8f = function(e, t) {
      r(e).timestampWrites = r(t);
    }, n.wbg.__wbg_settopology_3d9b2f0ffe2e350c = function(e, t) {
      r(e).topology = xt[t];
    }, n.wbg.__wbg_settype_0b59dd5f4721c490 = function(e, t) {
      r(e).type = vt[t];
    }, n.wbg.__wbg_settype_8c8bbfab4cf7e32e = function(e, t) {
      r(e).type = dt[t];
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
      r(e).viewDimension = me[t];
    }, n.wbg.__wbg_setviewdimension_f5d4b5336a27d302 = function(e, t) {
      r(e).viewDimension = me[t];
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
      const o = r(t).stack, a = G(o, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
      S().setInt32(e + 4 * 1, _, true), S().setInt32(e + 4 * 0, a, true);
    }, n.wbg.__wbg_static_accessor_GLOBAL_88a902d13a557d07 = function() {
      const e = typeof global > "u" ? null : global;
      return D(e) ? 0 : b(e);
    }, n.wbg.__wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0 = function() {
      const e = typeof globalThis > "u" ? null : globalThis;
      return D(e) ? 0 : b(e);
    }, n.wbg.__wbg_static_accessor_SELF_37c5d418e4bf5819 = function() {
      const e = typeof self > "u" ? null : self;
      return D(e) ? 0 : b(e);
    }, n.wbg.__wbg_static_accessor_WINDOW_5de37043a91a9c40 = function() {
      const e = typeof window > "u" ? null : window;
      return D(e) ? 0 : b(e);
    }, n.wbg.__wbg_status_f6360336ca686bf0 = function(e) {
      return r(e).status;
    }, n.wbg.__wbg_submit_252766c4e0945cee = function(e, t) {
      r(e).submit(r(t));
    }, n.wbg.__wbg_text_7805bea50de2af49 = function() {
      return P(function(e) {
        const t = r(e).text();
        return b(t);
      }, arguments);
    }, n.wbg.__wbg_then_44b73946d2fb3e7d = function(e, t) {
      const o = r(e).then(r(t));
      return b(o);
    }, n.wbg.__wbg_then_48b406749878a531 = function(e, t, o) {
      const a = r(e).then(r(t), r(o));
      return b(a);
    }, n.wbg.__wbg_toString_5285597960676b7b = function(e) {
      const t = r(e).toString();
      return b(t);
    }, n.wbg.__wbg_toString_c813bbd34d063839 = function(e) {
      const t = r(e).toString();
      return b(t);
    }, n.wbg.__wbg_url_8f9653b899456042 = function(e, t) {
      const o = r(t).url, a = G(o, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
      S().setInt32(e + 4 * 1, _, true), S().setInt32(e + 4 * 0, a, true);
    }, n.wbg.__wbg_width_5dde457d606ba683 = function(e) {
      return r(e).width;
    }, n.wbg.__wbg_writeBuffer_3193eaacefdcf39a = function() {
      return P(function(e, t, o, a, _, c) {
        r(e).writeBuffer(r(t), o, r(a), _, c);
      }, arguments);
    }, n.wbg.__wbg_writeTexture_cd7877c213ee5704 = function() {
      return P(function(e, t, o, a, _) {
        r(e).writeTexture(r(t), r(o), r(a), r(_));
      }, arguments);
    }, n.wbg.__wbindgen_cb_drop = function(e) {
      const t = ne(e).original;
      return t.cnt-- == 1 ? (t.a = 0, true) : false;
    }, n.wbg.__wbindgen_closure_wrapper2361 = function(e, t, o) {
      const a = Re(e, t, 193, ft);
      return b(a);
    }, n.wbg.__wbindgen_closure_wrapper2392 = function(e, t, o) {
      const a = Re(e, t, 202, bt);
      return b(a);
    }, n.wbg.__wbindgen_debug_string = function(e, t) {
      const o = ve(r(t)), a = G(o, i.__wbindgen_export_2, i.__wbindgen_export_3), _ = L;
      S().setInt32(e + 4 * 1, _, true), S().setInt32(e + 4 * 0, a, true);
    }, n.wbg.__wbindgen_is_function = function(e) {
      return typeof r(e) == "function";
    }, n.wbg.__wbindgen_is_null = function(e) {
      return r(e) === null;
    }, n.wbg.__wbindgen_is_undefined = function(e) {
      return r(e) === void 0;
    }, n.wbg.__wbindgen_memory = function() {
      const e = i.memory;
      return b(e);
    }, n.wbg.__wbindgen_number_get = function(e, t) {
      const o = r(t), a = typeof o == "number" ? o : void 0;
      S().setFloat64(e + 8 * 1, D(a) ? 0 : a, true), S().setInt32(e + 4 * 0, !D(a), true);
    }, n.wbg.__wbindgen_number_new = function(e) {
      return b(e);
    }, n.wbg.__wbindgen_object_clone_ref = function(e) {
      const t = r(e);
      return b(t);
    }, n.wbg.__wbindgen_object_drop_ref = function(e) {
      ne(e);
    }, n.wbg.__wbindgen_string_get = function(e, t) {
      const o = r(t), a = typeof o == "string" ? o : void 0;
      var _ = D(a) ? 0 : G(a, i.__wbindgen_export_2, i.__wbindgen_export_3), c = L;
      S().setInt32(e + 4 * 1, c, true), S().setInt32(e + 4 * 0, _, true);
    }, n.wbg.__wbindgen_string_new = function(e, t) {
      const o = p(e, t);
      return b(o);
    }, n.wbg.__wbindgen_throw = function(e, t) {
      throw new Error(p(e, t));
    }, n;
  }
  function kt(n, e) {
    return i = n.exports, Xe.__wbindgen_wasm_module = e, q = null, Z = null, J = null, i;
  }
  async function Xe(n) {
    if (i !== void 0) return i;
    typeof n < "u" && (Object.getPrototypeOf(n) === Object.prototype ? { module_or_path: n } = n : console.warn("using deprecated parameters for the initialization function; pass a single object instead")), typeof n > "u" && (n = new URL("/infinite-chat/assets/infinite_chat_wasm_bg-BVB18jos.wasm", import.meta.url));
    const e = Ft();
    (typeof n == "string" || typeof Request == "function" && n instanceof Request || typeof URL == "function" && n instanceof URL) && (n = fetch(n));
    const { instance: t, module: o } = await Et(await n, e);
    return kt(t, o);
  }
  let fe = false, _e = null;
  const He = /* @__PURE__ */ new Map();
  let Se = 0;
  Bt = function() {
    return fe;
  };
  function Ke(n, e) {
    if (!fe || Se === 0) return null;
    const t = He.get(n);
    return t == null ? null : t * e / Se;
  }
  Lt = function(n, e = "/infinite-chat/fonts/lxgw-msdf") {
    if (fe) return Promise.resolve();
    if (_e) return _e;
    const t = e.replace(/[^/]+$/, "");
    return _e = (async () => {
      const o = await fetch(`${e}.json`).then((s) => s.json()), a = o.chars;
      Se = o.info.size;
      const _ = new Uint32Array(a.length), c = new Float32Array(a.length * 7);
      a.forEach((s, g) => {
        _[g] = s.id, c.set([
          s.x,
          s.y,
          s.width,
          s.height,
          s.xoffset,
          s.yoffset,
          s.page
        ], g * 7), He.set(s.id, s.xadvance);
      });
      const u = [];
      for (const s of o.pages) {
        const g = await fetch(`${t}${s}`).then((M) => M.blob()), f = await createImageBitmap(g), v = new OffscreenCanvas(f.width, f.height).getContext("2d");
        if (!v) throw new Error("MSDF \u89E3\u7801:\u65E0 2D \u4E0A\u4E0B\u6587");
        v.drawImage(f, 0, 0);
        const F = v.getImageData(0, 0, f.width, f.height).data;
        u.push(new Uint8Array(F.buffer.slice(0)));
      }
      n.load_msdf({
        atlasW: o.common.scaleW,
        atlasH: o.common.scaleH,
        fontSize: o.info.size,
        ids: _,
        cells: c,
        pixels: u
      }), fe = true, console.info(`[msdf] loaded ${a.length} glyphs / ${o.pages.length} pages`);
    })(), _e;
  };
  let se = null, ze = false;
  function Ot(n, e = "/infinite-chat/fonts/katex-msdf") {
    if (ze) return Promise.resolve();
    if (se) return se;
    const t = e.replace(/[^/]+$/, "");
    return se = (async () => {
      const o = await fetch(`${e}.json`).then((s) => s.json()), a = o.chars, _ = new Uint32Array(a.length), c = new Float32Array(a.length * 7);
      a.forEach((s, g) => {
        _[g] = s.id, c.set([
          s.x,
          s.y,
          s.width,
          s.height,
          s.xoffset,
          s.yoffset,
          s.page
        ], g * 7);
      });
      const u = [];
      for (const s of o.pages) {
        const g = await fetch(`${t}${s}`).then((M) => M.blob()), f = await createImageBitmap(g), v = new OffscreenCanvas(f.width, f.height).getContext("2d");
        if (!v) throw new Error("MSDF \u89E3\u7801:\u65E0 2D \u4E0A\u4E0B\u6587");
        v.drawImage(f, 0, 0);
        const F = v.getImageData(0, 0, f.width, f.height).data;
        u.push(new Uint8Array(F.buffer.slice(0)));
      }
      n.load_msdf({
        atlasW: o.common.scaleW,
        atlasH: o.common.scaleH,
        fontSize: o.info.size,
        ids: _,
        cells: c,
        pixels: u
      }), ze = true, console.info(`[math-msdf] loaded ${a.length} glyphs`);
    })(), se;
  }
  const We = Object.freeze(Object.defineProperty({
    __proto__: null,
    loadMathMsdf: Ot,
    loadMsdf: Lt,
    msdfAdvancePx: Ke,
    msdfLoaded: Bt
  }, Symbol.toStringTag, {
    value: "Module"
  })), ce = {
    table: {
      vAlign: "center",
      hAlign: "auto"
    },
    tableRender: {
      lineColor: [
        0.26,
        0.29,
        0.36,
        0.9
      ],
      headerFill: [
        0.16,
        0.18,
        0.24,
        0.6
      ],
      aoColor: [
        1,
        1,
        1
      ],
      lineW: 1,
      ao: 0.12,
      aoWidth: 10,
      radius: 4
    }
  }, Ye = "infinite-chat.styleConfig";
  function Ne(n) {
    return JSON.parse(JSON.stringify(n));
  }
  let Ae = Rt();
  function Rt() {
    try {
      const n = localStorage.getItem(Ye);
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
        }
      };
    } catch {
      return Ne(ce);
    }
  }
  $t = function() {
    return Ae;
  };
  sn = function(n) {
    Ae = n;
    try {
      localStorage.setItem(Ye, JSON.stringify(Ae));
    } catch {
    }
  };
  const Te = typeof window < "u" && window.devicePixelRatio || 1, Dt = 16, A = Math.round(Dt * Te), z = Math.ceil(A * 1.4), $ = 128, re = 8, Gt = $ - 2 * re, be = {
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
  let Ce = "system", U = be.system.sans, H = be.system.mono;
  cn = function(n) {
    const e = be[n];
    return !e || n === Ce ? false : (Ce = n, U = e.sans, H = e.mono, true);
  };
  un = function() {
    return Ce;
  };
  let Me = "auto";
  fn = function(n) {
    Me = n;
  };
  function zt() {
    return Me === "auto" || Me === "msdf";
  }
  bn = function() {
    return Object.keys(be);
  };
  function Je(n) {
    switch (n) {
      case 1:
        return `bold ${A}px ${U}`;
      case 2:
        return `italic ${A}px ${U}`;
      case 3:
        return `bold italic ${A}px ${U}`;
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
        return `${A}px ${H}`;
      case 6:
      case 10:
      case 11:
      case 12:
      case 13:
      case 14:
        return `bold ${A}px ${U}`;
      case 8:
        return `italic ${A}px ${U}`;
      case 16:
        return `bold ${A}px ${U}`;
      case 17:
      case 18:
        return `${A}px ${H}`;
      case 19:
        return `bold ${A}px ${H}`;
      case 20:
        return `italic ${A}px ${H}`;
      case 21:
        return `${A}px ${H}`;
      case 26:
        return `${A}px KaTeX_Main`;
      case 27:
        return `bold ${A}px KaTeX_Main`;
      case 28:
        return `italic ${A}px KaTeX_Main`;
      case 29:
        return `bold italic ${A}px KaTeX_Main`;
      case 30:
        return `italic ${A}px KaTeX_Math`;
      case 31:
        return `${A}px KaTeX_AMS`;
      case 32:
        return `${A}px KaTeX_Size1`;
      case 33:
        return `${A}px KaTeX_Size2`;
      case 34:
        return `${A}px KaTeX_Size3`;
      case 35:
        return `${A}px KaTeX_Size4`;
      case 36:
        return `${A}px KaTeX_Caligraphic`;
      case 37:
        return `${A}px KaTeX_Fraktur`;
      case 38:
        return `${A}px KaTeX_SansSerif`;
      case 39:
        return `${A}px KaTeX_Script`;
      case 40:
        return `${A}px KaTeX_Typewriter`;
      default:
        return `${A}px ${U}`;
    }
  }
  function Wt(n) {
    return n === 5 || n >= 43 && n <= 50;
  }
  function Pe(n) {
    switch (n) {
      case 6:
        return 2;
      case 10:
        return 1.6;
      case 11:
        return 1.3;
      case 12:
        return 1.15;
      case 13:
        return 1;
      case 14:
        return 0.9;
      case 24:
        return 0.7;
      case 25:
        return 0.85;
      default:
        return 1;
    }
  }
  const Nt = new Intl.Segmenter(void 0, {
    granularity: "grapheme"
  });
  let he = null;
  function Ut() {
    if (!he) {
      const n = new OffscreenCanvas(8, 8).getContext("2d");
      if (!n) throw new Error("\u65E0\u6CD5\u521B\u5EFA 2D \u6D4B\u91CF\u4E0A\u4E0B\u6587");
      he = n;
    }
    return he;
  }
  function Vt(n, e) {
    if (zt()) {
      const o = [
        ...n
      ];
      if (o.length === 1) {
        const a = Ke(o[0].codePointAt(0) ?? 0, A);
        if (a != null) return a * Pe(e);
      }
    }
    const t = Ut();
    return t.font = Je(e), Math.max(1, t.measureText(n).width) * Pe(e);
  }
  function jt(n) {
    const e = n.codePointAt(0) ?? 0;
    return e >= 19968 && e <= 40959 || e >= 12352 && e <= 12543 || e >= 44032 && e <= 55203 || e >= 65280 && e <= 65519 || e >= 12288 && e <= 12351;
  }
  function qt(n) {
    return /\s/.test(n);
  }
  const Xt = "\u3002\uFF0C\u3001\uFF1B\uFF1A\uFF1F\uFF01\uFF09\u300D\u300F\u3011\u300B\u3009\u201D\u2019%\u2026\xB7.,;:?!)]}";
  function Ht(n) {
    return Xt.includes(n);
  }
  function Kt(n) {
    return n >= 17 && n <= 21;
  }
  const ie = Math.round(8 * Te), ye = Math.max(1, Math.round(Te));
  function Yt(n, e, t, o) {
    const a = [];
    let _ = e, c = 0, u = e;
    for (; u < t; ) {
      if (n[u].nl) {
        a.push([
          _,
          u
        ]), u++, _ = u, c = 0;
        continue;
      }
      let s = u, g = 0;
      if (n[u].cjk) s = u + 1, g = n[u].adv;
      else for (; s < t && !n[s].cjk && !n[s].nl && (g += n[s].adv, s++, !n[s - 1].space); ) ;
      c + g > o && c > 0 && (a.push([
        _,
        u
      ]), _ = u, c = 0), c += g, u = s;
    }
    return (_ < t || a.length === 0) && a.push([
      _,
      t
    ]), a;
  }
  function Jt(n, e, t, o, a, _) {
    const c = t.rows.length, u = Math.max(t.aligns.length, ...t.rows.map((l) => l.length)), s = (l, I) => {
      const E = t.rows[l][I];
      return E ? [
        o[E[0]],
        o[E[1]]
      ] : [
        0,
        0
      ];
    }, g = (l, I) => {
      let E = 0;
      for (let T = l; T < I; T++) n[T].nl || (E += n[T].adv);
      return E;
    }, f = new Array(u).fill(0);
    for (let l = 0; l < c; l++) for (let I = 0; I < t.rows[l].length; I++) {
      const [E, T] = s(l, I);
      f[I] = Math.max(f[I], g(E, T));
    }
    const w = u * ie * 2 + Math.max(0, u - 1) * ye, v = _ - w, F = f.reduce((l, I) => l + I, 0);
    if (F > v && v > 0) {
      const l = Math.round(4 * A), I = f.map((B) => B / F * v);
      let E = 0, T = 0;
      for (const B of I) B < l ? E += l - B : T += B - l;
      for (let B = 0; B < u; B++) I[B] < l ? f[B] = l : f[B] = E <= T && T > 0 ? I[B] - (I[B] - l) / T * E : I[B];
    }
    const M = new Array(u).fill(0);
    for (let l = 1; l < u; l++) M[l] = M[l - 1] + f[l - 1] + ie * 2 + ye;
    let y = a;
    const x = [];
    for (let l = 0; l < c; l++) {
      x.push(y);
      const I = [];
      let E = 1;
      for (let R = 0; R < t.rows[l].length; R++) {
        const [X, K] = s(l, R), Y = Yt(n, X, K, f[R]);
        I.push(Y), Y.length > E && (E = Y.length);
      }
      const T = $t().table, B = T.hAlign === "auto" ? null : T.hAlign === "right" ? 2 : T.hAlign === "center" ? 1 : 0;
      for (let R = 0; R < t.rows[l].length; R++) {
        const X = B ?? t.aligns[R] ?? 0, K = I[R], Y = (K.length - 1) * z + A, Ie = Math.max(0, E * z - Y), tt = T.vAlign === "top" ? 0 : T.vAlign === "bottom" ? Ie : Ie / 2;
        for (let ae = 0; ae < K.length; ae++) {
          const [Ee, Fe] = K[ae], ke = f[R] - g(Ee, Fe), nt = X === 2 ? ke : X === 1 ? ke / 2 : 0;
          let Be = M[R] + ie + Math.max(0, nt);
          const rt = y + tt + ae * z;
          for (let V = Ee; V < Fe; V++) {
            const W = n[V];
            e[V * 4] = Be - W.off, e[V * 4 + 1] = rt + (z - W.cell) * 0.5 - W.off, e[V * 4 + 2] = W.nl ? 0 : W.cell, e[V * 4 + 3] = W.nl ? 0 : W.cell, W.nl || (Be += W.adv);
          }
        }
      }
      y += E * z;
    }
    const m = [];
    for (let l = 1; l < u; l++) m.push(M[l] - ye * 0.5);
    const d = x.slice(1), h = M[u - 1] + f[u - 1] + 2 * ie, C = c >= 2 ? x[1] : a, k = {
      x: 0,
      y: a,
      w: h,
      h: y - a,
      headerBottom: C,
      cols: m,
      rows: d
    };
    return {
      height: y - a,
      panel: k
    };
  }
  function Zt(n) {
    const e = [];
    for (const t of n) e.push(t.x, t.y, t.w, t.h, t.headerBottom, t.cols.length, t.rows.length, ...t.cols, ...t.rows);
    return new Float32Array(e);
  }
  function Ze(n, e, t, o) {
    const a = A / Gt, _ = [], c = [];
    for (let y = 0; y < n.length; y++) {
      c.push(_.length);
      const x = e[y] ?? 0, m = Pe(x);
      for (const { segment: d } of Nt.segment(n[y])) {
        const h = d === `
`;
        _.push({
          cluster: d,
          role: x,
          adv: h ? 0 : Vt(d, x),
          cell: $ * a * m,
          off: re * a * m,
          lineH: Math.ceil(z * m),
          nl: h,
          cjk: !h && jt(d),
          space: !h && qt(d),
          inTable: false
        });
      }
    }
    c.push(_.length);
    const u = /* @__PURE__ */ new Map();
    if (o) for (const y of o) {
      if (!y.rows.length || !y.rows[0].length) continue;
      const x = c[y.rows[0][0][0]];
      let m = x;
      for (const d of y.rows) for (const [, h] of d) m = Math.max(m, c[h]);
      u.set(x, {
        region: y,
        gEnd: m
      });
    }
    {
      let y = 0;
      for (let x = 0; x <= _.length; x++) if (x === _.length || _[x].nl) {
        let m = false;
        for (let d = y; d < x; d++) if (Kt(_[d].role)) {
          m = true;
          break;
        }
        if (m) for (let d = y; d < x; d++) _[d].inTable = true;
        y = x + 1;
      }
    }
    const s = new Float32Array(_.length * 4);
    let g = 0, f = 0, w = z;
    const v = (y, x) => {
      s[x * 4] = g - y.off, s[x * 4 + 1] = f + (y.lineH - y.cell) * 0.5 - y.off, s[x * 4 + 2] = y.cell, s[x * 4 + 3] = y.cell, g += y.adv, y.lineH > w && (w = y.lineH);
    }, F = [];
    let M = 0;
    for (; M < _.length; ) {
      const y = u.get(M);
      if (y) {
        const C = Jt(_, s, y.region, c, f, t);
        f += C.height, F.push(C.panel), g = 0, w = z, M = y.gEnd;
        continue;
      }
      const x = _[M];
      if (x.nl) {
        s[M * 4] = g - x.off, s[M * 4 + 1] = f - x.off, s[M * 4 + 2] = 0, s[M * 4 + 3] = 0, g = 0, f += w, w = z, M++;
        continue;
      }
      const m = x.inTable || Wt(x.role);
      let d = M, h = 0;
      if (m) for (; d < _.length && !_[d].nl; ) h += _[d].adv, d++;
      else if (x.cjk) d = M + 1, h = x.adv;
      else for (; d < _.length && !_[d].nl && !_[d].cjk && (h += _[d].adv, d++, !_[d - 1].space); ) ;
      !m && g + h > t && g > 0 && !Ht(x.cluster) && (g = 0, f += w, w = z);
      for (let C = M; C < d; C++) v(_[C], C);
      M = d;
    }
    return F.length > 0 ? {
      positions: s,
      tables: Zt(F)
    } : s;
  }
  const Q = /* @__PURE__ */ new Map(), Qt = 4096;
  let Qe = 0, et = 0;
  function en(n, e, t) {
    if (n.length === 0) return new Float32Array([
      0,
      0
    ]);
    const o = `${n.join("")}${Array.from(e).join(",")}@${Math.round(t)}`, a = Q.get(o);
    if (a) return Qe++, a;
    et++;
    const _ = Ze(n, e, t), c = _ instanceof Float32Array ? _ : _.positions;
    let u = 0, s = 0;
    for (let f = 0; f + 3 < c.length; f += 4) {
      const w = c[f + 2];
      w > 0 && (u = Math.max(u, c[f] + w)), s = Math.max(s, c[f + 1] + c[f + 3]);
    }
    const g = new Float32Array([
      Math.min(u, t),
      s
    ]);
    return Q.size >= Qt && Q.clear(), Q.set(o, g), g;
  }
  gn = function() {
    return {
      hits: Qe,
      misses: et,
      size: Q.size
    };
  };
  const tn = 8, oe = 1e20;
  let xe = null;
  function nn() {
    if (!xe) {
      const n = new OffscreenCanvas($, $).getContext("2d", {
        willReadFrequently: true
      });
      if (!n) throw new Error("\u65E0\u6CD5\u521B\u5EFA SDF \u5149\u6805\u4E0A\u4E0B\u6587");
      xe = n;
    }
    return xe;
  }
  function Ue(n, e, t, o) {
    const a = new Float64Array(o), _ = new Int32Array(o), c = new Float64Array(o + 1);
    for (let s = 0; s < o; s++) a[s] = n[e + s * t];
    _[0] = 0, c[0] = -oe, c[1] = oe;
    let u = 0;
    for (let s = 1; s < o; s++) {
      let g = (a[s] + s * s - (a[_[u]] + _[u] * _[u])) / (2 * s - 2 * _[u]);
      for (; g <= c[u]; ) u--, g = (a[s] + s * s - (a[_[u]] + _[u] * _[u])) / (2 * s - 2 * _[u]);
      u++, _[u] = s, c[u] = g, c[u + 1] = oe;
    }
    u = 0;
    for (let s = 0; s < o; s++) {
      for (; c[u + 1] < s; ) u++;
      const g = s - _[u];
      n[e + s * t] = a[_[u]] + g * g;
    }
  }
  function Ve(n, e, t) {
    for (let o = 0; o < e; o++) Ue(n, o, e, t);
    for (let o = 0; o < t; o++) Ue(n, o * e, 1, e);
  }
  function rn(n, e, t = 1) {
    const o = nn();
    o.clearRect(0, 0, $, $);
    const a = $ - 2 * re;
    o.font = Je(e).replace(/^\s*(bold |italic )*\d+px/, `$1${a}px`), o.textBaseline = "top", o.fillStyle = "#ffffff", o.fillText(n, re, re);
    const _ = o.getImageData(0, 0, $, $).data, c = $ * $;
    if (t === 3) return new Uint8Array(_);
    if (t === 0) {
      const f = new Uint8Array(c * 4);
      for (let w = 0; w < c; w++) {
        const v = _[w * 4 + 3];
        f[w * 4] = v, f[w * 4 + 1] = v, f[w * 4 + 2] = v, f[w * 4 + 3] = 255;
      }
      return f;
    }
    const u = new Float64Array(c), s = new Float64Array(c);
    for (let f = 0; f < c; f++) {
      const w = _[f * 4 + 3] / 255;
      if (w === 1) u[f] = 0, s[f] = oe;
      else if (w === 0) u[f] = oe, s[f] = 0;
      else {
        const v = Math.max(0, 0.5 - w), F = Math.max(0, w - 0.5);
        u[f] = v * v, s[f] = F * F;
      }
    }
    Ve(u, $, $), Ve(s, $, $);
    const g = new Uint8Array(c * 4);
    for (let f = 0; f < c; f++) {
      const w = Math.sqrt(u[f]) - Math.sqrt(s[f]), v = Math.max(0, Math.min(255, Math.round(255 * (0.5 - w / (2 * tn)))));
      g[f * 4] = v, g[f * 4 + 1] = v, g[f * 4 + 2] = v, g[f * 4 + 3] = 255;
    }
    return g;
  }
  function on(n, e) {
    const t = () => window.devicePixelRatio || 1;
    let o = 0;
    const a = (g) => {
      g.preventDefault();
      const f = t();
      if (g.ctrlKey) {
        const w = Math.exp(-g.deltaY * 0.01), v = n.getBoundingClientRect();
        e.zoom_at(w, (g.clientX - v.left) * f, (g.clientY - v.top) * f);
      } else {
        const w = n.getBoundingClientRect(), v = (g.clientX - w.left) * f, F = (g.clientY - w.top) * f, M = e.code_block_at_screen(v, F);
        if (M) {
          o += g.deltaY / z;
          const y = Math.trunc(o);
          o -= y, (y !== 0 || g.deltaX !== 0) && e.scroll_code_block(M, g.deltaX * f, y);
        } else e.pan_by(g.deltaX * f, g.deltaY * f);
      }
    };
    n.addEventListener("wheel", a, {
      passive: false
    });
    let _ = false;
    const c = (g) => {
      g.button === 0 && (_ = true, n.setPointerCapture(g.pointerId), n.style.cursor = "grabbing");
    }, u = (g) => {
      if (!_) return;
      const f = t();
      e.pan_by(-g.movementX * f, -g.movementY * f);
    }, s = (g) => {
      _ && (_ = false, n.hasPointerCapture(g.pointerId) && n.releasePointerCapture(g.pointerId), n.style.cursor = "");
    };
    return n.addEventListener("pointerdown", c), n.addEventListener("pointermove", u), n.addEventListener("pointerup", s), n.addEventListener("pointercancel", s), () => {
      n.removeEventListener("wheel", a), n.removeEventListener("pointerdown", c), n.removeEventListener("pointermove", u), n.removeEventListener("pointerup", s), n.removeEventListener("pointercancel", s);
    };
  }
  async function an() {
    const n = document.getElementById("chat"), e = window.devicePixelRatio || 1, t = n.clientWidth || window.innerWidth, o = n.clientHeight || window.innerHeight;
    n.width = Math.round(t * e), n.height = Math.round(o * e);
    const a = await Xe(), _ = new URLSearchParams(location.search), c = _.get("server") ?? void 0, u = _.get("session") ?? void 0, s = !c, { loadReplayConfig: g } = await O(async () => {
      const { loadReplayConfig: d } = await import("./replay-config-BGpffVO3.js");
      return {
        loadReplayConfig: d
      };
    }, []), f = g(), w = _.get("replay"), v = w ?? f.case ?? (s ? "showcase" : void 0), F = Number(_.get("speed") ?? "") || f.speed || 1;
    let M;
    if (v) try {
      M = await (await O(async () => {
        const { loadCase: d } = await import("./replay-Cvqaq-72.js");
        return {
          loadCase: d
        };
      }, [])).loadCase(v, F);
    } catch (d) {
      console.warn(`[replay] \u52A0\u8F7D\u5931\u8D25,\u8DF3\u8FC7\u91CD\u653E: ${v}`, d);
    }
    const y = _.has("bench");
    let x = 0;
    if (y) {
      const d = _.get("lines") ?? "10k", h = Number(_.get("spread") ?? "") || 100;
      try {
        const k = await (await fetch(`/infinite-chat/replays/longsession-${d}.json`)).json();
        M = k.map((l) => ({
          t: l.t * h,
          raw: l.raw
        })), x = k.length, console.info(`[bench] longsession-${d}: ${k.length} turns, spread=${h}ms`);
      } catch (C) {
        console.warn("[bench] \u8F7D\u957F\u4F1A\u8BDD\u5931\u8D25(\u5148\u8DD1 node scripts/gen-longsession.mjs):", C);
      }
    }
    const m = new It(n, {
      layout: Ze,
      measure: en,
      rasterize: rn,
      serverUrl: c,
      sessionId: u,
      replay: M
    });
    if (m.set_math_em(A), m.start(), c) {
      const { SseClient: d } = await O(async () => {
        const { SseClient: h } = await import("./sse-client-UdbDDm5a.js");
        return {
          SseClient: h
        };
      }, []);
      new d({
        url: `${c}/event`,
        onEvent: (h) => m.push_event(h)
      }).start();
    }
    if (!_.has("noinput") && !s) {
      const { mountChatInput: d, parseModel: h } = await O(async () => {
        const { mountChatInput: k, parseModel: l } = await import("./chat-input-RVCfwI0i.js");
        return {
          mountChatInput: k,
          parseModel: l
        };
      }, []), C = h(_.get("model") ?? "aliyuntokenplan/qwen3.7-max");
      d({
        serverUrl: c ?? "http://localhost:4096",
        sessionId: u,
        model: C,
        canvasLive: !!c,
        parent: document.body
      });
    }
    on(n, m);
    {
      const { pumpImageLoads: d } = await O(async () => {
        const { pumpImageLoads: h } = await import("./image-loader-DEJYb4Ur.js");
        return {
          pumpImageLoads: h
        };
      }, []);
      setInterval(() => d(m), 120);
    }
    {
      const { pumpEmbedOverlay: d } = await O(async () => {
        const { pumpEmbedOverlay: T } = await import("./embed-overlay-CS5y01LF.js");
        return {
          pumpEmbedOverlay: T
        };
      }, []), { pumpCopyButtons: h } = await O(async () => {
        const { pumpCopyButtons: T } = await import("./copy-button-CE6UyqNK.js");
        return {
          pumpCopyButtons: T
        };
      }, []), { pumpTextLayer: C, attachSelection: k } = await O(async () => {
        const { pumpTextLayer: T, attachSelection: B } = await import("./text-layer-4lqMGK5f.js");
        return {
          pumpTextLayer: T,
          attachSelection: B
        };
      }, []), { mountFindBar: l } = await O(async () => {
        const { mountFindBar: T } = await import("./find-bar-BQX65CJo.js");
        return {
          mountFindBar: T
        };
      }, []), { pumpDock: I } = await O(async () => {
        const { pumpDock: T } = await import("./dock-BKnaER4q.js");
        return {
          pumpDock: T
        };
      }, []);
      k(m), l(m);
      const E = () => {
        d(m), h(m), C(m, n), I(m), requestAnimationFrame(E);
      };
      requestAnimationFrame(E);
    }
    if (window.__chat = m, _.has("debug")) {
      const d = document.createElement("div");
      d.style.cssText = "position:fixed;top:8px;right:8px;z-index:9999;display:flex;flex-direction:column;gap:8px;align-items:flex-end", document.body.appendChild(d);
      const { mountDebugPanel: h } = await O(async () => {
        const { mountDebugPanel: k } = await import("./debug-panel-BYWudt6Y.js");
        return {
          mountDebugPanel: k
        };
      }, __vite__mapDeps([0,1]));
      h(m, d);
      const { mountStylePanel: C } = await O(async () => {
        const { mountStylePanel: k } = await import("./style-panel-Df3RoVv5.js");
        return {
          mountStylePanel: k
        };
      }, []);
      C(m, d);
    }
    if (_.has("msdf") || _.has("asciimsdf")) {
      const { loadMsdf: d } = await O(async () => {
        const { loadMsdf: k } = await Promise.resolve().then(() => We);
        return {
          loadMsdf: k
        };
      }, void 0), h = "/infinite-chat/", C = _.has("msdf") ? h + "fonts/lxgw-msdf" : h + "fonts/ascii-msdf";
      d(m, C).catch((k) => console.warn("[msdf] load skipped (\u56DE\u9000 TinySDF)", k));
    }
    {
      const { loadMathMsdf: d } = await O(async () => {
        const { loadMathMsdf: C } = await Promise.resolve().then(() => We);
        return {
          loadMathMsdf: C
        };
      }, void 0), { loadMathFonts: h } = await O(async () => {
        const { loadMathFonts: C } = await import("./math-fonts-CSpAQE3e.js");
        return {
          loadMathFonts: C
        };
      }, []);
      Promise.all([
        d(m).catch((C) => console.error("[math-msdf] load failed", C)),
        h().catch((C) => console.error("[math-fonts] load failed", C))
      ]).then(() => m.refresh_fonts());
    }
    if (_.has("verify")) {
      let d = 0;
      const h = setInterval(() => {
        m.set_debug_geometry(true), ++d > 20 && clearInterval(h);
      }, 200);
    }
    if (_.has("gallery")) {
      let d = 0;
      const h = setInterval(() => {
        m.set_shaderbox_gallery(true), ++d > 20 && clearInterval(h);
      }, 200);
    }
    if (y) {
      m.set_stream_rate(1e9), m.set_reveal_cps(Number.POSITIVE_INFINITY), _.has("sizefold") && m.set_bench_fold_width(true), _.has("novirt") && m.set_virtualize(false);
      const d = [];
      let h = -1, C = 0;
      const k = setInterval(() => {
        var _a;
        const l = m.stats(), I = a.memory.buffer.byteLength, E = {
          turns: l.retainedViews,
          storeChars: l.storeChars,
          retainedGlyphs: l.retainedGlyphs,
          retainedNodes: l.retainedNodes,
          frameGlyphs: l.glyphsVisible,
          fps: Math.round(l.fps),
          frameMsAvg: Number(l.frameMsAvg.toFixed(2)),
          wasmMiB: Number((I / 1048576).toFixed(1)),
          phAdvance: Number(l.phAdvance.toFixed(2)),
          phBfLayout: Number(l.phBfLayout.toFixed(2)),
          phBfGrid: Number(l.phBfGrid.toFixed(2)),
          phBfEmit: Number(l.phBfEmit.toFixed(2)),
          phBfTotal: Number(l.phBfTotal.toFixed(2)),
          phAdvIngest: Number(l.phAdvIngest.toFixed(2)),
          phAdvRoles: Number(l.phAdvRoles.toFixed(2)),
          phAdvReveal: Number(l.phAdvReveal.toFixed(2)),
          phAdvEnsure: Number(l.phAdvEnsure.toFixed(2)),
          phAdvSchedule: Number(l.phAdvSchedule.toFixed(2)),
          tierHot: l.tierHot,
          tierWarm: l.tierWarm,
          rebuilds: l.rebuilds
        };
        if (d.push(E), console.table([
          E
        ]), (x > 0 ? l.retainedViews >= x : l.retainedGlyphs > 0) && l.retainedGlyphs === h) {
          if (++C >= 8) {
            const R = [
              Object.keys(d[0]).join(","),
              ...d.map((X) => Object.values(X).join(","))
            ].join(`
`);
            window.__benchCSV = R, console.log(`[bench] done \u2014 CSV (window.__benchCSV):
${R}`), (_a = navigator.clipboard) == null ? void 0 : _a.writeText(R).catch(() => {
            }), clearInterval(k);
          }
        } else C = 0, h = l.retainedGlyphs;
      }, 1e3);
    }
    if (console.info("[harness] ChatCanvas started", {
      mode: c ? `live: ${c}` : "synthetic demo"
    }), s && _n(), s && !w) {
      const { mountFilm: d } = await O(async () => {
        const { mountFilm: h } = await import("./scenes-BXUVMCrq.js");
        return {
          mountFilm: h
        };
      }, []);
      d(m);
    }
  }
  function _n() {
    const n = "/infinite-chat/", e = document.createElement("div");
    e.style.cssText = "position:fixed;top:0;left:0;right:0;z-index:9998;display:flex;gap:14px;align-items:center;padding:8px 14px;font:13px/1.4 system-ui,sans-serif;color:#cdd3e0;background:linear-gradient(#0d0f17ee,#0d0f1700);pointer-events:none;";
    const t = (o, a) => `<a href="${a}" style="pointer-events:auto;color:#3df5d0;text-decoration:none;border:1px solid #3df5d066;border-radius:6px;padding:3px 9px">${o}</a>`;
    e.innerHTML = '<b style="color:#fff;letter-spacing:.3px">infinite-chat</b><span style="opacity:.6">intro film \xB7 SDF / \u6D41\u5F0F / \u65E0\u9650\u753B\u5E03</span><span style="flex:1"></span>' + t("\u{1F3A8} Icon Gallery", `${n}gallery.html`) + t("\u{1F4C4} Markdown demo", "?replay=showcase&speed=0.5") + t("GitHub", "https://github.com/OhBonsai/infinite-chat"), document.body.appendChild(e);
  }
  an().catch((n) => console.error("[harness] \u521D\u59CB\u5316\u5931\u8D25", n));
})();
export {
  __tla,
  cn as a,
  Bt as b,
  un as c,
  fn as d,
  bn as f,
  $t as g,
  Lt as l,
  gn as m,
  sn as s
};
