# 调研:代码块语法高亮(warp / jcode)+ 本项目选型与集成

- 日期:2026-06-19
- 目的:为本项目 **H5 代码块语法高亮**(TODO §Plan2 欠账 / [plan2 §H5](../plan/plan2-usable-chat.md) / [plan5_progress 非表格完备])定**引擎选型 + 集成路径**。现状:`StyleRole::CodeBlock` 单一色 + 背景 rect,**无逐 token 高亮**;`vendor/jcode-render-core` 只给结构(`BlockKind::CodeBlock{language}`,围栏语言已捕获),不高亮。
- 方法:实读 `~/w/agentscode/warp`(tree-sitter)与 `~/w/agentscode/jcode`(syntect)源码,带 file:line。
- 关联:[0010 markdown 解析](../decision/0010-markdown-parsing-strategy.md)(pulldown-cmark)、[0021 样式数据驱动/调色板](../decision/0021-js-rust-boundary-and-configurable-render.md)、[0011 逐字 SDF / per-glyph 色](../decision/0011-gpu-text-as-sdf-primitive.md)、[0018 面板装饰(代码块底)](../decision/0018-sdf-panel-decoration-primitive.md)、[0017 流式提交前沿](../decision/0017-markdown-streaming-landing.md)。

---

## 0. 一句话结论

- **warp = tree-sitter**(`arborium`),AST 精准 + 增量编辑;但**每语言一份 C 语法**(28+),wasm/包体故事差——warp 是桌面端,无 wasm 约束才敢这么选。
- **jcode = syntect + `regex-fancy`**(纯 Rust、wasm 安全、bundled 二进制语法/主题免 yaml),`highlight_code_cached(code,lang)→逐 token 色`,LRU 缓存 `(code,lang)` hash,**每次流式更新整块重高亮**(试过拼接增量、因正确性回退)。
- **本项目选 syntect + fancy-regex**(= 与项目自己的 [plan2 §H5] 既定一致,jcode 证明 wasm 安全,纯 Rust 无 C 依赖、守"轻包体" [0000]);**tree-sitter 否决**(C 语法 × wasm 包体)。
- **集成关键(借 warp 的"8 色"洞察 + 本项目 0021 角色色模型)**:syntect 的 scope **塌缩成 ~8 个代码语义角色**(keyword/string/comment/function/type/number/...)→ **新增 `StyleRole::Code*`** → 走现有 **per-glyph role→色(0021 调色板,亮暗免费)**。**不引入任意 RGB 逐字通道**(守 opinionated 单实现)。
- **流式**:只重高亮**活动(未 settle)块** + settled 冻结 + `(hash,lang)` 缓存 → 量级有界,天然贴本项目 settle 模型。

---

## 1. warp(tree-sitter / arborium · 桌面端)

- **引擎**:`arborium`(tree-sitter 封装 + bundled 高亮 query),Cargo `arborium.workspace`(`warp/Cargo.toml:268`),28 语言;**不写自定义 `highlights.scm`,用 arborium 内置 query**(`crates/languages/grammars/README.md:12`)。
- **输出**:`HighlightQuery::get_highlighted_chunks()`(`crates/syntax_tree/src/queries/highlight_query.rs:42`)→ `RangeMap<CharOffset, ColorU>`;query capture 名(`keyword.control`/`string.quoted`…)→ 色,经 **8 色 `ColorMap`**(`highlight_query.rs:82`:keyword/function/string/type/number/comment/property/tag)。
- **流式**:**非逐行**——tree-sitter 增量编辑(`InputEdit` → `tree.edit`)+ **异步重解析**(`syntax_tree/src/lib.rs:352`,旧树先 edit 防闪)+ **单条目按 (version,range,lang) 缓存**(`lib.rs:50`);`MAX_SYNTAX_TREES=3`。
- **语言判定**:围栏别名归一(`languages/src/lib.rs:88` go→golang/py→python)+ 扩展名;无内容嗅探。**语法 bundled**(`RustEmbed` 编译期内嵌)+ **按语言懒加载** Arc 缓存。
- **主题**:8 色 `ColorMap` 由 app 层注入,`set_color_map` 切换即清缓存(亮暗在 app 层)。
- **chrome**:半透明底 `CODE_BLOCK_BACKGROUND=0x00000055` + **10px 圆角** + 整行宽 + 4 空格 tab(`formatted_text_element.rs`);**无行号 / 无语言标签渲染 / 无复制按钮**(语言串解析了但没画)。
- **wasm/perf**:桌面端,无 wasm 约束;regex 用 tree-sitter 自带,无 onig/fancy 之辩。

