# 决策记录 0039:crate 分层 —— primitives / components 抽离(纯搬家)

- 日期: 2026-07-14(Plan 33 C0;GOAL 原文写 0036,该号已被 plan31 svg ADR 占用 → 顺延本号)
- 状态: 已采纳
- 前置: [0032](0032-interaction-architecture-sdf-world-not-component-framework.md)(分层≠框架)·
  [0033](0033-part-render-contract.md)(渲染契约)· [0037](0037-declarative-part-decoration-contract.md)
  (装饰契约 —— 组件自治的前提)· [dev-practices §4.2 CR 铁律](../dev-practices.md)
- 范围: workspace 从 {core, render, wasm} 扩为 {primitives, components, core, render, wasm};纯移动零行为变化。

## 1. 背景 / 问题

app.rs 7.6k 行 / core 17k+ 行巨石化;part 组件(diff 卡/tool 卡/code view)已凭 0033+0037
契约自治(纯函数「语义 → spans + decorations」),却仍住在 core —— 无法被第二宿主消费,
编译不并行,测试边界糊。

## 2. 决策

```
crates/
  primitives/   图元词汇 + token(零 workspace 依赖;unicode-segmentation/serde 可)
      frame.rs · shaderbox.rs · theme.rs · motion.rs
      + 类型下沉:StyleRole / StyledSpan(自 content)· DecorationOp/DecorKind/DecorSlot/
        RenderOutput(自 partrender)· graphemes(自 support)
  components/   part 组件 = 纯函数注册表(只依赖 primitives;dev-dep core 供测试注入真 parse)
      partrender.rs · partspecific.rs · codeblock.rs · highlight.rs
  core/         引擎机制(store/fsm/content 解析主体/reveal/rhythm/app/spatial/camera/…)
  render/ wasm/ 不动(use 路径改指 primitives)
依赖方向:primitives ← {components, core, render};components ← core(core 注入 registry);禁反向。
```

- **parse 注入**(C0 审计定):partrender/partspecific 共 7 处调 `content::parse_markdown/plain`
  (解析主体留 core —— 它咬 nodes/reveal/highlight/jcode)。`RenderCtx` 增
  `parse: fn(&str) -> Vec<StyledSpan>` 函数指针(Copy 保持),core 构造处注入真 parse;
  components 测试经 dev-dep core 注入同一真 parse(cargo 允许 dev 环)→ 断言不变。
- **过渡镜像**:core `pub use` 重导出下沉/搬走的项,外部(render/wasm/测试)API 不变;
  镜像只活到 C3(删空 + grep 断言)。
- **审计结论**(2026-07-14 grep):frame/shaderbox/theme/highlight 零 crate 依赖;
  motion→theme(同迁);codeblock 仅 doc 注释提 app(改注释即净);support 整体不迁
  (咬 content/frame 的测试 harness),只下沉 `graphemes`。

## 3. 备选与否决理由

- **components 直依赖 core 拿 parse**:核心依赖方向反转(components→core→components 环);否决。
- **把 content 解析主体一并搬 components**:parse 咬 nodes(0020 身份)/reveal(0019 gate),
  那是引擎不是组件;否决。
- **搬家顺手重构**(改名/收 API/优化):违本 plan 第一铁律,行为 diff 不可审;否决 —— 想改的记遗留。

## 4. 影响

- 正:core 减 ~3.3k 行;组件可被第二宿主消费(examples/render_part_dump 为证);
  primitives/components 编译并行;测试边界随 crate 清晰。
- 负:一次性全仓 use 路径变动(执行窗口内禁其它 plan 交叉);RenderCtx 增 parse 字段
  (构造点全数补;fn 指针无 Default → 显式注入,杜绝静默空实现)。
- **防过度拆分条款**:再拆新 crate 须同时满足 ①有第二消费者 ②依赖面经 grep 审计零反向
  ③搬移 ≥500 行;三者缺一留 core。store/fsm/reveal/spatial **明确不拆**(引擎机制,
  无独立消费场景)。

## 5. 实现入口

`crates/primitives/src/` · `crates/components/src/` · workspace `Cargo.toml`;
搬家清单与编译时间对比见 [plan33_progress](../plan/plan33_progress.md)。
