const h = [{ key: "home", label: "\u9996\u9875", href: "" }, { key: "chat", label: "\u5BF9\u8BDD", href: "chat/" }, { key: "markdown", label: "Markdown", href: "markdown/" }, { key: "components", label: "\u7EC4\u4EF6", href: "components/" }, { key: "gallery", label: "\u753B\u5ECA", href: "gallery.html" }];
function m(l, c = document.body) {
  const r = "/infinite-chat/", e = document.createElement("nav");
  e.className = "p-nav";
  const n = document.createElement("a");
  n.className = "p-brand", n.href = r, n.textContent = "INFINITE-CHAT", e.appendChild(n);
  for (const o of h) {
    const a = document.createElement("a");
    a.href = r + o.href, a.textContent = o.label, o.key === l && a.setAttribute("aria-current", "page"), e.appendChild(a);
  }
  const t = document.createElement("a");
  t.href = "https://github.com/OhBonsai/infinite-chat", t.textContent = "GitHub", t.target = "_blank", t.rel = "noopener", e.appendChild(t), c.prepend(e);
}
export {
  m
};
