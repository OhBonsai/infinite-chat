# 决策记录 0038:滚动跟随显式 FSM —— Following / Released / Anchoring

- 日期: 2026-07-13(Plan 34 S1)
- 状态: 已采纳
- 前置: [0002 §6](0002-render-engine-architecture.md)(锚底阈值与视口裁剪)·
  [0031](0031-event-fsm-resilience-and-js-rust-boundary.md)(事件 FSM 韧性 —— 本 FSM 是相机层姊妹篇)·
  [0032](0032-interaction-architecture-sdf-world-not-component-framework.md)(交互 = 引擎内命中,非组件树)·
  [research/chat-ui-libs-ai-elements-shadcn R10](../research/chat-ui-libs-ai-elements-shadcn.md) ·
  [research/streaming-chat-ux-standards §1](../research/streaming-chat-ux-standards.md)
- 范围: 画布相机的"跟随新内容"行为从隐式布尔升级为显式三态 FSM;释放信号全集与重进入条件成文。

## 1. 背景 / 问题

锚底跟随一直是 `stick_to_bottom: bool` + 分散在各入口的置假:上滚/横移/缩放/jump 置 false,
`build_frame` 里"距底 ≤ 48px 即置 true"。两个成文缺陷(research R10 点名):

1. **被动复跟随**:置 true 的判定每帧跑 —— 用户上滚 20px(仍在 48px 阈值内)会被下一帧
   立刻拉回底部,"绝不违背读者意图移动"(shadcn 15 条第 1 条)被违反。
2. **释放信号不成集**:选区拖拽不释放跟随;信号清单只存在于代码分支,无契约可测。

## 2. 决策

相机层引入显式三态(core 纯逻辑,R8 确定性,输入信号全部经既有 Engine API 注入):

```
            用户上滚/横移/缩放/选区拖拽/jump-to-view
  Following ────────────────────────────────────────▶ Released
      ▲                                                   │
      │ |pan−底| < 0.5(平滑到位)                          │ scroll_to_latest()
      │                                                   ▼
  Anchoring ◀──────────────────────────────────────── (显式跳最新)
      ▲                                                   │
      └──── 用户向下滚动**主动**抵底(≤48px 阈值)◀─────────┘(Released 直接吸附回 Following)
```

- **Following**:锚底跟「已揭示底」,临界阻尼 smooth-damp(行为 == 旧 `stick_to_bottom=true`,
  golden 逐字节回归基线);落后超一屏直接到位(历史瞬显不慢滚)。
- **Released**:相机纹丝不动;流式增高**零位移**(内容只向下生长,pan 不变仅 clamp 上界),
  "读者视口内容屏幕位移 == 0" 是硬不变量(streaming-chat-ux §1②)。
- **Anchoring**:`scroll_to_latest()` 触发的过渡态 —— 复用 Following 的 smooth-damp 通道
  平滑滚向底部,`|pan−底| < 0.5` 即转 Following;期间任何释放信号可再打断回 Released。

**释放信号全集**(全部已收敛到 Engine API 入口,web 输入层零新增路由):

| 信号 | 入口 | 迁移 |
|---|---|---|
| 滚轮/键盘滚动键上滚 | `scroll_by(dy<0)` | → Released |
| 触摸/拖滚动条/两指上滚或横移 | `pan_by(dx≠0 或 dy<0)` | → Released |
| 缩放 | `zoom_by` | → Released |
| 选区拖拽(非空) | `set_selection(非空)` | → Released(**新增信号**) |
| 显式 jump(Cmd+F 命中) | `scroll_to(view)` | → Released |

**重进入**(Released → Following)只在**用户主动向下**滚动抵底时发生:`scroll_by`/`pan_by`
的 `dy>0` 记一帧「向下意图」,`build_frame` 仅在该意图存在且 `pan ≥ 底 − 48px` 时吸附;
内容增长本身永不触发重进入(修缺陷 1)。

## 3. 备选与否决理由

- **沿用布尔 + 补丁式修被动复跟随**:信号集仍散落无契约,9 场景无法逐条对照测试;否决。
- **FSM 放 web 输入层(TS)**:违 CR1/0031(状态机进 core 保重放确定性);web 只是信号搬运工;否决。
- **引入 IntersectionObserver 式"贴底检测"**:DOM 思路,画布世界里 pan/底距离是一手数据,无需间接层;否决。

## 4. 影响

- 正:9 条 shadcn 场景可逐条测试化;选区拖拽期间不再被内容增长拽走;Released 语义可宣称
  "布局稳定";`scroll_to_latest` 给 jump-to-latest 按钮一个正路 API。
- 负:相机状态从 1 bit 变 3 态 + 1 意图位,`build_frame` 锚底段分支变宽;以 native 枚举
  遍历测试(迁移表与本 ADR 一致)兜住。
- 兼容:Following 态行为与旧实现逐字节等价(golden 回归);`stick_to_bottom` 字段被
  `FollowState` 取代,wasm API 面不变(无导出变化)。

## 5. 实现入口

`crates/core/src/app.rs`:`FollowState` 枚举 + `scroll_by/pan_by/zoom_by/scroll_to/set_selection/scroll_to_latest` 迁移 + `build_frame` 锚底段;测试同文件 `follow_fsm_*`。
