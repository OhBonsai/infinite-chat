// wasm-pack(--target web)产物的最小类型声明,供 tsc/编辑器在 pkg 生成前也能通过。
// 实际产物由 `npm run build:wasm` 生成到 web/pkg/。
declare module "*infinite_chat_wasm.js" {
  /** 加载并实例化 wasm 模块。 */
  export default function init(input?: unknown): Promise<unknown>;

  export interface ChatCanvasConfig {
    layout: (
      runTexts: string[],
      runRoles: Uint32Array,
      maxWidth: number,
      tables?: unknown,
    ) => Float32Array | { positions: Float32Array; tables: Float32Array };
    rasterize: (cluster: string, style: number, kind: number) => Uint8Array;
    serverUrl?: string;
    sessionId?: string;
    /// Plan 5D 重放:预录事件 [{t, raw}](raw = opencode 信封 JSON 串)。
    replay?: { t: number; raw: string }[];
  }

  export interface ChatStats {
    fps: number;
    frameMsAvg: number;
    frameMsMax: number;
    dropped: number;
    glyphsVisible: number;
    glyphsTotal: number;
    blocksVisible: number;
    blocksTotal: number;
    atlasUsed: number;
    atlasCap: number;
    atlasEvict: number;
    camZoom: number;
    paused: number;
    srcBitmap: number;
    srcTinySdf: number;
    srcMsdf: number;
    srcRgba: number;
  }

  export class ChatCanvas {
    constructor(canvas: HTMLCanvasElement, config: ChatCanvasConfig);
    start(): void;
    stats(): ChatStats;
    set_paused(paused: boolean): void;
    step(): void;
    set_debug_geometry(on: boolean): void;
    refresh_fonts(): void;
    set_glyph_mode(mode: number): void;
    /** 平移画布(屏幕/设备像素;Plan 6 web 层输入)。dy>0 看更新内容,dx>0 看右侧。 */
    pan_by(dx: number, dy: number): void;
    /** 围绕屏幕点(设备像素)缩放;factor>1 放大。 */
    zoom_at(factor: number, sx: number, sy: number): void;
    /** 设表格面板渲染样式(实时,无需重排/reload)。颜色分量 0..1。 */
    set_table_style(cfg: {
      lineColor?: [number, number, number, number];
      headerFill?: [number, number, number, number];
      aoColor?: [number, number, number];
      lineW?: number;
      ao?: number;
      aoWidth?: number;
      radius?: number;
    }): void;
    load_msdf(meta: {
      atlasW: number;
      atlasH: number;
      fontSize: number;
      ids: Uint32Array;
      cells: Float32Array;
      pixels: Uint8Array[];
    }): void;
  }
}
