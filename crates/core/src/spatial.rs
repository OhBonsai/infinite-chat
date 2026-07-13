//! spatial(M8)— 块高度区间索引(Plan 29 V1,0029 §84 / agent-ui 调研 §1「前缀和路线」)。
//!
//! Plan 19 实测真凶之一:旧 `SpatialGrid`(均匀格 HashMap)**每帧 clear+insert O(总块)**,
//! 长会话下 grid 段 ~1.8ms/帧(plan29 before)。聊天列是**单列纵排**(boxlayout 文档序,
//! y 单调不减)→ 可见性查询本质是一维区间问题:**按 y 排序的前缀区间数组 + 二分**即得
//! O(log N + K) 查询,零 HashMap 翻搅、buffer 帧间复用零分配。
//!
//! 不变式(GOAL §6 校验器):`query` 结果 == 线性扫描结果(`assert_matches_linear`,测试用);
//! 输入若非 y 有序,`build` 会补一次稳定排序(容错,不破正确性)。
//! CR1 纯逻辑,native 可测;确定性(同输入同输出,R8)。

use crate::camera::Rect;

/// 一条块区间(世界 y0..y1 + 块 id)。x 不参与索引(单列;x 裁剪由调用方 narrow phase 兜)。
#[derive(Debug, Clone, Copy, PartialEq)]
struct Span {
    y0: f32,
    y1: f32,
    id: usize,
}

/// 块高度区间索引:按 `y0` 升序的区间数组。每帧 `build`(复用容量,O(N) 填充、常序零排序),
/// `query` 二分定位首个可能相交项 → 向后线性收集(O(log N + K))。
#[derive(Default)]
pub struct HeightIndex {
    spans: Vec<Span>,
    /// 到 i 为止(含 i)的最大 y1 前缀 —— 区间可变高,仅 y0 有序不足以早停;
    /// `max_y1_prefix` 让二分下界正确(找首个 `max_y1 > query.y0` 的位置)。
    max_y1_prefix: Vec<f32>,
}

