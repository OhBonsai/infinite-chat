# Vue harness(Plan 34 S5)

**框架无关性回归保护**(0021 划界 / research chat-ui-libs R8):同一份 `web/pkg` wasm 与
同一套桥(`web/src/boot.ts` 及其依赖),Vue 只是壳(模板 + 生命周期 + 响应式状态条)。

- 独立子工程独立 lockfile,**不进主 `web/package.json`**。
- **零改 wasm API**:若接 Vue 需要动 `crates/wasm` 导出,即框架耦合,判负。
- 跑法:`npm install && npm run dev`(端口 5174),或 `bash scripts/vue-smoke.sh`(装依赖 +
  起服 + 跑 smoke)。
- Smoke:`web/tests/vue-smoke.spec.ts`(加载 / 流式渲染 / 滚动跟随 / 链接回调);
  依赖未装时显式 skip(fail-loud 标注,不静默)。
