// home-bloom.ts — Plan 42 G4:首页「绽放」高潮幕驱动器。scene enter 只做引擎框取/预设;
// 这里按幕内时间每帧推进 formation_progress、成花期撒花瓣、转发指针风场、点击绽放,离幕散场回文。
// 引擎侧接口(plan42):formation_begin/progress/end/petals + set_wind(屏幕/设备 px)。

interface BloomEngine {
  formation_begin(json: string): boolean;
  formation_progress(p: number): void;
  formation_end(): void;
  formation_petals(count: number): void;
  set_wind(x: number, y: number, radius: number, strength: number): void;
}

const IN_MS = 2600; // 聚花
const HOLD_MS = 4400; // 成花停留(撒瓣 + 指针可玩)
const OUT_MS = 2600; // 散场回文

function easeInOut(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

export class BloomController {
  private active = false;
  private start = 0;
  private raf = 0;
  private lastPetal = 0;
  private readonly dpr = window.devicePixelRatio || 1;

  constructor(
    private engine: BloomEngine,
    private canvas: HTMLCanvasElement,
  ) {
    this.onMove = this.onMove.bind(this);
    this.onLeave = this.onLeave.bind(this);
    this.onClick = this.onClick.bind(this);
  }

  /** 幕切回调:进入 bloom 幕 → 开始;离开 → 散场收尾。 */
  onScene(sceneId: string): void {
    if (sceneId === "bloom") this.begin();
    else this.end();
  }

  private begin(): void {
    if (this.active) return;
    this.active = true;
    this.start = performance.now();
    this.lastPetal = 0;
    this.engine.formation_begin(JSON.stringify({ shape: "rose", a: 300, k: 5, seed: 2 }));
    this.canvas.addEventListener("pointermove", this.onMove);
    this.canvas.addEventListener("pointerleave", this.onLeave);
    this.canvas.addEventListener("pointerdown", this.onClick);
    this.raf = requestAnimationFrame(this.tick);
  }

  private end(): void {
    if (!this.active) return;
    this.active = false;
    cancelAnimationFrame(this.raf);
    this.engine.set_wind(0, 0, 0, 0);
    this.engine.formation_progress(0);
    this.engine.formation_end();
    this.canvas.removeEventListener("pointermove", this.onMove);
    this.canvas.removeEventListener("pointerleave", this.onLeave);
    this.canvas.removeEventListener("pointerdown", this.onClick);
  }

  private tick = (now: number): void => {
    if (!this.active) return;
    const t = now - this.start;
    let p: number;
    if (t < IN_MS) {
      p = easeInOut(t / IN_MS);
    } else if (t < IN_MS + HOLD_MS) {
      p = 1;
      // 成花期分批撒花瓣(每 ~650ms 一小把)。
      if (now - this.lastPetal > 650) {
        this.engine.formation_petals(18);
        this.lastPetal = now;
      }
    } else if (t < IN_MS + HOLD_MS + OUT_MS) {
      p = 1 - easeInOut((t - IN_MS - HOLD_MS) / OUT_MS);
    } else {
      p = 0;
    }
    this.engine.formation_progress(p);
    this.raf = requestAnimationFrame(this.tick);
  };

  private windAt(e: PointerEvent): [number, number] {
    const r = this.canvas.getBoundingClientRect();
    return [(e.clientX - r.left) * this.dpr, (e.clientY - r.top) * this.dpr];
  }

  private onMove(e: PointerEvent): void {
    const [x, y] = this.windAt(e);
    this.engine.set_wind(x, y, 220 * this.dpr, 110 * this.dpr);
  }

  private onLeave(): void {
    this.engine.set_wind(0, 0, 0, 0);
  }

  // 点击绽放:撒一把花瓣 + 一记向外风脉冲(强 → 400ms 衰减 → 0)。
  private onClick(e: PointerEvent): void {
    this.engine.formation_petals(40);
    const [x, y] = this.windAt(e);
    const t0 = performance.now();
    const pulse = (now: number): void => {
      if (!this.active) return;
      const k = 1 - Math.min((now - t0) / 400, 1);
      if (k <= 0) return; // 衰减完由 onMove/onLeave 接管
      this.engine.set_wind(x, y, 300 * this.dpr, 260 * this.dpr * k);
      requestAnimationFrame(pulse);
    };
    requestAnimationFrame(pulse);
  }

  dispose(): void {
    this.end();
  }
}
