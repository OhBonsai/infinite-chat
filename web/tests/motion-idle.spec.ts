// motion-idle.spec.ts — Plan 25 M2a(GOAL M2 验收两断言):
// ①「idle 回冻结」:settled 后活跃动效组归零(orb 静态冻结、光标/指示条退场,0028 护栏)。
// ②「动画组 ≤4」:流式中生命感在场(≥1)且不超同屏预算(设计原则 4 / MOT 上限)。
// 计数源 = core FrameStats.anim_groups(经 chat.stats().animGroups,`?debug` 同源)。
import { test, expect } from "@playwright/test";
import { bootVisible } from "./helpers";

test("M2a idle 回冻结:settled 后活跃动效组归零", async ({ page }) => {
  await bootVisible(page); // showcase 全揭示 settled
  await expect
    .poll(() => page.evaluate(() => window.__chat.stats().animGroups), { timeout: 10_000 })
    .toBe(0);
});

test("M2a 流式中生命感在场且 ≤4", async ({ page }) => {
  await bootVisible(page);
  // 切 reader 慢速 → push 长文本 → 滚到底(动效只对可见块发射)→ 揭示中应有光标/指示条/orb。
  await page.evaluate(() => {
    window.__chat.set_reveal_preset("reader");
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: {
            type: "text",
            id: "pm",
            messageID: "mm",
            text: "生命感测试:一段足够长的文本,让揭示持续数秒钟,期间行内光标与左缘指示条应当在场,且随 settled 退场。",
          },
          time: 1,
        },
      }),
    );
    window.__chat.pan_by(0, 1e6); // 滚到底(新消息在 showcase 之下)
  });
  await expect
    .poll(() => page.evaluate(() => window.__chat.stats().animGroups), { timeout: 10_000 })
    .toBeGreaterThanOrEqual(1);
  const peak = await page.evaluate(() => window.__chat.stats().animGroups);
  expect(peak, "同屏活跃动效组 ≤4(设计原则 4)").toBeLessThanOrEqual(4);
});
