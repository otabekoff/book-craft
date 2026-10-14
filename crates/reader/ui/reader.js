/* Book Craft reader. Loads /book.json and renders chapters from the book. */
(function () {
  "use strict";

  const $ = (id) => document.getElementById(id);
  const state = {
    book: null,
    chapter: -1,
    chapterCache: new Map(),
    searchIndex: null,
    lastView: "home",
    saveTimer: 0,
  };
  let prefs = {};
  let storeKey = "bookcraft";

  /* ---------------------------------------------------------------- storage */

  function load(key, fallback) {
    try {
      const v = localStorage.getItem(storeKey + ":" + key);
      return v == null ? fallback : JSON.parse(v);
    } catch (e) {
      return fallback;
    }
  }
  function save(key, value) {
    try {
      localStorage.setItem(storeKey + ":" + key, JSON.stringify(value));
    } catch (e) {
      /* storage can be unavailable */
    }
  }

  /* ---------------------------------------------------------------- helpers */

  function esc(s) {
    return String(s == null ? "" : s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  }
  function el(tag, attrs, text) {
    const e = document.createElement(tag);
    if (attrs) for (const [k, v] of Object.entries(attrs)) if (v != null) e.setAttribute(k, v);
    if (text != null) e.textContent = text;
    return e;
  }
  function chapterNumberLabel(i) {
    const c = state.book.chapters[i];
    if (c.kind && c.kind !== "chapter") return "";
    let n = 0;
    for (let j = 0; j <= i; j++) if (!state.book.chapters[j].kind || state.book.chapters[j].kind === "chapter") n++;
    return String(n);
  }

  /* ---------------------------------------------------------------- theme & display */

  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");
  function effectiveTheme() {
    const t = prefs.theme || state.book.reader.defaultTheme || "light";
    if (t === "system") return systemDark.matches ? "dark" : "light";
    return t;
  }
  function applyDisplay() {
    const theme = effectiveTheme();
    document.documentElement.setAttribute("data-theme", theme);
    if (window.BookBlocks) window.BookBlocks.setTheme(theme, $("chapter"));
    const root = document.documentElement.style;
    if (prefs.size) root.setProperty("--bc-size", prefs.size + "px");
    else root.removeProperty("--bc-size");
    if (prefs.font) root.setProperty("--bc-font-body", prefs.font);
    else root.removeProperty("--bc-font-body");
    root.removeProperty("--bc-leading");
    if (prefs.leading === "compact") root.setProperty("--bc-leading", "1.45");
    if (prefs.leading === "relaxed") root.setProperty("--bc-leading", "1.9");
    root.removeProperty("--bc-measure");
    if (prefs.width === "narrow") root.setProperty("--bc-measure", "600px");
    if (prefs.width === "wide") root.setProperty("--bc-measure", "900px");

    document.querySelectorAll("[data-theme-choice]").forEach((b) =>
      b.setAttribute("aria-pressed", String((prefs.theme || state.book.reader.defaultTheme) === b.dataset.themeChoice))
    );
    document.querySelectorAll("[data-leading]").forEach((b) => b.setAttribute("aria-pressed", String((prefs.leading || "") === b.dataset.leading)));
    document.querySelectorAll("[data-width]").forEach((b) => b.setAttribute("aria-pressed", String((prefs.width || "") === b.dataset.width)));
    $("size-value").textContent = currentSize() + " px";
    $("font-select").value = prefs.font || "";
    renderSettings();
  }
  function currentSize() {
    return prefs.size || parseInt(getComputedStyle(document.documentElement).getPropertyValue("--bc-size"), 10) || 19;
  }
  function setPref(key, value) {
    if (value === "" || value == null) delete prefs[key];
    else prefs[key] = value;
    save("prefs", prefs);
    applyDisplay();
  }
  systemDark.addEventListener("change", applyDisplay);

  /* ---------------------------------------------------------------- views */

  function show(view) {
    ["home", "reader", "about"].forEach((v) => ($(v).hidden = v !== view));
    if (view !== "about") state.lastView = view;
    closePanels();
  }

  function route() {
    const hash = decodeURIComponent(location.hash.replace(/^#/, ""));
    if (hash === "about") {
      show("about");
      return;
    }
    if (hash === "back") {
      history.back();
      return;
    }
    const m = hash.match(/^read\/(\d+)(?:\/(.+))?$/);
    if (m) {
      openChapter(parseInt(m[1], 10), m[2] || null);
      return;
    }
    if (hash !== "home" && !state.book.reader.showHomeScreen) {
      // No home screen: go straight to the last place (or the start).
      const pos = state.book.reader.rememberPosition ? load("position", null) : null;
      location.replace("#read/" + (pos ? pos.chapter : 0));
      return;
    }
    renderHome();
    show("home");
  }

  /* ---------------------------------------------------------------- home */

  function renderCover(target) {
    const meta = state.book.meta;
    target.innerHTML = "";
    target.setAttribute("aria-label", "Cover of " + meta.title + (meta.author ? " by " + meta.author : ""));
    if (meta.cover) {
      target.classList.add("has-image");
      target.appendChild(el("img", { src: "/" + meta.cover.replace(/^\/+/, ""), alt: "" }));
      return;
    }
    const top = el("div");
    top.appendChild(el("div", { class: "cover-rule" }));
    top.appendChild(el("div", { class: "cover-title" }, meta.title));
    const bottom = el("div");
    if (meta.author) bottom.appendChild(el("div", { class: "cover-author" }, meta.author));
    if (meta.publisher) bottom.appendChild(el("div", { class: "cover-publisher" }, meta.publisher));
    target.append(top, bottom);
  }

  function renderHome() {
    const b = state.book;
    const meta = b.meta;
    renderCover($("cover"));
    $("home-eyebrow").textContent = [meta.publisher, meta.copyright && (meta.copyright.match(/\d{4}/) || [])[0]].filter(Boolean).join(" · ");
    $("home-title").textContent = meta.title;
    $("home-subtitle").textContent = meta.subtitle || "";
    $("home-subtitle").hidden = !meta.subtitle;
    $("home-author").textContent = meta.author ? "by " + meta.author : "";
    $("home-description").textContent = meta.description || "";
    $("home-description").hidden = !meta.description;
    $("home-copyright").textContent = [meta.copyright, meta.publisher && "Published by " + meta.publisher].filter(Boolean).join(" · ");

    const pos = b.reader.rememberPosition ? load("position", null) : null;
    const cont = $("home-continue");
    if (pos && pos.chapter < b.chapters.length && (pos.chapter > 0 || pos.ratio > 0.02)) {
      cont.href = "#read/" + pos.chapter;
      cont.querySelector(".btn-label").textContent = "Continue reading";
      $("home-continue-sub").textContent = b.chapters[pos.chapter].title;
      $("home-start").hidden = false;
      const pct = overallPercent(pos.chapter, pos.ratio || 0);
      $("home-progress-wrap").hidden = !b.reader.showProgress;
      $("home-progress").style.width = pct + "%";
      $("home-progress-text").textContent = pct + "% read · " + b.chapters.length + " chapters";
    } else {
      cont.href = "#read/0";
      cont.querySelector(".btn-label").textContent = "Start reading";
      $("home-continue-sub").textContent = minutes(b.words) ;
      $("home-start").hidden = true;
      $("home-progress-wrap").hidden = true;
    }

    const list = $("home-toc");
    list.innerHTML = "";
    b.chapters.forEach((c, i) => {
      const li = el("li");
      const a = el("a", { href: "#read/" + i });
      a.appendChild(el("span", { class: "num" }, chapterNumberLabel(i)));
      a.appendChild(el("span", null, c.title));
      li.appendChild(a);
      list.appendChild(li);
    });
  }

  function minutes(words) {
    if (!words) return "";
    const m = Math.max(1, Math.round(words / 230));
    return m < 60 ? "About " + m + " min" : "About " + Math.round(m / 6) / 10 + " h";
  }

  /* ---------------------------------------------------------------- reader */

  function renderToc() {
    const list = $("toc-list");
    list.innerHTML = "";
    state.book.chapters.forEach((c, i) => {
      const li = el("li", { class: "kind-" + (c.kind || "chapter") });
      const a = el("a", { href: "#read/" + i, "data-index": i });
      a.appendChild(el("span", { class: "num" }, chapterNumberLabel(i)));
      a.appendChild(el("span", null, c.title));
      li.appendChild(a);
      list.appendChild(li);
      if (c.headings && c.headings.length) {
        const sub = el("ol", { class: "sub", "data-for": i, hidden: "" });
        c.headings
          .filter((h) => h.level === 2)
          .forEach((h) => {
            const sli = el("li");
            sli.appendChild(el("a", { href: "#read/" + i + "/" + encodeURIComponent(h.id) }, h.text));
            sub.appendChild(sli);
          });
        list.appendChild(sub);
      }
    });
    $("toc-book").textContent = state.book.meta.title;
  }

  async function fetchChapter(i) {
    if (state.chapterCache.has(i)) return state.chapterCache.get(i);
    const res = await fetch("/" + state.book.chapters[i].file);
    const html = res.ok ? await res.text() : "<p>This chapter could not be loaded.</p>";
    state.chapterCache.set(i, html);
    return html;
  }

  async function openChapter(i, anchor, opts) {
    const b = state.book;
    if (i < 0 || i >= b.chapters.length) i = 0;
    show("reader");
    const article = $("chapter");
    const scroller = $("scroller");
    if (state.chapter !== i) {
      saveNow();
      state.loading = true;
      state.chapter = i;
      article.className = b.chapterClass || "bc-chapter";
      article.innerHTML = await fetchChapter(i);
      article.setAttribute("lang", b.meta.language || "en");
      if (window.BookBlocks) {
        window.BookBlocks.hydrate(article, { vendor: "/_runtime/vendor/", theme: effectiveTheme() });
      }
      const pos = load("position", null);
      if (!anchor && pos && pos.chapter === i && b.reader.rememberPosition && !(opts && opts.top)) {
        requestAnimationFrame(() => (scroller.scrollTop = pos.ratio * (scroller.scrollHeight - scroller.clientHeight)));
      } else {
        scroller.scrollTop = 0;
      }
      requestAnimationFrame(() => (state.loading = false));
    }
    if (anchor) {
      const target = document.getElementById(anchor);
      if (target) requestAnimationFrame(() => target.scrollIntoView({ block: "start" }));
    }
    $("toolbar-title").textContent = b.chapters[i].title;
    document.title = b.chapters[i].title + " — " + b.meta.title;
    document.querySelectorAll("#toc-list > li > a").forEach((a) => {
      if (Number(a.dataset.index) === i) a.setAttribute("aria-current", "page");
      else a.removeAttribute("aria-current");
    });
    document.querySelectorAll("#toc-list .sub").forEach((s) => (s.hidden = Number(s.dataset.for) !== i));
    $("prev").disabled = i === 0;
    $("next").disabled = i === b.chapters.length - 1;
    $("next").textContent = i === b.chapters.length - 1 ? "The end" : "Next chapter →";
    updateProgress();
    if (state.pendingSearch) {
      highlightSearch(state.pendingSearch);
      state.pendingSearch = null;
    }
    if (window.innerWidth < 760) $("reader").classList.add("toc-hidden-narrow");
  }

  function scrollRatio() {
    const s = $("scroller");
    const max = s.scrollHeight - s.clientHeight;
    return max > 0 ? Math.min(1, Math.max(0, s.scrollTop / max)) : 0;
  }
  function overallPercent(chapter, ratio) {
    const cs = state.book.chapters;
    const total = cs.reduce((n, c) => n + Math.max(1, c.words), 0);
    let done = 0;
    for (let j = 0; j < chapter; j++) done += Math.max(1, cs[j].words);
    done += Math.max(1, cs[chapter].words) * ratio;
    return Math.round((done / total) * 100);
  }
  function updateProgress() {
    if (state.chapter < 0) return;
    const pct = overallPercent(state.chapter, scrollRatio());
    $("progress-chapter").textContent = "Chapter " + (state.chapter + 1) + " of " + state.book.chapters.length;
    $("progress-fill").style.width = pct + "%";
    $("progress-pct").textContent = pct + "% read";
  }
  function saveNow() {
    if (state.chapter < 0 || state.loading || !state.book.reader.rememberPosition) return;
    save("position", { chapter: state.chapter, ratio: scrollRatio(), at: Date.now() });
  }

  /* ---------------------------------------------------------------- links & viewer */

  function chapterIndexForSource(src) {
    return state.book.chapters.findIndex((c) => c.source === src);
  }

  function openViewer(href, title) {
    $("viewer-title").textContent = title || href.split("/").pop();
    $("viewer-frame").src = href;
    $("viewer").hidden = false;
    $("viewer-close").focus();
  }
  function closeViewer() {
    $("viewer").hidden = true;
    $("viewer-frame").src = "about:blank";
  }

  document.addEventListener("click", (e) => {
    const a = e.target.closest && e.target.closest("a[href]");
    if (!a) return;
    const href = a.getAttribute("href");
    if (href.startsWith("#chapter:")) {
      e.preventDefault();
      const [src, frag] = href.slice(9).split("#");
      const i = chapterIndexForSource(src);
      if (i >= 0) location.hash = "#read/" + i + (frag ? "/" + encodeURIComponent(frag) : "");
      return;
    }
    if (a.closest("#chapter") && href.startsWith("#") && !href.startsWith("#read/")) {
      e.preventDefault();
      const t = document.getElementById(decodeURIComponent(href.slice(1)));
      if (t) t.scrollIntoView({ block: "start", behavior: "smooth" });
      return;
    }
    if (a.dataset.viewer || /^\/assets\/.+\.(pdf|txt|png|jpe?g|gif|webp|svg|mp4|webm|mp3|wav|ogg)$/i.test(href)) {
      e.preventDefault();
      openViewer(href, a.textContent.trim());
    }
  });

  /* ---------------------------------------------------------------- search */

  async function buildIndex() {
    if (state.searchIndex) return state.searchIndex;
    $("search-status").textContent = "Preparing search…";
    const index = [];
    for (let i = 0; i < state.book.chapters.length; i++) {
      const html = await fetchChapter(i);
      const div = document.createElement("div");
      div.innerHTML = html;
      div.querySelectorAll("script, style, .bc-code-tools").forEach((n) => n.remove());
      index.push(div.textContent.replace(/\s+/g, " "));
    }
    state.searchIndex = index;
    return index;
  }

  let searchTimer = 0;
  async function runSearch(q) {
    const results = $("search-results");
    results.innerHTML = "";
    q = q.trim();
    if (q.length < 2) {
      $("search-status").textContent = "";
      return;
    }
    const index = await buildIndex();
    const needle = q.toLocaleLowerCase();
    let count = 0;
    index.forEach((text, i) => {
      const lower = text.toLocaleLowerCase();
      let at = lower.indexOf(needle);
      let perChapter = 0;
      while (at >= 0 && perChapter < 20 && count < 200) {
        const start = Math.max(0, at - 50);
        const end = Math.min(text.length, at + needle.length + 70);
        const li = el("li");
        const a = el("a", { href: "#read/" + i });
        a.innerHTML =
          '<span class="where">' + esc(state.book.chapters[i].title) + "</span>" +
          (start > 0 ? "…" : "") + esc(text.slice(start, at)) + "<mark>" + esc(text.slice(at, at + needle.length)) + "</mark>" +
          esc(text.slice(at + needle.length, end)) + (end < text.length ? "…" : "");
        const occurrence = perChapter;
        a.addEventListener("click", (e) => {
          e.preventDefault();
          state.pendingSearch = { q, occurrence };
          if (state.chapter === i) {
            highlightSearch(state.pendingSearch);
            state.pendingSearch = null;
          } else location.hash = "#read/" + i;
        });
        li.appendChild(a);
        results.appendChild(li);
        count++;
        perChapter++;
        at = lower.indexOf(needle, at + needle.length);
      }
    });
    $("search-status").textContent = count ? count + (count >= 200 ? "+" : "") + " result" + (count === 1 ? "" : "s") : "No results";
  }

  function highlightSearch({ q, occurrence }) {
    if (!window.CSS || !CSS.highlights) return;
    const article = $("chapter");
    const walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
    const needle = q.toLocaleLowerCase();
    const ranges = [];
    let node;
    while ((node = walker.nextNode())) {
      const text = node.nodeValue.toLocaleLowerCase();
      let at = text.indexOf(needle);
      while (at >= 0) {
        const r = new Range();
        r.setStart(node, at);
        r.setEnd(node, at + needle.length);
        ranges.push(r);
        at = text.indexOf(needle, at + needle.length);
      }
    }
    CSS.highlights.set("bc-search", new Highlight(...ranges));
    const target = ranges[Math.min(occurrence || 0, ranges.length - 1)];
    if (target) {
      const rect = target.getBoundingClientRect();
      const s = $("scroller");
      s.scrollTop += rect.top - s.getBoundingClientRect().top - s.clientHeight / 3;
    }
  }

  /* ---------------------------------------------------------------- bookmarks */

  function bookmarks() {
    return load("bookmarks", []);
  }
  function renderBookmarks() {
    const list = $("bookmark-list");
    list.innerHTML = "";
    const all = bookmarks();
    if (!all.length) {
      list.appendChild(el("li", { class: "side-status" }, "No bookmarks yet."));
      return;
    }
    all.forEach((bm, n) => {
      const li = el("li");
      const a = el("a", { href: "#read/" + bm.chapter });
      a.innerHTML = '<span class="where">' + esc((state.book.chapters[bm.chapter] || {}).title || "") + "</span>" + esc(bm.text || "");
      a.addEventListener("click", (e) => {
        e.preventDefault();
        closePanels();
        save("position", { chapter: bm.chapter, ratio: bm.ratio, at: Date.now() });
        if (state.chapter === bm.chapter) {
          const s = $("scroller");
          s.scrollTop = bm.ratio * (s.scrollHeight - s.clientHeight);
        } else {
          state.chapter = -1;
          location.hash = "#read/" + bm.chapter;
          openChapter(bm.chapter);
        }
      });
      const rm = el("button", { type: "button", class: "remove", "aria-label": "Remove bookmark" }, "✕");
      rm.addEventListener("click", () => {
        const next = bookmarks();
        next.splice(n, 1);
        save("bookmarks", next);
        renderBookmarks();
      });
      li.append(a, rm);
      list.appendChild(li);
    });
  }
  function addBookmark() {
    const s = $("scroller");
    const top = s.getBoundingClientRect().top + 40;
    let text = "";
    const blocks = $("chapter").querySelectorAll("p, h1, h2, h3, li");
    for (const b of blocks) {
      if (b.getBoundingClientRect().bottom > top) {
        text = b.textContent.trim().slice(0, 140);
        break;
      }
    }
    const all = bookmarks();
    all.unshift({ chapter: state.chapter, ratio: scrollRatio(), text, at: Date.now() });
    save("bookmarks", all.slice(0, 200));
    renderBookmarks();
  }

  /* ---------------------------------------------------------------- about & settings */

  function renderAbout() {
    const m = state.book.meta;
    const rows = [
      ["Title", m.title],
      ["Subtitle", m.subtitle],
      ["Author", m.author],
      ["Publisher", m.publisher],
      ["Language", m.language],
      ["ISBN", m.isbn],
      ["Edition", m.version],
      ["Package ID", m.packageId, "mono"],
      ["Chapters", String(state.book.chapters.length)],
      ["Words", state.book.words ? state.book.words.toLocaleString() : ""],
    ];
    const dl = $("about-details");
    dl.innerHTML = "";
    rows.filter((r) => r[1]).forEach(([k, v, cls]) => {
      dl.appendChild(el("dt", null, k));
      dl.appendChild(el("dd", cls ? { class: cls } : null, v));
    });
    if (m.website) {
      dl.appendChild(el("dt", null, "Website"));
      const dd = el("dd");
      dd.appendChild(el("a", { href: m.website }, m.website));
      dl.appendChild(dd);
    }
    $("about-copyright").textContent = m.copyright || "";
    $("about-rights").textContent = m.rights || "All rights reserved.";
  }

  function renderSettings() {
    const r = state.book.reader;
    const body = $("about-settings-body");
    if (!body) return;
    body.innerHTML = "";
    if (r.allowThemeChange) {
      const row = el("div", { class: "setting" });
      row.appendChild(el("span", { class: "pop-label" }, "Theme"));
      const seg = el("div", { class: "seg seg-text", role: "group", "aria-label": "Theme" });
      [["light", "Light"], ["sepia", "Sepia"], ["dark", "Dark"], ["system", "Match Windows"]].forEach(([v, label]) => {
        const b = el("button", { type: "button", "aria-pressed": String((prefs.theme || r.defaultTheme) === v) }, label);
        b.addEventListener("click", () => setPref("theme", v));
        seg.appendChild(b);
      });
      row.appendChild(seg);
      body.appendChild(row);
    }
    if (r.allowFontSizeChange) {
      const row = el("div", { class: "setting" });
      row.appendChild(el("label", { for: "about-size" }, "Text size: " + currentSize() + " px"));
      const input = el("input", { id: "about-size", type: "range", min: "14", max: "30", value: String(currentSize()) });
      input.style.accentColor = "var(--bc-accent)";
      input.addEventListener("change", () => setPref("size", parseInt(input.value, 10)));
      row.appendChild(input);
      body.appendChild(row);
    }
    if (r.rememberPosition) {
      const label = el("label", { class: "check" });
      const box = el("input", { type: "checkbox" });
      box.checked = load("remember", true);
      box.addEventListener("change", () => save("remember", box.checked));
      label.append(box, document.createTextNode("Remember my place between sessions"));
      body.appendChild(label);
    }
    const reset = el("button", { type: "button", class: "btn btn-secondary" }, "Reset reading settings");
    reset.addEventListener("click", () => {
      prefs = {};
      save("prefs", prefs);
      applyDisplay();
    });
    body.appendChild(reset);
  }

  /* ---------------------------------------------------------------- panels & keys */

  function closePanels() {
    ["display-panel", "search-panel", "bookmark-panel"].forEach((p) => ($(p).hidden = true));
    $("btn-display").setAttribute("aria-expanded", "false");
  }
  function toggle(panel) {
    const open = $(panel).hidden;
    closePanels();
    $(panel).hidden = !open;
    if (panel === "display-panel") $("btn-display").setAttribute("aria-expanded", String(open));
    if (open && panel === "search-panel") $("search-input").focus();
    if (open && panel === "bookmark-panel") renderBookmarks();
  }

  function wire() {
    const r = state.book.reader;
    $("btn-toc").addEventListener("click", () => {
      const reader = $("reader");
      if (window.innerWidth < 760) reader.classList.toggle("toc-hidden-narrow");
      else {
        reader.classList.toggle("toc-hidden");
        save("tocHidden", reader.classList.contains("toc-hidden"));
      }
    });
    $("btn-display").addEventListener("click", () => toggle("display-panel"));
    $("btn-search").addEventListener("click", () => toggle("search-panel"));
    $("btn-bookmark").addEventListener("click", () => toggle("bookmark-panel"));
    $("add-bookmark").addEventListener("click", addBookmark);
    document.querySelectorAll("[data-close]").forEach((b) => b.addEventListener("click", () => ($(b.dataset.close).hidden = true)));
    $("prev").addEventListener("click", () => (location.hash = "#read/" + Math.max(0, state.chapter - 1)));
    $("next").addEventListener("click", () => {
      if (state.chapter < state.book.chapters.length - 1) location.hash = "#read/" + (state.chapter + 1);
    });
    document.querySelectorAll("[data-theme-choice]").forEach((b) => b.addEventListener("click", () => setPref("theme", b.dataset.themeChoice)));
    document.querySelectorAll("[data-leading]").forEach((b) => b.addEventListener("click", () => setPref("leading", b.dataset.leading)));
    document.querySelectorAll("[data-width]").forEach((b) => b.addEventListener("click", () => setPref("width", b.dataset.width)));
    $("size-down").addEventListener("click", () => setPref("size", Math.max(14, currentSize() - 1)));
    $("size-up").addEventListener("click", () => setPref("size", Math.min(30, currentSize() + 1)));
    $("font-select").addEventListener("change", (e) => setPref("font", e.target.value));
    $("search-input").addEventListener("input", (e) => {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => runSearch(e.target.value), 180);
    });
    $("viewer-close").addEventListener("click", closeViewer);
    $("about-back").addEventListener("click", (e) => {
      e.preventDefault();
      if (state.lastView === "reader" && state.chapter >= 0) location.hash = "#read/" + state.chapter;
      else location.hash = "#home";
    });
    $("scroller").addEventListener("scroll", () => {
      updateProgress();
      clearTimeout(state.saveTimer);
      state.saveTimer = setTimeout(saveNow, 400);
    });
    window.addEventListener("beforeunload", saveNow);
    window.addEventListener("hashchange", route);
    document.addEventListener("keydown", (e) => {
      const typing = /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement && document.activeElement.tagName);
      if (e.key === "Escape") {
        if (!$("viewer").hidden) closeViewer();
        else closePanels();
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === "f" && r.enableSearch) {
        e.preventDefault();
        if ($("reader").hidden) location.hash = "#read/" + Math.max(0, state.chapter);
        toggle("search-panel");
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === "d" && r.enableBookmarks && !$("reader").hidden) {
        e.preventDefault();
        toggle("bookmark-panel");
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === "b") {
        e.preventDefault();
        $("btn-toc").click();
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === "p" && !r.allowPrint) {
        e.preventDefault();
        return;
      }
      if (e.ctrlKey && (e.key === "=" || e.key === "+") && r.allowFontSizeChange) {
        e.preventDefault();
        setPref("size", Math.min(30, currentSize() + 1));
        return;
      }
      if (e.ctrlKey && e.key === "-" && r.allowFontSizeChange) {
        e.preventDefault();
        setPref("size", Math.max(14, currentSize() - 1));
        return;
      }
      if (typing || $("reader").hidden) return;
      if (e.key === "ArrowRight" && !e.altKey && state.chapter < state.book.chapters.length - 1) {
        location.hash = "#read/" + (state.chapter + 1);
      } else if (e.key === "ArrowLeft" && !e.altKey && state.chapter > 0) {
        location.hash = "#read/" + (state.chapter - 1);
      }
    });
    if (!r.allowCopy) {
      document.body.classList.add("no-copy");
      document.addEventListener("copy", (e) => {
        if (e.target.closest && e.target.closest("#chapter") && !e.target.closest(".bc-code")) e.preventDefault();
      });
    }
    if (!r.allowPrint) window.addEventListener("beforeprint", (e) => e.preventDefault());

    $("btn-search").hidden = !r.enableSearch;
    $("btn-bookmark").hidden = !r.enableBookmarks;
    $("btn-home").hidden = !r.showHomeScreen;
    $("row-theme").hidden = !r.allowThemeChange;
    $("row-size").hidden = !r.allowFontSizeChange;
    $("row-font").hidden = !r.allowFontChange;
    $("progress-bar").hidden = !r.showProgress;
    if (!r.showToc) $("reader").classList.add("toc-hidden");
    else if (load("tocHidden", false)) $("reader").classList.add("toc-hidden");
  }

  /* ---------------------------------------------------------------- start */

  async function start() {
    try {
      const res = await fetch("/book.json");
      state.book = await res.json();
    } catch (e) {
      document.body.textContent = "This book could not be opened.";
      return;
    }
    storeKey = "bookcraft:" + (state.book.meta.packageId || "book");
    prefs = load("prefs", {});
    if (!load("remember", true)) state.book.reader.rememberPosition = false;
    document.title = state.book.meta.title;
    document.documentElement.lang = state.book.meta.language || "en";
    wire();
    applyDisplay();
    renderToc();
    renderAbout();
    $("app").removeAttribute("aria-busy");
    route();
  }

  start();
})();
