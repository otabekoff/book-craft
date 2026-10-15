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
  const pager = { page: 0, pages: 1, cols: 1, n: 1, pw: 0, gap: 0, step: 0, padX: 0, padTop: 0, h: 0, fw: 0, fh: 0, curl: null };
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
    ["fade", "slide", "flip", "none"].forEach((t) => reader.classList.toggle("turn-" + t, currentTurn() === t));
    $("row-spread").hidden = !isPaged();
    $("row-turn").hidden = !isPaged();
    $("row-numbers").hidden = !isPaged();
    $("row-arrows").hidden = !isPaged();
    reader.classList.toggle("arrows-on", currentArrows() === "on");

    document.querySelectorAll("[data-theme-choice]").forEach((b) =>
      b.setAttribute("aria-pressed", String((prefs.theme || state.book.reader.defaultTheme) === b.dataset.themeChoice))
    );
    document.querySelectorAll("[data-leading]").forEach((b) => b.setAttribute("aria-pressed", String((prefs.leading || "") === b.dataset.leading)));
    document.querySelectorAll("[data-width]").forEach((b) => b.setAttribute("aria-pressed", String(width === b.dataset.width)));
    document.querySelectorAll("[data-layout]").forEach((b) => b.setAttribute("aria-pressed", String(currentLayout() === b.dataset.layout)));
    document.querySelectorAll("[data-spread]").forEach((b) => b.setAttribute("aria-pressed", String(currentSpread() === b.dataset.spread)));
    document.querySelectorAll("[data-paper]").forEach((b) => b.setAttribute("aria-pressed", String(currentPaper() === b.dataset.paper)));
    document.querySelectorAll("[data-turn]").forEach((b) => b.setAttribute("aria-pressed", String(currentTurn() === b.dataset.turn)));
    document.querySelectorAll("[data-numbers]").forEach((b) => b.setAttribute("aria-pressed", String(currentNumbers() === b.dataset.numbers)));
    document.querySelectorAll("[data-arrows]").forEach((b) => b.setAttribute("aria-pressed", String(currentArrows() === b.dataset.arrows)));
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
    return prefs.turn || state.book.reader.pageTurn || "fade";
  }
  function currentArrows() {
    return prefs.arrows || (state.book.reader.pageArrows === false ? "off" : "on");
  }
  function currentNumbers() {
    return prefs.numbers || (state.book.reader.pageNumbers === false ? "off" : "on");
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

  function icon(name) {
    const span = el("span", { "data-icon": name, "aria-hidden": "true" });
    span.innerHTML = (window.ReaderIcons || {})[name] || "";
    return span;
  }
  function renderHome() {
    const b = state.book;
    const meta = b.meta;
    renderCover($("cover"));
    const backdrop = $("home-backdrop");
    backdrop.classList.toggle("has-image", !!meta.cover);
    backdrop.style.backgroundImage = meta.cover ? 'url("/' + meta.cover.replace(/^\/+/, "") + '")' : "";
    $("home-eyebrow").textContent = [meta.publisher, meta.copyright && (meta.copyright.match(/\d{4}/) || [])[0]].filter(Boolean).join(" · ");
    $("home-title").textContent = meta.title;
    $("home-subtitle").textContent = meta.subtitle || "";
    $("home-subtitle").hidden = !meta.subtitle;
    $("home-author").textContent = meta.author ? "by " + meta.author : "";
    $("home-description").textContent = meta.description || "";
    $("home-description").hidden = !meta.description;
    $("home-copyright").textContent = [meta.copyright, meta.publisher && "Published by " + meta.publisher].filter(Boolean).join(" · ");

    const chapterCount = b.chapters.filter((c) => !c.kind || c.kind === "chapter").length || b.chapters.length;
    const facts = $("home-facts");
    facts.innerHTML = "";
    [
      ["list", chapterCount + (chapterCount === 1 ? " chapter" : " chapters")],
      ["clock", b.words ? minutes(b.words).replace(/^About /, "") + " read" : ""],
      ["languages", meta.language ? languageName(meta.language) : ""],
    ]
      .filter(([, text]) => text)
      .forEach(([name, text]) => {
        const li = el("li");
        li.append(icon(name), document.createTextNode(text));
        facts.appendChild(li);
      });

    const pos = b.reader.rememberPosition ? load("position", null) : null;
    const started = pos && pos.chapter < b.chapters.length && (pos.chapter > 0 || pos.ratio > 0.02);
    const cont = $("home-continue");
    if (started) {
      cont.href = "#read/" + pos.chapter;
      cont.querySelector(".btn-label").textContent = "Continue reading";
      $("home-continue-sub").textContent = b.chapters[pos.chapter].title;
      $("home-start").hidden = false;
      const pct = overallPercent(pos.chapter, pos.ratio || 0);
      $("home-progress-wrap").hidden = !b.reader.showProgress;
      $("home-progress").style.width = pct + "%";
      $("home-progress-text").textContent = pct + "% read";
    } else {
      cont.href = "#read/0";
      cont.querySelector(".btn-label").textContent = "Start reading";
      $("home-continue-sub").textContent = b.chapters[0] ? b.chapters[0].title : "";
      $("home-start").hidden = true;
      $("home-progress-wrap").hidden = true;
    }

    $("home-contents-meta").textContent = [chapterCount + (chapterCount === 1 ? " chapter" : " chapters"), b.words ? minutes(b.words) : ""].filter(Boolean).join(" · ");
    const list = $("home-toc");
    list.innerHTML = "";
    b.chapters.forEach((c, i) => {
      const li = el("li", { class: c.kind && c.kind !== "chapter" ? "kind-" + c.kind : "" });
      const status = started ? (i < pos.chapter ? "done" : i === pos.chapter ? "current" : "") : "";
      const a = el("a", { href: "#read/" + i, class: "toc-card" + (status ? " " + status : "") });
      const num = chapterNumberLabel(i);
      a.appendChild(el("span", { class: "num" }, num ? num.padStart(2, "0") : "•"));
      const body = el("span", { class: "body" });
      body.appendChild(el("span", { class: "t" }, c.title));
      const metaLine = el("span", { class: "m" });
      if (c.words) metaLine.appendChild(document.createTextNode(Math.max(1, Math.round(c.words / 230)) + " min"));
      if (status === "done") {
        const st = el("span", { class: "state" });
        st.append(icon("check"), document.createTextNode("Read"));
        metaLine.appendChild(st);
      } else if (status === "current") {
        metaLine.appendChild(el("span", { class: "state" }, "Reading · " + Math.round((pos.ratio || 0) * 100) + "%"));
      }
      body.appendChild(metaLine);
      a.appendChild(body);
      a.appendChild(icon("chevron-right"));
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
      const drawn = window.BookBlocks ? window.BookBlocks.hydrate(article, { vendor: "/_runtime/vendor/", theme: effectiveTheme() }) : null;
      Promise.resolve(drawn).then(() => state.chapter === i && paintHighlights());
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
        if (currentTurn() === "fade" && !reducedMotion.matches) article.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 260, easing: "ease-out" });
      } else if (ratio) {
        requestAnimationFrame(() => setRatio(ratio));
      }
      requestAnimationFrame(() => (state.loading = false));
    }
    if (anchor) {
      const target = document.getElementById(anchor);
      if (target)
        requestAnimationFrame(() => {
          reveal(target);
          flash(target);
        });
    }
    if (state.pendingMark) {
      const mark = state.pendingMark;
      state.pendingMark = null;
      requestAnimationFrame(() => revealMark(mark));
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
    endCurl();
    $("reader").classList.remove("spread-2");
    $("page-numbers").textContent = "";
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
    endCurl();
    Object.assign(pager, { n, pw, gap, step: contentW + gap, padX: pad.x, padTop: pad.top, h: contentH, fw: contentW + pad.x * 2, fh: availH });
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
    endCurl();
    const moved = !!animate && !reducedMotion.matches && pager.page !== from;
    if (moved && currentTurn() === "fade") crossfade(spreadCopy(from));
    article.classList.toggle("turning", moved && currentTurn() === "slide");
    article.style.transform = "translateX(" + -pager.page * pager.step + "px)";
    renderPageNumbers();
    $("arrow-prev").disabled = pager.page === 0 && state.chapter === 0;
    $("arrow-next").disabled = pager.page >= pager.pages - 1 && state.chapter === state.book.chapters.length - 1;
    hideSelMenu();
    updateProgress();
    clearTimeout(state.saveTimer);
    state.saveTimer = setTimeout(saveNow, 400);
  }
  /** Copies of a whole spread over the chapter, used to fade from it. */
  function spreadCopy(page) {
    const frame = $("page-frame");
    const layer = document.createElement("div");
    layer.className = "curl-layer";
    layer.setAttribute("aria-hidden", "true");
    layer.style.cssText = "left:" + frame.offsetLeft + "px;top:" + frame.offsetTop + "px;width:" + pager.fw + "px;height:" + pager.fh + "px";
    if (pager.n === 2) {
      const spine = pager.padX + pager.pw + pager.gap / 2;
      const right = pageSheet(page * 2 + 1, pager.fw - spine, pager.gap / 2);
      right.style.left = spine + "px";
      layer.append(pageSheet(page * 2, spine, pager.padX), right);
    } else layer.appendChild(pageSheet(page, pager.fw, pager.padX));
    return layer;
  }
  let fading = null;
  function crossfade(layer) {
    if (fading) fading.remove();
    fading = layer;
    $("scroller").appendChild(layer);
    const anim = layer.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 260, easing: "ease-out" });
    anim.finished.then(
      () => layer.remove(),
      () => layer.remove()
    );
  }
  /** A number under each page, like a printed book. */
  function renderPageNumbers() {
    const box = $("page-numbers");
    box.textContent = "";
    if (currentNumbers() === "off") return;
    for (let i = 0; i < pager.n; i++) {
      const col = pager.page * pager.n + i;
      if (col >= pager.cols) break;
      const num = el("span", { class: "page-num" }, String(col + 1));
      num.style.left = pager.padX + i * (pager.pw + pager.gap) + pager.pw / 2 + "px";
      box.appendChild(num);
    }
  }
  /** Next or previous page, crossing into the neighbouring chapter at either end. */
  function turn(dir) {
    if (state.chapter < 0) return;
    if (currentTurn() === "flip" && !reducedMotion.matches && isPaged()) {
      autoCurl(dir);
      return;
    }
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
  /* ------------------------------------------------------------ page curl
     A real-book turn. The page is folded along the line halfway between its corner and the pointer:
     the part past the fold flips over and shows the next page on its back, and the page underneath
     shows through where it lifted. Everything is drawn with copies of the pages over the chapter;
     the chapter itself only moves once the turn is done.

     "Sheet space" has its origin at the spine (the page's inner edge), x toward the outer edge (W),
     y down (H). A backward turn is the same fold played in reverse with the earlier spread. */

  /** A full-height sheet of paper holding one page's text and number (or blank when col is null). */
  function pageSheet(col, width, contentX) {
    const sheet = document.createElement("div");
    sheet.className = "curl-sheet";
    sheet.style.width = width + "px";
    sheet.style.height = pager.fh + "px";
    if (col == null || col < 0 || col >= pager.cols) return sheet;
    const box = document.createElement("div");
    box.className = "curl-content";
    box.style.cssText = "left:" + contentX + "px;top:" + pager.padTop + "px;width:" + pager.pw + "px;height:" + pager.h + "px";
    const a = $("chapter");
    const copy = a.cloneNode(true);
    copy.removeAttribute("id");
    copy.removeAttribute("tabindex");
    copy.classList.remove("turning");
    copy.style.transform = "translateX(" + -col * (pager.pw + pager.gap) + "px)";
    const src = a.querySelectorAll("canvas"); // canvases lose their drawing when copied
    copy.querySelectorAll("canvas").forEach((d, i) => {
      try {
        d.getContext("2d").drawImage(src[i], 0, 0);
      } catch (e) {
        /* empty or tainted */
      }
    });
    box.appendChild(copy);
    sheet.appendChild(box);
    if (currentNumbers() !== "off") {
      const num = el("span", { class: "page-num" }, String(col + 1));
      num.style.left = contentX + pager.pw / 2 + "px";
      sheet.appendChild(num);
    }
    return sheet;
  }
  function strip(cls, width) {
    const d = document.createElement("div");
    d.className = "curl-strip " + cls;
    d.style.width = width + "px";
    return d;
  }
  /** Prepare a turn in one direction. Returns null when there is nowhere to go. */
  function startCurl(dir, top) {
    endCurl();
    countPages();
    const s = pager.page;
    const last = state.chapter === state.book.chapters.length - 1;
    const cross = dir > 0 ? s >= pager.pages - 1 : s <= 0;
    if (cross && (dir > 0 ? last : state.chapter === 0)) return null;
    const n = pager.n;
    const target = cross ? null : s + dir;
    const col = (spread, i) => (spread == null ? null : spread * n + i);
    let spineX, contentX, front, back, under, base = null;
    if (n === 2) {
      spineX = pager.padX + pager.pw + pager.gap / 2;
      contentX = pager.gap / 2;
      if (dir > 0) {
        front = col(s, 1);
        back = col(target, 0);
        under = col(target, 1);
      } else {
        front = col(target, 1);
        back = col(s, 0);
        under = col(s, 1);
        base = col(target, 0);
      }
    } else {
      spineX = 0;
      contentX = pager.padX;
      front = dir > 0 ? col(s, 0) : col(target, 0);
      under = dir > 0 ? col(target, 0) : col(s, 0);
      back = null; // the back of a single page is blank paper
    }
    const W = pager.fw - spineX;
    const H = pager.fh;
    const frame = $("page-frame");
    const layer = document.createElement("div");
    layer.className = "curl-layer";
    layer.setAttribute("aria-hidden", "true");
    layer.style.cssText = "left:" + frame.offsetLeft + "px;top:" + frame.offsetTop + "px;width:" + pager.fw + "px;height:" + H + "px";
    if (n === 2 && dir < 0) {
      const left = pageSheet(base, spineX, pager.padX);
      left.classList.add("curl-base");
      layer.appendChild(left);
    }
    const space = document.createElement("div");
    space.className = "curl-space";
    space.style.cssText = "left:" + spineX + "px;width:" + W + "px;height:" + H + "px";
    const underEl = pageSheet(under, W, contentX);
    const underShadow = strip("curl-cast", 70);
    underEl.appendChild(underShadow);
    const frontEl = pageSheet(front, W, contentX);
    const frontShade = strip("curl-bend", 40);
    frontEl.appendChild(frontShade);
    const flap = document.createElement("div");
    flap.className = "curl-flap";
    const backEl = pageSheet(back, W, n === 2 ? pager.padX : contentX);
    backEl.classList.add("curl-back");
    if (n === 1) backEl.classList.add("blank");
    const backShade = strip("curl-gloss", 60);
    backEl.appendChild(backShade);
    flap.appendChild(backEl);
    space.append(underEl, frontEl, flap);
    layer.appendChild(space);
    $("scroller").appendChild(layer);
    const curl = { dir, cross, target, W, H, cy: top ? 0 : H, x: 0, y: 0, layer, frontEl, backEl, underShadow, frontShade, backShade, anim: 0 };
    pager.curl = curl;
    return curl;
  }
  function clipPoly(pts, side) {
    const out = [];
    for (let i = 0; i < pts.length; i++) {
      const a = pts[i];
      const b = pts[(i + 1) % pts.length];
      const fa = side(a);
      const fb = side(b);
      if (fa >= 0) out.push(a);
      if (fa >= 0 !== fb >= 0) {
        const t = fa / (fa - fb);
        out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
      }
    }
    return out;
  }
  function polygon(pts) {
    if (pts.length < 3) return "polygon(0 0, 0 0, 0 0)";
    return "polygon(" + pts.map((p) => p[0].toFixed(1) + "px " + p[1].toFixed(1) + "px").join(", ") + ")";
  }
  function placeStrip(elm, x, y, angle, before) {
    const len = 4 * Math.hypot(pager.fw, pager.fh);
    elm.style.height = len + "px";
    elm.style.transform =
      "translate(" + x + "px," + y + "px) rotate(" + angle + "rad) translate(" + (before ? -parseFloat(elm.style.width) : 0) + "px," + -len / 2 + "px)";
  }
  /** Fold the page so its corner sits at (px, py) in sheet space. */
  function curlTo(c, px, py) {
    const { W, H, cy } = c;
    const cx = W;
    let x = px;
    let y = py;
    // Paper does not stretch: the corner stays within reach of the spine's ends.
    [
      [0, cy, W],
      [0, H - cy, Math.hypot(W, H)],
    ].forEach(([ax, ay, reach]) => {
      const dx = x - ax;
      const dy = y - ay;
      const d = Math.hypot(dx, dy);
      if (d > reach) {
        x = ax + (dx * reach) / d;
        y = ay + (dy * reach) / d;
      }
    });
    c.x = x;
    c.y = y;
    let nx = cx - x;
    let ny = cy - y;
    const len = Math.hypot(nx, ny);
    if (len < 0.5) {
      c.frontEl.style.clipPath = "none";
      c.backEl.style.visibility = "hidden";
      c.underShadow.style.opacity = "0";
      return;
    }
    nx /= len;
    ny /= len;
    const mx = (cx + x) / 2;
    const my = (cy + y) / 2;
    const dist = (q) => (q[0] - mx) * nx + (q[1] - my) * ny;
    const rect = [
      [0, 0],
      [W, 0],
      [W, H],
      [0, H],
    ];
    const cut = clipPoly(rect, dist);
    c.frontEl.style.clipPath = polygon(clipPoly(rect, (q) => -dist(q)));
    // The back: its own x runs from the outer edge to the spine; fold it over the line.
    c.backEl.style.visibility = "visible";
    c.backEl.style.clipPath = polygon(cut.map((q) => [W - q[0], q[1]]));
    const k = 2 * (mx * nx + my * ny);
    const a = -(1 - 2 * nx * nx);
    const b = 2 * nx * ny;
    const cc = -2 * nx * ny;
    const d = 1 - 2 * ny * ny;
    const e = (1 - 2 * nx * nx) * W + k * nx;
    const f = -2 * nx * ny * W + k * ny;
    c.backEl.style.transform = "matrix(" + [a, b, cc, d, e, f].map((v) => v.toFixed(5)).join(",") + ")";
    // Shading: the shadow the lifted page casts, the bend near the fold, and light on the flap.
    const angle = Math.atan2(ny, nx);
    const depth = Math.min(1, len / (W * 0.6));
    placeStrip(c.underShadow, mx, my, angle, false);
    c.underShadow.style.opacity = String(0.35 + 0.65 * depth);
    placeStrip(c.frontShade, mx, my, angle, true);
    placeStrip(c.backShade, W - mx, my, Math.atan2(ny, -nx), false);
  }
  /** Move the corner to (tx, ty) along a gentle arc, then finish or settle back. */
  function curlAnimate(c, tx, ty, ms, done) {
    cancelAnimationFrame(c.anim);
    const x0 = c.x;
    const y0 = c.y;
    const lift = (c.cy === 0 ? 1 : -1) * c.H * 0.1 * Math.min(1, Math.abs(tx - x0) / (2 * c.W));
    const t0 = performance.now();
    const ease = (t) => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2);
    const step = (now) => {
      const t = Math.min(1, (now - t0) / ms);
      const e = ease(t);
      curlTo(c, x0 + (tx - x0) * e, y0 + (ty - y0) * e + lift * Math.sin(Math.PI * e));
      if (t < 1) c.anim = requestAnimationFrame(step);
      else {
        c.anim = 0;
        done();
      }
    };
    c.anim = requestAnimationFrame(step);
  }
  /** The page has landed: show the new spread under the copies, then remove them. */
  function landCurl(c) {
    if (pager.curl !== c) return;
    if (c.cross) {
      if (c.dir < 0) state.openAtEnd = true;
      location.hash = "#read/" + (state.chapter + c.dir);
      setTimeout(() => pager.curl === c && endCurl(), 1500); // the new chapter normally replaces it first
      return;
    }
    pager.curl = null; // keep goto from removing it twice
    goto(c.target, false);
    c.layer.remove();
  }
  function endCurl() {
    const c = pager.curl;
    if (!c) return;
    pager.curl = null;
    cancelAnimationFrame(c.anim);
    c.layer.remove();
  }
  const TURN_MS = 760;
  function autoCurl(dir) {
    const busy = pager.curl;
    if (busy) {
      // Finish the turn in progress at once, then start the next.
      cancelAnimationFrame(busy.anim);
      if (busy.settling) endCurl();
      else landCurl(busy);
      if (busy.cross) return;
    }
    const c = startCurl(dir, false);
    if (!c) return;
    if (dir > 0) {
      curlTo(c, c.W, c.H);
      curlAnimate(c, -c.W, c.H, TURN_MS, () => landCurl(c));
    } else {
      curlTo(c, -c.W, c.H);
      curlAnimate(c, c.W, c.H, TURN_MS, () => landCurl(c));
    }
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
  /** Which way a press outside the text turns: the grips and the desk on either side. */
  function sideOf(target, x) {
    if (target.closest(".grip-prev")) return -1;
    if (target.closest(".grip-next")) return 1;
    if (target.id === "scroller" || target.id === "page-frame") {
      const r = $("page-frame").getBoundingClientRect();
      return x < r.left + r.width / 2 ? -1 : 1;
    }
    return 0;
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
    // Click the page edges (the grips) or the desk beside the page to turn.
    scroller.addEventListener("click", (e) => {
      if (state.suppressClick) {
        state.suppressClick = false;
        return;
      }
      if (!isPaged() || e.button !== 0) return;
      const dir = sideOf(e.target, e.clientX);
      if (dir) turn(dir);
    });
    // Drag to turn: grab a page edge or the desk with a mouse (the text stays selectable), or drag
    // anywhere with a finger or pen. With Flip the page curls under the pointer.
    let drag = null;
    scroller.addEventListener("pointerdown", (e) => {
      if (!isPaged() || e.button !== 0 || e.target.closest(INTERACTIVE)) return;
      const side = sideOf(e.target, e.clientX);
      if (e.pointerType === "mouse" && !side) return;
      drag = { id: e.pointerId, x: e.clientX, y: e.clientY, t: performance.now(), side, moved: false, curl: null, vx: 0, lx: e.clientX, lt: performance.now() };
      if (e.pointerType === "mouse") e.preventDefault();
    });
    scroller.addEventListener("pointermove", (e) => {
      if (!drag || e.pointerId !== drag.id) return;
      const dx = e.clientX - drag.x;
      const dy = e.clientY - drag.y;
      const now = performance.now();
      drag.vx = (e.clientX - drag.lx) / Math.max(1, now - drag.lt);
      drag.lx = e.clientX;
      drag.lt = now;
      if (!drag.moved) {
        if (Math.hypot(dx, dy) < 6 || (!drag.side && Math.abs(dx) < Math.abs(dy))) return;
        drag.moved = true;
        try {
          scroller.setPointerCapture(e.pointerId);
        } catch (err) {
          /* the pointer is already gone */
        }
        $("reader").classList.add("dragging");
        if (currentTurn() === "flip" && !reducedMotion.matches) {
          const r = $("page-frame").getBoundingClientRect();
          drag.curl = startCurl(drag.side || (dx < 0 ? 1 : -1), drag.y < r.top + r.height / 2);
        } else endCurl();
      }
      if (drag.curl) {
        // The corner starts where the page lies (right edge, or the left page's outer edge when
        // turning back) and moves with the pointer, lifting a little as it crosses, like paper does.
        const c = drag.curl;
        const startX = c.dir > 0 ? c.W : -c.W;
        const progress = Math.min(1, Math.abs(dx) / (2 * c.W));
        const lift = (c.cy === 0 ? 1 : -1) * c.H * 0.1 * Math.sin(Math.PI * progress);
        curlTo(c, startX + dx, c.cy + dy + lift);
      } else if (currentTurn() === "slide") {
        // Follow the pointer, with resistance where there is no page to go to.
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
      const ok = e.type === "pointerup";
      if (d.curl) {
        const c = d.curl;
        // Turn when the corner passed the middle, or with a quick flick the right way.
        const flick = c.dir > 0 ? d.vx < -0.45 : d.vx > 0.45;
        const finish = ok && (flick || (c.dir > 0 ? c.x < 0 : c.x > 0));
        const tx = finish === c.dir > 0 ? -c.W : c.W;
        const ms = 260 + 260 * (Math.abs(tx - c.x) / (2 * c.W));
        if (finish) curlAnimate(c, tx, c.cy, ms, () => landCurl(c));
        else {
          c.settling = true;
          curlAnimate(c, tx, c.cy, ms, () => pager.curl === c && endCurl());
        }
        return;
      }
      const fast = Math.abs(dx) / Math.max(1, performance.now() - d.t) > 0.5;
      const page = pager.page;
      const chapter = state.chapter;
      if (ok && (Math.abs(dx) > 70 || (fast && Math.abs(dx) > 20))) turn(dx < 0 ? 1 : -1);
      if (pager.page === page && state.chapter === chapter) goto(page, currentTurn() === "slide"); // snap back
    };
    scroller.addEventListener("pointerup", endDrag);
    scroller.addEventListener("pointercancel", endDrag);
    // Tabbing to a link on another page turns to it.
    $("chapter").addEventListener("focusin", (e) => {
      if (isPaged() && e.target.id !== "chapter") reveal(e.target);
    });
    // Late images and diagrams change the page count.
    $("chapter").addEventListener("load", () => isPaged() && !pager.curl && (countPages(), goto(pager.page, false)), true);
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
      if (t) {
        reveal(t, "smooth");
        flash(t);
      }
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
  function openMark(mark) {
    closePanels();
    if (state.chapter === mark.chapter) {
      revealMark(mark);
      return;
    }
    state.pendingMark = mark;
    location.hash = "#read/" + mark.chapter;
  }
  function renderBookmarks() {
    const r = state.book.reader;
    $("bookmarks-section").hidden = !r.enableBookmarks;
    $("highlights-section").hidden = !canSelect();
    $("marks-title").textContent = r.enableBookmarks && canSelect() ? "Bookmarks & highlights" : r.enableBookmarks ? "Bookmarks" : "Highlights";
    const list = $("bookmark-list");
    list.innerHTML = "";
    const all = bookmarks();
    if (!all.length) list.appendChild(el("li", { class: "side-status" }, "No bookmarks yet."));
    all.forEach((bm, n) => {
      const li = el("li");
      const a = el("a", { href: "#read/" + bm.chapter });
      a.innerHTML = '<span class="where">' + esc((state.book.chapters[bm.chapter] || {}).title || "") + "</span>" + esc(bm.text || "");
      a.addEventListener("click", (e) => {
        e.preventDefault();
        save("position", { chapter: bm.chapter, ratio: bm.ratio, at: Date.now() });
        openMark(bm);
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
    const hl = $("highlight-list");
    hl.innerHTML = "";
    const marks = highlights()
      .slice()
      .sort((x, y) => x.chapter - y.chapter || x.start - y.start);
    if (!marks.length) hl.appendChild(el("li", { class: "side-status" }, "Select text in the book to highlight it."));
    marks.forEach((h) => {
      const li = el("li");
      const a = el("a", { href: "#read/" + h.chapter });
      a.innerHTML =
        '<span class="where"><span class="hl-mark hl-' + esc(h.color) + '"></span>' + esc((state.book.chapters[h.chapter] || {}).title || "") + "</span>" + esc(h.text.slice(0, 160));
      a.addEventListener("click", (e) => {
        e.preventDefault();
        openMark(h);
      });
      const rm = el("button", { type: "button", class: "remove", "aria-label": "Remove highlight" }, "✕");
      rm.addEventListener("click", () => {
        save("highlights", highlights().filter((x) => x.id !== h.id));
        paintHighlights();
        renderBookmarks();
      });
      li.append(a, rm);
      hl.appendChild(li);
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

  /* ---------------------------------------------------------------- highlights & the selection menu
     Highlights are kept as text offsets within the chapter (plus the text itself, to find them again
     if the chapter changed) and drawn with the CSS Highlight API, so the page's HTML is never touched. */

  const SKIP_TEXT = ".katex, .mermaid, svg, script, style, .bc-code-tools, button";
  function canSelect() {
    return state.book.reader.allowSelect !== false;
  }
  function highlights() {
    return load("highlights", []);
  }
  function textIndex() {
    const nodes = [];
    let total = 0;
    const walker = document.createTreeWalker($("chapter"), NodeFilter.SHOW_TEXT, {
      acceptNode: (n) => (n.parentElement && n.parentElement.closest(SKIP_TEXT) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
    });
    let n;
    while ((n = walker.nextNode())) {
      nodes.push({ node: n, start: total });
      total += n.nodeValue.length;
    }
    return { nodes, total, text: nodes.map((e) => e.node.nodeValue).join("") };
  }
  function boundaryOffset(idx, container, offset) {
    if (container.nodeType === 3) {
      const hit = idx.nodes.find((e) => e.node === container);
      if (hit) return hit.start + offset;
    }
    const point = document.createRange();
    point.setStart(container, offset);
    for (const e of idx.nodes) if (point.comparePoint(e.node, 0) >= 0) return e.start;
    return idx.total;
  }
  function locate(idx, pos, end) {
    for (const e of idx.nodes) {
      const len = e.node.nodeValue.length;
      if (pos < e.start + len || (end && pos === e.start + len)) return [e.node, Math.max(0, pos - e.start)];
    }
    const last = idx.nodes[idx.nodes.length - 1];
    return last ? [last.node, last.node.nodeValue.length] : null;
  }
  /** Where a saved passage is now: its offsets if the text still matches, else the nearest copy of the text. */
  function findPassage(idx, mark) {
    const len = (mark.text || "").length;
    if (!len) return null;
    if (idx.text.slice(mark.start, mark.start + len) === mark.text) return mark.start;
    let best = -1;
    for (let at = idx.text.indexOf(mark.text); at >= 0; at = idx.text.indexOf(mark.text, at + 1)) {
      if (best < 0 || Math.abs(at - mark.start) < Math.abs(best - mark.start)) best = at;
    }
    return best >= 0 ? best : null;
  }
  function passageRange(idx, start, len) {
    const a = locate(idx, start, false);
    const b = locate(idx, start + len, true);
    if (!a || !b) return null;
    const range = document.createRange();
    range.setStart(a[0], a[1]);
    range.setEnd(b[0], b[1]);
    return range;
  }
  let painted = [];
  function paintHighlights() {
    if (!window.CSS || !CSS.highlights || state.chapter < 0) return;
    const groups = { yellow: [], green: [], blue: [], pink: [] };
    painted = [];
    const idx = textIndex();
    highlights()
      .filter((h) => h.chapter === state.chapter)
      .forEach((h) => {
        const at = findPassage(idx, h);
        if (at == null) return;
        const range = passageRange(idx, at, h.text.length);
        if (!range) return;
        (groups[h.color] || groups.yellow).push(range);
        painted.push({ h, range });
      });
    Object.entries(groups).forEach(([color, ranges]) => CSS.highlights.set("bc-hl-" + color, new Highlight(...ranges)));
  }
  /** Scroll or turn to a bookmarked passage or a highlight, and flash it. */
  function revealMark(mark) {
    if (mark.start == null || !mark.text) {
      setRatio(mark.ratio || 0, true);
      return;
    }
    const idx = textIndex();
    const at = findPassage(idx, mark);
    const range = at == null ? null : passageRange(idx, at, mark.text.length);
    if (!range) {
      setRatio(mark.ratio || 0, true);
      return;
    }
    reveal(range, "third");
    if (window.CSS && CSS.highlights) {
      CSS.highlights.set("bc-flash", new Highlight(range));
      setTimeout(() => CSS.highlights.delete("bc-flash"), 1600);
    }
  }
  function flash(target) {
    target.classList.remove("bc-flash");
    void target.offsetWidth; // restart the animation
    target.classList.add("bc-flash");
    setTimeout(() => target.classList.remove("bc-flash"), 1700);
  }
  function toast(message) {
    const t = el("div", { class: "toast", role: "status" }, message);
    document.querySelector(".reader-main").appendChild(t);
    setTimeout(() => t.remove(), 1900);
  }

  const menu = { range: null, mark: null };
  function chapterSelection() {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !sel.rangeCount) return null;
    const range = sel.getRangeAt(0);
    if (!$("chapter").contains(range.commonAncestorContainer)) return null;
    return range.toString().trim() ? range : null;
  }
  function showSelMenu(rect, existing) {
    const r = state.book.reader;
    const m = $("sel-menu");
    $("sel-colors").hidden = !canSelect();
    $("sel-bookmark").hidden = !r.enableBookmarks;
    $("sel-copy").hidden = !r.allowCopy;
    $("sel-remove").hidden = !existing;
    m.querySelectorAll("[data-hl]").forEach((b) => b.setAttribute("aria-pressed", String(!!existing && existing.color === b.dataset.hl)));
    if ([...m.children].every((c) => c.hidden || c.classList.contains("sel-sep"))) return;
    m.hidden = false;
    const w = m.offsetWidth;
    const h = m.offsetHeight;
    let top = rect.top - h - 10;
    if (top < 60) top = rect.bottom + 10;
    const left = Math.max(8, Math.min(window.innerWidth - w - 8, rect.left + rect.width / 2 - w / 2));
    m.style.left = left + "px";
    m.style.top = Math.min(window.innerHeight - h - 8, top) + "px";
  }
  function hideSelMenu() {
    const m = $("sel-menu");
    if (m) m.hidden = true;
    menu.range = null;
    menu.mark = null;
  }
  function markFromRange(range, color) {
    const idx = textIndex();
    const start = boundaryOffset(idx, range.startContainer, range.startOffset);
    const end = boundaryOffset(idx, range.endContainer, range.endOffset);
    const text = idx.text.slice(start, end);
    if (!text.trim()) return null;
    return { id: Date.now().toString(36) + Math.random().toString(36).slice(2, 6), chapter: state.chapter, start, text, color, at: Date.now() };
  }
  function hitHighlight(x, y) {
    const caret = document.caretRangeFromPoint ? document.caretRangeFromPoint(x, y) : null;
    if (!caret) return null;
    const hit = painted.find((p) => p.range.isPointInRange(caret.startContainer, caret.startOffset));
    return hit ? hit.h : null;
  }
  function wireSelection() {
    const m = $("sel-menu");
    const article = $("chapter");
    // Keep the text selected while using the menu.
    m.addEventListener("pointerdown", (e) => e.preventDefault());
    const check = (e) => {
      setTimeout(() => {
        const range = chapterSelection();
        if (range && canSelect()) {
          menu.range = range.cloneRange();
          menu.mark = null;
          showSelMenu(range.getBoundingClientRect(), null);
          return;
        }
        const hit = e && e.type === "pointerup" ? hitHighlight(e.clientX, e.clientY) : null;
        if (hit) {
          menu.range = null;
          menu.mark = hit;
          const p = painted.find((x) => x.h.id === hit.id);
          showSelMenu(p.range.getBoundingClientRect(), hit);
          return;
        }
        hideSelMenu();
      }, 0);
    };
    article.addEventListener("pointerup", check);
    article.addEventListener("keyup", (e) => e.shiftKey && check(e));
    document.addEventListener("selectionchange", () => {
      if (!m.hidden && menu.range && !chapterSelection()) hideSelMenu();
    });
    document.addEventListener("pointerdown", (e) => {
      if (!m.hidden && !m.contains(e.target) && !article.contains(e.target)) hideSelMenu();
    });
    $("scroller").addEventListener("scroll", hideSelMenu, { passive: true });
    m.querySelectorAll("[data-hl]").forEach((b) =>
      b.addEventListener("click", () => {
        const color = b.dataset.hl;
        let all = highlights();
        if (menu.mark) all = all.map((h) => (h.id === menu.mark.id ? Object.assign({}, h, { color }) : h));
        else if (menu.range) {
          const mark = markFromRange(menu.range, color);
          if (mark) all.push(mark);
        }
        save("highlights", all.slice(-2000));
        paintHighlights();
        window.getSelection().removeAllRanges();
        hideSelMenu();
        renderBookmarks();
      })
    );
    $("sel-remove").addEventListener("click", () => {
      if (menu.mark) save("highlights", highlights().filter((h) => h.id !== menu.mark.id));
      paintHighlights();
      hideSelMenu();
      renderBookmarks();
    });
    $("sel-bookmark").addEventListener("click", () => {
      const mark = menu.mark || (menu.range && markFromRange(menu.range, ""));
      if (!mark) return;
      const all = bookmarks();
      all.unshift({ chapter: state.chapter, ratio: positionRatio(), start: mark.start, text: mark.text.trim().slice(0, 200), at: Date.now() });
      save("bookmarks", all.slice(0, 200));
      window.getSelection().removeAllRanges();
      hideSelMenu();
      renderBookmarks();
      toast("Bookmarked");
    });
    $("sel-copy").addEventListener("click", async () => {
      const text = menu.mark ? menu.mark.text : menu.range ? menu.range.toString() : "";
      try {
        await navigator.clipboard.writeText(text);
        toast("Copied");
      } catch (e) {
        toast("Could not copy");
      }
      hideSelMenu();
    });
  }

  /* ---------------------------------------------------------------- about & settings */

  function languageName(code) {
    try {
      return new Intl.DisplayNames([navigator.language || "en"], { type: "language" }).of(code) || code;
    } catch (e) {
      return code;
    }
  }
  function renderAbout() {
    const b = state.book;
    const m = b.meta;
    renderCover($("about-cover"));
    $("about-title").textContent = m.title;
    $("about-byline").textContent = [m.author && "by " + m.author, m.publisher].filter(Boolean).join(" · ");
    $("about-description").textContent = m.description || "";
    $("about-description").hidden = !m.description;
    const rows = [
      ["Subtitle", m.subtitle],
      ["Publisher", m.publisher],
      ["Language", m.language && languageName(m.language)],
      ["Edition", m.version],
      ["ISBN", m.isbn],
      ["Chapters", String(b.chapters.length)],
      ["Length", b.words ? b.words.toLocaleString() + " words · " + minutes(b.words).replace(/^About /, "about ") : ""],
      ["Package ID", m.packageId, "mono"],
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
      dd.appendChild(el("a", { href: m.website }, m.website.replace(/^https?:\/\//, "")));
      dl.appendChild(dd);
    }
    $("about-copyright").textContent = m.copyright || (m.author ? "© " + m.author : "");
    $("about-rights").textContent = m.rights || "All rights reserved.";
    renderKeys();
  }
  function renderKeys() {
    const r = state.book.reader;
    const keys = [
      [["←", "→"], "Previous or next page (chapter when scrolling)"],
      [["Space"], "Next page in page mode"],
      r.enableSearch && [["Ctrl", "F"], "Search the book"],
      (r.enableBookmarks || canSelect()) && [["Ctrl", "D"], "Bookmarks and highlights"],
      r.showToc && [["Ctrl", "B"], "Show or hide the contents"],
      r.allowFontSizeChange && [["Ctrl", "+"], "Larger or smaller text (Ctrl −)"],
      [["F"], "Focus mode"],
      [["F11"], "Full screen"],
      [["Esc"], "Close panels, leave focus or full screen"],
    ].filter(Boolean);
    const dl = $("about-keys");
    dl.innerHTML = "";
    keys.forEach(([combo, what]) => {
      const dt = el("dt");
      combo.forEach((k) => dt.appendChild(el("kbd", null, k)));
      dl.append(dt, el("dd", null, what));
    });
  }

  function settingRow(label, control) {
    const row = el("div", { class: "setting-row" });
    row.append(el("span", { class: "setting-label" }, label), control);
    return row;
  }
  function renderSettings() {
    const r = state.book.reader;
    const body = $("about-settings-body");
    if (!body) return;
    body.innerHTML = "";
    if (r.allowThemeChange) {
      const seg = el("div", { class: "seg swatches", role: "group", "aria-label": "Theme" });
      [["light", "Light"], ["sepia", "Sepia"], ["dark", "Dark"], ["system", "Match Windows"]].forEach(([v, label]) => {
        const b = el("button", { type: "button", class: "swatch swatch-" + v, "aria-label": label, title: label, "aria-pressed": String((prefs.theme || r.defaultTheme) === v) });
        b.addEventListener("click", () => setPref("theme", v));
        seg.appendChild(b);
      });
      body.appendChild(settingRow("Theme", seg));
    }
    if (r.allowFontSizeChange) {
      const seg = el("div", { class: "seg" });
      const down = el("button", { type: "button", class: "size-btn small", "aria-label": "Smaller text" }, "A");
      const up = el("button", { type: "button", class: "size-btn large", "aria-label": "Larger text" }, "A");
      down.addEventListener("click", () => setPref("size", Math.max(14, currentSize() - 1)));
      up.addEventListener("click", () => setPref("size", Math.min(30, currentSize() + 1)));
      seg.append(down, el("span", { class: "size-value" }, currentSize() + " px"), up);
      body.appendChild(settingRow("Text size", seg));
    }
    const layout = el("div", { class: "seg seg-text", role: "group", "aria-label": "Layout" });
    [["scroll", "Scroll"], ["pages", "Pages"]].forEach(([v, label]) => {
      const b = el("button", { type: "button", "aria-pressed": String(currentLayout() === v) }, label);
      b.addEventListener("click", () => setPref("layout", v));
      layout.appendChild(b);
    });
    body.appendChild(settingRow("Layout", layout));
    if (r.rememberPosition) {
      const label = el("label", { class: "switch" });
      const box = el("input", { type: "checkbox", role: "switch" });
      box.checked = load("remember", true);
      box.addEventListener("change", () => save("remember", box.checked));
      label.append(el("span", null, "Remember my place"), box);
      body.appendChild(label);
    }
    const reset = el("button", { type: "button", class: "btn btn-secondary" }, "Reset reading settings");
    reset.addEventListener("click", () => {
      prefs = {};
      save("prefs", prefs);
      applyDisplay();
      relayout();
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
    document.querySelectorAll("[data-numbers]").forEach((b) => b.addEventListener("click", () => setPref("numbers", b.dataset.numbers)));
    document.querySelectorAll("[data-arrows]").forEach((b) => b.addEventListener("click", () => setPref("arrows", b.dataset.arrows)));
    $("arrow-prev").addEventListener("click", () => turn(-1));
    $("arrow-next").addEventListener("click", () => turn(1));
    wireSelection();
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
        if (!$("sel-menu").hidden) hideSelMenu();
        else if (!$("viewer").hidden) closeViewer();
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
      if (e.ctrlKey && e.key.toLowerCase() === "d" && (r.enableBookmarks || canSelect()) && !$("reader").hidden) {
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
    // Selecting and copying are separate choices. Code blocks' copy buttons always work.
    if (!canSelect()) document.body.classList.add("no-select");
    if (!r.allowCopy) {
      document.body.classList.add("no-copy");
      const block = (e) => {
        const sel = window.getSelection();
        const node = sel && sel.anchorNode;
        const at = node && (node.nodeType === 1 ? node : node.parentElement);
        if (at && at.closest("#chapter") && !at.closest(".bc-code")) e.preventDefault();
      };
      document.addEventListener("copy", block);
      document.addEventListener("cut", block);
    }
    if (!r.allowPrint) window.addEventListener("beforeprint", (e) => e.preventDefault());

    $("btn-search").hidden = !r.enableSearch;
    $("btn-bookmark").hidden = !(r.enableBookmarks || canSelect());
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
