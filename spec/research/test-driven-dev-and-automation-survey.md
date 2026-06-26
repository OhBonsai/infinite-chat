# 测试驱动研发 + 自动化测试体系调研(适配 infinite-chat)

- 日期:2026-06-24
- 状态:研究 / 设计依据(供后续"测试体系" plan/ADR;行业综述 + 本项目落地建议)
- 范围:为一个 **Rust + WASM + WebGPU 流式渲染引擎 + 大量 AI agent 协作研发**的项目,建立**功能 + 性能**的自动化测试,且**测试执行与开发 agent session 解耦**、结果**结构化回喂 agent 判断**。
- 来源:5 路并行 web 调研(图形/WebGPU 视觉测试、确定性重放/仿真、性能回归/持续基准、agent-in-loop/LLM 裁决、Rust/wasm 工具链),一手来源为主,见文末 §12。可信度标注:多为官方文档/论文(高);软件光栅性能数字、厂商定价、MCP 测试回灌为二手(中),正文已标。

---

## 0. 结论速览(TL;DR)

1. **本项目已踩在行业最优实践的地基上**:`R8 确定性`(注入 `dt_ms`、record/replay)正是 FoundationDB/TigerBeetle 那套 **DST(确定性仿真测试)** 的前提;`CR1 平台无关 core` 正是 wgpu/业界推崇的 **"testable core / 平台边界下沉"**;`?verify 自绘几何` + Plan 18 `?bench` + `proptest` 已分别是视觉、性能、属性测试的雏形。**缺的不是地基,是把它们组织成一条"解耦 + 结构化 + agent 可裁决"的流水线。**
2. **分四层测**(金字塔,越下越快越多):L1 native 逻辑(`nextest`+`proptest`+`insta`)→ **L2 确定性重放回归(本项目最强、最该重投的一层)**→ L3 性能护栏(帧时分位 + 指令数代理 + **时间序列变更点检测**,不要 per-PR % 门)→ L4 视觉/感知(黄金样张**像素 exact**守几何不跳变 + **LLM pairwise 判图**守主观观感)。
3. **dev↔test 解耦的关键 = 数据契约**:测试在**独立进程/CI** 跑,产一份**机器可读 report(JUnit/JSON + 一份 agent 摘要 + SARIF 给 lint)**;dev agent **只读 report,不自评**;验收用 **FAIL_TO_PASS ∧ PASS_TO_PASS** 合同 + **独立 verification subagent** 判主观项。复用项目已有的 `DEVMEM.md` 文件 handoff 习惯,扩成 `test-report.*`。
4. **硬门走确定性**(确定性测试 + 像素 exact + 指令数),**软信号走 LLM**(感知质量、观感),**人最终拍板观感**。绝不让"写代码的 session"既当运动员又当裁判。

---

# 第一部分:行业实践综述

## 1. 图形 / 渲染 / WebGPU 的自动化测试

### 1.1 根本难点:同一 draw 跨机不同字节

