// math-fonts(Plan 12 / 0013 §8)— 懒加载 KaTeX 字体(woff2)用于数学字形栅化。
//
// 数学排版在 core(RaTeX 给绝对坐标 + KaTeX 字族名);web 只负责把对应 KaTeX woff2 注册成
// `@font-face`,使离屏 canvas(glyph-raster)能用 `fontForRole(26..40)` 返回的 `KaTeX_*` 字族
// 栅化字形。**懒加载**:首个数学公式出现时才拉(无数学零成本,守体积,plan12 §2①/§4)。
//
// 字体文件:从 `RaTeX/platforms/web/fonts/` 复制到 `web/public/fonts/katex/`(见
// `scripts/copy-katex-fonts.mjs`;与 lxgw-msdf 同样 gitignore,可由 RaTeX 重生)。

// (CSS 字族名, woff2 文件基名, 字重, 斜体)—— 与 layout-bridge.fontForRole 的 KaTeX_* 对齐。
const FACES: ReadonlyArray<[family: string, file: string, weight: string, style: string]> = [
  ["KaTeX_Main", "KaTeX_Main-Regular", "normal", "normal"],
  ["KaTeX_Main", "KaTeX_Main-Bold", "bold", "normal"],
  ["KaTeX_Main", "KaTeX_Main-Italic", "normal", "italic"],
  ["KaTeX_Main", "KaTeX_Main-BoldItalic", "bold", "italic"],
  ["KaTeX_Math", "KaTeX_Math-Italic", "normal", "italic"],
  ["KaTeX_Math", "KaTeX_Math-BoldItalic", "bold", "italic"],
  ["KaTeX_AMS", "KaTeX_AMS-Regular", "normal", "normal"],
  ["KaTeX_Size1", "KaTeX_Size1-Regular", "normal", "normal"],
  ["KaTeX_Size2", "KaTeX_Size2-Regular", "normal", "normal"],
  ["KaTeX_Size3", "KaTeX_Size3-Regular", "normal", "normal"],
  ["KaTeX_Size4", "KaTeX_Size4-Regular", "normal", "normal"],
  ["KaTeX_Caligraphic", "KaTeX_Caligraphic-Regular", "normal", "normal"],
  ["KaTeX_Fraktur", "KaTeX_Fraktur-Regular", "normal", "normal"],
  ["KaTeX_SansSerif", "KaTeX_SansSerif-Regular", "normal", "normal"],
  ["KaTeX_Script", "KaTeX_Script-Regular", "normal", "normal"],
  ["KaTeX_Typewriter", "KaTeX_Typewriter-Regular", "normal", "normal"],
];

let started: Promise<void> | null = null;

/// 是否已加载完成(glyph-raster 在加载完成前对数学字形回退,加载后 `refresh_fonts` 重栅)。
let ready = false;
export function mathFontsReady(): boolean {
  return ready;
}

/// 懒加载所有 KaTeX 字体(幂等:多次调用复用同一 Promise)。`base` = 字体目录(默认
/// `/fonts/katex/`)。完成后 `mathFontsReady()` 转真;调用方宜在 `.then` 里 `chat.refresh_fonts()`
/// 让已出现的公式用真字体重栅。
export function loadMathFonts(base = import.meta.env.BASE_URL + "fonts/katex/"): Promise<void> {
  if (started) return started;
  started = (async () => {
    const fontset = (document as Document & { fonts?: FontFaceSet }).fonts;
    if (!fontset) return; // 无 FontFaceSet(老环境)→ 跳过,靠系统回退
    await Promise.all(
      FACES.map(async ([family, file, weight, style]) => {
        try {
          const face = new FontFace(family, `url(${base}${file}.woff2) format("woff2")`, {
            weight,
            style,
          });
          await face.load();
          fontset.add(face);
        } catch (e) {
          console.warn(`[math-fonts] ${file} 加载失败`, e);
        }
      }),
    );
    ready = true;
  })();
  return started;
}
