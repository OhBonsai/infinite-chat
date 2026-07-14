// feature-manifest.test.ts — Plan 39 R0:manifest schema 锁(五层门 unit 层;保清单不腐)。
import { describe, expect, it } from "vitest";
import manifest from "../../test/feature-manifest.json";

const SCENES = ["showcase", "chat-full", "empty", "gallery", "debug"];
const OPS = /^(settle|pause|tapFold|tapLink|waitMs:\d+|panUp:\d+|pushText:[\s\S]+|pushTool:[\s\S]+|removePart:.+|js:[\s\S]+)$/;

describe("feature-manifest schema(plan39 单一真值源)", () => {
  const groups = manifest.groups.map((g) => g.id);
  const items = manifest.items;

  it("结构:九大项、40+ 小项、id 唯一、组引用有效", () => {
    expect(manifest.version).toBe(1);
    expect(groups).toHaveLength(9);
    expect(new Set(groups).size).toBe(9);
    expect(items.length).toBeGreaterThanOrEqual(40);
    expect(new Set(items.map((i) => i.id)).size).toBe(items.length);
    for (const it of items) expect(groups, it.id).toContain(it.group);
    for (const g of groups)
      expect(items.some((i) => i.group === g), `组 ${g} 至少一项`).toBe(true);
  });

  it("捕获:kind/scene/steps/assert 合法;webm 有固定时长;每项有人话解释与出处", () => {
    for (const it of items) {
      expect(["png", "webm"], it.id).toContain(it.kind);
      expect(SCENES, it.id).toContain(it.capture.scene);
      expect(it.capture.steps.length, it.id).toBeGreaterThan(0);
      for (const s of it.capture.steps) expect(s, `${it.id}: ${s}`).toMatch(OPS);
      const a = it.capture.assert as Record<string, unknown>;
      expect(a && ("textVisible" in a || "ink" in a), `${it.id} 必带断言`).toBeTruthy();
      if (it.kind === "webm") {
        const d = (it.capture as { durationMs?: number }).durationMs ?? 0;
        expect(d, `${it.id} webm 固定时长`).toBeGreaterThanOrEqual(3000);
        expect(d).toBeLessThanOrEqual(8000);
      }
      expect(it.desc, it.id).toContain("你会看到");
      expect(it.desc, it.id).toContain("出处");
    }
  });

  it("retina:每大项 ≥1 张 dpr=2 png(DoD-2)", () => {
    for (const g of groups) {
      const has = items.some(
        (i) => i.group === g && i.kind === "png" && (i.capture as { dpr?: number }).dpr === 2,
      );
      expect(has, `组 ${g} 缺 dpr=2 样张`).toBe(true);
    }
  });

  it("webm 段数 8–12(DoD-3)", () => {
    const n = items.filter((i) => i.kind === "webm").length;
    expect(n).toBeGreaterThanOrEqual(8);
    expect(n).toBeLessThanOrEqual(12);
  });
});
