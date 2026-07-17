import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// Node 全局(tsconfig types:[] 未引 @types/node;config 在 Node 下由 vite/esbuild 执行)。
declare const process: { env: Record<string, string | undefined> };

// wasm() 让 wasm-pack 产物可被 import;pkg 排除出依赖预打包(它是本地生成的 ESM)。
// pretext(@chenglou/pretext)是发布的 npm 包,走正常 dependencies + 预打包,无需 alias。
export default defineConfig({
  // GitHub Pages 子路径:项目页托管在 https://<user>.github.io/<repo>/,资源/路由需带 base。
  // CI 里 `PAGES_BASE=/infinite-chat/ npm run build`;本地 dev/实时默认 "/" 不受影响。
  // 运行时 fetch(cases/fonts)统一用 import.meta.env.BASE_URL(= 此 base),见 replay/msdf/math-fonts。
  base: process.env.PAGES_BASE || "/",
  plugins: [wasm(), topLevelAwait()],
  // Plan 40:四页站 —— landing(/)+ chat + markdown + gallery(public 静态)。
  // 开发 harness 移 dev.html(/dev.html);build 产各页到 dist 对应子路径。
  build: {
    rollupOptions: {
      input: {
        // Plan 40 四页站:landing(index)/ chat / markdown;dev = 原开发 harness(不上导航)。
        main: new URL("index.html", import.meta.url).pathname,
        chat: new URL("chat/index.html", import.meta.url).pathname,
        markdown: new URL("markdown/index.html", import.meta.url).pathname,
        components: new URL("components/index.html", import.meta.url).pathname,
        dev: new URL("dev.html", import.meta.url).pathname,
      },
    },
  },
  optimizeDeps: {
    exclude: ["pkg", "infinite-chat-wasm"],
  },
  server: {
    port: 5173,
    // 用 ?server=/opencode 走这个代理 → 同源,绕开 CORS;Vite 转发到本地 opencode。
    // SSE(/opencode/event)也走代理。改端口就改 target。
    proxy: {
      "/opencode": {
        target: "http://localhost:4096",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/opencode/, ""),
      },
    },
  },
});
