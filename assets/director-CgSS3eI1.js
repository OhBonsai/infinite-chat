var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
class l {
  constructor(s, e, t) {
    __publicField(this, "totalMs");
    __publicField(this, "marks");
    __publicField(this, "clock", 0);
    __publicField(this, "rate", 1);
    __publicField(this, "playing", false);
    __publicField(this, "starts", []);
    __publicField(this, "sceneIdx", -1);
    __publicField(this, "fired", /* @__PURE__ */ new Set());
    __publicField(this, "onUpdate");
    this.scenes = s, this.chat = e, this.teaser = t;
    let c = 0;
    this.marks = s.map((i) => {
      const a = { id: i.id, title: i.title, startMs: c, durationMs: i.durationMs };
      return this.starts.push(c), c += i.durationMs, a;
    }), this.totalMs = c;
  }
  ctx() {
    return { rate: this.rate, teaser: this.teaser, call: (s, ...e) => {
      try {
        const t = this.chat[s];
        typeof t == "function" && t(...e);
      } catch (t) {
        console.warn(`[film] ${s} \u5931\u8D25`, t);
      }
    } };
  }
  play() {
    this.playing = true, this.ctx().call("set_paused", false), this.emit();
  }
  pause() {
    this.playing = false, this.ctx().call("set_paused", true), this.emit();
  }
  toggle() {
    this.playing ? this.pause() : this.play();
  }
  setRate(s) {
    this.rate = s, this.emit();
  }
  seek(s) {
    this.clock = Math.max(0, Math.min(s, this.totalMs));
    const e = this.sceneAt(this.clock);
    this.enterScene(e, true), this.emit();
  }
  tick(s) {
    if (!this.playing) return;
    this.clock += s * this.rate, this.clock >= this.totalMs && (this.clock = this.totalMs, this.playing = false);
    const e = this.sceneAt(this.clock);
    e !== this.sceneIdx && this.enterScene(e, false);
    const t = this.scenes[e];
    if (t == null ? void 0 : t.cues) {
      const c = this.clock - this.starts[e];
      for (let i = 0; i < t.cues.length; i++) {
        const a = `${e}:${i}`;
        c >= t.cues[i].at && !this.fired.has(a) && (this.fired.add(a), t.cues[i].fn(this.ctx()));
      }
    }
    this.emit();
  }
  sceneAt(s) {
    let e = 0;
    for (let t = 0; t < this.starts.length; t++) s >= this.starts[t] && (e = t);
    return e;
  }
  enterScene(s, e) {
    var _a, _b;
    this.sceneIdx = s, this.teaser.hide();
    const t = this.scenes[s];
    for (let c = 0; c < (((_a = t == null ? void 0 : t.cues) == null ? void 0 : _a.length) ?? 0); c++) {
      const i = `${s}:${c}`, a = this.clock - this.starts[s];
      e && t.cues[c].at <= a ? this.fired.add(i) : this.fired.delete(i);
    }
    (_b = t == null ? void 0 : t.enter) == null ? void 0 : _b.call(t, this.ctx());
  }
  emit() {
    var _a;
    (_a = this.onUpdate) == null ? void 0 : _a.call(this, { clock: this.clock, total: this.totalMs, sceneIdx: this.sceneIdx, playing: this.playing, rate: this.rate });
  }
}
export {
  l as F
};
