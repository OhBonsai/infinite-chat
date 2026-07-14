// feature/playwright.config.ts — Plan 39 捕获专用(不入五层门;由 test/feature-report.mjs 驱动)。
// 与主 config 同 WebGPU flags;单 worker(捕获确定性);视频 720p。
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  timeout: 120_000,
  workers: 1,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:5173",
    viewport: { width: 1280, height: 900 },
    launchOptions: {
      args: [
        "--enable-unsafe-webgpu",
        "--enable-features=Vulkan",
        "--enable-webgpu-developer-features",
        "--ignore-gpu-blocklist",
        "--use-angle=default",
      ],
    },
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:5173",
    timeout: 180_000,
    reuseExistingServer: true,
    stdout: "ignore",
    stderr: "pipe",
  },
});
