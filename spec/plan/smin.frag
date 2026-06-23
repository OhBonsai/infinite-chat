// smin —— 多项式 smooth-min(IQ)演示:几个圆 SDF 平滑融合成 metaball。
// 对应引擎 base/sdf.wgsl 的 op_smin。输出 rgb(青/靛)。

float opSmin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

void mainImage(out vec4 o, in vec2 fc) {
    vec2 uv = (fc - 0.5 * iResolution.xy) / iResolution.y; // 居中 + 纵横比校正
    float t = iTime;

    float d = 1e9;
    for (int i = 0; i < 5; i++) {
        float fi = float(i);
        vec2 c = 0.30 * vec2(cos(t * 0.6 + fi * 1.7), sin(t * 0.8 + fi * 2.1));
        float r = 0.11 + 0.035 * sin(t * 1.3 + fi);
        d = opSmin(d, length(uv - c) - r, 0.13);   // 平滑并
    }

    vec3 bg = vec3(0.102, 0.063, 0.251);
    vec3 fg = vec3(0.239, 0.961, 0.816);
    float inside = smoothstep(0.006, -0.006, d);
    // 内部等距 iso 线(显出 smin 融合的等值面)
    float iso = smoothstep(0.6, 1.0, abs(sin(d * 90.0))) * inside * 0.25;
    // 外缘柔光
    float rim = smoothstep(0.05, 0.0, abs(d)) * (1.0 - inside);

    vec3 col = mix(bg, fg, inside);
    col -= iso * 0.4;
    col += fg * rim * 0.6;
    o = vec4(col, 1.0);
}