impl HeightIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 以 (id, y0, 高) 序列重建索引(每帧;容量复用)。输入应按文档序(y0 单调不减);
    /// 若乱序(防御)则排序矫正。
    pub fn build(&mut self, items: impl IntoIterator<Item = (usize, f32, f32)>) {
        self.spans.clear();
        for (id, y0, h) in items {
            self.spans.push(Span {
                y0,
                y1: y0 + h.max(0.0),
                id,
            });
        }
        if !self.spans.is_sorted_by(|a, b| a.y0 <= b.y0) {
            self.spans.sort_by(|a, b| a.y0.total_cmp(&b.y0));
        }
        self.max_y1_prefix.clear();
        let mut m = f32::NEG_INFINITY;
        for s in &self.spans {
            m = m.max(s.y1);
            self.max_y1_prefix.push(m);
        }
    }

    /// 查询与视口 `r` 纵向相交的块 id(升序,确定性)。O(log N + K)。
    #[must_use]
    pub fn query(&self, r: &Rect) -> Vec<usize> {
        let (top, bottom) = (r.y, r.y + r.h);
        // 二分:首个「前缀最大 y1 > top」的位置(其之前的所有区间都整体在视口上方)。
        let start = self.max_y1_prefix.partition_point(|&m| m <= top);
        let mut out = Vec::new();
        for s in &self.spans[start..] {
            if s.y0 >= bottom {
                break; // y0 有序 → 后续全在视口下方
            }
            if s.y1 > top {
                out.push(s.id);
            }
        }
        out.sort_unstable(); // id 升序(输入即文档序时为常序,近零开销)
        out
    }

    /// 不变式校验(GOAL §6):二分查询 == 线性扫描。测试用。
    #[cfg(test)]
    pub(crate) fn assert_matches_linear(&self, r: &Rect) {
        let (top, bottom) = (r.y, r.y + r.h);
        let mut linear: Vec<usize> = self
            .spans
            .iter()
            .filter(|s| s.y1 > top && s.y0 < bottom)
            .map(|s| s.id)
            .collect();
        linear.sort_unstable();
        assert_eq!(self.query(r), linear, "二分命中必须 == 线性扫描");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_returns_overlapping_only() {
        let mut ix = HeightIndex::new();
        ix.build([(0, 0.0, 50.0), (1, 1000.0, 50.0)]);
        assert_eq!(ix.query(&Rect::new(0.0, 0.0, 200.0, 200.0)), vec![0]);
        ix.assert_matches_linear(&Rect::new(0.0, 0.0, 200.0, 200.0));
    }

    #[test]
    fn edge_touching_excluded() {
        let mut ix = HeightIndex::new();
        ix.build([(0, 0.0, 100.0), (1, 100.0, 100.0), (2, 200.0, 100.0)]);
        // 视口 [100,200):恰触 0 的底边(y1==100,不算相交)、含 1、触 2 顶边(不算)。
        assert_eq!(ix.query(&Rect::new(0.0, 100.0, 10.0, 100.0)), vec![1]);
    }

    #[test]
    fn unsorted_input_is_corrected() {
        let mut ix = HeightIndex::new();
        ix.build([(2, 200.0, 50.0), (0, 0.0, 50.0), (1, 100.0, 50.0)]);
        assert_eq!(ix.query(&Rect::new(0.0, 0.0, 10.0, 260.0)), vec![0, 1, 2]);
        ix.assert_matches_linear(&Rect::new(0.0, 90.0, 10.0, 20.0));
    }

    #[test]
    fn variable_heights_no_false_negatives() {
        // 变高:早出现的超高块(如整屏代码块)必须在后续视口仍命中(max_y1 前缀保证)。
        let mut ix = HeightIndex::new();
        ix.build([(0, 0.0, 5000.0), (1, 100.0, 50.0), (2, 4000.0, 50.0)]);
        let vis = Rect::new(0.0, 3900.0, 10.0, 300.0);
        assert_eq!(ix.query(&vis), vec![0, 2]);
        ix.assert_matches_linear(&vis);
    }

    #[test]
    fn rebuild_reuses_and_replaces() {
        let mut ix = HeightIndex::new();
        ix.build([(0, 0.0, 10.0)]);
        ix.build([(5, 500.0, 10.0)]);
        assert!(ix.query(&Rect::new(0.0, 0.0, 10.0, 100.0)).is_empty());
        assert_eq!(ix.query(&Rect::new(0.0, 490.0, 10.0, 100.0)), vec![5]);
    }

    #[test]
    #[ignore = "计时基准(release 手跑):cargo test -p infinite-chat-core --release bench_height_index -- --ignored --nocapture"]
    fn bench_height_index_query_sublinear() {
        // GOAL §3 判据:总块 ×10 → 查询耗时不 ×10(实测应近常数)。测试环境计时,仅基准用
        //(R8 约束的是 core 运行时,测试计时不入渲染路径)。
        let mk = |n: usize| {
            let mut ix = HeightIndex::new();
            ix.build((0..n).map(|i| (i, i as f32 * 100.0, 80.0)));
            ix
        };
        let time = |ix: &HeightIndex, y: f32| {
            let r = Rect::new(0.0, y, 10.0, 300.0);
            let t = std::time::Instant::now();
            let mut acc = 0usize;
            for k in 0..10_000 {
                acc += ix
                    .query(&Rect::new(0.0, y + (k % 7) as f32, 10.0, r.h))
                    .len();
            }
            let dt = t.elapsed().as_secs_f64();
            (dt, acc)
        };
        let small = mk(10_000);
        let large = mk(100_000);
        let (ts, a1) = time(&small, 400_000.0);
        let (tl, a2) = time(&large, 4_000_000.0);
        println!(
            "# height-index query: 10k 块 {ts:.4}s / 100k 块 {tl:.4}s(10k 次查询;acc {a1}/{a2})"
        );
        assert!(
            tl < ts * 5.0,
            "×10 块查询耗时应远低于 ×10(实测 {ts:.4}s → {tl:.4}s)"
        );
    }

    #[test]
    fn query_touch_count_constant_as_blocks_x10() {
        // GOAL §3:总块 ×10 → 查询代价不 ×10。确定性代理:固定视口的命中数为常数
        //(二分 start 之后的线性段只到视口底,与 N 无关);耗时基准见 bench_height_index。
        let mk = |n: usize| {
            let mut ix = HeightIndex::new();
            ix.build((0..n).map(|i| (i, i as f32 * 100.0, 80.0)));
            ix
        };
        let r = Rect::new(0.0, 50_000.0, 10.0, 300.0);
        let small = mk(1_000);
        let large = mk(10_000);
        small.assert_matches_linear(&r);
        large.assert_matches_linear(&r);
        // 1k 时视口在末尾之外(空);10k 时视口内 ~4 块 —— 都是常数级,与总量无关。
        assert!(small.query(&r).len() <= 5 && large.query(&r).len() <= 5);
        assert_eq!(large.query(&r), vec![500, 501, 502]);
    }
}
