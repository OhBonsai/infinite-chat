// home-bg.ts — Plan 41 补丁:首页环境背景 shader —— 左右两侧「流光线」(silk/aurora 竖向光帘)。
// 低成本:半分辨率 backing store + 便宜 fbm;**恒定环境**(与幕内容无关,只随时间流动);
// 页隐藏暂停。取 pages-theme 金/hextech 调。参考 spec/research/shader-effects-catalog §5(aurora/silk)。
const VERT = `#version 300 es
in vec2 p; void main(){ gl_Position = vec4(p, 0.0, 1.0); }`;

// 竖向流光光帘:fbm 域扭曲 + 正弦成线 + 沿 y 上升流动;仅左右两侧(中间留给内容)。
const FRAG = `#version 300 es
precision highp float;
uniform vec2 uRes; uniform float uTime; uniform float uAmp;
out vec4 o;
float hash(vec2 p){ p=fract(p*vec2(123.34,456.21)); p+=dot(p,p+45.32); return fract(p.x*p.y); }
float noise(vec2 p){ vec2 i=floor(p),f=fract(p); f=f*f*(3.-2.*f);
  float a=hash(i),b=hash(i+vec2(1,0)),c=hash(i+vec2(0,1)),d=hash(i+vec2(1,1));
  return mix(mix(a,b,f.x),mix(c,d,f.x),f.y); }
float fbm(vec2 p){ float v=0.,a=.5; for(int i=0;i<4;i++){ v+=a*noise(p); p=p*2.0+7.0; a*=.5; } return v; }
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  float t = uTime * 0.08;
  // 竖向域扭曲(沿 y 上升流动)→ 丝绸/光帘的飘动
  float warp = fbm(vec2(uv.x * 1.6, uv.y * 1.0 - t * 1.5));
  float warp2 = fbm(vec2(uv.x * 3.2 + 5.0, uv.y * 1.6 - t * 2.2));
  // 两层不同频的**细亮竖线**(pow 收窄成线;相位随时间/warp 飘)
  float ph1 = (uv.x * 9.0 + warp * 3.5) * 6.2831 + t * 2.5;
  float ph2 = (uv.x * 15.0 + warp2 * 4.0) * 6.2831 - t * 1.7;
  float lines = pow(0.5 + 0.5 * sin(ph1), 7.0) * 0.7
              + pow(0.5 + 0.5 * sin(ph2), 9.0) * 0.5;
  // 沿光帘的亮度流动(明暗随 y 上涌)
  float flow = smoothstep(0.25, 0.95, fbm(vec2(uv.x * 2.2, uv.y * 1.3 - t * 2.6)));
  float streak = lines * (0.25 + 0.9 * flow);
  // 左右边缘 mask:中间 (|x-.5|<~0.30) 透明,向两侧升起
  float d = abs(uv.x - 0.5) * 2.0;
  float mask = smoothstep(0.40, 0.92, d);
  // 顶/底(nav/chrome 处)淡出
  mask *= smoothstep(0.0, 0.16, uv.y) * smoothstep(1.0, 0.80, uv.y);
  float a = streak * mask * uAmp;
  // 干净鎏金:暗金 → 亮金(随流动),偶掺一丝 hextech 冷调
  vec3 col = mix(vec3(0.66, 0.54, 0.32), vec3(0.95, 0.84, 0.58), flow);
  col = mix(col, vec3(0.10, 0.62, 0.70), smoothstep(0.7, 1.0, flow) * 0.25);
  o = vec4(col * a, a); // 预乘,配 ONE/1-src alpha 混合
}`;

export interface BgHandle {
  destroy: () => void;
}

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader | null {
  const s = gl.createShader(type);
  if (!s) return null;
  gl.shaderSource(s, src);
  gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    console.warn("[home-bg] shader 编译失败", gl.getShaderInfoLog(s));
    return null;
  }
  return s;
}

/** 挂流光背景到 canvas(WebGL2)。失败(无 WebGL2)→ 静默返回,页面照常(纯装饰)。 */
export function mountBackground(canvas: HTMLCanvasElement, opts: { amp?: number } = {}): BgHandle {
  const gl = canvas.getContext("webgl2", { alpha: true, premultipliedAlpha: true, antialias: false });
  if (!gl) return { destroy: () => {} };
  const vs = compile(gl, gl.VERTEX_SHADER, VERT);
  const fs = compile(gl, gl.FRAGMENT_SHADER, FRAG);
  if (!vs || !fs) return { destroy: () => {} };
  const prog = gl.createProgram()!;
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  gl.useProgram(prog);

  const buf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buf);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW); // 覆盖屏三角
  const loc = gl.getAttribLocation(prog, "p");
  gl.enableVertexAttribArray(loc);
  gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA); // 预乘 alpha

  const uRes = gl.getUniformLocation(prog, "uRes");
  const uTime = gl.getUniformLocation(prog, "uTime");
  const uAmp = gl.getUniformLocation(prog, "uAmp");
  gl.uniform1f(uAmp, opts.amp ?? 0.5);

  // 低成本:backing store 走 0.5× CSS 尺寸(流光糊一点无妨),CSS 拉满。
  const SCALE = 0.5;
  function resize(): void {
    const w = Math.max(1, Math.round(canvas.clientWidth * SCALE));
    const h = Math.max(1, Math.round(canvas.clientHeight * SCALE));
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
      gl!.viewport(0, 0, w, h);
    }
    gl!.uniform2f(uRes, canvas.width, canvas.height);
  }
  const ro = new ResizeObserver(resize);
  ro.observe(canvas);
  resize();

  let raf = 0;
  let running = true;
  const t0 = performance.now();
  const frame = (now: number): void => {
    if (!running) return;
    gl!.uniform1f(uTime, (now - t0) / 1000);
    gl!.drawArrays(gl!.TRIANGLES, 0, 3);
    raf = requestAnimationFrame(frame);
  };
  raf = requestAnimationFrame(frame);

  const onVis = (): void => {
    if (document.hidden) {
      running = false;
      cancelAnimationFrame(raf);
    } else if (!running) {
      running = true;
      raf = requestAnimationFrame(frame);
    }
  };
  document.addEventListener("visibilitychange", onVis);

  return {
    destroy: () => {
      running = false;
      cancelAnimationFrame(raf);
      ro.disconnect();
      document.removeEventListener("visibilitychange", onVis);
    },
  };
}
