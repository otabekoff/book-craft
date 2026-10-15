// The sandboxed preview page. It loads the book's real stylesheet and block
// runtime from bookasset:// and re-renders when the editor posts new HTML.
import { ASSET_BASE } from "./api";

export function previewDocument(theme: string, cssVersion: number): string {
  return `<!doctype html>
<html data-theme="${theme}">
<head>
<meta charset="utf-8">
<base href="${ASSET_BASE}/project/">
<link rel="stylesheet" href="${ASSET_BASE}/_runtime/book.css?v=${cssVersion}">
<script src="${ASSET_BASE}/_runtime/blocks.js"><\/script>
<style>
html, body { margin: 0; background: var(--bc-paper); color: var(--bc-ink); }
/* No rubber-band stretch when scrolling past either end. */
html, body { overscroll-behavior: none; }
body { padding: 40px 28px 120px; }
.empty { font-family: var(--bc-font-ui); color: var(--bc-muted); text-align: center; margin-top: 30vh; }
/* The page's own scrollbar is hidden; a slim overlay one (below) takes no space, like the studio's. */
html { scrollbar-width: none; }
html::-webkit-scrollbar { display: none; }
.bc-ov { position: fixed; top: 2px; bottom: 2px; right: 1px; width: 12px; z-index: 50; opacity: 0; transition: opacity 0.25s; }
.bc-ov.on, .bc-ov:hover, .bc-ov.drag { opacity: 1; }
.bc-ov-thumb { position: absolute; top: 0; right: 2px; width: 6px; border-radius: 999px; background: color-mix(in srgb, var(--bc-ink) 26%, transparent); transition: width 0.15s, background 0.15s; }
.bc-ov:hover .bc-ov-thumb, .bc-ov.drag .bc-ov-thumb { width: 8px; background: color-mix(in srgb, var(--bc-ink) 42%, transparent); }
</style>
</head>
<body>
<article id="a" class="bc-chapter"><p class="empty">Start writing…</p></article>
<script>
const a = document.getElementById("a");
const vendor = "${ASSET_BASE}/_runtime/vendor/";
function setTheme(theme) {
  if (!theme || document.documentElement.dataset.theme === theme) return;
  document.documentElement.dataset.theme = theme;
  if (window.BookBlocks) BookBlocks.setTheme(theme, a);
}
function render(html, cls, theme) {
  const y = scrollY;
  if (theme) document.documentElement.dataset.theme = theme;
  a.className = cls;
  a.innerHTML = html || '<p class="empty">Start writing…</p>';
  if (window.BookBlocks) BookBlocks.hydrate(a, { vendor, theme: document.documentElement.dataset.theme });
  scrollTo(0, y);
}
// No browser menu on right-click in the preview.
addEventListener("contextmenu", (e) => e.preventDefault());
addEventListener("message", (e) => {
  if (e.source !== parent) return;
  const d = e.data || {};
  if (d.type === "render") render(d.html, d.cls, d.theme);
  if (d.type === "theme") setTheme(d.theme);
  if (d.type === "scroll") {
    const max = document.documentElement.scrollHeight - innerHeight;
    scrollTo(0, Math.max(0, max * d.ratio));
  }
  if (d.type === "reveal") {
    const t = document.getElementById(d.id);
    if (t) t.scrollIntoView({ block: "start", behavior: "smooth" });
  }
});
document.addEventListener("click", (e) => {
  const l = e.target.closest && e.target.closest("a[href]");
  if (!l) return;
  const h = l.getAttribute("href");
  e.preventDefault();
  if (h.startsWith("#") && !h.startsWith("#chapter:")) {
    const t = document.getElementById(decodeURIComponent(h.slice(1)));
    if (t) t.scrollIntoView({ behavior: "smooth" });
  }
});
// Overlay scrollbar: shows while scrolling or near the right edge; drag the thumb, click the track to page.
(function () {
  const bar = document.createElement("div");
  bar.className = "bc-ov";
  const thumb = document.createElement("div");
  thumb.className = "bc-ov-thumb";
  bar.appendChild(thumb);
  document.body.appendChild(bar);
  const se = document.scrollingElement;
  let timer = 0;
  function measure() {
    const view = innerHeight, full = se.scrollHeight, track = view - 4;
    if (full <= view + 1) { bar.style.display = "none"; return; }
    bar.style.display = "";
    const size = Math.max(28, (track * view) / full);
    thumb.style.height = size + "px";
    thumb.style.transform = "translateY(" + ((track - size) * se.scrollTop) / (full - view) + "px)";
  }
  function wake() {
    bar.classList.add("on");
    clearTimeout(timer);
    timer = setTimeout(() => bar.classList.remove("on"), 900);
  }
  addEventListener("scroll", () => { measure(); wake(); }, { passive: true });
  addEventListener("resize", measure);
  new ResizeObserver(measure).observe(document.body);
  document.addEventListener("mousemove", (e) => { if (innerWidth - e.clientX < 28) wake(); });
  thumb.addEventListener("pointerdown", (e) => {
    e.preventDefault();
    thumb.setPointerCapture(e.pointerId);
    bar.classList.add("drag");
    const y0 = e.clientY, s0 = se.scrollTop;
    const ratio = (se.scrollHeight - innerHeight) / Math.max(1, innerHeight - 4 - thumb.offsetHeight);
    const move = (ev) => { se.scrollTop = s0 + (ev.clientY - y0) * ratio; };
    const up = () => {
      bar.classList.remove("drag");
      thumb.removeEventListener("pointermove", move);
      thumb.removeEventListener("pointerup", up);
      thumb.removeEventListener("pointercancel", up);
    };
    thumb.addEventListener("pointermove", move);
    thumb.addEventListener("pointerup", up);
    thumb.addEventListener("pointercancel", up);
  });
  bar.addEventListener("pointerdown", (e) => {
    if (e.target !== bar) return;
    const r = thumb.getBoundingClientRect();
    scrollBy({ top: (e.clientY < r.top ? -1 : 1) * innerHeight * 0.9, behavior: "smooth" });
  });
  measure();
})();
parent.postMessage({ type: "ready" }, "*");
<\/script>
</body>
</html>`;
}
