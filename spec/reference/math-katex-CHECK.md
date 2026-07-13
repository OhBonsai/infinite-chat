# math ↔ KaTeX 参考对照(Plan 31 R1 / 0035 §4)

- 生成:web/scripts/check-math-katex.mjs;容差 max(0.04em, 3%)。
- 结果:**4/4 PASS**

| 度量(em) | RaTeX | KaTeX | Δ | 判定 |
|---|---|---|---|---|
| sup 抬升(x^2) | 0.3630 | 0.3630 | 0.0000 | PASS |
| frac 分子→杠 | 0.4070 | 0.4070 | 0.0000 | PASS |
| frac 杠→分母 | 0.9560 | 0.9560 | 0.0000 | PASS |
| sub 下沉(x_1) | 0.1500 | 0.1500 | 0.0000 | PASS |
