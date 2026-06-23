// 路线图 teaser —— COMING SOON 卡(DOM 叠加,0022 风格)。不伪造效果,诚实占位。
export function createTeaser(parent: HTMLElement = document.body) {
  const el = document.createElement("div");
  el.style.cssText =
    "position:fixed;left:50%;top:46%;transform:translate(-50%,-50%) scale(0.96);z-index:9990;" +
    "min-width:260px;max-width:360px;padding:18px 20px;border:1px solid #3df5d055;border-radius:12px;" +
    "background:#1a1040ee;color:#e8eaf2;font:14px/1.5 system-ui,sans-serif;" +
    "opacity:0;transition:opacity .35s ease,transform .35s cubic-bezier(0.34,1.56,0.64,1);pointer-events:none;";
  el.innerHTML =
    `<div style="font:700 10px/1 'JetBrains Mono',monospace;letter-spacing:.24em;color:#3df5d0;margin-bottom:8px">COMING SOON</div>` +
    `<div class="t-title" style="font-size:16px;font-weight:600;margin-bottom:6px"></div>` +
    `<div class="t-body" style="color:#bfeee2;opacity:.85"></div>`;
  parent.appendChild(el);
  const title = el.querySelector(".t-title") as HTMLElement;
  const body = el.querySelector(".t-body") as HTMLElement;
  return {
    show(t: string, b: string) {
      title.textContent = t;
      body.textContent = b;
      el.style.opacity = "1";
      el.style.transform = "translate(-50%,-50%) scale(1)";
    },
    hide() {
      el.style.opacity = "0";
      el.style.transform = "translate(-50%,-50%) scale(0.96)";
    },
  };
}
