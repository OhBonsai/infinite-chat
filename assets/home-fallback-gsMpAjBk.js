const i = { title: "reveal-typing.webm", claim: "canvas-zoom.png", table: "md-codeblock.png", card: "card-diff.png", bloom: "fx-dissolve.webm" };
function d(n, a) {
  n.style.display = "block";
  const c = /* @__PURE__ */ new Map();
  for (const [l, o] of Object.entries(i)) {
    const t = `${a}pages-assets/${o}`;
    let s;
    if (o.endsWith(".webm")) {
      const e = document.createElement("video");
      e.src = t, e.muted = true, e.loop = true, e.playsInline = true, e.preload = "auto", s = e;
    } else {
      const e = document.createElement("img");
      e.src = t, e.loading = "eager", s = e;
    }
    s.className = "fb-slide", n.appendChild(s), c.set(l, s);
  }
  return { show: (l) => {
    for (const [o, t] of c) {
      const s = o === l;
      t.classList.toggle("on", s), t instanceof HTMLVideoElement && (s ? t.play().catch(() => {
      }) : t.pause());
    }
  } };
}
export {
  d as mountFallbackSlides
};