## 2. jcode(syntect + regex-fancy · TUI)

- **引擎**:`syntect` v5,`default-features=false, features=["default-syntaxes","default-themes","regex-fancy"]`(`jcode-tui-markdown/Cargo.toml:15`)——**fancy-regex(非 onig)**避 C FFI、wasm 友好;**bundled 二进制语法/主题**避 yaml-rust RUSTSEC。
- **输出**:`highlight_code_cached(code,lang)`(`markdown_render_support.rs:253`)→ 查缓存 → `highlight_code()`(:281)逐行 `HighlightLines::highlight_line()` → syntect `(Style,&str)` 区间 → ratatui `Span{fg:rgb}`(`syntect_to_ratatui_style` :319,**只取前景色**)→ `Vec<Line>`。**高亮在上层 TUI crate,render-core 不依赖 syntect**(后端中立)。
- **流式**:**整块重 render 每次更新**(`markdown_incremental.rs:36`);注释明言曾试"从 checkpoint 拼接增量",因 markdown 块分隔/列表连续性导致重影 → **回退为整体重渲,重正确性**。但 `HighlightLines` 逐行**带状态**(行间复用解析态)。
- **缓存**:`(code,lang)` DefaultHasher,**LRU max 256**(`lib.rs:141`),命中/未命中入 debug stats。
- **语言判定**:围栏串 `find_syntax_by_token(lang)`,缺省 plain text(`markdown_render_support.rs:286`);文件路径走扩展名。语法**全量启动加载**(`SyntaxSet::load_defaults_newlines`,`LazyLock`)。
- **主题**:**硬编码 `base16-ocean.dark`**(:289);`rgb()` 按终端能力 → truecolor 或 xterm-256(`color_support.rs:73`)。
- **chrome**:盒框 `┌─ <lang> │ code └─`(`markdown_render_full.rs:468`,dim 灰);**复制目标可抽取**(`extract_copy_targets_from_rendered_lines`,`CopyTargetKind::CodeBlock{language}`);**无行号**;**代码块不折行**,长行交 TUI 视口横滚。
- **wasm/perf**:fancy-regex 纯 Rust wasm 兼容;LazyLock 语法 + (hash,lang) LRU;syntect→bincode 有 RUSTSEC-2025-0141(未维护传递依赖)需盯上游。

## 3. 对比 + 本项目选型

| 维度 | warp:tree-sitter | jcode:syntect+fancy | 本项目取舍 |
|---|---|---|---|
| 精度 | AST(高) | TextMate 正则(够用) | syntect 够用(聊天代码块非 IDE) |
| **wasm / 包体** | C 语法 ×N,wasm 难、包大 | **纯 Rust、bundled、wasm 证明可行** | **syntect 胜**(守 0000 轻包体) |
| 增量/流式 | 增量编辑(优) | 整块重高亮(够,逐行带状态) | 只重高亮活动块即可(§5) |
| 语言覆盖 | 28(按需) | syntect 默认全集 | syntect 默认够,可裁剪 |
| 主题/色 | 8 色 ColorMap | tmTheme | **8 色塌缩 → 0021 角色**(§4) |
| 既定 | — | = [plan2 §H5] 选型 | **沿用,不翻案** |

**结论**:`syntect`(`default-syntaxes`/`default-themes`/`regex-fancy`,关 yaml)。**tree-sitter 否决**理由 = wasm + 包体 + C 依赖,与护城河([0011] 纯 Rust/[0000] 轻包)冲突;其增量优势本项目用"只重高亮活动块"即可平替。

## 4. 集成本项目架构(关键)

本项目的色是 **role 驱动**:`StyledSpan{role, fill}` → 逐 glyph `style:u32` → [0021] 调色板取色(亮暗免费)。syntect 给的是**任意 tmTheme RGB**,两条路:

| 路 | 做法 | 利 | 弊 |
|---|---|---|---|
| **A 角色塌缩(选)** | syntect scope **塌缩成 ~8 个 `StyleRole::Code*`**(借 warp 8 色:`CodeKeyword/CodeString/CodeComment/CodeFunc/CodeType/CodeNumber/CodePunct/CodePlain`)→ 走现有 role→色 | 守 0021 数据驱动 + 亮暗免费 + 包体小 + opinionated 单实现一致 | 损失 tmTheme 精细分色(可接受) |
| B 任意 RGB 逐字 | 给 glyph 加任意 RGB fill 通道,直传 tmTheme 色 | 完全保真 | 新 per-glyph 色通道、亮暗要自管、与 0021 角色模型割裂 |

