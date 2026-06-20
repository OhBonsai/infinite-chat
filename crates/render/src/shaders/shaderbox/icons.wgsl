// shaderbox/icons.wgsl — PixelSpiritDeck 整盘 icon(Plan 16 §2.5 / 附录 A,逐字 WGSL 直译)。
// `shade()` 据 `p0.x` = icon_id 走 `switch` 分派(一条 pipeline,uniform 分支 coherent,廉价、守护栏)。
// GLSL→WGSL:`stroke/fill/flip/bridge/rotate/scale/S/C`→`sb_*`、`iTime`→`c.time`、`mod`→`sb_mod`、
// `atan(y,x)`→`atan2`、`mat2`→`mat2x2`(列主序已核)。单 box = 一个 icon 满格(源网格预览不译)。
// **许可:vendored-with-risk,个人/非商用 test 自用**(§2.5/§6)。

fn sb_mod(a: f32, b: f32) -> f32 { return a - b * floor(a / b); }

/// 画第 `icon` 号 icon 于 `st`(0..1)→ 覆盖率(0..1)。`t` = time(秒)。
fn icon_draw(uv: vec2<f32>, icon: i32, t: f32) -> f32 {
    var st = uv;
    var color = 0.0;
    switch icon {
        case 1: { color = sstep(0.5 * sb_S(t), st.x); }
        case 2: { color = sstep(0.5 + cos(st.y * SB_PI + t / 2.0) * 0.25, st.x); }
        case 3: { color = sstep(0.5, (st.x * sb_S(t) + st.y * sb_C(t)) * 0.5); }
        case 4: { color = sb_stroke(st.x, 0.5, 0.15 * sb_S(t)); }
        case 5: {
            let offset = cos(st.y * SB_PI + t) * 0.15;
            color = sb_stroke(st.x, 0.28 + offset, 0.1);
            color += sb_stroke(st.x, 0.5 + offset, 0.1);
            color += sb_stroke(st.x, 0.72 + offset, 0.1);
        }
        case 6: {
            let offset = 0.5 + (st.x - st.y) * 0.5;
            color = sb_stroke(offset, 0.5, 0.1 * sb_S(t));
        }
        case 7: {
            let sdf = 0.5 + (st.x - st.y) * 0.5;
            color = sb_stroke(sdf, 0.5, 0.1 * sb_C(t));
            let sdf_inv = (st.x + st.y) * 0.5;
            color += sb_stroke(sdf_inv, 0.5, 0.1 * sb_C(t));
        }
        case 8: { color = sb_stroke(circleSDF(st), 0.5 * sb_S(t), 0.05 * sb_C(t)); }
        case 9: {
            color = sb_fill(circleSDF(st), 0.65);
            let offset = vec2<f32>(0.1, 0.05);
            color -= sb_fill(circleSDF(st - offset * sb_S(t)), 0.5);
        }
        case 10: {
            let sdf = rectSDF(st, vec2<f32>(1.0));
            color = sb_stroke(sdf, 0.5 * sb_C(t), 0.125);
            color += sb_fill(sdf, 0.1 * sb_S(t));
        }
        case 11: {
            let rect = rectSDF(st, vec2<f32>(1.0));
            color = sb_fill(rect, 0.5);
            let cross = crossSDF(st, 1.0);
            color *= sstep(0.5, fract(cross * 3.0 + t));
            color *= sstep(1.0, cross);
            color += sb_fill(cross, 0.5);
            color += sb_stroke(rect, 0.65, 0.05);
            color += sb_stroke(rect, 0.75, 0.025);
        }
        case 12: {
            let rect = rectSDF(st, vec2<f32>(0.5, 1.0));
            let diag = (st.x * sb_C(t) + st.y * sb_S(t)) * 0.5;
            color = sb_flip(sb_fill(rect, 0.6), sb_stroke(diag, 0.5, 0.01));
        }
        case 13: {
            let offset = vec2<f32>(0.15 * sb_S(t), 0.0);
            let left = circleSDF(st + offset);
            let right = circleSDF(st - offset);
            color = sb_flip(sb_stroke(left, 0.5, 0.05), sb_fill(right, 0.525));
        }
        case 14: {
            let sdf = vesicaSDF(st, 0.2 * sb_S(t));
            color = sb_flip(sb_fill(sdf, 0.5), sstep((st.x + st.y) * 0.5, 0.5));
        }
        case 15: {
            st.y = 1.0 - st.y;
            let ts = vec2<f32>(st.x, 0.82 - st.y);
            color = sb_fill(triSDF(st), 0.7);
            color -= sb_fill(triSDF(ts), 0.36);
        }
        case 16: {
            let circle = circleSDF(st - vec2<f32>(0.0, 0.1));
            let triangle = triSDF(st + vec2<f32>(0.0, 0.1));
            color = sb_stroke(circle, 0.5 * sb_C(t), 0.1);
            color *= sstep(0.55, triangle);
            color += sb_fill(triangle, 0.45);
        }
        case 17: {
            let sdf = rhombSDF(st);
            color = sb_fill(sdf, 0.425 * sb_S(t));
            color += sb_stroke(sdf, 0.5 * sb_S(t), 0.05);
            color += sb_stroke(sdf, 0.6 * sb_C(t), 0.03);
        }
        case 18: { color = sb_flip(sb_fill(triSDF(st), 0.5), sb_fill(rhombSDF(st), 0.4)); }
        case 19: {
            st = sb_rotate(st, radians(-25.0) * sb_S(t));
            var sdf = triSDF(st);
            sdf /= triSDF(st + vec2<f32>(0.0, 0.2 * sb_C(t)));
            color = sb_fill(abs(sdf), 0.56);
        }
        case 20: {
            st = sb_rotate(st, radians(45.0));
            color = sb_fill(rectSDF(st, vec2<f32>(1.0)), 0.4);
            color *= 1.0 - sb_stroke(st.x, 0.5 * sb_S(t), 0.02);
            color *= 1.0 - sb_stroke(st.y, 0.5 * sb_C(t), 0.02);
        }
        case 21: {
            st = sb_rotate(st, radians(-45.0));
            let off = 0.12 * sb_S(t);
            let s = vec2<f32>(1.0);
            color = sb_fill(rectSDF(st + off, s), 0.2 * sb_C(t));
            color += sb_fill(rectSDF(st - off, s), 0.2 * sb_C(t));
            let r = rectSDF(st, s);
            color *= sstep(0.33, r);
            color += sb_fill(r, 0.3);
        }
        case 22: {
            st = sb_rotate(vec2<f32>(st.x, 1.0 - st.y), radians(45.0));
            let s = vec2<f32>(1.0);
            color += sb_fill(rectSDF(st - 0.025 * sb_S(t), s), 0.4);
            color += sb_fill(rectSDF(st + 0.025, s), 0.4);
            color *= sstep(0.38, rectSDF(st + 0.025, s));
        }
        case 23: {
            st = sb_rotate(st, radians(-45.0));
            let s = vec2<f32>(1.0);
            let o = 0.05 * sb_S(t) * 1.5;
            color += sb_flip(sb_fill(rectSDF(st - o, s), 0.4), sb_fill(rectSDF(st + o, s), 0.4));
        }
        case 24: {
            st = sb_rotate(st, radians(45.0));
            let r1 = rectSDF(st, vec2<f32>(1.0) * sb_S(t));
            let r2 = rectSDF(st + 0.15 * sb_S(t), vec2<f32>(1.0));
            color += sb_stroke(r1, 0.5, 0.05);
            color *= sstep(0.325, r2);
            color += sb_stroke(r2, 0.325, 0.05) * sb_fill(r1, 0.525);
            color += sb_stroke(r2, 0.2, 0.05);
        }
        case 25: {
            st = sb_rotate(st, radians(-45.0)) - 0.08;
            for (var i = 0; i < 4; i = i + 1) {
                let r = rectSDF(st, vec2<f32>(1.0) * sb_S(t));
                color += sb_stroke(r, 0.19, 0.04);
                st = st + 0.05;
            }
        }
        case 26: {
            let d1 = polySDF(st, 5);
            let ts = vec2<f32>(st.x, 1.0 - st.y);
            let d2 = polySDF(ts, 5);
            color = sb_fill(d1, 0.75) * sb_fill(fract(d1 * 5.0 - t / 2.0), 0.5);
            color -= sb_fill(d1, 0.6) * sb_fill(fract(d2 * 4.9 - t / 2.0), 0.45);
        }
        case 27: {
            st = st.yx;
            color = sb_stroke(hexSDF(st), 0.6 * sb_C(t), 0.1);
            color += sb_fill(hexSDF(st - vec2<f32>(-0.06, -0.1) * sb_S(t)), 0.15);
            color += sb_fill(hexSDF(st - vec2<f32>(-0.06, 0.1) * sb_S(t)), 0.15);
            color += sb_fill(hexSDF(st - vec2<f32>(0.11, 0.0) * sb_S(t)), 0.15);
        }
        case 28: {
            color += sb_stroke(circleSDF(st), 0.8 * sb_C(t), 0.05);
            st.y = 1.0 - st.y;
            let s = starSDF(st.yx, 5, 0.1);
            color *= sstep(0.7 * sb_C(t), s);
            color += sb_stroke(s, 0.4 * sb_S(t), 0.1);
        }
        case 29: {
            let bgv = starSDF(st, 16, 0.1 * sb_S(t));
            color += sb_fill(bgv, 1.3);
            var l = 0.0;
            for (var i = 0.0; i < 8.0; i = i + 1.0) {
                var xy = sb_rotate(st, SB_QTR_PI * i + t / 4.0);
                xy.y -= 0.3;
                let tri = polySDF(xy, 3);
                color += sb_fill(tri, 0.3);
                l += sb_stroke(tri, 0.3 * sb_S(t), 0.03);
            }
            color *= 1.0 - l;
            let cc = polySDF(st, 8);
            color -= sb_stroke(cc, 0.15, 0.04);
        }
        case 30: {
            color = sb_stroke(raysSDF(st, 8), 0.5, 0.15 * sb_C(t) * 2.0);
            let inner = starSDF(st.xy, 6, 0.09 * sb_S(t));
            let outer = starSDF(st.yx, 6, 0.09 * sb_S(t));
            color *= sstep(0.7, outer);
            color += sb_fill(outer, 0.5);
            color -= sb_stroke(inner, 0.25, 0.06);
            color += sb_stroke(outer, 0.6, 0.05);
        }
        case 31: {
            color = sb_flip(sb_stroke(raysSDF(sb_rotate(st, -t / 8.0), 28), 0.5, 0.2), sb_fill(st.y, 0.5));
            let rect = rectSDF(st, vec2<f32>(1.0) * sb_S(t));
            color *= sstep(0.25, rect);
            color += sb_fill(rect, 0.2);
        }
        case 32: {
            let sdf = polySDF(sb_rotate(st.yx, sb_C(t)), 8);
            color = sb_fill(sdf, 0.5);
            color *= sb_stroke(raysSDF(sb_rotate(st, sb_C(t)), 8), 0.5, 0.2);
            color *= sstep(0.27, sdf);
            color += sb_stroke(sdf, 0.2, 0.05);
            color += sb_stroke(sdf, 0.6, 0.1);
        }
        case 33: {
            let v1 = vesicaSDF(st, 0.5);
            let st2 = st.yx + vec2<f32>(0.04, 0.0);
            let v2 = vesicaSDF(st2, 0.7);
            color = sb_stroke(v2, 1.0, 0.05);
            st = sb_rotate(st, t / 2.0);
            color += sb_fill(v2, 1.0) * sb_stroke(circleSDF(st - vec2<f32>(0.05)), 0.3, 0.05);
            color += sb_fill(raysSDF(st, 50), 0.2) * sb_fill(v1, 1.25) * sstep(1.0, v2);
        }
        case 34: {
            color = sb_fill(heartSDF(st), 0.5 * sb_C(t) * 1.2);
            color -= sb_stroke(polySDF(st, 3), 0.15 * sb_S(t) * 1.1, 0.05);
        }
        case 35: {
            st.x = sb_flip(st.x, step(0.5, st.y));
            let offset = vec2<f32>(0.15 * sb_S(t), 0.0);
            let left = circleSDF(st + offset);
            let right = circleSDF(st - offset);
            color = sb_stroke(left, 0.4 * sb_S(t), 0.075);
            color = sb_bridge(color, right, 0.4 * sb_S(t), 0.075);
        }
        case 36: {
            st = st.yx;
            st.x = mix(1.0 - st.x, st.x, step(0.5, st.y));
            let o = vec2<f32>(0.1, 0.0);
            let s = vec2<f32>(1.0) * sb_C(t);
            let a = radians(45.0) + t / 2.0;
            let lft = rectSDF(sb_rotate(st + o, a), s);
            let rgt = rectSDF(sb_rotate(st - o, -a), s);
            color = sb_stroke(lft, 0.3, 0.1);
            color = sb_bridge(color, rgt, 0.3, 0.1);
            color += sb_fill(rhombSDF(abs(st.yx - vec2<f32>(0.0, 0.5))), 0.1);
        }
        case 37: {
            st.x = mix(1.0 - st.x, st.x, step(0.5, st.y));
            let o = vec2<f32>(0.05, 0.0);
            let s = vec2<f32>(1.0);
            let a = radians(45.0);
            let lft = rectSDF(sb_rotate(st + o, a * sb_S(t)), s);
            let rgt = rectSDF(sb_rotate(st - o, -a * sb_S(t)), s);
            color = sb_stroke(lft, 0.145, 0.098);
            color = sb_bridge(color, rgt, 0.145, 0.098);
        }
        case 38: {
            let r1 = rectSDF(st, vec2<f32>(1.0));
            let r2 = rectSDF(sb_rotate(st, radians(45.0)), vec2<f32>(1.0));
            var inv = step(0.5, (st.x + st.y) * 0.5);
            inv = sb_flip(inv, step(0.5, 0.5 + (st.x - st.y) * 0.5));
            let w = 0.075 * sb_S(t) * 1.2;
            color = sb_stroke(r1, 0.5, w) + sb_stroke(r2, 0.5, w);
            let bridges = mix(r1, r2, inv);
            color = sb_bridge(color, bridges, 0.5, w);
        }
        case 39: {
            let inv = sstep(0.5, st.y);
            st = sb_rotate(st, radians(-45.0)) - 0.2;
            st = mix(st, 0.6 - st, sstep(0.5, inv));
            for (var i = 0; i < 5; i = i + 1) {
                let r = rectSDF(st, vec2<f32>(1.0));
                var s = 0.25;
                s -= abs(f32(i) * 0.1 - 0.2);
                color = sb_bridge(color, r, s, 0.05 * sb_S(t));
                st = st + 0.1;
            }
        }
        case 40: {
            st = sb_rotate(st, radians(-60.0) + t / 4.0);
            st.y = sb_flip(st.y, step(0.5, st.x));
            st.y += 0.25;
            let down = polySDF(st, 3);
            st.y = 1.5 - st.y;
            let top = polySDF(st, 3);
            color = sb_stroke(top, 0.4, 0.15 * sb_S(t));
            color = sb_bridge(color, down, 0.4, 0.15 * sb_S(t));
        }
        case 41: {
            st.y = 1.0 - st.y;
            let s = 0.25 * sb_C(t) * 1.3;
            let t1 = polySDF(st + vec2<f32>(0.0, 0.175), 3);
            let t2 = polySDF(st + vec2<f32>(0.1, 0.0), 3);
            let t3 = polySDF(st - vec2<f32>(0.1, 0.0), 3);
            color = sb_stroke(t1, s, 0.08) + sb_stroke(t2, s, 0.08) + sb_stroke(t3, s, 0.08);
            let bridges = mix(mix(t1, t2, step(0.5, st.y)), mix(t3, t2, step(0.5, st.y)), step(0.5, st.x));
            color = sb_bridge(color, bridges, s, 0.08);
        }
        case 42: {
            let n = 12.0;
            let a = SB_TAU / n;
            for (var i = 0.0; i < n; i = i + 1.0) {
                var xy = sb_rotate(st, a * i);
                xy.y -= 0.189;
                let vsc = vesicaSDF(xy, 0.3);
                color *= 1.0 - sb_stroke(vsc, 0.45 * sb_S(t), 0.1) * sstep(0.5, xy.y);
                color += sb_stroke(vsc, 0.45 * sb_S(t), 0.05);
            }
        }
        case 43: {
            let n = 3.0;
            let a = SB_TAU / n;
            for (var i = 0.0; i < n * 2.0; i = i + 1.0) {
                var xy = sb_rotate(st, a * i);
                xy.y -= 0.09;
                let vsc = vesicaSDF(xy, 0.3);
                color = mix(
                    color + sb_stroke(vsc, 0.5, 0.1 * sb_S(t)),
                    mix(color, sb_bridge(color, vsc, 0.5, 0.1 * sb_S(t)), step(xy.x, 0.5) - step(xy.y, 0.4)),
                    step(3.0, i)
                );
            }
        }
        case 44: {
            let star = starSDF(st, 8, 0.063);
            color += sb_fill(star, 1.22);
            let n = 8.0;
            let a = SB_TAU / n;
            for (var i = 0.0; i < n; i = i + 1.0) {
                var xy = sb_rotate(st, 0.39 + a * i);
                xy = sb_scale(xy, vec2<f32>(1.0, 0.72) * sb_S(t));
                xy.y -= 0.125;
                color *= sstep(0.235, rhombSDF(xy));
            }
        }
        case 45: {
            st -= 0.5;
            let r = dot(st, st);
            let a = atan2(st.y, st.x) / SB_PI;
            var uvp = vec2<f32>(a, r);
            let grid = vec2<f32>(5.0, log(r) * 20.0 * sb_S(t));
            let uv_i = floor(uvp * grid);
            uvp.x += 0.5 * sb_mod(uv_i.y, 2.0);
            let uv_f = fract(uvp * grid);
            let shape = rhombSDF(uv_f);
            color += sb_fill(shape, 0.9) * sstep(0.75, 1.0 - r);
        }
        case 46: {
            color = sb_fill(flowerSDF(sb_rotate(st, -t / 4.0), 5), 0.25 * sb_C(t));
            color -= sstep(0.95, starSDF(sb_rotate(st, 0.628 - t / 4.0), 5, 0.1 * sb_S(t)));
            color = clamp(color, 0.0, 1.0);
            let circle = circleSDF(st);
            color -= sb_stroke(circle, 0.1, 0.05);
            color += sb_stroke(circle, 0.8, 0.07);
        }
        case 47: { color = sstep(0.5, spiralSDF(sb_rotate(st, t / 2.0), 0.13 * sb_S(t))); }
        case 48: { color = 1.0; }
        case 49: {
            st = sb_rotate(st, -t / 4.0);
            let d = 0.15;
            let r = 0.3 * sb_S(t);
            color = sb_fill(circleSDF(st - vec2<f32>(cos(SB_TAU / 3.0), sin(SB_TAU / 3.0)) * d), r);
            color += sb_fill(circleSDF(st - vec2<f32>(cos(SB_TAU / 3.0 * 2.0), sin(SB_TAU / 3.0 * 2.0)) * d), r);
            color += sb_fill(circleSDF(st - vec2<f32>(d, 0.0)), r);
            st = st.yx;
            st.y = 1.0 - st.y;
            color *= 1.0 - sb_fill(triSDF(st - vec2<f32>(0.0, 0.02)), 0.13);
            color += sb_stroke(circleSDF(st), 0.8, 0.08);
        }
        default: { color = 0.0; } // 0 = void / 未知
    }
    return clamp(color, 0.0, 1.0);
}

/// effect 入口:白色 icon(覆盖率作 alpha),`p1.rgb`(非零时)作色;over 背景由 fs.wgsl 合成。
/// **morph 钩子**(Plan 16 §2.5,copy→✓ 等):`p0.x`=icon_a、`p0.y`=icon_b、`p0.z`=t(0..1)→
/// `mix(cov_a, cov_b, t)`。t=0 时只出 icon_a(向后兼容旧的单图标发射)。
fn shade(c: ShadeCtx) -> vec4<f32> {
    let cov_a = icon_draw(c.uv, i32(c.p0.x + 0.5), c.time);
    var cov = cov_a;
    let morph = clamp(c.p0.z, 0.0, 1.0);
    if (morph > 0.0) {
        let cov_b = icon_draw(c.uv, i32(c.p0.y + 0.5), c.time);
        cov = mix(cov_a, cov_b, morph);
    }
    var tint = vec3<f32>(0.85, 0.88, 0.95);
    if (c.p1.x + c.p1.y + c.p1.z > 0.0) {
        tint = c.p1.rgb;
    }
    return vec4<f32>(tint, cov);
}
