# GitHub Pages 演示部署

把 infinite-chat 引擎 + icon 画廊作为**静态站**部署到 GitHub Pages,在专用 `gh-pages` 分支。
CI 自动构建,你只需在仓库设置里点两下。

- 站点 URL(部署后):**https://ohbonsai.github.io/infinite-chat/**
- 引擎演示(根路径):离线**重放** `showcase` 会话,展示流式 markdown / 代码 / 表格 / 数学 / 任务 / CJK·emoji / 无限画布。
- 图标画廊:**…/infinite-chat/gallery.html**(16 个 tool 的 shaderbox 动态图标)。

---

## 一次性配置(3 步)

1. **合并本次改动到 `master`**(含 `.github/workflows/pages.yml`、`web/` 改动、`scripts/gen-*.mjs`、`web/public/cases/showcase.json`、`web/public/gallery.html`)。
2. push 后,**Actions** 标签页会自动跑 `Deploy demo to GitHub Pages`。首次约 3–6 分钟(编译 wasm)。它会把 `web/dist` 推到新分支 **`gh-pages`**。
3. 仓库 **Settings → Pages → Build and deployment**:
   - **Source** 选 **Deploy from a branch**;
   - **Branch** 选 **`gh-pages`** / **`/ (root)`** → Save。
   等 1 分钟,访问上面的 URL。

> 之后每次 push 到 `master`(动到 `crates/`、`web/`、生成脚本或本工作流)都会自动重建 + 部署。也可在 Actions 里手动 **Run workflow**。

---

## 工作原理(改了什么)

- **子路径适配**:项目页在 `/<repo>/` 下,故构建时 `PAGES_BASE=/infinite-chat/` → vite `base`;运行时所有 `fetch`(replay cases、字体)统一走 `import.meta.env.BASE_URL` 前缀(改了 `replay.ts` / `msdf.ts` / `math-fonts.ts` 的默认 base)。**换仓名就改 workflow 里的 `PAGES_BASE` 和 `vite.config` 注释**。
- **离线演示模式**:`VITE_DEMO=1` 构建 → `main.ts` 在无 `?server` 时默认重放 `showcase`、挂顶部链接栏(画廊 / GitHub)、关掉发不出去的输入框。本地 `npm run dev` 不受影响。
- **演示资产单一真源**:`scripts/gen-showcase.mjs`(导览会话)、`scripts/gen-gallery.mjs`(读 `spec/plan/tool-icons.frag` 内联成 `gallery.html`)。CI 每次重生成,改 `.frag` 即同步画廊。
- **部署**:`peaceiris/actions-gh-pages` 把 `web/dist` 推到 `gh-pages`(orphan,单提交干净),自动加 `.nojekyll`。

---

## 已知取舍 / 注意

- **WebGPU 浏览器支持**:引擎走 `wgpu`(WebGPU,降级 WebGL2)。**Chrome / Edge 最稳**;Safari/Firefox 视版本走 WebGL2 后备。画廊页是纯 WebGL2,兼容更广。
- **字体 MSDF(英文 + LaTeX 入库,CJK 不入库)**:
  - **LaTeX(`katex-msdf.*`,173 字 ~0.25MB)** 已**随库提交**(`.gitignore` 末尾负向放行)→ 数学直接 MSDF 锐利。
  - **英文/ASCII(`ascii-msdf.*`,95 字 ~0.1MB)**:**CI 自动烘**(`Bake ASCII MSDF` 步,`continue-on-error`)。也可本地一次性烘后提交进库:`cd web && npm run bake:ascii`(macOS 上 `msdf-bmfont-xml` 自带二进制可跑;**本沙箱 glibc 太旧跑不了**,故未在此预烘)。`main.ts` 默认加载 `ascii-msdf`,英文/ASCII 即锐利。
  - **CJK(`lxgw-msdf.*`,~10MB)** 体积大,**不入库**;需要时 `node scripts/bake-msdf.mjs` 本地烘,或加 `?msdf` 让运行时加载全集。未命中字 → TinySDF 回退(能显示,极限缩放稍软)。
- **不放版权 deck**:画廊只展示**我们自绘的 16 个 tool 图标**;PixelSpiritDeck 50 卡造型按 plan16 §6「公开分发即触发重审」**不上公开站**。
- **首次构建较慢**:wasm 全量编译;`Swatinem/rust-cache` 之后会显著加速。
- **想用官方 artifact 方式而非分支**:把部署步改成 `actions/upload-pages-artifact` + `actions/deploy-pages`(需 `permissions: pages: write, id-token: write`,Pages Source 选 **GitHub Actions**)。本工作流按你要求用**专用 `gh-pages` 分支**。

---

## 本地预演(可选)

```bash
cd web
PAGES_BASE=/infinite-chat/ VITE_DEMO=1 npm run build
npx vite preview --base /infinite-chat/   # 本地看子路径版
# 或不带 base 直接 npm run dev 看实时/合成模式
```
