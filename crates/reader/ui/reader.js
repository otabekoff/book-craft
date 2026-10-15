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
    focus: false,
    fullscreen: false,
    openAtEnd: false,
    lastWheel: 0,
  };
  // Page mode: the chapter is laid out in CSS columns inside a fixed frame, one column per page.
  const pager = { page: 0, pages: 1, cols: 1, n: 1, pw: 0, gap: 0, step: 0, padX: 0, padTop: 0, h: 0, flip: null };
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
    const width = currentWidth();
    if (width === "narrow") root.setProperty("--bc-measure", "600px");
    if (width === "wide") root.setProperty("--bc-measure", "900px");
    if (width === "full") root.setProperty("--bc-measure", "none");
    const reader = $("reader");
    reader.classList.toggle("paged", isPaged());
    reader.classList.toggle("paper", currentPaper() === "on");
    reader.classList.toggle("width-full", width === "full");
    $("row-spread").hidden = !isPaged();
    $("row-turn").hidden = !isPaged();

    document.querySelectorAll("[data-theme-choice]").forEach((b) =>
      b.setAttribute("aria-pressed", String((prefs.theme || state.book.reader.defaultTheme) === b.dataset.themeChoice))
    );
    document.querySelectorAll("[data-leading]").forEach((b) => b.setAttribute("aria-pressed", String((prefs.leading || "") === b.dataset.leading)));
    document.querySelectorAll("[data-width]").forEach((b) => b.setAttribute("aria-pressed", String(width === b.dataset.width)));
    document.querySelectorAll("[data-layout]").forEach((b) => b.setAttribute("aria-pressed", String(currentLayout() === b.dataset.layout)));
    document.querySelectorAll("[data-spread]").forEach((b) => b.setAttribute("aria-pressed", String(currentSpread() === b.dataset.spread)));
    document.querySelectorAll("[data-paper]").forEach((b) => b.setAttribute("aria-pressed", String(currentPaper() === b.dataset.paper)));
    document.querySelectorAll("[data-turn]").forEach((b) => b.setAttribute("aria-pressed", String(currentTurn() === b.dataset.turn)));
    $("size-value").textContent = currentSize() + " px";
    renderFontList();
    renderSettings();
  }
  // The book's own body font first, then the fonts the author lets readers pick.
  function renderFontList() {
    const list = $("font-list");
    if (!list) return;
    const choices = [{ key: "", name: "Book’s font", stack: "" }].concat(state.book.fontChoices || []);
    list.innerHTML = "";
    choices.forEach((f) => {
      const selected = (prefs.font || "") === f.stack;
      const b = el("button", { type: "button", role: "radio", "aria-checked": String(selected), class: "font-option" });
      const name = el("span", { class: "font-name" }, f.name);
      name.style.fontFamily = f.stack || "var(--bc-font-body)";
      const sample = el("span", { class: "font-sample" }, "Aa");
      sample.style.fontFamily = f.stack || "var(--bc-font-body)";
      b.append(sample, name);
      if (selected) b.insertAdjacentHTML("beforeend", (window.ReaderIcons || {}).check || "");
      b.addEventListener("click", () => setPref("font", f.stack));
      list.appendChild(b);
    });
  }

  function currentSize() {
    return prefs.size || parseInt(getComputedStyle(document.documentElement).getPropertyValue("--bc-size"), 10) || 19;
  }
  // Reader choices fall back to the author's defaults.
  function currentWidth() {
    return prefs.width || state.book.reader.defaultWidth || "normal";
  }
  function currentLayout() {
    return prefs.layout || state.book.reader.defaultLayout || "scroll";
  }
  function currentSpread() {
    return prefs.spread || state.book.reader.pageSpread || "auto";
  }
  function currentPaper() {
    return prefs.paper || (state.book.reader.paperLook ? "on" : "off");
  }
  function currentTurn() {
    return prefs.turn || state.book.reader.pageTurn || "slide";
  }
  function isPaged() {
    return currentLayout() === "pages";
  }
  function setPref(key, value) {
    const ratio = positionRatio();
    if (value === "" || value == null) delete prefs[key];
    else prefs[key] = value;
    save("prefs", prefs);
    applyDisplay();
    relayout(ratio);
    // Web fonts can arrive after the change; lay the pages out again once they have.
    if (document.fonts && isPaged()) document.fonts.ready.then(() => relayout(ratio));
  }
  systemDark.addEventListener("change", applyDisplay);

  /* ---------------------------------------------------------------- views */

  function show(view) {
    ["home", "reader", "about"].forEach((v) => ($(v).hidden = v !== view));
    if (view === "home") setSubtitle("");
    if (view === "about") setSubtitle("About this book");
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
      const atEnd = state.openAtEnd;
      state.openAtEnd = false;
      let ratio = 0;
      if (!anchor && pos && pos.chapter === i && b.reader.rememberPosition && !(opts && opts.top)) ratio = pos.ratio;
      scroller.scrollTop = 0;
      if (isPaged()) {
        paginate();
        setRatio(atEnd ? 1 : ratio, false);
        settlePages(i, atEnd ? 1 : ratio);
      } else if (ratio) {
        requestAnimationFrame(() => setRatio(ratio));
      }
      requestAnimationFrame(() => (state.loading = false));
    }
    if (anchor) {
      const target = document.getElementById(anchor);
      if (target) requestAnimationFrame(() => reveal(target));
    }
    $("toolbar-title").textContent = b.chapters[i].title;
    document.title = b.chapters[i].title + " — " + b.meta.title;
    setSubtitle(b.chapters[i].title);
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
  /** How far through the chapter the reader is, 0-1, in either layout. */
  function positionRatio() {
    if (state.chapter < 0) return 0;
    if ($("reader").classList.contains("paged")) return pager.pages > 1 ? pager.page / (pager.pages - 1) : 0;
    return scrollRatio();
  }
  function setRatio(ratio, animate) {
    if (isPaged()) {
      countPages();
      goto(Math.round((ratio || 0) * (pager.pages - 1)), animate);
    } else {
      const s = $("scroller");
      s.scrollTop = (ratio || 0) * (s.scrollHeight - s.clientHeight);
    }
  }
  /** Bring an element (or a Range) into view: scroll to it, or turn to its page. */
  function reveal(target, where) {
    if (!isPaged()) {
      if (where === "third") {
        const s = $("scroller");
        s.scrollTop += target.getBoundingClientRect().top - s.getBoundingClientRect().top - s.clientHeight / 3;
      } else target.scrollIntoView({ block: "start", behavior: where === "smooth" ? "smooth" : "auto" });
      return;
    }
    const left = target.getBoundingClientRect().left - $("chapter").getBoundingClientRect().left;
    countPages();
    goto(Math.floor((left + 1) / pager.step), true);
  }

  /* ---------------------------------------------------------------- page mode */

  function pagePadding() {
    const narrow = window.innerWidth < 760;
    return { x: narrow ? 24 : 48, top: narrow ? 28 : 44, bottom: 52 };
  }
  function clearPaging() {
    const article = $("chapter");
    $("page-frame").removeAttribute("style");
    ["width", "height", "columns", "column-gap", "transform", "--page-h"].forEach((k) => article.style.removeProperty(k));
    article.classList.remove("turning");
    finishFlip();
    $("reader").classList.remove("spread-2");
    $("page-number").textContent = "";
  }
  /** Size the frame and columns for the current window. Keeps the page index (clamped). */
  function paginate() {
    const scroller = $("scroller");
    const frame = $("page-frame");
    const article = $("chapter");
    const pad = pagePadding();
    const outer = currentPaper() === "on" ? (window.innerWidth < 760 ? 10 : 24) : 0;
    const gap = pad.x * 2;
    const availW = scroller.clientWidth - outer * 2 - pad.x * 2;
    const availH = scroller.clientHeight - outer * 2;
    const measure = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--bc-measure")) || Infinity;
    const spread = currentSpread();
    let n = spread === "two" ? 2 : spread === "one" ? 1 : availW >= 2 * 480 + gap && scroller.clientWidth > scroller.clientHeight ? 2 : 1;
    if (n === 2 && (availW - gap) / 2 < 280) n = 1;
    const pw = Math.floor(Math.max(200, n === 2 ? Math.min(measure, (availW - gap) / 2) : Math.min(measure, availW)));
    const contentW = n * pw + (n - 1) * gap;
    const contentH = Math.max(160, availH - pad.top - pad.bottom);
    frame.style.width = contentW + pad.x * 2 + "px";
    frame.style.height = availH + "px";
    frame.style.padding = pad.top + "px " + pad.x + "px " + pad.bottom + "px";
    article.style.width = contentW + "px";
    article.style.height = contentH + "px";
    article.style.columns = n + " " + pw + "px";
    article.style.columnGap = gap + "px";
    article.style.setProperty("--page-h", contentH + "px");
    $("reader").classList.toggle("spread-2", n === 2);
    Object.assign(pager, { n, pw, gap, step: contentW + gap, padX: pad.x, padTop: pad.top, h: contentH });
    countPages();
    goto(pager.page, false);
  }
  function countPages() {
    const article = $("chapter");
    pager.cols = Math.max(1, Math.round((article.scrollWidth + pager.gap) / (pager.pw + pager.gap)));
    pager.pages = Math.max(1, Math.ceil(pager.cols / pager.n));
  }
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  function goto(page, animate) {
    const article = $("chapter");
    const from = pager.page;
    pager.page = Math.max(0, Math.min(pager.pages - 1, page || 0));
    const style = animate && !reducedMotion.matches ? currentTurn() : "none";
    finishFlip();
    if (style === "flip" && Math.abs(pager.page - from) === 1) flip(from, pager.page);
    article.classList.toggle("turning", style === "slide");
    article.style.transform = "translateX(" + -pager.page * pager.step + "px)";
    const first = pager.page * pager.n + 1;
    const last = Math.min(pager.cols, first + pager.n - 1);
    $("page-number").textContent = (first === last ? first : first + "–" + last) + " of " + pager.cols;
    updateProgress();
    clearTimeout(state.saveTimer);
    state.saveTimer = setTimeout(saveNow, 400);
  }
  /** Next or previous page, crossing into the neighbouring chapter at either end. */
  function turn(dir) {
    if (state.chapter < 0) return;
    countPages();
    const count = state.book.chapters.length;
    if (dir > 0) {
      if (pager.page < pager.pages - 1) goto(pager.page + 1, true);
      else if (state.chapter < count - 1) location.hash = "#read/" + (state.chapter + 1);
    } else if (pager.page > 0) goto(pager.page - 1, true);
    else if (state.chapter > 0) {
      state.openAtEnd = true;
      location.hash = "#read/" + (state.chapter - 1);
    }
  }
  /** Math, diagrams and fonts finish after the first layout; recount a few times while the page is still untouched. */
  function settlePages(chapter, ratio) {
    let expect = pager.page;
    [150, 600, 1500].forEach((ms) =>
      setTimeout(() => {
        if (state.chapter !== chapter || !isPaged() || pager.page !== expect) return;
        countPages();
        goto(Math.round(ratio * (pager.pages - 1)), false);
        expect = pager.page;
      }, ms)
    );
  }
  let relayoutFrame = 0;
  function relayout(ratio) {
    if (state.chapter < 0 || $("reader").hidden) return;
    if (ratio == null) ratio = positionRatio();
    cancelAnimationFrame(relayoutFrame);
    relayoutFrame = requestAnimationFrame(() => {
      if (isPaged()) {
        paginate();
        setRatio(ratio, false);
      } else {
        clearPaging();
        setRatio(ratio);
      }
    });
  }
  /* A real-book turn: the leaf rotates around the spine (two pages) or the page edge (one page),
     drawn with copies of the pages. The chapter itself jumps at once and is uncovered as the leaf moves. */
  function pageCopy(col, x) {
    const box = document.createElement("div");
    box.className = "flip-page";
    box.style.cssText = "left:" + x + "px;top:" + pager.padTop + "px;width:" + pager.pw + "px;height:" + pager.h + "px";
    if (col >= 0 && col < pager.cols) {
      const a = $("chapter");
      const c = a.cloneNode(true);
      c.removeAttribute("id");
      c.removeAttribute("tabindex");
      c.classList.remove("turning");
      c.style.transform = "translateX(" + -col * (pager.pw + pager.gap) + "px)";
      // Canvases lose their drawing when copied.
      const src = a.querySelectorAll("canvas");
      c.querySelectorAll("canvas").forEach((d, i) => {
        try {
          d.getContext("2d").drawImage(src[i], 0, 0);
        } catch (e) {
          /* tainted or empty */
        }
      });
      box.appendChild(c);
    }
    return box;
  }
  function sheet(cls, left, width, height) {
    const d = document.createElement("div");
    d.className = cls;
    d.style.left = left + "px";
    d.style.width = width + "px";
    d.style.height = height + "px";
    return d;
  }
  function flip(from, to) {
    const frame = $("page-frame");
    const { n, pw, gap, padX } = pager;
    const fwd = to > from;
    const height = pager.padTop + pager.h + 12;
    const layer = document.createElement("div");
    layer.className = "flip-layer";
    layer.setAttribute("aria-hidden", "true");
    layer.style.cssText = "left:" + frame.offsetLeft + "px;top:" + frame.offsetTop + "px;width:" + frame.offsetWidth + "px;height:" + height + "px";
    let leaf;
    let frames;
    if (n === 2) {
      const half = pw + gap / 2;
      const spine = padX + half;
      // The page that does not move stays covered by its old content until the leaf lands on it.
      const still = sheet("flip-still", fwd ? 0 : spine, spine, height);
      still.appendChild(pageCopy(fwd ? from * 2 : from * 2 + 1, fwd ? padX : gap / 2));
      leaf = sheet(fwd ? "flip-leaf" : "flip-leaf rev", fwd ? spine : spine - half, half, height);
      leaf.style.transformOrigin = fwd ? "left center" : "right center";
      const front = sheet("flip-face", 0, half, height);
      front.appendChild(pageCopy(fwd ? from * 2 + 1 : from * 2, fwd ? gap / 2 : 0));
      const back = sheet("flip-face back", 0, half, height);
      back.appendChild(pageCopy(fwd ? to * 2 : to * 2 + 1, fwd ? 0 : gap / 2));
      leaf.append(front, back);
      layer.append(still, leaf);
      frames = [{ transform: "rotateY(0deg)" }, { transform: "rotateY(" + (fwd ? -180 : 180) + "deg)" }];
    } else {
      // One page: the old page swings away to the left, or the previous one swings back in over it.
      const w = frame.offsetWidth;
      leaf = sheet("flip-leaf single", 0, w, height);
      leaf.style.transformOrigin = "left center";
      leaf.appendChild(pageCopy(fwd ? from : to, padX));
      if (!fwd) {
        const still = sheet("flip-still", 0, w, height);
        still.appendChild(pageCopy(from, padX));
        layer.appendChild(still);
      }
      layer.appendChild(leaf);
      frames = fwd ? [{ transform: "rotateY(0deg)" }, { transform: "rotateY(-100deg)" }] : [{ transform: "rotateY(-100deg)" }, { transform: "rotateY(0deg)" }];
    }
    $("scroller").appendChild(layer);
    const anim = leaf.animate(frames, { duration: n === 2 ? 700 : 520, easing: fwd || n === 2 ? "cubic-bezier(0.45, 0.05, 0.3, 1)" : "cubic-bezier(0.2, 0.7, 0.3, 1)" });
    pager.flip = { layer, anim };
    anim.finished.then(
      () => pager.flip && pager.flip.anim === anim && finishFlip(),
      () => {}
    );
  }
  function finishFlip() {
    const f = pager.flip;
    if (!f) return;
    pager.flip = null;
    f.anim.cancel();
    f.layer.remove();
  }
  function scrollsItself(node, dy) {
    for (let n = node; n && n.id !== "chapter"; n = n.parentElement) {
      if (n.nodeType !== 1) continue;
      const oy = getComputedStyle(n).overflowY;
      if ((oy === "auto" || oy === "scroll") && n.scrollHeight > n.clientHeight + 1) {
        if ((dy > 0 && n.scrollTop + n.clientHeight < n.scrollHeight - 1) || (dy < 0 && n.scrollTop > 0)) return true;
      }
    }
    return false;
  }
  const INTERACTIVE = "a, button, input, textarea, select, summary, label, details, video, audio, iframe, canvas, [role=button], [contenteditable], .bc-code, .bc-flashcard, .bc-quiz";
  /** True over the page margins, the gutter or the desk: where a mouse can grab the page without selecting text. */
  function onMargin(x, y) {
    const r = $("page-frame").getBoundingClientRect();
    const left = r.left + pager.padX;
    const top = r.top + pager.padTop;
    const inText = x >= left && x <= r.right - pager.padX && y >= top && y <= top + pager.h;
    const inGutter = pager.n === 2 && x > left + pager.pw && x < left + pager.pw + pager.gap;
    return !inText || inGutter;
  }
  function wirePages() {
    const scroller = $("scroller");
    const article = $("chapter");
    // Wheel and touchpad: a mouse notch turns at once; a two-finger swipe turns once it has moved
    // far enough, and its momentum is ignored until the fingers pause or swipe again.
    const wheel = { acc: 0, last: 0, prev: 0, armed: true };
    scroller.addEventListener(
      "wheel",
      (e) => {
        if (!isPaged() || e.ctrlKey) return;
        const d = Math.abs(e.deltaY) >= Math.abs(e.deltaX) ? e.deltaY : e.deltaX;
        if (scrollsItself(e.target, d)) return;
        e.preventDefault();
        const now = performance.now();
        const size = Math.abs(d);
        if (now - wheel.last > 200) {
          wheel.armed = true;
          wheel.acc = 0;
        } else if (!wheel.armed && size > wheel.prev * 1.8 && size > 6) {
          wheel.armed = true; // a new swipe while the last one is still coasting
          wheel.acc = 0;
        }
        wheel.last = now;
        wheel.prev = size;
        if (!wheel.armed) return;
        wheel.acc += d;
        const threshold = e.deltaMode === 1 ? 1 : 60;
        if (Math.abs(wheel.acc) >= threshold) {
          turn(wheel.acc > 0 ? 1 : -1);
          wheel.armed = false;
          wheel.acc = 0;
        }
      },
      { passive: false }
    );
    // Click the outer part of a page (or the desk beside it) to turn.
    scroller.addEventListener("click", (e) => {
      if (state.suppressClick) {
        state.suppressClick = false;
        return;
      }
      if (!isPaged() || e.button !== 0) return;
      if (e.target.closest(INTERACTIVE)) return;
      if (String(window.getSelection && window.getSelection()).length) return;
      const r = $("page-frame").getBoundingClientRect();
      const zone = Math.min(160, r.width * 0.2);
      if (e.clientX < r.left + zone) turn(-1);
      else if (e.clientX > r.right - zone) turn(1);
    });
    // Drag to turn: touch and pen anywhere, a mouse from the margins (so text can still be selected).
    let drag = null;
    scroller.addEventListener("pointerdown", (e) => {
      if (!isPaged() || e.button !== 0 || e.target.closest(INTERACTIVE)) return;
      const mouse = e.pointerType === "mouse";
      if (mouse && !onMargin(e.clientX, e.clientY)) return;
      drag = { id: e.pointerId, x: e.clientX, y: e.clientY, t: performance.now(), moved: false };
      if (mouse) e.preventDefault();
    });
    scroller.addEventListener("pointermove", (e) => {
      if (!drag || e.pointerId !== drag.id) return;
      const dx = e.clientX - drag.x;
      const dy = e.clientY - drag.y;
      if (!drag.moved) {
        if (Math.abs(dx) < 8 || Math.abs(dx) < Math.abs(dy)) return;
        drag.moved = true;
        try {
          scroller.setPointerCapture(e.pointerId);
        } catch (err) {
          /* the pointer is already gone */
        }
        $("reader").classList.add("dragging");
        finishFlip();
      }
      if (currentTurn() === "slide") {
        // Follow the finger, with resistance where there is no page to go to.
        const atStart = pager.page === 0 && state.chapter === 0;
        const atEnd = pager.page === pager.pages - 1 && state.chapter === state.book.chapters.length - 1;
        const pull = (dx > 0 && atStart) || (dx < 0 && atEnd) ? dx * 0.25 : dx;
        article.classList.remove("turning");
        article.style.transform = "translateX(" + (-pager.page * pager.step + pull) + "px)";
      }
    });
    const endDrag = (e) => {
      if (!drag || e.pointerId !== drag.id) return;
      const d = drag;
      drag = null;
      if (!d.moved) return;
      $("reader").classList.remove("dragging");
      state.suppressClick = true;
      setTimeout(() => (state.suppressClick = false), 0);
      const dx = e.clientX - d.x;
      const fast = Math.abs(dx) / Math.max(1, performance.now() - d.t) > 0.5;
      const page = pager.page;
      const chapter = state.chapter;
      if (e.type === "pointerup" && (Math.abs(dx) > 70 || (fast && Math.abs(dx) > 20))) turn(dx < 0 ? 1 : -1);
      if (pager.page === page && state.chapter === chapter) goto(page, currentTurn() === "slide"); // snap back
    };
    scroller.addEventListener("pointerup", endDrag);
    scroller.addEventListener("pointercancel", endDrag);
    // Tabbing to a link on another page turns to it.
    $("chapter").addEventListener("focusin", (e) => {
      if (isPaged() && e.target.id !== "chapter") reveal(e.target);
    });
    // Late images and diagrams change the page count.
    $("chapter").addEventListener("load", () => isPaged() && (countPages(), goto(pager.page, false)), true);
    if (window.ResizeObserver) new ResizeObserver(() => isPaged() && relayout()).observe(scroller);
  }

  /* ---------------------------------------------------------------- focus & full screen */

  function setFocusMode(on) {
    state.focus = on;
    $("reader").classList.toggle("focus", on);
    $("btn-focus").setAttribute("aria-pressed", String(on));
    if (on) showChrome(1600); // so readers see how to leave
    else $("reader").classList.remove("chrome-on");
  }
  /* In focus mode the toolbar shows while the pointer is near the top (or a menu is open), then hides
     after a short pause, so a quick move off the edge does not lose it. */
  let chromeTimer = 0;
  function showChrome(hideAfter) {
    clearTimeout(chromeTimer);
    $("reader").classList.add("chrome-on");
    chromeTimer = setTimeout(hideChrome, hideAfter || 900);
  }
  function hideChrome() {
    clearTimeout(chromeTimer);
    const bar = document.querySelector(".reader .toolbar");
    if (panelsOpen() || bar.matches(":hover, :focus-within")) {
      chromeTimer = setTimeout(hideChrome, 600);
      return;
    }
    $("reader").classList.remove("chrome-on");
  }
  function wireFocusChrome() {
    document.addEventListener("mousemove", (e) => {
      if (!state.focus || $("reader").hidden) return;
      const top = document.querySelector(".reader-main").getBoundingClientRect().top;
      if (e.clientY - top < 72) showChrome();
    });
  }
  function tauriWindow() {
    const t = window.__TAURI__;
    return t && t.window ? t.window.getCurrentWindow() : null;
  }
  async function setFullscreen(on) {
    const win = tauriWindow();
    try {
      if (win) {
        // A maximized window going full screen keeps its old height, leaving the taskbar strip
        // uncovered. Restore it first, and maximize it again on the way out.
        if (on) {
          state.wasMaximized = await win.isMaximized();
          if (state.wasMaximized) await win.unmaximize();
        }
        await win.setFullscreen(on);
        if (!on && state.wasMaximized) await win.maximize();
      }
      else if (on) await document.documentElement.requestFullscreen();
      else if (document.fullscreenElement) await document.exitFullscreen();
    } catch (e) {
      return;
    }
    syncFullscreen(on);
  }
  function syncFullscreen(on) {
    state.fullscreen = on;
    $("app").classList.toggle("fullscreen", on);
    const b = $("btn-fullscreen");
    b.setAttribute("aria-pressed", String(on));
    b.setAttribute("aria-label", on ? "Leave full screen" : "Full screen");
    b.title = (on ? "Leave full screen" : "Full screen") + " (F11)";
    b.querySelector("[data-icon]").innerHTML = (window.ReaderIcons || {})[on ? "minimize" : "maximize"] || "";
  }
  const PANELS = ["text-panel", "page-panel", "search-panel", "bookmark-panel"];
  const POPOVER_BUTTONS = { "text-panel": "btn-text", "page-panel": "btn-page" };
  function panelsOpen() {
    return PANELS.some((p) => !$(p).hidden);
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
    const pct = overallPercent(state.chapter, positionRatio());
    $("progress-chapter").textContent = "Chapter " + (state.chapter + 1) + " of " + state.book.chapters.length;
    $("progress-fill").style.width = pct + "%";
    $("progress-pct").textContent = pct + "% read";
  }
  function saveNow() {
    if (state.chapter < 0 || state.loading || !state.book.reader.rememberPosition) return;
    save("position", { chapter: state.chapter, ratio: positionRatio(), at: Date.now() });
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
      if (t) reveal(t, "smooth");
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
    if (target) reveal(target, "third");
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
          setRatio(bm.ratio, true);
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
    const frame = $("page-frame").getBoundingClientRect();
    const paged = isPaged();
    let text = "";
    const blocks = $("chapter").querySelectorAll("p, h1, h2, h3, li");
    for (const b of blocks) {
      const rect = b.getBoundingClientRect();
      if (paged ? rect.right > frame.left && rect.left < frame.right : rect.bottom > top) {
        text = b.textContent.trim().slice(0, 140);
        break;
      }
    }
    const all = bookmarks();
    all.unshift({ chapter: state.chapter, ratio: positionRatio(), text, at: Date.now() });
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
    PANELS.forEach((p) => ($(p).hidden = true));
    Object.values(POPOVER_BUTTONS).forEach((b) => $(b).setAttribute("aria-expanded", "false"));
  }
  function toggle(panel) {
    const open = $(panel).hidden;
    closePanels();
    $(panel).hidden = !open;
    const button = POPOVER_BUTTONS[panel];
    if (button) {
      $(button).setAttribute("aria-expanded", String(open));
      if (open) placePopover($(panel), $(button));
    }
    if (open && panel === "search-panel") $("search-input").focus();
    if (open && panel === "bookmark-panel") renderBookmarks();
  }

  /** Open a menu just under its button, with a small gap, kept inside the window. */
  function placePopover(panel, button) {
    const main = document.querySelector(".reader-main").getBoundingClientRect();
    const b = button.getBoundingClientRect();
    const bar = document.querySelector(".reader .toolbar").getBoundingClientRect();
    panel.style.top = bar.bottom - main.top + 8 + "px";
    panel.style.right = Math.max(8, main.right - b.right - 4) + "px";
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
    $("btn-text").addEventListener("click", () => toggle("text-panel"));
    $("btn-page").addEventListener("click", () => toggle("page-panel"));
    // A click outside an open menu closes it. Moving the pointer never does.
    document.addEventListener("pointerdown", (e) => {
      for (const [panel, button] of Object.entries(POPOVER_BUTTONS)) {
        if (!$(panel).hidden && !$(panel).contains(e.target) && !$(button).contains(e.target)) {
          $(panel).hidden = true;
          $(button).setAttribute("aria-expanded", "false");
        }
      }
    });
    window.addEventListener("resize", () => {
      for (const [panel, button] of Object.entries(POPOVER_BUTTONS)) if (!$(panel).hidden) placePopover($(panel), $(button));
    });
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
    document.querySelectorAll("[data-layout]").forEach((b) => b.addEventListener("click", () => setPref("layout", b.dataset.layout)));
    document.querySelectorAll("[data-spread]").forEach((b) => b.addEventListener("click", () => setPref("spread", b.dataset.spread)));
    document.querySelectorAll("[data-paper]").forEach((b) => b.addEventListener("click", () => setPref("paper", b.dataset.paper)));
    document.querySelectorAll("[data-turn]").forEach((b) => b.addEventListener("click", () => setPref("turn", b.dataset.turn)));
    wireFocusChrome();
    $("btn-focus").addEventListener("click", () => setFocusMode(!state.focus));
    $("btn-fullscreen").addEventListener("click", () => setFullscreen(!state.fullscreen));
    document.addEventListener("fullscreenchange", () => !tauriWindow() && syncFullscreen(!!document.fullscreenElement));
    wirePages();
    $("size-down").addEventListener("click", () => setPref("size", Math.max(14, currentSize() - 1)));
    $("size-up").addEventListener("click", () => setPref("size", Math.min(30, currentSize() + 1)));
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
        else if (panelsOpen()) closePanels();
        else if (state.focus) setFocusMode(false);
        else if (state.fullscreen) setFullscreen(false);
        return;
      }
      if (e.key === "F11") {
        e.preventDefault();
        setFullscreen(!state.fullscreen);
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
      if (typing || $("reader").hidden || e.ctrlKey || e.metaKey) return;
      if (e.key.toLowerCase() === "f" && !e.altKey && !e.shiftKey) {
        setFocusMode(!state.focus);
        return;
      }
      if (isPaged() && !e.altKey) {
        const onControl = document.activeElement && document.activeElement.closest && document.activeElement.closest("button, a, summary, [role=button]");
        const space = e.key === " " && !onControl;
        if (e.key === "ArrowRight" || e.key === "ArrowDown" || e.key === "PageDown" || (space && !e.shiftKey)) turn(1);
        else if (e.key === "ArrowLeft" || e.key === "ArrowUp" || e.key === "PageUp" || (space && e.shiftKey)) turn(-1);
        else if (e.key === "Home") goto(0, true);
        else if (e.key === "End") (countPages(), goto(pager.pages - 1, true));
        else return;
        e.preventDefault();
        return;
      }
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

  /* ---------------------------------------------------------------- icons & title bar */

  function fillIcons(root) {
    const icons = window.ReaderIcons || {};
    root.querySelectorAll("[data-icon]").forEach((el) => {
      if (!el.firstChild && icons[el.dataset.icon]) el.innerHTML = icons[el.dataset.icon];
    });
  }

  function setSubtitle(text) {
    $("tb-sub").textContent = text || "";
  }

  async function setupTitleBar() {
    const tauri = window.__TAURI__;
    if (state.book.reader.titleBar === "native" || !tauri || !tauri.window) return;
    const win = tauri.window.getCurrentWindow();
    $("titlebar").hidden = false;
    $("app").classList.add("has-titlebar");
    $("tb-title").textContent = state.book.meta.title;
    $("tb-min").addEventListener("click", () => win.minimize());
    $("tb-max").addEventListener("click", () => win.toggleMaximize());
    $("tb-close").addEventListener("click", () => {
      saveNow();
      win.close();
    });
    const sync = async () => {
      let max = false;
      try {
        max = await win.isMaximized();
      } catch (e) {
        /* ignore */
      }
      const b = $("tb-max");
      b.querySelector(".glyph").innerHTML = max ? "&#xE923;" : "&#xE922;";
      b.setAttribute("aria-label", max ? "Restore" : "Maximize");
      b.title = max ? "Restore" : "Maximize";
    };
    try {
      await win.onResized(sync);
      await win.onFocusChanged(({ payload }) => $("titlebar").classList.toggle("blurred", !payload));
    } catch (e) {
      /* events not permitted: the buttons still work */
    }
    sync();
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
    fillIcons(document);
    setupTitleBar();
    wire();
    applyDisplay();
    renderToc();
    renderAbout();
    $("app").removeAttribute("aria-busy");
    route();
    window.__pager = pager;
  }

  start();
})();
