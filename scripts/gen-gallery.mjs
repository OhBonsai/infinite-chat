#!/usr/bin/env node
// gen-gallery —— 把 spec/plan/tool-icons.frag(单一真源)内联进 web/public/gallery.html。
// 一个自包含 WebGL2 页:4×4 动态 tool icon(我们自绘,不含版权 deck)+ 标签 + 青/靛配色。
// 用法:node scripts/gen-gallery.mjs
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const frag = readFileSync(resolve(ROOT, "spec/plan/tool-icons.frag"), "utf8");

// 与 tool-icons.frag 的 case 0..15(行主序,顶→底)对应。
const LABELS = [
  "read", "write", "edit", "grep",
  "glob", "shell", "task", "web fetch",
  "web search", "todo", "skill", "patch",
  "question", "plan · enter", "plan · exit", "invalid",
];

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>opencode · tool icons (shaderbox)</title>
<style>
  :root { --fg:#3df5d0; --bg:#1a1040; --ink:#0d0a22; }
  html,body { margin:0; background:var(--ink); color:#e8eaf2;
    font:15px/1.5 system-ui,-apple-system,Segoe UI,Roboto,sans-serif; }
  .wrap { max-width:760px; margin:0 auto; padding:28px 20px 60px; }
  h1 { font-size:20px; letter-spacing:.4px; margin:0 0 4px; }
  h1 b { color:var(--fg); }
  .sub { opacity:.65; margin:0 0 22px; font-size:13px; }
  .stage { position:relative; width:100%; aspect-ratio:1/1;
    border:1px solid #ffffff14; border-radius:12px; overflow:hidden; background:var(--bg); }
  canvas { display:block; width:100%; height:100%; }
  .labels { position:absolute; inset:0; display:grid;
    grid-template-columns:repeat(4,1fr); grid-template-rows:repeat(4,1fr); pointer-events:none; }
  .labels div { display:flex; align-items:flex-end; justify-content:center; padding-bottom:6px;
    font-size:11px; color:#bfeee2; opacity:.9; text-align:center; }
  .foot { margin-top:18px; font-size:12px; opacity:.6; }
  .foot a { color:var(--fg); text-decoration:none; }
</style>
</head>
<body>
<div class="wrap">
  <h1><b>opencode</b> · tool icons <span style="opacity:.5">— shaderbox / SDF</span></h1>
  <p class="sub">16 个内置工具的程序化动态图标。纯片元 SDF,任意缩放锐利。<a style="color:var(--fg);text-decoration:none" href="./">← 回引擎演示</a></p>
  <div class="stage">
    <canvas id="g"></canvas>
    <div class="labels">${LABELS.map((l) => `<div>${l}</div>`).join("")}</div>
  </div>
  <p class="foot">每个图标=一段 fragment shader(plan16 ShaderBox)。配色青 #3DF5D0 / 靛 #1A1040,可换。
    源:<a href="https://github.com/OhBonsai/infinite-chat">infinite-chat</a>。</p>
</div>

<script type="x-shader/x-vertex" id="vs">#version 300 es
void main(){ vec2 p=vec2((gl_VertexID<<1)&2, gl_VertexID&2); gl_Position=vec4(p*2.0-1.0,0.0,1.0); }
</script>

<script type="x-shader/x-fragment" id="fs">#version 300 es
precision highp float;
uniform vec2 iResolution;
uniform float iTime;
out vec4 outColor;

${frag}

void main(){
  vec4 c; mainImage(c, gl_FragCoord.xy);
  float l = c.r;                                   // mainImage 出灰度
  vec3 BG = vec3(0.102, 0.063, 0.251);             // 靛
  vec3 FG = vec3(0.239, 0.961, 0.816);             // 青
  outColor = vec4(mix(BG, FG, clamp(l,0.0,1.0)), 1.0);
}
</script>

<script>
const cv = document.getElementById("g");
const gl = cv.getContext("webgl2", { antialias:true });
if (!gl) { document.querySelector(".stage").innerHTML =
  "<p style='padding:20px;color:#fbb'>此浏览器不支持 WebGL2。请用 Chrome / Edge / Firefox。</p>"; }
else {
  const compile = (type, src) => { const s=gl.createShader(type); gl.shaderSource(s,src); gl.compileShader(s);
    if(!gl.getShaderParameter(s,gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s)); return s; };
  const prog = gl.createProgram();
  gl.attachShader(prog, compile(gl.VERTEX_SHADER, document.getElementById("vs").textContent.trim()));
  gl.attachShader(prog, compile(gl.FRAGMENT_SHADER, document.getElementById("fs").textContent.trim()));
  gl.linkProgram(prog);
  if(!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(prog));
  gl.useProgram(prog);
  const uRes = gl.getUniformLocation(prog, "iResolution");
  const uTime = gl.getUniformLocation(prog, "iTime");
  const vao = gl.createVertexArray(); gl.bindVertexArray(vao);
  function resize(){ const dpr=Math.min(window.devicePixelRatio||1, 2);
    const w=cv.clientWidth*dpr, h=cv.clientHeight*dpr;
    if(cv.width!==w||cv.height!==h){ cv.width=w; cv.height=h; gl.viewport(0,0,w,h); } }
  const t0 = performance.now();
  function frame(){ resize(); gl.uniform2f(uRes, cv.width, cv.height);
    gl.uniform1f(uTime, (performance.now()-t0)/1000);
    gl.drawArrays(gl.TRIANGLES, 0, 3); requestAnimationFrame(frame); }
  requestAnimationFrame(frame);
}
</script>
</body>
</html>
`;

const out = resolve(ROOT, "web/public/gallery.html");
writeFileSync(out, html);
console.log(`wrote ${out} (${(html.length / 1024).toFixed(1)} KB, frag inlined)`);
