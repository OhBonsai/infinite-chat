// Vue harness(Plan 34 S5):独立子工程 —— 复用主 web/ 的 wasm pkg 与桥(../../src),
// 证明「行为在 wasm+bridge,框架只是壳」(0021 划界 / research R8)。
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  server: {
    fs: { allow: ["../..", "."] }, // 允许引 ../../src 与 ../../pkg
  },
});
