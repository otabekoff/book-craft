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
parent.postMessage({ type: "ready" }, "*");
<\/script>
</body>
</html>`;
}
