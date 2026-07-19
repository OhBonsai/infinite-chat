import { e as S, _ as T, __tla as __tla_0 } from "./boot-B4nH8qNC.js";
import { m as k } from "./pages-nav-BiSBOohm.js";
Promise.all([
  (() => {
    try {
      return __tla_0;
    } catch {
    }
  })()
]).then(async () => {
  k("components");
  const t = (n) => ({
    kind: "text",
    text: n
  }), d = (n, s) => ({
    kind: "tool",
    tool: n,
    state: s
  }), w = [
    {
      id: "sec-agent",
      span: [
        4,
        1
      ],
      title: "",
      section: true,
      content: [
        t("## Agent \u4F1A\u8BDD\u5361 \xB7 \u4E5D\u7EC4\u4EF6")
      ]
    },
    {
      id: "a-user",
      span: [
        2,
        1
      ],
      title: "USER",
      content: [
        t("\u628A\u9996\u9875\u516D\u5E55\u91CD\u6784\u6210\u7EC4\u4EF6\u5899,tile \u8981 SDF \u771F\u6E32\u67D3\u3002")
      ]
    },
    {
      id: "a-asst",
      span: [
        2,
        1
      ],
      title: "ASSISTANT",
      content: [
        t("\u5DF2\u62C6\u6210 **4 \u5217\u57FA\u7F51\u683C** + skyline \u6446\u4F4D\u3002\u6BCF\u683C\u662F\u771F panel \u56FE\u5143,\u7F29\u653E\u9510\u5229\u3002")
      ]
    },
    {
      id: "a-reason",
      span: [
        2,
        1
      ],
      title: "REASONING",
      content: [
        t("\u5148\u786E\u8BA4 span \u8BBE\u8BA1\u8868 \u2192 \u518D\u5199\u5E03\u5C40\u5668 \u2192 \u6052\u7B49\u786C\u95E8\u5148\u9A8C\u3002")
      ]
    },
    {
      id: "a-read",
      span: [
        2,
        1
      ],
      title: "TOOL \xB7 READ",
      content: [
        d("read", {
          status: "running",
          input: {
            path: "crates/core/src/tilelayout.rs"
          }
        })
      ]
    },
    {
      id: "a-bash",
      span: [
        2,
        2
      ],
      title: "TOOL \xB7 BASH",
      content: [
        d("bash", {
          status: "completed",
          input: {
            cmd: "cargo test -p core tile"
          },
          metadata: {
            output: `running 8 tests
test result: ok. 8 passed`
          }
        })
      ]
    },
    {
      id: "a-ask",
      span: [
        2,
        2
      ],
      title: "ASK",
      content: [
        d("ask", {
          status: "pending",
          metadata: {
            question: "\u5141\u8BB8\u5199\u5165 tilelayout.rs?",
            options: [
              "\u5141\u8BB8",
              "\u62D2\u7EDD"
            ]
          }
        })
      ]
    },
    {
      id: "a-diff",
      span: [
        4,
        3
      ],
      title: "TOOL \xB7 EDIT",
      content: [
        d("edit", {
          status: "completed",
          input: {
            path: "crates/core/src/tilelayout.rs"
          },
          metadata: {
            filediff: `@@ -40,3 +40,6 @@
 ctx let col_w = grid(w);
-place_naive(t);
+let row = skyline_min(cols, sw);
+for i in col..col+sw { skyline[i] = row + sh; }
+rect(col, row, sw, sh) // \u843D\u7F51\u683C\u7EBF
`
          }
        })
      ]
    },
    {
      id: "a-compact",
      span: [
        4,
        1
      ],
      title: "COMPACTION",
      content: [
        t("\u2014 \u4E0A\u4E0B\u6587\u5DF2\u538B\u7F29 \xB7 \u4FDD\u7559 12 \u6761 \u2014")
      ]
    },
    {
      id: "a-error",
      span: [
        2,
        1
      ],
      title: "ERROR",
      content: [
        d("bash", {
          status: "error",
          input: {
            cmd: "curl api"
          },
          error: "APIError: rate limited (429) \u2014 \u5DF2\u91CD\u8BD5 3 \u6B21"
        })
      ]
    },
    {
      id: "sec-md",
      span: [
        4,
        1
      ],
      title: "",
      section: true,
      content: [
        t("## Markdown \u5168\u7C7B\u578B \xB7 \u5341\u4E5D\u578B")
      ]
    },
    {
      id: "m-head",
      span: [
        2,
        2
      ],
      title: "HEADINGS",
      content: [
        t(`# H1 \u6807\u9898
## H2
### H3`)
      ]
    },
    {
      id: "m-emph",
      span: [
        2,
        1
      ],
      title: "EMPHASIS",
      content: [
        t("**\u7C97** \xB7 *\u659C* \xB7 ~~\u5220~~ \xB7 `\u884C\u5185\u7801`")
      ]
    },
    {
      id: "m-link",
      span: [
        1,
        1
      ],
      title: "LINK",
      content: [
        t("[GitHub \u2197](https://github.com/OhBonsai/infinite-chat)")
      ]
    },
    {
      id: "m-emoji",
      span: [
        1,
        1
      ],
      title: "EMOJI",
      content: [
        t("\u5B57 \u{1F338} \u6DF7\u6392 \u2705")
      ]
    },
    {
      id: "m-ul",
      span: [
        2,
        2
      ],
      title: "LIST \xB7 UL",
      content: [
        t(`- \u4E00
- \u4E8C
  - \u5D4C\u5957`)
      ]
    },
    {
      id: "m-ol",
      span: [
        2,
        2
      ],
      title: "LIST \xB7 OL",
      content: [
        t(`1. \u4E00
2. \u4E8C
3. \u4E09`)
      ]
    },
    {
      id: "m-task",
      span: [
        2,
        1
      ],
      title: "TASKS",
      content: [
        t(`- [x] \u5DF2\u5B8C\u6210
- [ ] \u5F85\u529E`)
      ]
    },
    {
      id: "m-quote",
      span: [
        2,
        1
      ],
      title: "QUOTE",
      content: [
        t("> \u5F15\u64CE\u5728\u8DD1,\u4F60\u770B\u5230\u7684\u5C31\u662F\u5BA3\u4F20\u7247\u672C\u8EAB\u3002")
      ]
    },
    {
      id: "m-note",
      span: [
        2,
        1
      ],
      title: "ALERT \xB7 NOTE",
      content: [
        t(`> [!NOTE]
> SDF \u6E32\u67D3,\u4EFB\u610F\u7F29\u653E\u9510\u5229\u3002`)
      ]
    },
    {
      id: "m-warn",
      span: [
        2,
        1
      ],
      title: "ALERT \xB7 WARN",
      content: [
        t(`> [!WARNING]
> \u524D\u6CBF\u7559 hold \u533A,\u4E0D\u63D0\u4EA4\u6B67\u4E49\u5B57\u8282\u3002`)
      ]
    },
    {
      id: "m-hr",
      span: [
        4,
        1
      ],
      title: "RULE",
      content: [
        t(`\u5206\u8282\u7EBF

---`)
      ]
    },
    {
      id: "m-code",
      span: [
        4,
        2
      ],
      title: "CODE BLOCK",
      content: [
        t("```rust\nfn place(t: &Tile, sky: &mut [u32]) -> Rect {\n    let (c, row) = skyline_min(sky, t.span[0]);\n    for i in c..c + t.span[0] { sky[i] = row + t.span[1]; }\n    Rect::grid(c, row, t.span) // \u56DB\u8FB9\u843D\u7F51\u683C\u7EBF\n}\n```")
      ]
    },
    {
      id: "m-codefold",
      span: [
        2,
        2
      ],
      title: "CODE \xB7 FOLD",
      content: [
        t("```rust\n" + Array.from({
          length: 28
        }, (n, s) => `let v${s} = ${s} * col_w;`).join(`
`) + "\n```")
      ]
    },
    {
      id: "m-table",
      span: [
        4,
        2
      ],
      title: "TABLE",
      content: [
        t(`| \u80FD\u529B | \u624B\u6CD5 | \u89C4\u6A21 |
| --- | --- | --- |
| \u6587\u5B57 | SDF / MSDF | \u4EFB\u610F\u7F29\u653E\u9510\u5229 |
| \u6D41\u5F0F | \u9010\u5B57 reveal | \u5168\u7A0B\u65E0\u8DF3\u53D8 |
| \u5386\u53F2 | settled \u6D3E\u7ED8 | 100+ \u8F6E\u4E1D\u6ED1 |`)
      ]
    },
    {
      id: "m-foot",
      span: [
        2,
        1
      ],
      title: "FOOTNOTE",
      content: [
        t(`\u6B63\u6587\u6709\u811A\u6CE8[^1]

[^1]: \u811A\u6CE8\u5185\u5BB9\u3002`)
      ]
    },
    {
      id: "m-imath",
      span: [
        1,
        1
      ],
      title: "MATH \xB7 IN",
      content: [
        t("\u8D28\u80FD $E=mc^2$")
      ]
    },
    {
      id: "m-bmath",
      span: [
        2,
        2
      ],
      title: "MATH \xB7 BLOCK",
      content: [
        t("$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$")
      ]
    },
    {
      id: "m-svg",
      span: [
        2,
        1
      ],
      title: "SVG",
      content: [
        t("![\u6D41\u7A0B](data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='120' height='40'><rect x='2' y='8' width='50' height='24' rx='4' fill='none' stroke='%23c8aa6e'/><text x='27' y='25' fill='%23c8aa6e' font-size='11' text-anchor='middle'>SDF</text></svg>)")
      ]
    }
  ], h = "cm", A = 16;
  function D(n) {
    return [
      ...n.section ? [] : [
        t(`**${n.title}**`)
      ],
      ...n.content
    ];
  }
  function I(n, s, p, i) {
    var _a;
    const m = i.kind === "text" ? {
      type: "text",
      id: p,
      messageID: s,
      sessionID: h,
      text: i.text
    } : {
      type: "tool",
      id: p,
      messageID: s,
      sessionID: h,
      tool: i.tool,
      state: i.state
    };
    (_a = n.push_event) == null ? void 0 : _a.call(n, JSON.stringify({
      type: "message.part.updated",
      properties: {
        part: m,
        time: 1
      }
    }));
  }
  async function x() {
    const n = document.getElementById("cm-canvas");
    if (!n) return;
    const { chat: s, ok: p } = await S({
      canvasId: "cm-canvas",
      glyphMode: 1
    });
    if (!p || !s) {
      await L();
      return;
    }
    const i = s, m = (e = 0) => {
      const o = i.effect_preset_name;
      if (!o || o.call(s) === "") {
        e < 600 && requestAnimationFrame(() => m(e + 1));
        return;
      }
      O();
    };
    let y = 0;
    const g = /* @__PURE__ */ new Map();
    function O() {
      var _a, _b, _c;
      (_a = i.set_reveal_cps) == null ? void 0 : _a.call(i, 1e9);
      for (const a of w) {
        (_b = s.push_event) == null ? void 0 : _b.call(s, JSON.stringify({
          type: "message.updated",
          properties: {
            info: {
              id: a.id,
              role: "assistant",
              sessionID: h
            }
          }
        }));
        const c = [];
        for (const r of D(a)) {
          const l = `${a.id}-p${y++}`;
          I(i, a.id, l, r), c.push(l);
        }
        g.set(a.id, c);
      }
      const e = {
        cols: 4,
        gap: A,
        row_h: 150,
        pad: 13,
        title_h: 26,
        tiles: w.map((a) => ({
          id: a.id,
          span: a.span,
          title: a.title
        }))
      };
      ((_c = i.set_tile_spec) == null ? void 0 : _c.call(i, JSON.stringify(e))) || console.warn("[components] tile spec \u88AB\u62D2(\u9000\u5355\u5217)");
    }
    const u = window.devicePixelRatio || 1;
    let f = "", _ = 0;
    const E = (e) => {
      var _a, _b;
      const o = w.find((l) => l.id === e);
      if (!o || o.section || f === e) return;
      f = e, (_a = i.set_reveal_cps) == null ? void 0 : _a.call(i, 42);
      const a = g.get(e) ?? [], c = [];
      for (const l of o.content) {
        const v = `${e}-r${y++}`;
        I(i, e, v, l), c.push(v);
      }
      const r = a[0];
      for (const l of a.slice(o.section ? 0 : 1)) (_b = s.push_event) == null ? void 0 : _b.call(s, JSON.stringify({
        type: "message.part.removed",
        properties: {
          sessionID: h,
          messageID: e,
          partID: l
        }
      }));
      g.set(e, r ? [
        r,
        ...c
      ] : c), window.clearTimeout(_), _ = window.setTimeout(() => {
        var _a2;
        (_a2 = i.set_reveal_cps) == null ? void 0 : _a2.call(i, 1e9), f = "";
      }, 2600);
    };
    n.addEventListener("pointermove", (e) => {
      var _a;
      const o = n.getBoundingClientRect(), a = (e.clientX - o.left) * u, c = (e.clientY - o.top) * u, r = ((_a = i.tile_hit) == null ? void 0 : _a.call(i, a, c)) ?? "";
      r && f === "" && E(r);
    }), n.addEventListener("click", (e) => {
      var _a;
      const o = n.getBoundingClientRect();
      (_a = i.tap) == null ? void 0 : _a.call(i, (e.clientX - o.left) * u, (e.clientY - o.top) * u);
    }), m();
  }
  async function L() {
    const n = document.getElementById("cm-canvas");
    n && (n.style.display = "none");
    const { mountComponentsFallback: s } = await T(async () => {
      const { mountComponentsFallback: p } = await import("./components-fallback-CSvhB7bd.js");
      return {
        mountComponentsFallback: p
      };
    }, []);
    s(document.getElementById("cm-fallback"), "/infinite-chat/");
  }
  x();
});
