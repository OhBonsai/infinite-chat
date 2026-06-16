// style-config(Plan 6)— web 层样式配置(Figma 式属性面板的数据源)。
//
// 渲染样式里"非内容、可调观感"的部分放这里(表格垂直对齐…),由排版(layout-bridge)读取、
// 样式面板(style-panel)写入。改值后宿主调 `chat.refresh_fonts()` 作废排版缓存 → 下一帧重排。
// 结构按元素分组(table / list / div …),便于面板分节渲染,也便于后续扩展。

export type VAlign = "top" | "center" | "bottom";
/// 水平对齐;`auto` = 跟随 markdown 列对齐(`:--` / `:-:` / `--:`),其余为全表覆盖。
export type HAlign = "auto" | "left" | "center" | "right";
export type RGBA = [number, number, number, number]; // 分量 0..1
export type RGB = [number, number, number];

/// 表格面板渲染样式(0..1 颜色;字段名与 wasm `set_table_style` 一致 → 可直接透传)。
export interface TableRender {
  lineColor: RGBA; // 网格线 / 外框
  headerFill: RGBA; // 表头底
  aoColor: RGB; // AO 辉光色(暗色主题取白)
  lineW: number; // 线宽 px
  ao: number; // AO 强度 0..~0.5
  aoWidth: number; // AO 向内淡出 px
  radius: number; // 圆角 px
}

export interface StyleConfig {
  table: {
    /// 单元格文字在行内的垂直对齐(多行/不等高行显著;单行行内即整体上/中/下)。**布局**(走重排)。
    vAlign: VAlign;
    /// 水平对齐覆盖(auto = 用列对齐)。**布局**(走重排)。
    hAlign: HAlign;
  };
  /// 表格面板**渲染**样式(颜色/AO/线宽/圆角)。走 wasm `set_table_style`,实时、不重排。
  tableRender: TableRender;
  // 占位:后续元素分组(list 标记/缩进、div 容器内边距…)接到这里,面板自动出节。
}

const DEFAULT: StyleConfig = {
  table: { vAlign: "center", hAlign: "auto" },
  // 默认 = core `TableStyle::default()`(theme::TABLE_RULE / TABLE_HEADER_BG)。
  tableRender: {
    lineColor: [0.26, 0.29, 0.36, 0.9],
    headerFill: [0.16, 0.18, 0.24, 0.6],
    aoColor: [1, 1, 1],
    lineW: 1,
    ao: 0.12,
    aoWidth: 10,
    radius: 4,
  },
};

const KEY = "infinite-chat.styleConfig";

function clone(c: StyleConfig): StyleConfig {
  return JSON.parse(JSON.stringify(c)) as StyleConfig;
}

let config: StyleConfig = load();

function load(): StyleConfig {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return clone(DEFAULT);
    const c = JSON.parse(raw) as Partial<StyleConfig>;
    return {
      table: { ...DEFAULT.table, ...(c.table ?? {}) },
      tableRender: { ...DEFAULT.tableRender, ...(c.tableRender ?? {}) },
    };
  } catch {
    return clone(DEFAULT);
  }
}

/// 当前配置(排版热路径直接读;返回引用,勿原地改——改用 `setStyleConfig`)。
export function getStyleConfig(): StyleConfig {
  return config;
}

/// 整体替换并持久化(面板写入用)。
export function setStyleConfig(next: StyleConfig): void {
  config = next;
  try {
    localStorage.setItem(KEY, JSON.stringify(config));
  } catch {
    /* localStorage 不可用 → 忽略,仅本会话生效 */
  }
}
