/* Book Craft block runtime — makes rendered chapters interactive.
   Used by the studio preview and every generated reader.

   BookBlocks.hydrate(rootElement, { vendor: "/_runtime/vendor/", theme: "light" })
   BookBlocks.setTheme("dark")  // re-renders diagrams for the new theme
*/
(function () {
  "use strict";

  const loaded = new Map();
  let vendorBase = "/_runtime/vendor/";
  let currentTheme = "light";
  let runCounter = 0;

  function loadScript(src) {
    if (!loaded.has(src)) {
      loaded.set(
        src,
        new Promise((resolve, reject) => {
          const s = document.createElement("script");
          s.src = src;
          s.onload = () => resolve();
          s.onerror = () => reject(new Error("Could not load " + src));
          document.head.appendChild(s);
        })
      );
    }
    return loaded.get(src);
  }

  function loadStyle(href) {
    if (!loaded.has(href)) {
      const l = document.createElement("link");
      l.rel = "stylesheet";
      l.href = href;
      document.head.appendChild(l);
      loaded.set(href, Promise.resolve());
    }
    return loaded.get(href);
  }

  function el(tag, attrs, text) {
    const e = document.createElement(tag);
    if (attrs) for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
    if (text != null) e.textContent = text;
    return e;
  }

  /* ---------------------------------------------------------------- math */

  async function hydrateMath(root) {
    const nodes = root.querySelectorAll("[data-math-style]:not([data-bc-done])");
    if (!nodes.length) return;
    try {
      loadStyle(vendorBase + "katex/katex.min.css");
      await loadScript(vendorBase + "katex/katex.min.js");
      await loadScript(vendorBase + "katex/contrib/mhchem.min.js").catch(() => {});
    } catch (e) {
      nodes.forEach((n) => n.classList.add("bc-math-pending"));
      return;
    }
    nodes.forEach((n) => {
      const tex = n.textContent;
      n.setAttribute("data-tex", tex);
      try {
        window.katex.render(tex, n, {
          displayMode: n.getAttribute("data-math-style") === "display",
          throwOnError: false,
          trust: false,
          strict: "ignore",
        });
      } catch (e) {
        n.classList.add("bc-math-pending");
      }
      n.setAttribute("data-bc-done", "1");
    });
  }

  /* ---------------------------------------------------------------- diagrams */

  async function hydrateMermaid(root, force) {
    const nodes = Array.from(root.querySelectorAll("pre.mermaid"));
    if (!nodes.length) return;
    nodes.forEach((n) => {
      if (!n.hasAttribute("data-src")) n.setAttribute("data-src", n.textContent);
      if (force) {
        n.removeAttribute("data-processed");
        n.textContent = n.getAttribute("data-src");
      }
    });
    try {
      await loadScript(vendorBase + "mermaid.min.js");
    } catch (e) {
      return;
    }
    window.mermaid.initialize(mermaidConfig(root));
    try {
      await window.mermaid.run({ nodes: nodes.filter((n) => !n.hasAttribute("data-processed")) });
    } catch (e) {
      /* a broken diagram shows its source */
    }
  }

  // Mermaid needs plain colours, so CSS colours (including color-mix) are resolved
  // to rgb() by painting them on a 1×1 canvas.
  const probe = document.createElement("canvas").getContext("2d", { willReadFrequently: true });
  function resolveColor(cssValue, scope) {
    const el = document.createElement("span");
    el.style.color = cssValue;
    el.style.display = "none";
    (scope || document.body).appendChild(el);
    const computed = getComputedStyle(el).color;
    el.remove();
    probe.clearRect(0, 0, 1, 1);
    probe.fillStyle = "#000";
    probe.fillStyle = computed;
    probe.fillRect(0, 0, 1, 1);
    const [r, g, b] = probe.getImageData(0, 0, 1, 1).data;
    return "#" + [r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("");
  }

  function mermaidConfig(root) {
    const scope = root && root.nodeType === 1 ? root : document.body;
    const c = (v) => resolveColor(`var(${v})`, scope);
    const font = getComputedStyle(scope).getPropertyValue("--bc-font-ui").trim() || "Segoe UI, sans-serif";
    const paper = c("--bc-paper");
    const surface = c("--bc-surface-2");
    const ink = c("--bc-ink");
    const muted = c("--bc-muted");
    const border = c("--bc-border-strong");
    const accent = c("--bc-accent-ink");
    const soft = c("--bc-soft");
    return {
      startOnLoad: false,
      securityLevel: "strict",
      theme: "base",
      look: "classic",
      fontFamily: font,
      flowchart: { curve: "basis", padding: 14, htmlLabels: true },
      themeVariables: {
        darkMode: currentTheme === "dark",
        fontFamily: font,
        fontSize: "15px",
        background: paper,
        mainBkg: surface,
        primaryColor: surface,
        primaryTextColor: ink,
        primaryBorderColor: border,
        secondaryColor: soft,
        secondaryTextColor: ink,
        secondaryBorderColor: accent,
        tertiaryColor: paper,
        tertiaryTextColor: ink,
        tertiaryBorderColor: border,
        lineColor: muted,
        textColor: ink,
        nodeBorder: border,
        clusterBkg: paper,
        clusterBorder: border,
        edgeLabelBackground: paper,
        titleColor: ink,
        actorBkg: surface,
        actorBorder: border,
        actorTextColor: ink,
        signalColor: muted,
        signalTextColor: ink,
        noteBkgColor: soft,
        noteTextColor: ink,
        noteBorderColor: accent,
      },
      themeCSS:
        ".node rect, .node polygon, .actor, .note, rect.task { rx: 8px; ry: 8px; } " +
        ".node rect, .node polygon, .node circle { filter: none !important; stroke-width: 1px; } " +
        ".edgeLabel, .edgeLabel p { background: " + paper + "; color: " + muted + "; } " +
        ".marker { fill: " + muted + "; stroke: " + muted + "; }",
    };
  }

  /* ---------------------------------------------------------------- code */

  function codeText(figure) {
    return Array.from(figure.querySelectorAll(".bc-line"))
      .map((l) => (l.classList.contains("bc-cmd") ? l.textContent.replace(/^(\S*>|\$|#|%)\s/, "") : l.textContent))
      .join("\n");
  }

  function hydrateCode(root) {
    root.querySelectorAll("figure.bc-code:not([data-bc-done])").forEach((fig) => {
      fig.setAttribute("data-bc-done", "1");
      const tools = el("div", { class: "bc-code-tools" });
      if (fig.dataset.run === "js") {
        const run = el("button", { type: "button", class: "bc-run" }, "Run");
        run.addEventListener("click", () => runJs(fig));
        tools.appendChild(run);
      }
      if (fig.hasAttribute("data-copy")) {
        const copy = el("button", { type: "button", "aria-label": "Copy code" }, "Copy");
        copy.addEventListener("click", async () => {
          try {
            await navigator.clipboard.writeText(codeText(fig));
            copy.textContent = "Copied";
            copy.classList.add("bc-done");
          } catch (e) {
            copy.textContent = "Select and press Ctrl+C";
          }
          setTimeout(() => {
            copy.textContent = "Copy";
            copy.classList.remove("bc-done");
          }, 1600);
        });
        tools.appendChild(copy);
      }
      if (tools.children.length) fig.appendChild(tools);
    });
  }

  function runJs(fig) {
    let out = fig.querySelector(".bc-code-output");
    if (!out) {
      out = el("pre", { class: "bc-code-output", "aria-live": "polite" });
      fig.appendChild(out);
    }
    out.textContent = "";
    const old = fig.querySelector("iframe.bc-runner");
    if (old) old.remove();
    const id = "run" + ++runCounter;
    const code = codeText(fig);
    // The sandbox has no same-origin access: it cannot reach the book, the disk or the network APIs of the app.
    const html =
      "<!doctype html><meta charset=utf-8><script>" +
      "const send=(k,a)=>parent.postMessage({bcRun:" + JSON.stringify(id) + ",kind:k,text:a.map(v=>{try{return typeof v==='string'?v:JSON.stringify(v,null,2)}catch(e){return String(v)}}).join(' ')},'*');" +
      "['log','info','warn','error'].forEach(k=>{console[k]=(...a)=>send(k,a)});" +
      "window.onerror=(m)=>{send('error',[String(m)]);};" +
      "window.onunhandledrejection=(e)=>send('error',[String(e.reason)]);" +
      "<\/script><script>" + code.replace(/<\/script/gi, "<\\/script") + "\n;parent.postMessage({bcRun:" + JSON.stringify(id) + ",kind:'done'},'*');<\/script>";
    const frame = el("iframe", { sandbox: "allow-scripts", class: "bc-runner", title: "Code runner", hidden: "" });
    frame.srcdoc = html;
    const timer = setTimeout(() => finish("Stopped after 5 seconds."), 5000);
    function finish(note) {
      clearTimeout(timer);
      window.removeEventListener("message", onMessage);
      if (note) out.appendChild(el("span", { class: "bc-out-error" }, note + "\n"));
      if (!out.textContent) out.textContent = "(no output)";
      frame.remove();
    }
    function onMessage(e) {
      if (e.source !== frame.contentWindow || !e.data || e.data.bcRun !== id) return;
      if (e.data.kind === "done") return finish();
      const line = el("span", e.data.kind === "error" ? { class: "bc-out-error" } : null, e.data.text + "\n");
      out.appendChild(line);
    }
    window.addEventListener("message", onMessage);
    fig.appendChild(frame);
  }

  /* ---------------------------------------------------------------- layout */

  function hydrateTabs(root) {
    root.querySelectorAll(".bc-tabs:not([data-bc-done])").forEach((tabs, ti) => {
      tabs.setAttribute("data-bc-done", "1");
      const panels = Array.from(tabs.children).filter((c) => c.classList.contains("bc-tab"));
      if (!panels.length) return;
      const bar = el("div", { class: "bc-tabbar", role: "tablist" });
      panels.forEach((p, i) => {
        const pid = p.id || "bc-tab-" + ti + "-" + i + "-" + Math.random().toString(36).slice(2, 7);
        p.id = pid;
        p.setAttribute("role", "tabpanel");
        const b = el("button", { type: "button", role: "tab", "aria-controls": pid }, p.dataset.title || "Tab " + (i + 1));
        b.addEventListener("click", () => select(i));
        b.addEventListener("keydown", (e) => {
          if (e.key === "ArrowRight") select((i + 1) % panels.length, true);
          if (e.key === "ArrowLeft") select((i - 1 + panels.length) % panels.length, true);
        });
        bar.appendChild(b);
      });
      tabs.insertBefore(bar, tabs.firstChild);
      function select(n, focus) {
        panels.forEach((p, i) => (p.hidden = i !== n));
        Array.from(bar.children).forEach((b, i) => {
          b.setAttribute("aria-selected", String(i === n));
          b.tabIndex = i === n ? 0 : -1;
          if (focus && i === n) b.focus();
        });
      }
      select(0);
    });
  }

  /* ---------------------------------------------------------------- learning */

  function hydrateQuiz(root) {
    root.querySelectorAll(".bc-quiz:not([data-bc-done])").forEach((quiz) => {
      quiz.setAttribute("data-bc-done", "1");
      const list = quiz.querySelector("ul.contains-task-list, ul");
      if (!list) return;
      const multiple = quiz.dataset.multiple === "true";
      const options = [];
      list.querySelectorAll(":scope > li").forEach((li) => {
        const box = li.querySelector("input[type=checkbox]");
        const correct = !!(box && box.checked);
        if (box) box.remove();
        const btn = el("button", { type: "button", class: "bc-quiz-option", "aria-pressed": "false" });
        while (li.firstChild) btn.appendChild(li.firstChild);
        li.appendChild(btn);
        li.className = "";
        options.push({ btn, correct });
        btn.addEventListener("click", () => {
          if (quiz.classList.contains("bc-answered")) return;
          if (!multiple) options.forEach((o) => o.btn.setAttribute("aria-pressed", "false"));
          btn.setAttribute("aria-pressed", btn.getAttribute("aria-pressed") === "true" ? "false" : "true");
          if (!multiple) check();
        });
      });
      list.classList.remove("contains-task-list");
      list.style.listStyle = "none";
      list.style.paddingLeft = "0";
      list.style.display = "grid";
      list.style.gap = "0.5em";
      const actions = el("div", { class: "bc-quiz-actions" });
      const result = el("span", { class: "bc-quiz-result", "aria-live": "polite" });
      if (multiple) {
        const checkBtn = el("button", { type: "button" }, "Check");
        checkBtn.addEventListener("click", check);
        actions.appendChild(checkBtn);
      }
      const retry = el("button", { type: "button", class: "bc-secondary", hidden: "" }, "Try again");
      retry.addEventListener("click", () => {
        quiz.classList.remove("bc-answered");
        options.forEach((o) => {
          o.btn.classList.remove("bc-correct", "bc-wrong");
          o.btn.setAttribute("aria-pressed", "false");
        });
        result.textContent = "";
        retry.hidden = true;
      });
      actions.append(retry, result);
      list.after(actions);
      function check() {
        let ok = true;
        options.forEach((o) => {
          const picked = o.btn.getAttribute("aria-pressed") === "true";
          if (picked !== o.correct) ok = false;
          if (o.correct) o.btn.classList.add("bc-correct");
          else if (picked) o.btn.classList.add("bc-wrong");
        });
        quiz.classList.add("bc-answered");
        result.textContent = ok ? "Correct!" : "Not quite.";
        result.style.color = ok ? "var(--bc-green)" : "var(--bc-red)";
        retry.hidden = false;
      }
    });
  }

  function hydrateFlashcards(root) {
    root.querySelectorAll(".bc-flashcards:not([data-bc-done])").forEach((box) => {
      box.setAttribute("data-bc-done", "1");
      box.querySelectorAll("dl").forEach((dl) => {
        const grid = el("div", { class: "bc-cards" });
        let front = null;
        Array.from(dl.children).forEach((c) => {
          if (c.tagName === "DT") front = c;
          else if (c.tagName === "DD" && front) {
            const card = el("button", { type: "button", class: "bc-flashcard", "aria-label": "Flash card: " + front.textContent + ". Press to turn over." });
            const inner = el("span", { class: "bc-flashcard-inner" });
            const f = el("span", { class: "bc-flashcard-face bc-flashcard-front" });
            const b = el("span", { class: "bc-flashcard-face bc-flashcard-back" });
            while (front.firstChild) f.appendChild(front.firstChild);
            f.appendChild(el("span", { class: "bc-flashcard-hint" }, "Click to turn"));
            while (c.firstChild) b.appendChild(c.firstChild);
            inner.append(f, b);
            card.appendChild(inner);
            card.addEventListener("click", () => card.classList.toggle("bc-flipped"));
            grid.appendChild(card);
            front = null;
          }
        });
        dl.replaceWith(grid);
      });
    });
  }

  /* ---------------------------------------------------------------- media */

  function hydrateMedia(root) {
    root.querySelectorAll(".bc-compare-frame:not([data-bc-done])").forEach((f) => {
      f.setAttribute("data-bc-done", "1");
      const top = f.querySelector(".bc-compare-top");
      const range = f.querySelector(".bc-compare-range");
      if (!top || !range) return;
      const update = () => {
        top.style.clipPath = "inset(0 " + (100 - range.value) + "% 0 0)";
        f.style.setProperty("--bc-split", range.value + "%");
      };
      range.addEventListener("input", update);
      update();
    });
    root.querySelectorAll(".spoiler:not([data-bc-done])").forEach((s) => {
      s.setAttribute("data-bc-done", "1");
      s.tabIndex = 0;
      s.addEventListener("click", () => s.classList.toggle("bc-revealed"));
    });
    root.querySelectorAll(".bc-image img, .bc-figure img, .bc-gallery img").forEach((img) => {
      if (img.dataset.bcZoom) return;
      img.dataset.bcZoom = "1";
      img.addEventListener("click", () => {
        const box = el("div", { class: "bc-lightbox", role: "dialog", "aria-label": img.alt || "Image" });
        const big = el("img", { src: img.currentSrc || img.src, alt: img.alt || "" });
        box.appendChild(big);
        const close = () => {
          box.remove();
          document.removeEventListener("keydown", onKey);
        };
        const onKey = (e) => e.key === "Escape" && close();
        box.addEventListener("click", close);
        document.addEventListener("keydown", onKey);
        document.body.appendChild(box);
      });
    });
  }

  /* ---------------------------------------------------------------- drop caps */

  // A drop cap sinks two lines, so it looks broken on a first paragraph that fits on one.
  // Measure the paragraph without it and keep it only when the text wraps; re-check on resize.
  let dropCapWatch = null;
  function checkDropCap(p) {
    p.classList.add("bc-dropcap-short");
    const lineHeight = parseFloat(getComputedStyle(p).lineHeight) || 1;
    const lines = Math.round(p.getBoundingClientRect().height / lineHeight);
    if (lines >= 2) p.classList.remove("bc-dropcap-short");
  }
  function hydrateDropCaps(root) {
    const paragraphs = root.querySelectorAll(":scope.bc-dropcaps > h1 + p, .bc-dropcaps > h1 + p");
    if (!paragraphs.length) return;
    if (!dropCapWatch && window.ResizeObserver) {
      let lastWidth = new WeakMap();
      dropCapWatch = new ResizeObserver((entries) => {
        entries.forEach((e) => {
          const w = Math.round(e.contentRect.width);
          if (lastWidth.get(e.target) === w) return; // only a width change can change the line count
          lastWidth.set(e.target, w);
          checkDropCap(e.target);
        });
      });
    }
    paragraphs.forEach((p) => {
      checkDropCap(p);
      if (dropCapWatch) dropCapWatch.observe(p);
    });
  }

  /* ---------------------------------------------------------------- api */

  async function hydrate(root, options) {
    options = options || {};
    if (options.vendor) vendorBase = options.vendor;
    if (options.theme) currentTheme = options.theme;
    hydrateCode(root);
    hydrateTabs(root);
    hydrateQuiz(root);
    hydrateFlashcards(root);
    hydrateMedia(root);
    hydrateDropCaps(root);
    await Promise.all([hydrateMath(root), hydrateMermaid(root, false)]);
  }

  function setTheme(theme, root) {
    if (theme === currentTheme) return;
    currentTheme = theme;
    if (window.mermaid) hydrateMermaid(root || document, true);
  }

  window.BookBlocks = { hydrate, setTheme, version: 1 };
})();