**选 A**。落点:
- **新模块** `crates/core/src/highlight.rs`:`fn highlight(code:&str, lang:Option<&str>) -> Vec<(区间, CodeRole)>`,内部 syntect `HighlightLines` 逐行 → scope 映射函数(syntect `Scope` / `StyleModifier` → 8 个 `CodeRole`)。
- **StyleRole 追加** `Code*`(数值尾部追加,不移位,守 0001/0026 数值稳定先例,同 Plan 11/12 加角色)。
- **content.rs**:`BlockKind::CodeBlock{language}` 的代码行,span 角色由 `highlight()` 赋 `Code*`(替现单一 `CodeBlock`);未识别语言/纯文本 → `CodePlain`(= 现 CodeBlock 观感)。
- **0021 调色板**:加 8 个 code role 的色(亮暗两套),style 面板可调(同 TableStyle 实时 setter)。GitHub `.pl-*` 调色板可抄(TODO H5 原注)。
- **render 零改**:逐 glyph 仍 role→色,8 个新 role 自动生效。

## 5. 流式高亮(贴 settle 模型)

- **只重高亮活动块**:代码块在活动尾部 append-only([0017] 提交前沿);**只对未 settle 的代码块重跑 `highlight()`**,settled 块冻结(同 0025 §4 / 块冻结)→ 量级有界(1 块)。jcode "整块重渲" 的代价在本项目被"仅活动块"进一步收窄。
- **逐行带状态**:syntect `HighlightLines` 行间复用解析态;活动块重高亮 = 从块首逐行(块通常短),或缓存到上一完整行的 `ParseState` 续算(进阶)。
- **多行构造回退色**:未闭合字符串/注释跨行,后续行到达会改前面色——本项目代码块 reveal 本就逐行揭示,重高亮活动块自然吸收(不像 jcode 跨块拼接的坑)。
- **缓存**:`(code_hash, lang)` → `Vec<(区间,CodeRole)>`,块 settle 后冻结复用(= plan2 §H5 + jcode LRU)。
- **与 reveal(0019)正交**:高亮赋的是 role(色),reveal 定 `spawn_time`(何时上屏);两者不冲突——color 在 layout 期定,reveal 只控揭示节奏。

## 6. 代码块 chrome

- **背景**:已有 rect(`app.rs::code_block_emits_background_rect`)→ 迁 [0018] 面板(圆角 + AO,同表格),warp 同款(半透明 + 圆角)。
- **语言标签**:块顶左 dim 小字(jcode `┌─ lang`;warp 解析未画)——本项目可作 `Dim` span 或面板内饰,**易做、建议 v1 带**。
- **复制按钮**:需交互/hit-test 层(TODO Q),**defer**;但可先按 jcode 思路保留"代码块 = 可抽取文本块"的结构(0020 节点 range 已能定位)。
- **行号**:两者都没做;**defer / 可选**(0021 开关)。
- **长行**:横向滚动需 scissor(= 表格 C 同款,TODO),**defer**;v1 先折行或溢出。

## 7. 建议 + 开放问题

**建议**:
1. **引擎**:`syntect`(`default-syntaxes`+`default-themes`+`regex-fancy`,关 yaml);盯 `bincode` RUSTSEC(deny.toml 记)。
2. **集成**:路 A 角色塌缩——8 个 `Code*` role + scope→role 映射 + 0021 调色板(亮暗),render 零改。
3. **流式**:只重高亮活动块 + settle 冻结 + (hash,lang) 缓存。
4. **chrome v1**:面板底(0018)+ 语言标签;复制/行号/横滚 defer。

**开放问题**:
- scope→8 role 的映射表粒度(syntect scope 上百 → 8;参考 warp `ColorMap` 8 类 + base16 分组)。是否要可配(0021)?
- syntect 默认语法全集进 wasm 的**包体增量**(需实测;可裁剪到常用语言子集,同 K′ MSDF 烘常用字思路)。
- `bincode`/`yaml` 供应链(关 yaml 已避一半;bincode 传递依赖盯上游)。
- 是否新开 **ADR**(代码高亮引擎 + 角色塌缩模型)?建议落地前开一条(承 0010/0021)。
- 进阶:`ParseState` 跨帧缓存做真增量(暂不需,活动块重跑已够)。

---

> 参照源码:warp `crates/{syntax_tree/src/queries/highlight_query.rs, syntax_tree/src/lib.rs, languages/src/lib.rs, warpui_core/src/elements/formatted_text_element.rs}`;jcode `crates/{jcode-tui-markdown/src/markdown_render_support.rs, markdown_incremental.rs, markdown_render_full.rs}` + `jcode-tui-workspace/src/color_support.rs`。
