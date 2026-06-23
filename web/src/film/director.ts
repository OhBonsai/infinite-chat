// FilmDirector —— 导演时钟:按 scene 时间轴推进,在入点/cue 调 ChatCanvas API。
// 见 spec/plan/plan17-intro-film.md。引擎调用全 guard(方法随版本变,失败不打断)。

export type EngineApi = Record<string, ((...a: unknown[]) => unknown) | undefined>;

export interface FilmCtx {
  /// guard 调用引擎方法(不存在/抛错都安全)。
  call: (name: string, ...args: unknown[]) => void;
  /// 当前调速(0.5/1/2),cue 里需要时可读。
  rate: number;
  /// 路线图 teaser 控制(COMING SOON 卡)。
  teaser: { show: (title: string, body: string) => void; hide: () => void };
}

export interface Scene {
  id: string;
  title: string; // 进度条幕刻度名
  durationMs: number;
  enter?: (ctx: FilmCtx) => void; // 幕起点:置引擎到该幕状态(可重入,供 seek)
  cues?: { at: number; fn: (ctx: FilmCtx) => void }[]; // 幕内一次性触发(at=幕内 ms)
}

export interface SceneMark {
  id: string;
  title: string;
  startMs: number;
  durationMs: number;
}

export class FilmDirector {
  readonly totalMs: number;
  readonly marks: SceneMark[];
  clock = 0;
  rate = 1;
  playing = false;

  private starts: number[] = [];
  private sceneIdx = -1;
  private fired = new Set<string>();

  /// clock/rate/scene 变化时回调(播放器 UI 用)。
  onUpdate?: (info: { clock: number; total: number; sceneIdx: number; playing: boolean; rate: number }) => void;

  constructor(
    private scenes: Scene[],
    private chat: EngineApi,
    private teaser: FilmCtx["teaser"],
  ) {
    let acc = 0;
    this.marks = scenes.map((s) => {
      const m = { id: s.id, title: s.title, startMs: acc, durationMs: s.durationMs };
      this.starts.push(acc);
      acc += s.durationMs;
      return m;
    });
    this.totalMs = acc;
  }

  private ctx(): FilmCtx {
    return {
      rate: this.rate,
      teaser: this.teaser,
      call: (name, ...args) => {
        try {
          const fn = this.chat[name];
          if (typeof fn === "function") fn(...args);
        } catch (e) {
          console.warn(`[film] ${name} 失败`, e);
        }
      },
    };
  }

  play() {
    this.playing = true;
    this.ctx().call("set_paused", false);
    this.emit();
  }
  pause() {
    this.playing = false;
    this.ctx().call("set_paused", true);
    this.emit();
  }
  toggle() {
    this.playing ? this.pause() : this.play();
  }
  setRate(r: number) {
    this.rate = r;
    this.emit();
  }

  /// 跳到绝对 ms:重入所属幕(置引擎到幕首),标记此前 cue 已触发(不补放)。
  seek(ms: number) {
    this.clock = Math.max(0, Math.min(ms, this.totalMs));
    const idx = this.sceneAt(this.clock);
    this.enterScene(idx, /*fromSeek*/ true);
    this.emit();
  }

  /// 主循环每帧调:推进 clock,跨幕则 enter,触发到点的 cue。
  tick(dtMs: number) {
    if (!this.playing) return;
    this.clock += dtMs * this.rate;
    if (this.clock >= this.totalMs) {
      this.clock = this.totalMs;
      this.playing = false; // 不循环(决议):停在收尾
    }
    const idx = this.sceneAt(this.clock);
    if (idx !== this.sceneIdx) this.enterScene(idx, false);
    // 触发当前幕内已到点的 cue
    const s = this.scenes[idx];
    if (s?.cues) {
      const local = this.clock - this.starts[idx];
      for (let i = 0; i < s.cues.length; i++) {
        const key = `${idx}:${i}`;
        if (local >= s.cues[i].at && !this.fired.has(key)) {
          this.fired.add(key);
          s.cues[i].fn(this.ctx());
        }
      }
    }
    this.emit();
  }

  private sceneAt(ms: number): number {
    let idx = 0;
    for (let i = 0; i < this.starts.length; i++) if (ms >= this.starts[i]) idx = i;
    return idx;
  }

  private enterScene(idx: number, fromSeek: boolean) {
    this.sceneIdx = idx;
    this.teaser.hide(); // 切幕先清 teaser
    // 重置本幕 cue 触发标记;seek 时把"已过去"的 cue 标记为已触发(不补放)
    const s = this.scenes[idx];
    for (let i = 0; i < (s?.cues?.length ?? 0); i++) {
      const key = `${idx}:${i}`;
      const local = this.clock - this.starts[idx];
      if (fromSeek && s!.cues![i].at <= local) this.fired.add(key);
      else this.fired.delete(key);
    }
    s?.enter?.(this.ctx());
  }

  private emit() {
    this.onUpdate?.({
      clock: this.clock,
      total: this.totalMs,
      sceneIdx: this.sceneIdx,
      playing: this.playing,
      rate: this.rate,
    });
  }
}