"同一个绘制调用**不产生相同的字节**":GPU 厂商/驱动版本、OS 文字光栅器、亚像素定位、抗锯齿、色彩管理都会扰动输出。所以**单张 byte-exact 黄金图先天不稳**。两条逃生路各有代价:**fuzzy/感知容差**可能放过真 bug;**多张有效基线**(Skia Gold)避免误放但要 triage 一组组合爆炸的有效图。([Skia Gold](https://skia.org/docs/dev/testing/skiagold/)、[Chromium GPU Pixel Testing With Gold](https://chromium.googlesource.com/chromium/src.git/+/master/docs/gpu/gpu_pixel_testing_with_gold.md))

### 1.2 代表做法

- **Google Skia Gold**:黄金图比对做成**外部云服务**(`goldctl` 把图的哈希比对一组**已批准哈希**,未命中→上传→GUI 里人审标 positive/negative)。三大优势:triage 快(不走提交队列)、**一个测试可有多张有效图**(图形场景刚需)、图存 GCS 不撑爆 git。噪声测试可选 `matching_algorithm`(fuzzy + Sobel 边缘),并附 `determine_gold_inexact_parameters.py` 调参。([Skia Gold](https://skia.org/docs/dev/testing/skiagold/))
- **Chromium 像素 / Web 测试**:`-expected.png` 基线**按平台分目录**;reftest 用 `<meta name=fuzzy content="maxDifference=0-4; totalPixels=0-100">` 容差(限每通道色差 + 容许差异像素数);`rebaseline-cl`/`--reset-results` 重基线;**能用行为断言(testharness.js)就别用像素**。([Chromium Web Tests](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/web_tests.md)、[Chromium Chronicle #10](https://developer.chrome.com/blog/chromium-chronicle-10/))
- **WebGPU CTS(`gpuweb/cts`)**:**规范行为/API 一致性**套件(validation 规则、资源语义、数值正确性),**不是截图黄金套件**;仅在规范定死确切结果处做 texel readback。wgpu 用基于 Deno 的 `cts_runner` 跑;Dawn 经 `telemetry_gpu_integration_test` 跑。**对用 wgpu 的项目**:CTS 验"你的 WebGPU 层规范正确",你自己的视觉黄金测验"上层渲染正确"。([gpuweb/cts](https://github.com/gpuweb/cts))
- **软件 / headless GPU 适配器**:`SwiftShader`(Google,CPU Vulkan,Chrome CI 首选,**为可靠性**而非速度)、`Mesa lavapipe`(CPU Vulkan,常更快;Chrome 曾用 LLVMpipe 后因 flaky 换 SwiftShader——"软件=确定"**不自动成立,要钉死版本**)、`WARP`(D3D12 软光栅)、`Dawn`(本身不是软光栅,可挂上述软适配器)。**软适配器**:同版本内可复现、无需硬件、可进容器;**但慢、≠真机输出(软件上烘的黄金图在真机不匹配)、且光栅器自身版本漂移仍 flaky**。([lavapipe vs SwiftShader](https://airlied.blogspot.com/2021/03/sketchy-vulkan-benchmarks-lavapipe-vs.html) 二手性能;Chrome 选型为一手)

### 1.3 感知 diff 工具矩阵

| 工具 | 是什么 | 取舍 / 边界 |
|---|---|---|
| **pixelmatch** | JS 像素 diff,YIQ 色空间 + `threshold` + 抗锯齿检测 | 纯本地像素 diff,分不清有意义改动 vs AA/字体噪声,靠阈值补;免费可嵌 |
| **odiff** | 原生(OCaml)像素 diff,大图远快于 pixelmatch | 同样的像素 diff 局限,胜在速度 |
| **Playwright `toHaveScreenshot`** | 内置视觉断言(底层 pixelmatch),**重试到连续两张一致**抗 flake,基线按 `浏览器+平台` 命名 | 本地免费;基线**强平台相关**,必须在与 CI 同环境生成 |
| **Percy** | 抓 **DOM 快照**,云端真浏览器**重渲染**再 diff(比本地截图更确定) | SaaS、按快照计费;**只适用 DOM,不适用 canvas/WebGPU**(画布对 DOM 不透明) |
| **Applitools Eyes** | "Visual AI" 感知模型,比"像素的含义" | 企业定价;**不适合需要 byte-exact 图形验证**的场景 |
| **Chromatic** | Storybook 原生组件视觉测试,Git 感知基线 + TurboSnap | 组件级,非全画布;已用 Storybook 才划算 |

**三轴取舍**:像素 diff(便宜但 AA/字体噪声大,需容差)/ 感知 diff(容噪但黑箱,精确图形难推理)/ DOM 快照(集中重渲染确定,但**canvas/WebGPU 无效**)。**对 canvas/WebGPU:DOM 路全废,只能像素/texel readback**——这也是 CTS 尽量用行为断言、像素留给必须处的原因。

> **§1 给本项目的结论**:本项目是 canvas/WebGPU + **自带字体栈(单实现、不追兼容)+ 确定性渲染**,所以**比通用 web 视觉测试好做**——可以钉死单一环境(headless Chromium + 固定适配器 + 自带字体)追求**近 exact**,而不必陷入 Skia Gold 那种多机多有效图的 triage 地狱。详见 §7-L4。

## 2. 确定性重放 / 仿真测试 / 快照(本项目最强武器)

### 2.1 DST(Deterministic Simulation Testing)

把系统放进全仿真环境:墙钟、线程调度、RNG、网络/磁盘 IO **全部可插拔、由单一 seed 驱动**,整系统**单线程单进程**步进虚拟时钟 → 输出是 `(seed, 代码版本)` 的纯函数 → **任何 bug 可按 seed 逐字节重放**。常配属性测试 + 故障注入。([Antithesis DST primer](https://antithesis.com/docs/resources/deterministic_simulation_testing/))

代表:**FoundationDB**(Flow actor 语言,同源编译生产/仿真;模拟网络丢包、磁盘满、宕机重启、"swizzle-clogging";真:模拟时间 ≈ 10:1)、**TigerBeetle VOPR**(stub 掉 clock/network/disk,seed 调故障;**seed+commit 复现**;**生产期断言常开,宁停不错**;校验副本数据文件 byte 一致;1 分钟 ≈ 数天真实测试)、**Antithesis**(确定性 hypervisor 跑普通容器)。([FoundationDB testing](https://apple.github.io/foundationdb/testing.html)、[TigerBeetle VOPR](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md))

### 2.2 重放作 test oracle + 不变量检查

"**同输入(seed)→ 同输出**"本身就是 oracle。两层用法:(a)**差分/回归 oracle**:存的快照/重放历史就是期望,任何偏离即失败(Temporal 的"重放期 command 与 history 比对"即此);(b)**属性/不变量检查**:不固定黄金输出,而是断言跨状态空间必须成立的性质(FDB 的 cycle test 断言事务隔离不破环;TigerBeetle 校验副本字节一致;Antithesis 的 "Sometimes" 断言查可达性)。因确定性,违反即得可重放的反例 seed。([FoundationDB testing](https://apple.github.io/foundationdb/testing.html))

### 2.3 快照测试(insta)

`insta`(Rust):文件快照(`.snap`)或内联快照(`@"..."`);`cargo insta review` 逐个 accept/reject/skip;**redaction**(`{".id" => "[id]"}`、`sorted_redaction()`)稳住时间戳/随机 ID/map 序等易变字段。**适用**:解析/codegen/IR/序列化结构/渲染文本。**陷阱**:大快照被盲目 `-u` → bug 连同快照一起被"盖章";Jest 官方明言要"对抗失败就重生成快照的习惯"。**对策**:快照当代码审、保持小而聚焦、禁大快照、CI 不自动更新。([insta docs](https://insta.rs/docs/snapshot-types/)、[Jest Snapshot](https://jestjs.io/docs/snapshot-testing))

### 2.4 "可 DST/replay 测"的前提(本项目逐条已满足)

> 1. 不直接碰墙钟/RNG/调度/IO,全走可注入接口;2. 单 seed 驱动一切;3. 单线程事件驱动 core(状态机/step 函数);4. IO/外部依赖可 mock;5. 代码内丰富不变量/断言常开;6. 失败带 `seed+commit` 可重放;7. 不可避免的易变输出做 redaction。
>
> **infinite-chat 对照**:1 ✅(`seam` trait 注入 Clock/Connection/Layout/RenderSink,CR2)、2/3 ✅(`dt_ms` 注入逐帧累加、Engine 每帧 step、R8)、4 ✅(`CollectSink`/`NullSink`/native stub)、6 ✅(record/replay + `seek_reveal` 单帧步进)。**这意味着 DST 那套最难的前提本项目天生具备——这是本项目测试体系最大的杠杆点。**

## 3. 性能回归 / 持续基准

### 3.1 测量层:wall-time 统计 vs 指令数代理

- **criterion(Rust,wall-time 统计)**:warmup → 测量(线性回归取每次迭代耗时)→ bootstrap 置信区间(默认 0.95)→ 与存档基线做 **bootstrap T 检验**报 % 变化 + 显著性;Tukey 标记离群但不丢弃;噪声阈值(如 ±1%)内当噪声。**弱点(其自述)**:"极度敏感,后台进程都能误报"——**wall-time 方差是核心软肋,CI 尤甚**。([criterion analysis](https://bheisler.github.io/criterion.rs/book/analysis.html))
- **iai-callgrind / CodSpeed(指令数/周期,CI 稳)**:在 Valgrind 虚拟 CPU 上**只跑一次**,数指令/缓存/周期 → **完全确定、机器无关、CI 噪声免疫、能测更小变化**;代价:只**相关**于 wall-time(不直接测)、不含 syscall/IO、无 Windows(iai)。CodSpeed 宣称 <1% 方差、PR 集成,但其自述**工具链/runner 漂移(glibc 版本)仍可"无代码改动而回归"**。([criterion vs iai](https://bheisler.github.io/criterion.rs/book/iai/comparison.html)、[CodSpeed CPU](https://codspeed.io/docs/instruments/cpu/overview))

### 3.2 追踪/门控层:Bencher + 变更点检测

- **Bencher.dev**:不测量,**追踪 + 统计门控**——按 `Branch×Testbed×Measure` 存时间序列,Threshold = 统计 Test(static/percentage/z_score/t_test/log_normal/iqr…)+ 边界,越界报 Alert(`--error-on-alert` 失败构建)。**对噪声友好**:建每个 benchmark 的历史分布,按噪声选合适检验(t_test/log_normal/IQR),而非脆弱的固定 %。([Bencher Thresholds](https://bencher.dev/docs/explanation/thresholds/))
- **核心教训:别用 per-PR 固定 % 门**。MongoDB 公开复盘:10% 阈值"漏掉小回归 + 噪声测试大量误报 + 报错在错的 commit"。成熟做法是**把问题从"两次构建差 X%?"重构为"哪个 commit 真正改变了性能?"= 时间序列变更点检测**(E-Divisive,进 Evergreen CI;Datadog 开源 **Hunter** 同族;Chromium perf bots 同思路 + bisect 定位)。([MongoDB Change Point Detection](https://www.mongodb.com/company/blog/engineering/using-change-point-detection-find-performance-regressions))

### 3.3 headless 浏览器的 fps 不可信

headless 无真 vsync/合成器,Chrome **缺同步源时假设 60fps 走内部近似**,`rAF` 被钳到 ~60Hz。Chromium 自己的 smoothness 基准**不信单个 fps**:抓 trace 取逐帧时间戳,算 `frame_times`(原始分布)/`mean_frame_time`/**`jank`(时间戳不规则度,一次长停≈其毫秒数)**/`mostly_smooth`(95% 帧达 60fps);且测量须 `--disable-gpu-vsync` 否则被刷新率钳住。**实践:用帧时分位(P50/P95/P99)+ jank,长程采样,不用瞬时 fps。**([Chromium Rendering Benchmarks](https://www.chromium.org/developers/design-documents/rendering-benchmarks/))

> **§3 对本项目**:Plan 18 已经踩对了一半(帧时 `frameMsAvg`、长程采样、CSV 时间序列);但用了**单次跑 + 看 fps 绝对值**(headless 不可信)。升级方向见 §7-L3:**指令数代理给 CI 硬门 + 帧时 P95/jank + 变更点检测**。

## 4. Agent-in-the-loop / LLM 可消费的测试输出 + 解耦

### 4.1 结构化结果格式

- **JUnit XML**:事实标准,几乎所有 runner 可出、所有 CI 可读;携带 pass/fail/error/skip 计数 + 每测失败信息/栈。**agent 友好**:可过滤到"只看失败"。
- **per-test JSON**(`cargo test --message-format=json` 不稳;**nextest `libtest-json-plus`** 带额外 metadata,版本化):agent 想程序化过滤失败时最佳。
- **SARIF(OASIS 标准)**:**静态分析**结果的 JSON 交换格式,GitHub code scanning 直接渲染成 alert;给 agent **file+line+rule+severity**。**边界:SARIF 是给 lint/SAST,不是测试 pass/fail**——测试结果用 JUnit/JSON,lint/clippy 用 SARIF。([SARIF v2.1.0](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)、[GitHub SARIF](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning))

### 4.2 Claude Code 的 verifier loop(一手指引)

Anthropic《Best practices for Claude Code》明确:**"给 Claude 一个能验证自己工作的东西"——产生 pass/fail 的东西,循环就自闭合**(测试套件、构建退出码、linter、与 fixture 比对的脚本、与设计比对的截图)。配套:**TDD**(先写复现失败的测试再修)、**"给证据不给断言"**(贴测试输出/命令返回/截图)、**concise 失败摘要而非原始日志**(context 填满则性能下降)。门控强度递增:in-prompt → `/goal` 每轮复检 → **Stop hook 脚本硬门(不过不放行)** → **verification subagent(新模型试图反驳,写代码的不当裁判)**。([Best practices for Claude Code](https://code.claude.com/docs/en/best-practices))

### 4.3 LLM-as-judge

奠基论文《Judging LLM-as-a-Judge》(MT-Bench,arXiv 2306.05685):强判官(GPT-4)与人类一致率 **>80%(等于人类间一致率)**;但有**位置偏置、冗长偏置、自我偏好偏置**。判分模式:pointwise(单测打分)/ **pairwise(成对偏好,通常更贴人类)**/ reference-guided。**多模态判图**(arXiv 2402.04788):MLLM 在**成对比较像人,但绝对打分/批量排序显著偏离人类**——**判截图/观感要用 pairwise,别用绝对分**。框架:promptfoo / DeepEval(G-Eval)/ OpenAI Evals。**边界:LLM 判官只用于精确匹配测不了的主观项(风格、观感),是补充非替代,且必须去偏(换位、控长度、不自评)。**([2306.05685](https://arxiv.org/abs/2306.05685)、[2402.04788](https://arxiv.org/abs/2402.04788))

### 4.4 SWE-bench 合同(解耦 + 可复现的"测试即裁判")

SWE-bench(arXiv 2310.06770)用**仓库自身测试套件在隔离容器里**判 agent 补丁。核心规则可直接借:**FAIL_TO_PASS**(修前失败的测试现在必须过,证 bug 已修)∧ **PASS_TO_PASS**(原本绿的必须仍绿,证无回归)——**防止删测试/禁套件来骗过**。现代 harness:每任务一个 Docker 容器,**把候选当 git diff 应用**再跑套件;judge 是容器外算的二元值,**agent 从不自评**。([SWE-bench](https://arxiv.org/abs/2310.06770)、[SWE-bench Verified](https://openai.com/index/introducing-swe-bench-verified/))

### 4.5 解耦执行(本诉求 4/5 的直接答案)

Claude Code 文档支持多种解耦:**沙箱/独立进程 runner**;**CI 作 verifier**(`claude -p "…" --output-format json` 非交互入 CI,结果可程序化解析);**artifact handoff**(验证信号 = 与 fixture 比对的脚本/结果文件,agent 读回而非肉眼看日志);**独立 verification subagent**(新 context 只看 diff + 标准);**Stop hook 确定性门**。**MCP**(JSON-RPC,server 暴露 tools/resources)是把测试/CI 结果**结构化回灌**给 agent 的干净通道(测试结果 schema 为社区实践,非 spec 强制——中可信度)。([Best practices](https://code.claude.com/docs/en/best-practices)、[MCP spec](https://modelcontextprotocol.io/specification/2025-11-25))

## 5. Rust + WASM + WebGPU 工具链

| 工具 | 是什么 / 怎么用 | 边界 |
|---|---|---|
| **cargo-nextest** | 下一代 runner:**进程隔离(每测一进程)**、per-test 超时/取消、**retry + flaky 检测**、**JUnit XML**、`--partition` 分片。**wgpu 强制用它**(图形上下文每进程只能一线程建) | 不跑 wasm-in-browser;极短测试有建进程开销 | ([why process-per-test](https://nexte.st/docs/design/why-process-per-test/)) |
| **proptest** | 属性测试:Strategy 生成 + **自动 shrink 到最小反例** + **失败 seed 入 `proptest-regressions/`(应提交)**;策略须确定性 | 纯逻辑/平台无关 core 是甜区 | ([Failure Persistence](https://altsysrq.github.io/proptest-book/proptest/failure-persistence.html)) |
| **wasm-bindgen-test** | `#[wasm_bindgen_test]` 跑 wasm 测试于 Node 或 **headless 浏览器**(`wasm-pack test --headless --chrome`);`wasm_bindgen_test_configure!(run_in_browser)` 锁浏览器;支持 async/DOM/WebGPU | 慢(浏览器+WebDriver),**只给真需 JS/web/wasm 运行时的代码** | ([wasm-bindgen test](https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/usage.html)) |
| **trybuild** | 编译失败 UI 测试(断言"误用不编译" + 比对 stderr);**wgpu 用它**验生命周期误用不编译 | 主要 proc-macro / API 误用守护 | ([trybuild](https://docs.rs/trybuild/latest/trybuild/)) |
| **insta** | 快照测试;**naga 用它**做 WGSL 解析/codegen 的数据驱动快照("WGSL 进 → 各后端出") | 大快照盖章风险,需 review | ([insta](https://insta.rs/docs/cli/)) |
| **wgpu 自身测法** | `#[gpu_test]` 自定义 harness(跑系统所有适配器 + **per-adapter 预期失败**表)+ **`noop` 后端**(不连真 GPU 的 validation/trace 测,快)+ CTS via Deno;**"不需要真 GPU 的就别写成 GPU 测"** | 真 GPU 测慢、需硬件 | ([wgpu testing.md](https://github.com/gfx-rs/wgpu/blob/trunk/docs/testing.md)) |

**分层铁律(wgpu/业界共识)**:**把逻辑推进平台无关 core crate(无 wasm-bindgen/web-sys/wgpu)→ 跑 native `cargo nextest`(+proptest+insta),毫秒级**;**仅不可约的平台边界下沉 headless 浏览器,仅不可约的硬件行为下沉 GPU harness**。一句话:*在干净 core 上用 native nextest 测最大面,只把无法避免的边界降级到 wasm/browser、把无法避免的硬件行为降级到 GPU 测。*

---

# 第二部分:适配 infinite-chat

## 6. 现状盘点:杠杆点已就位

| 行业最优前提 | infinite-chat 现状 | 杠杆 |
|---|---|---|
| DST/replay 可测(无墙钟/单 seed/单线程/可注入) | ✅ R8 + `dt_ms` + `seam` trait + record/replay + `seek_reveal` 单帧步进 | **最大**:几乎白送一套 DST |
| testable core(平台无关) | ✅ CR1:`crates/core` 零 wasm/web-sys/wgpu | 大:native 测最大面 |
| 视觉测试钩子 | 🟡 `?verify` 自绘几何(块 AABB/baseline),黄金样张未落地(TODO V) | 中:差临门一脚 |
| 性能基准 | 🟡 Plan 18 `?bench`+Playwright+CSV+native `bench_scale_before`+`scale_stats_grow_with_history` 回归 | 中:差"可信度 + 时间序列门" |
| 属性测试 | 🟡 `proptest` 已在 core dev-deps,用得少 | 中 |
| 构建期校验 | ✅ naga WGSL 校验(render dev-dep) | — |
| agent 协作 + 文件 handoff | ✅ 文档驱动 + `DEVMEM.md` scratchpad + skills(`/test-write` `/test-run` `/replay-debug` `/dev-diagnose`) | 大:解耦回路的载体已有 |
| 观感取向 | ✅ opinionated 单实现(中英+markdown 一条路径) | **利好视觉测试**:可钉死单环境追近 exact |

**判断:不是从零建,是把散件组织成"解耦 + 结构化 + agent 可裁决"的一条流水线。**

## 7. 分层测试策略(金字塔 · 标 where-it-runs + 对应北极星断言)

### L1 — core 逻辑(native `nextest` + `proptest` + `insta`,毫秒级,占比最大)

- 跑哪:`crates/core`,`cargo nextest`(换掉裸 `cargo test`,白得进程隔离 + JUnit + flaky retry + 分片)。
- 测什么(**功能**):`store` 对账幂等/乱序归位(proptest:任意 delta/updated 序列 → 同末状态)、`reveal` gate **单调只增**(proptest)、`boxlayout` 前缀和等价(已有思路)、**0029 虚拟化 release↔rebuild 往返逐字节等价**(proptest,R8 命脉)、`protocol` 未知类型→`Ignored` 向前兼容、`smoother` grapheme 不切碎。
- `insta` 快照:markdown→`StyledSpan`/`NodeTree`、math `MathLayout`、`FrameData` 结构(redact 掉 spawn_time 等时变)。**保持小、review、CI 不自动更新。**

### L2 — 确定性重放回归(**本项目最强、最该重投的一层**)

- 跑哪:native(core 重放,无 GPU)为主,关键路径再上 headless。
- 机制:**录像(`Recorder`)→ 重放到固定 `target_ms`(`seek_reveal`)→ 把"末状态 + 关键帧 `FrameData`(几何/spawn/节点树)"序列化 → `insta` 快照 diff**。这就是 §2.2 的"重放作 oracle"。
- 北极星断言(借 FDB/TigerBeetle 的**属性检查**,不止快照):
  - **流式不闪**:同一 case 任意 `target_ms`,结构块(表格/代码/列表)在 raw 抑制门前不出现 raw glyph(reveal gate 不变量)。
  - **settled 冻结**:settled 块跨帧 `FrameData` 几何恒定(0025 §4)。
  - **重放确定性**:同 case 同 seed → 同末状态(差分 oracle)。
  - **0029 后**:滚动来回 `retained_glyphs` 回落 + 块 y 不变(release 安全)。
- 价值:**绝大多数"丑/闪/跳变/卡死"类回归都能在 native 毫秒级抓到,无需 GPU/浏览器**——这是本项目区别于普通 web 渲染测试的护城河。

### L3 — 性能护栏(升级 Plan 18)

- **CI 硬门走指令数代理**:把 `bench_scale_before` 的核心(纯 CPU retention/排版热路径)接 **iai-callgrind**,指令数确定、CI 噪声免疫、能抓 sub-% 回归(§3.1)。`build_frame`/`ensure_layouts`/`sizes` 各立一个 iai 基准 → 直接守 0029/Plan 19 的"O(历史)→O(views)"。
- **帧时走分位 + jank,不用 fps**:`?bench` 改报 **P50/P95/P99 帧时 + jank**,`--disable-gpu-vsync`,**N≥5 次取中位**(§3.3 修 Plan 18 单次/fps 短板)。
- **门控走时间序列变更点,不用 per-PR %**:把每次 bench 入 **Bencher**(或自建 CSV 时间序列 + Hunter 式 E-Divisive),**检测 step 回归并定位 commit**,而非 per-PR 固定阈值(§3.2)。before 基线 Plan 18 已有。
- 真机:headless 仅作 CI 趋势门;关键点真机复测定绝对值。

### L4 — 视觉 / 感知(分两半:exact 硬门 + LLM 软信号)

- **几何 exact(确定性硬门)**:本项目渲染确定 + 单环境可钉死 → `?verify` 自绘几何 + 截图,**像素 diff 守"几何/baseline/不跳变"**。因确定性,可追求**近 exact**(钉死 headless Chromium + 固定适配器 + 自带字体),用 Playwright `toHaveScreenshot` 紧容差,**而非** Skia Gold 那种多有效图 triage(§1.2 我们环境单一,不必)。
- **观感感知(LLM 软信号,契合 opinionated 主观审美)**:截图交 **LLM pairwise 判图**(§4.3:多模态判官**成对可靠、绝对分不可靠**)——"新版 vs 黄金版,哪个更符合作者认可的观感?"。**只作软信号,人最终拍板**;去偏(换位、不自评)。这正好承接 TODO V "作者认可的观感不回退"。

> **金字塔比例**:L1+L2 占绝大多数(native 毫秒、确定、可属性化)→ L3 性能少量但关键 → L4 视觉最少最慢(浏览器/截图/LLM),只守 L1/L2 测不到的像素与观感。

## 8. dev↔test 数据契约(核心诉求 4/5 的设计)

### 8.1 解耦形态

```
开发 Claude Code session(只写码 + 读 report)
        │ 产出 diff
        ▼
独立 verify runner(进程/CI,不在 dev session)
   just verify  →  跑 L1 nextest + L2 replay + L3 bench + L4 视觉
        │ 产出 report artifact(机器可读)
        ▼
report/  ──读回──▶ dev session 裁决(失败摘要回灌,迭代)
        └──────▶ verification subagent(判主观/观感,独立 context)
```

要点:**测试在独立进程/CI 跑,dev agent 只消费 report,绝不自评**(§4.2/4.4)。复用项目已有的 `DEVMEM.md` 文件 handoff 直觉,扩成结构化 report。

### 8.2 report 契约(每类测试 → 机器可读 + agent 摘要)

`report/`(git-ignored 临时 + CI artifact):

| 文件 | 内容 | 格式 | 给谁 |
|---|---|---|---|
| `junit.xml` | L1/L2 pass/fail + 失败名/断言/file:line | JUnit XML(nextest 出) | CI + agent |
| `nextest.json` | per-test 结构化(过滤失败用) | nextest `libtest-json-plus` | agent |
| `perf.json` | iai 指令数 + 帧时 P50/P95/P99 + jank + retained_* + Δvs基线 + 变更点判定 | 自定义 JSON | agent + Bencher |
| `clippy.sarif` | lint/clippy 发现 file+line+rule | SARIF | GitHub + agent |
| `visual/` | `?verify` 截图 + 像素 diff + LLM pairwise 裁决 | png + JSON | verification subagent + 人 |
| **`TESTREPORT.md`** | **一份 agent 摘要**:总判定 + 失败 N 条(名+断言+file:line)+ 性能 Δ + 视觉判定 + seed/commit | Markdown | **dev agent 首读**(concise,不灌原始日志) |

**关键:`TESTREPORT.md` 是 §4.2 "concise 摘要而非原始日志"的落点**——把上面所有结构化结果**蒸馏成几十行**喂回 dev agent。

### 8.3 验收合同(借 SWE-bench)

- **FAIL_TO_PASS ∧ PASS_TO_PASS**:每个 plan 的 DoD 测试必须**红→绿**(证目标达成),且既有套件**保持绿**(证无回归)。**防 agent 删测试/放宽断言骗过**(§4.4)。本项目已有的 `scale_stats_grow_with_history` 等回归断言就是 PASS_TO_PASS 集。
- **seed+commit 入 report**:任何失败可按 §2 重放复现(record/replay + `seek_reveal`)。

## 9. 自动化与 agent 裁决回路(具体形态)

1. **一条命令出全量 report**:`just verify`(或 `cargo xtask verify`)→ 跑四层 → 写 `report/`。本地与 CI **同一命令**(避免"CI 才暴露")。
2. **硬门(确定性,机器判)**:L1/L2 必须全绿(确定性,零容忍)、L3 指令数门 + 帧时变更点不回归、L4 几何 exact 不变 → 任一红则 `--error-on-alert` 失败,`TESTREPORT.md` 标红 + 摘要。
3. **软信号(主观,LLM 判 + 人拍板)**:L4 观感 LLM pairwise → 写 `visual/verdict.json`,**只警示不阻断**,人 review。
4. **回灌**:dev agent 读 `TESTREPORT.md` → 失败则迭代;**用 Stop hook 把 `just verify` 设成确定性门**(不过不放行),或非交互 `claude -p --output-format json` 在 CI 跑(§4.5)。
5. **裁判隔离**:主观项交**独立 verification subagent**(新 context、只看 diff + report + 标准),不让写码 session 自判(§4.2/4.4)。
6. (可选)**MCP / CI artifact** 作 report 回灌通道,替代贴日志。

## 10. 落地路线(分阶段,建议接 plan 编号)

1. **P1 地基(最高性价比)**:`cargo test`→`nextest`(白得 JUnit/flaky/隔离);`just verify` 骨架 + `TESTREPORT.md` 摘要生成器;把现有 native 测 + `scale` 回归纳入。**1 个 PR,立刻有"一条命令 + 一份 agent 可读 report"。**
2. **P2 确定性重放层(护城河)**:录像→`seek_reveal`→`FrameData`/末状态 `insta` 快照 + reveal/settled **属性断言**(proptest)。补几条北极星 case(表格不闪 raw、settled 冻结、0029 滚动回落)。
3. **P3 性能可信化**:`iai-callgrind` 接 `build_frame`/`sizes` 等热路径(CI 硬门);`?bench` 改帧时 P95/jank + N 次中位;接 Bencher/变更点检测;补 §7-L3。
4. **P4 视觉/感知**:落地黄金样张(`?verify` 截图 + Playwright `toHaveScreenshot` 紧容差,exact 硬门);LLM pairwise 判图软信号 + 人审。
5. **P5 解耦闭环**:Stop hook / CI 非交互 + verification subagent 判主观 + (可选)MCP 回灌;FAIL_TO_PASS∧PASS_TO_PASS 写进每个 plan 的 DoD。

## 11. 取舍与不做(明确边界)

- **不追跨平台/跨浏览器像素一致**(与 opinionated 单实现、BiDi 非目标同源):钉死单一 headless 环境追近 exact,**不进 Skia Gold 多有效图 triage**。
- **软件适配器要不要**:为 CI 复现性可固定一个(SwiftShader/lavapipe),但**必须钉死版本**(否则仍 flaky),且明确"软件烘的图≠真机";v1 也可先用钉死的 headless Chromium WebGPU,不引软适配器。
- **LLM 判图只作软信号**,绝不当硬门(绝对分不可靠;成对也只辅助);**人对观感有最终否决权**。
- **性能硬门用指令数代理,不用 headless fps 绝对值**;wall-time 帧时只作趋势 + 真机复测。
- **快照不滥用**:大 `FrameData` 快照保持聚焦 + review + CI 不自动更新,防"bug 连快照一起盖章"。
- **CTS 暂不自建**:用 wgpu 即继承其 CTS;我们只测上层渲染,不测 WebGPU 规范一致性。

---

## 12. 参考(Sources)

**图形/WebGPU 视觉**:[Skia Gold](https://skia.org/docs/dev/testing/skiagold/) · [Chromium GPU Pixel Testing With Gold](https://chromium.googlesource.com/chromium/src.git/+/master/docs/gpu/gpu_pixel_testing_with_gold.md) · [Chromium Web Tests](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/web_tests.md) · [Chromium Chronicle #10 (Pixel Tests)](https://developer.chrome.com/blog/chromium-chronicle-10/) · [gpuweb/cts](https://github.com/gpuweb/cts) · [WebGPU CTS runner](https://gpuweb.github.io/cts/) · [Dawn webgpu-cts](https://dawn.googlesource.com/dawn/+/HEAD/webgpu-cts/) · [Playwright Visual comparisons](https://playwright.dev/docs/test-snapshots) · [pixelmatch](https://github.com/mapbox/pixelmatch) · [lavapipe vs swiftshader (二手)](https://airlied.blogspot.com/2021/03/sketchy-vulkan-benchmarks-lavapipe-vs.html) · [Percy/Applitools/Chromatic 对比 (二手)](https://crosscheck.cloud/blogs/percy-vs-applitools-vs-chromatic-visual-regression-testing/)

**确定性重放/仿真/快照**:[FoundationDB Simulation and Testing](https://apple.github.io/foundationdb/testing.html) · [Will Wilson, Testing Distributed Systems w/ DST (Strange Loop 2014)](https://www.youtube.com/watch?v=4fFDFbi3toc) · [Antithesis DST primer](https://antithesis.com/docs/resources/deterministic_simulation_testing/) · [TigerBeetle VOPR](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md) · [TigerBeetle: Simulation Testing for Liveness](https://tigerbeetle.com/blog/2023-07-06-simulation-testing-for-liveness/) · [insta Snapshot Types](https://insta.rs/docs/snapshot-types/) · [insta Redactions](https://insta.rs/docs/redactions/) · [Jest Snapshot Testing](https://jestjs.io/docs/snapshot-testing) · [Temporal Event History](https://docs.temporal.io/workflow-execution/event)

**性能回归/持续基准**:[Criterion.rs Analysis](https://bheisler.github.io/criterion.rs/book/analysis.html) · [Criterion vs Iai](https://bheisler.github.io/criterion.rs/book/iai/comparison.html) · [iai-callgrind](https://github.com/iai-callgrind/iai-callgrind) · [Bencher Thresholds](https://bencher.dev/docs/explanation/thresholds/) · [CodSpeed CPU Instrument](https://codspeed.io/docs/instruments/cpu/overview) · [Catch2 benchmarks](https://github.com/catchorg/Catch2/blob/devel/docs/benchmarks.md) · [nanobench](https://github.com/martinus/nanobench) · [MongoDB Change Point Detection](https://www.mongodb.com/company/blog/engineering/using-change-point-detection-find-performance-regressions) · [ICPE 2020 paper](https://dl.acm.org/doi/abs/10.1145/3358960.3375791) · [Chromium Rendering Benchmarks](https://www.chromium.org/developers/design-documents/rendering-benchmarks/)

**Agent-in-loop / LLM 裁决 / 解耦**:[Best practices for Claude Code](https://code.claude.com/docs/en/best-practices) · [How Anthropic teams use Claude Code](https://claude.com/blog/how-anthropic-teams-use-claude-code) · [Judging LLM-as-a-Judge (2306.05685)](https://arxiv.org/abs/2306.05685) · [MLLM-as-a-Judge (2402.04788)](https://arxiv.org/abs/2402.04788) · [SWE-bench (2310.06770)](https://arxiv.org/abs/2310.06770) · [SWE-bench Verified (OpenAI)](https://openai.com/index/introducing-swe-bench-verified/) · [SARIF v2.1.0 (OASIS)](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html) · [GitHub SARIF support](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning) · [nextest machine-readable](https://nexte.st/docs/machine-readable/) · [promptfoo LLM-as-judge](https://www.promptfoo.dev/docs/guides/llm-as-a-judge/) · [DeepEval G-Eval](https://deepeval.com/docs/metrics-llm-evals) · [MCP spec](https://modelcontextprotocol.io/specification/2025-11-25)

**Rust/WASM/WebGPU 工具链**:[nextest why-process-per-test](https://nexte.st/docs/design/why-process-per-test/) · [nextest retries/flaky](https://nexte.st/docs/features/retries/) · [nextest JUnit](https://nexte.st/docs/machine-readable/junit/) · [nextest partitioning](https://nexte.st/docs/ci-features/partitioning/) · [proptest Failure Persistence](https://altsysrq.github.io/proptest-book/proptest/failure-persistence.html) · [proptest vs quickcheck](https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html) · [wasm-bindgen-test usage](https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/usage.html) · [wasm-bindgen-test headless browsers](https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/browsers.html) · [trybuild](https://docs.rs/trybuild/latest/trybuild/) · [insta CLI](https://insta.rs/docs/cli/) · [wgpu testing.md](https://github.com/gfx-rs/wgpu/blob/trunk/docs/testing.md) · [wgpu README (backend env)](https://github.com/gfx-rs/wgpu/blob/trunk/README.md)

> 可信度:奠基论文(2306.05685/2402.04788/2310.06770)、OASIS/GitHub SARIF、Anthropic Claude Code 文档、各官方工具文档(nextest/proptest/insta/criterion/bencher/codspeed/wgpu/chromium/skia/playwright)、FDB/TigerBeetle/Antithesis 一手 = 高。软件光栅性能数字、视觉 SaaS 定价、"测试结果走 MCP" = 二手/社区,正文已标。
