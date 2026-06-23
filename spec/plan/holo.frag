// holo —— 全息卡片:角向彩虹 hue + 扫描线 + 斜向高光扫过 + 边缘 fresnel 辉光,随时间流动。
// 输出 rgb(彩虹)。圆角矩形 SDF 作卡片轮廓。

vec3 hue(float h) { return 0.5 + 0.5 * cos(6.28318 * (h + vec3(0.0, 0.33, 0.67))); }

float rrect(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void mainImage(out vec4 o, in vec2 fc) {
    vec2 uv = (fc - 0.5 * iResolution.xy) / iResolution.y; // 居中 + 纵横比校正
    float t = iTime;

    float d = rrect(uv, vec2(0.34, 0.26), 0.05);
    float card = smoothstep(0.004, -0.004, d);

    // 全息色:角度 + 位置 + 时间 驱动 hue
    float ang = atan(uv.y, uv.x);
    float h = fract(0.5 * ang / 3.14159 + uv.x * 0.7 + uv.y * 0.5 + t * 0.12);
    vec3 holo = hue(h);

    // 扫描线 + 斜向高光扫过
    float scan = 0.82 + 0.18 * sin(uv.y * 140.0 - t * 6.0);
    float sweep = smoothstep(0.16, 0.0, abs(sin((uv.x + uv.y) * 2.2 - t * 1.4)));

    vec3 col = holo * scan + sweep * 0.45;

    // fresnel 边缘辉光(贴近卡片边)
    float rim = smoothstep(0.05, 0.0, abs(d));
    col += hue(h + 0.3) * rim * 1.1;

    vec3 bg = vec3(0.06, 0.04, 0.16);
    o = vec4(mix(bg, col, card), 1.0);
}
