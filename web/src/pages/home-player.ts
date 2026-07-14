// home-player.ts — Plan 41 首页自动播放控制器(围绕现成 FilmDirector,不改 director)。
// 职责:自动推进 + 循环(director「到尾停」→ 这里 seek(0) 回 S1)· 手动接管(←/→ / 滚轮 / 进度点)
// · 接管后停自动、空闲 idleMs 恢复 · 暂停/播放 · 隐藏暂停。director 只被 tick/seek/play/pause 驱动。
import type { FilmDirector } from "../film/director";

export class HomePlayer {
  private takeover = false; // 手动接管中:停自动推进(引擎仍渲当前幕)
  private paused = false; // 显式暂停:引擎冻帧
  private idleTimer = 0;
  private wheelLock = 0;
  private last = 0;
  private raf = 0;
  private readonly idleMs: number;
  private readonly auto: boolean; // false = 不自动播(reduced-motion:静帧 + 只手动)
  /** 接管/暂停态变化回调(chrome 反映)。 */
  onState?: (s: { paused: boolean; takeover: boolean }) => void;

  constructor(
    private dir: FilmDirector,
    opts: { idleMs?: number; auto?: boolean } = {},
  ) {
    this.idleMs = opts.idleMs ?? 30_000;
    this.auto = opts.auto ?? true;
  }

  start(): void {
    this.dir.seek(0);
    if (this.auto) this.dir.play();
    else this.takeover = true; // 不自动播:停在 S1,等手动
    this.raf = requestAnimationFrame(this.loop);
    this.bindInput();
  }

  stop(): void {
    cancelAnimationFrame(this.raf);
    window.clearTimeout(this.idleTimer);
  }

  private loop = (now: number): void => {
    if (!this.last) this.last = now;
    const dt = now - this.last;
    this.last = now;
    if (!this.paused && !this.takeover && this.dir.playing) {
      this.dir.tick(dt);
      if (this.dir.clock >= this.dir.totalMs) {
        this.dir.seek(0); // 循环回 S1(director 自身不循环)
        this.dir.play();
      }
    }
    this.raf = requestAnimationFrame(this.loop);
  };

  /** 当前幕 index(由 clock 落在哪个 mark 段判定)。 */
  currentIdx(): number {
    const marks = this.dir.marks;
    let idx = 0;
    for (let i = 0; i < marks.length; i++) if (this.dir.clock >= marks[i].startMs) idx = i;
    return idx;
  }

  /** 跳到某幕(进度点/键盘/滚轮):接管 + seek 到幕首 + 引擎续渲;重置空闲计时。 */
  gotoScene(i: number): void {
    const n = this.dir.marks.length;
    const idx = Math.max(0, Math.min(i, n - 1));
    this.takeover = true;
    this.dir.seek(this.dir.marks[idx].startMs);
    if (!this.paused) this.dir.play(); // 保引擎不冻(接管只停「自动推进」)
    this.resetIdle();
    this.emitState();
  }

  next(): void {
    this.gotoScene(this.currentIdx() + 1);
  }
  prev(): void {
    this.gotoScene(this.currentIdx() - 1);
  }

  togglePause(): void {
    this.paused = !this.paused;
    if (this.paused) {
      this.dir.pause();
      window.clearTimeout(this.idleTimer);
    } else {
      this.dir.play();
      if (this.takeover) this.resetIdle();
    }
    this.emitState();
  }

  /** 是否显式暂停中(隐藏页恢复时判断)。 */
  isPaused(): boolean {
    return this.paused;
  }

  private resetIdle(): void {
    window.clearTimeout(this.idleTimer);
    if (!this.auto) return; // reduced-motion:接管后不自动恢复
    this.idleTimer = window.setTimeout(() => {
      this.takeover = false; // 空闲恢复自动推进(从当前 clock 续)
      this.emitState();
    }, this.idleMs);
  }

  private emitState(): void {
    this.onState?.({ paused: this.paused, takeover: this.takeover });
  }

  private bindInput(): void {
    window.addEventListener("keydown", (e) => {
      if (e.key === "ArrowRight") {
        e.preventDefault();
        this.next();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        this.prev();
      } else if (e.key === " " || e.key === "Spacebar") {
        e.preventDefault();
        this.togglePause();
      }
    });
    // 滚轮切幕(节流:一次滚动一幕)。
    window.addEventListener(
      "wheel",
      (e) => {
        const now = e.timeStamp;
        if (now - this.wheelLock < 700) return;
        if (Math.abs(e.deltaY) < 12) return;
        this.wheelLock = now;
        e.deltaY > 0 ? this.next() : this.prev();
      },
      { passive: true },
    );
    // 触摸横滑切幕(移动端):水平位移 > 阈值且大于纵向 → 切幕。
    let tx = 0;
    let ty = 0;
    window.addEventListener("touchstart", (e) => {
      tx = e.changedTouches[0].clientX;
      ty = e.changedTouches[0].clientY;
    }, { passive: true });
    window.addEventListener("touchend", (e) => {
      const dx = e.changedTouches[0].clientX - tx;
      const dy = e.changedTouches[0].clientY - ty;
      if (Math.abs(dx) > 48 && Math.abs(dx) > Math.abs(dy)) dx < 0 ? this.next() : this.prev();
    }, { passive: true });
  }
}
