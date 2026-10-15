<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { openSearchPanel } from "@codemirror/search";
import { markdown } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import { open } from "@tauri-apps/plugin-dialog";
import { api, ASSET_BASE, errorText, type Heading } from "../api";
import { BLOCKS, type BlockItem } from "../blocks";
import { previewDocument } from "../previewDoc";
import { loadAssets, notify, onFlush, store } from "../store";
import {
  Bold, ChevronDown, Code, Columns2, Eye, Heading1, Heading2, Heading3, Heading4, Heading5, Heading6, Pilcrow, Highlighter, Image as ImageIcon, ImageUp, Italic, Link,
  List, ListOrdered, ListTree, Moon, PenLine, Plus, Quote, Search, Strikethrough, Sun, SunMoon, Coffee, Table, TriangleAlert, X,
} from "@lucide/vue";
import { UiButton, UiIconButton, UiInput, UiMenu, UiSegmented, type MenuItem } from "../ui";
import { appTheme } from "../appTheme";

const layoutOptions = [
  { value: "editor" as const, icon: PenLine, title: "Editor only" },
  { value: "split" as const, icon: Columns2, title: "Editor and preview" },
  { value: "preview" as const, icon: Eye, title: "Preview only" },
];
const themeOptions = [
  { value: "auto", icon: SunMoon, title: "Match the app" },
  { value: "light", icon: Sun, title: "Light" },
  { value: "sepia", icon: Coffee, title: "Sepia" },
  { value: "dark", icon: Moon, title: "Dark" },
];

const editorEl = ref<HTMLElement>();
const frame = ref<HTMLIFrameElement>();
const view = shallowRef<EditorView>();
const status = ref<"saved" | "dirty" | "saving" | "error">("saved");
const loadedFile = ref("");
const words = ref(0);
const warnings = ref<string[]>([]);
const headings = ref<Heading[]>([]);
const layout = ref<"split" | "editor" | "preview">((localStorage.getItem("bc.layout") as any) || "split");
// "auto" follows the studio's own light/dark theme.
const previewTheme = ref(localStorage.getItem("bc.previewTheme") || "auto");
const effectiveTheme = computed(() => (previewTheme.value === "auto" ? appTheme.value : previewTheme.value));
const insertOpen = ref(false);
const insertFilter = ref("");
const imagePicker = ref(false);
const frameReady = ref(false);
const lastHtml = ref({ html: "", cls: "" });

let saveTimer = 0;
let renderTimer = 0;

const outlineItems = computed<MenuItem[]>(() =>
  headings.value.map((h) => ({ label: h.text, indent: h.level - 1, action: () => jumpTo(h) })),
);
const moreHeadings: MenuItem[] = [
  { label: "Heading 4", icon: Heading4, action: () => heading(4) },
  { label: "Heading 5", icon: Heading5, action: () => heading(5) },
  { label: "Heading 6", icon: Heading6, action: () => heading(6) },
  { separator: true },
  { label: "Paragraph", icon: Pilcrow, action: () => heading(0) },
];

const chapterTitle = computed(() => store.chapters.find((c) => c.file === store.currentChapter)?.title ?? "");
const disabledModules = computed(() => {
  const m = store.config?.modules as Record<string, { enabled: boolean }> | undefined;
  return new Set(Object.entries(m ?? {}).filter(([, v]) => !v.enabled).map(([k]) => k));
});
const filteredGroups = computed(() => {
  const q = insertFilter.value.trim().toLowerCase();
  return BLOCKS.map((g) => ({
    ...g,
    items: g.items.filter((i) => !q || i.label.toLowerCase().includes(q) || g.name.toLowerCase().includes(q)),
  })).filter((g) => g.items.length);
});
const images = computed(() => store.assets.filter((a) => a.kind === "image"));

/* ------------------------------------------------------------ editor */

const highlight = HighlightStyle.define([
  { tag: t.heading1, fontWeight: "700", fontSize: "1.25em", color: "var(--accent)" },
  { tag: t.heading2, fontWeight: "700", fontSize: "1.12em", color: "var(--accent)" },
  { tag: [t.heading3, t.heading4, t.heading5, t.heading6], fontWeight: "700", color: "var(--accent)" },
  { tag: t.strong, fontWeight: "700" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.strikethrough, textDecoration: "line-through" },
  { tag: [t.link, t.url], color: "var(--accent)" },
  { tag: t.quote, color: "var(--muted)", fontStyle: "italic" },
  { tag: t.monospace, color: "#b45309" },
  { tag: [t.processingInstruction, t.meta, t.contentSeparator], color: "var(--muted)" },
  { tag: t.list, color: "var(--ink)" },
]);

const theme = EditorView.theme({
  "&": { height: "100%", fontSize: "14.5px", backgroundColor: "var(--panel)", color: "var(--ink)" },
  ".cm-scroller": { fontFamily: "var(--mono)", lineHeight: "1.65" },
  ".cm-content": { padding: "20px 0 120px", caretColor: "var(--accent)", maxWidth: "860px" },
  ".cm-line": { padding: "0 22px" },
  ".cm-gutters": { backgroundColor: "var(--panel)", color: "var(--muted)", border: "none" },
  ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--accent) 5%, transparent)" },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--ink)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": { backgroundColor: "color-mix(in srgb, var(--accent) 22%, transparent) !important" },
  ".cm-cursor": { borderLeftColor: "var(--accent)" },
  ".cm-panels": { backgroundColor: "var(--panel)", color: "var(--ink)" },
  ".cm-panels.cm-panels-bottom": { borderTop: "1px solid var(--line)", boxShadow: "0 -8px 24px rgb(0 0 0 / 0.08)" },
  ".cm-searchMatch": { backgroundColor: "color-mix(in srgb, #facc15 40%, transparent)" },
  ".cm-searchMatch-selected": { backgroundColor: "color-mix(in srgb, #f59e0b 55%, transparent)" },
  /* Find & replace, in the studio's own controls: two rows, toggle chips, a close button. */
  ".cm-search": {
    position: "relative",
    display: "flex",
    flexWrap: "wrap",
    alignItems: "center",
    gap: "8px",
    padding: "12px 52px 12px 16px",
    fontFamily: "var(--font)",
    fontSize: "13px",
  },
  // Wraps on narrow panes; the invisible break keeps Replace on its own row.
  ".cm-search::before": { content: '""', order: "8", flexBasis: "100%", height: "0" },
  ".cm-search br": { display: "none" },
  ".cm-search [name=search]": { order: "1", flex: "1 1 200px" },
  ".cm-search [name=next]": { order: "2" },
  ".cm-search [name=prev]": { order: "3" },
  ".cm-search [name=select]": { order: "4" },
  ".cm-search > label": { order: "5" },
  ".cm-search input[name=replace]": { order: "9", flex: "1 1 200px" },
  ".cm-search button[name=replace]": { order: "10" },
  ".cm-search [name=replaceAll]": { order: "11" },
  ".cm-search .cm-textfield": {
    height: "32px",
    minWidth: "0",
    margin: "0",
    padding: "0 10px",
    border: "1px solid var(--line)",
    borderRadius: "8px",
    background: "var(--panel-2)",
    color: "var(--ink)",
    font: "inherit",
    outline: "none",
  },
  ".cm-search .cm-textfield:focus": { borderColor: "var(--accent)", boxShadow: "0 0 0 3px var(--accent-soft)" },
  ".cm-search .cm-button": {
    height: "32px",
    margin: "0",
    padding: "0 12px",
    border: "1px solid var(--line) !important",
    borderRadius: "8px",
    backgroundImage: "none !important",
    backgroundColor: "var(--panel) !important",
    color: "var(--ink)",
    font: "inherit",
    whiteSpace: "nowrap",
    cursor: "pointer",
  },
  ".cm-search .cm-button:hover": { backgroundColor: "var(--panel-2) !important" },
  ".cm-search label": {
    position: "relative",
    display: "inline-flex",
    alignItems: "center",
    height: "32px",
    margin: "0",
    padding: "0 12px",
    border: "1px solid var(--line)",
    borderRadius: "999px",
    color: "var(--muted)",
    fontSize: "12.5px",
    whiteSpace: "nowrap",
    cursor: "pointer",
    userSelect: "none",
  },
  ".cm-search label:has(input:checked)": { backgroundColor: "var(--accent-soft)", borderColor: "var(--accent)", color: "var(--accent)" },
  ".cm-search label:focus-within": { outline: "2px solid var(--accent)", outlineOffset: "1px" },
  ".cm-search input[type=checkbox]": { position: "absolute", opacity: "0", width: "0", height: "0", margin: "0" },
  ".cm-search [name=close]": {
    position: "absolute",
    top: "12px",
    right: "12px",
    width: "32px",
    height: "32px",
    padding: "0",
    border: "none",
    borderRadius: "8px",
    background: "none",
    color: "var(--muted)",
    fontSize: "20px",
    lineHeight: "1",
    cursor: "pointer",
  },
  ".cm-search [name=close]:hover": { backgroundColor: "var(--panel-2)", color: "var(--ink)" },
});

/** Clearer words for the find & replace panel. */
const phrases = EditorState.phrases.of({
  Find: "Find in chapter",
  Replace: "Replace with",
  next: "Next",
  previous: "Previous",
  all: "Select all",
  "match case": "Match case",
  regexp: "Regex",
  "by word": "Whole word",
  replace: "Replace",
  "replace all": "Replace all",
  close: "Close",
});

/** Ctrl+H: find & replace, with the cursor in the Replace field. */
function openReplace(v: EditorView) {
  openSearchPanel(v);
  setTimeout(() => v.dom.querySelector<HTMLInputElement>(".cm-search input[name=replace]")?.focus(), 0);
  return true;
}

function makeState(doc: string) {
  return EditorState.create({
    doc,
    extensions: [
      basicSetup,
      markdown(),
      syntaxHighlighting(highlight),
      EditorView.lineWrapping,
      theme,
      phrases,
      keymap.of([
        { key: "Mod-s", preventDefault: true, run: () => (saveNow(), true) },
        { key: "Mod-b", run: () => (wrap("**"), true) },
        { key: "Mod-i", run: () => (wrap("*"), true) },
        { key: "Mod-k", run: () => (link(), true) },
        { key: "Mod-e", run: () => (wrap("`"), true) },
        { key: "Mod-Shift-i", run: () => ((insertOpen.value = true), true) },
        { key: "Mod-h", preventDefault: true, run: openReplace },
      ]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) changed();
      }),
      EditorView.domEventHandlers({ scroll: () => syncScroll() }),
    ],
  });
}

function changed() {
  status.value = "dirty";
  clearTimeout(saveTimer);
  saveTimer = window.setTimeout(saveNow, 900);
  clearTimeout(renderTimer);
  renderTimer = window.setTimeout(renderNow, 220);
}

async function saveNow() {
  clearTimeout(saveTimer);
  if (!view.value || !loadedFile.value || status.value === "saved") return;
  const file = loadedFile.value;
  const text = view.value.state.doc.toString();
  status.value = "saving";
  try {
    store.chapters = await api.writeChapter(file, text);
    if (loadedFile.value === file && view.value.state.doc.toString() === text) status.value = "saved";
    else status.value = "dirty";
  } catch (e) {
    status.value = "error";
    notify(errorText(e), "error");
  }
}

async function load(file: string) {
  if (!file || !view.value) return;
  await saveNow();
  try {
    const text = await api.readChapter(file);
    loadedFile.value = file;
    view.value.setState(makeState(text));
    status.value = "saved";
    renderNow();
  } catch (e) {
    notify(errorText(e), "error");
  }
}

/* ------------------------------------------------------------ editing helpers */

function wrap(mark: string, end = mark) {
  const v = view.value!;
  const { from, to } = v.state.selection.main;
  const sel = v.state.sliceDoc(from, to);
  if (sel.startsWith(mark) && sel.endsWith(end) && sel.length >= mark.length + end.length) {
    v.dispatch({ changes: { from, to, insert: sel.slice(mark.length, sel.length - end.length) } });
  } else {
    v.dispatch({
      changes: { from, to, insert: mark + sel + end },
      selection: sel ? { anchor: from, head: to + mark.length + end.length } : { anchor: from + mark.length },
    });
  }
  v.focus();
}

function prefixLines(prefix: string | ((i: number) => string)) {
  const v = view.value!;
  const sel = v.state.selection.main;
  const first = v.state.doc.lineAt(sel.from).number;
  const last = v.state.doc.lineAt(sel.to).number;
  const changes = [];
  for (let n = first, i = 0; n <= last; n++, i++) {
    const line = v.state.doc.line(n);
    const p = typeof prefix === "string" ? prefix : prefix(i);
    if (typeof prefix === "string" && line.text.startsWith(p)) changes.push({ from: line.from, to: line.from + p.length, insert: "" });
    else changes.push({ from: line.from, insert: p });
  }
  const cs = v.state.changes(changes);
  // Map with assoc 1 so a cursor at the line start lands after the inserted marker.
  const selection = sel.empty ? { anchor: cs.mapPos(sel.head, 1) } : { anchor: cs.mapPos(sel.anchor, 1), head: cs.mapPos(sel.head, 1) };
  v.dispatch({ changes: cs, selection, scrollIntoView: true });
  v.focus();
}

/** Toggle a heading level on the current line; level 0 turns it back into a paragraph. */
function heading(level: number) {
  const v = view.value!;
  const head = v.state.selection.main.head;
  const line = v.state.doc.lineAt(head);
  // A heading replaces a list marker too (a list item can't be a heading).
  const marker = line.text.match(level ? /^(#{1,6}\s+|[-*+]\s+(\[[ xX]\]\s+)?|\d+[.)]\s+)/ : /^#{1,6}\s+/)?.[0] ?? "";
  const current = marker.startsWith("#") ? marker.trim().length : 0;
  const next = level === 0 || current === level ? "" : "#".repeat(level) + " ";
  // Keep the cursor at the same spot in the text, or at the end when it sat inside the old marker.
  const offset = Math.max(0, head - line.from - marker.length);
  const atStart = head - line.from <= marker.length;
  const textLen = line.length - marker.length;
  v.dispatch({
    changes: { from: line.from, to: line.from + marker.length, insert: next },
    selection: { anchor: line.from + next.length + (atStart ? textLen : offset) },
    scrollIntoView: true,
  });
  v.focus();
}

function link() {
  const v = view.value!;
  const { from, to } = v.state.selection.main;
  const sel = v.state.sliceDoc(from, to) || "link text";
  const insert = `[${sel}](https://)`;
  v.dispatch({ changes: { from, to, insert }, selection: { anchor: from + sel.length + 3, head: from + insert.length - 1 } });
  v.focus();
}

function insertBlock(item: BlockItem) {
  const v = view.value!;
  const { from, to } = v.state.selection.main;
  const sel = v.state.sliceDoc(from, to);
  let text = item.snippet.replace("$SEL", sel);
  let cursor = text.indexOf("$0");
  text = text.replace("$0", "");
  if (cursor < 0) cursor = text.length;
  let at = from;
  let end = to;
  let prefix = "";
  let suffix = "";
  if (!item.inline) {
    const line = v.state.doc.lineAt(from);
    if (!sel && line.text.trim() !== "") {
      at = end = line.to;
      prefix = "\n\n";
    } else if (from !== line.from) {
      prefix = "\n\n";
    } else if (line.number > 1 && v.state.doc.line(line.number - 1).text.trim() !== "") {
      prefix = "\n";
    }
    const after = v.state.doc.sliceString(end, Math.min(v.state.doc.length, end + 2));
    suffix = after.startsWith("\n\n") ? "" : after.startsWith("\n") ? "\n" : end < v.state.doc.length ? "\n\n" : "\n";
  }
  v.dispatch({
    changes: { from: at, to: end, insert: prefix + text + suffix },
    selection: { anchor: at + prefix.length + cursor },
    scrollIntoView: true,
  });
  insertOpen.value = false;
  insertFilter.value = "";
  v.focus();
  if (item.module && disabledModules.value.has(item.module)) {
    notify(`This block's module is turned off. Turn it on in Blocks & features to see it in the book.`);
  }
}

function insertImage(path: string) {
  const name = path.split("/").pop()!.replace(/\.[^.]+$/, "").replace(/[-_]+/g, " ");
  insertBlock({ label: "Image", snippet: `![${name}](${path} "$0")` });
  imagePicker.value = false;
}

async function importImage() {
  const picked = await open({ multiple: true, filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp", "svg", "avif"] }] });
  if (!picked) return;
  const list = Array.isArray(picked) ? picked : [picked];
  try {
    const added = await api.importAssets(list, "images");
    await loadAssets();
    if (added[0]) insertImage(added[0]);
  } catch (e) {
    notify(errorText(e), "error");
  }
}

function jumpTo(h: Heading) {
  const v = view.value!;
  const doc = v.state.doc;
  for (let n = 1; n <= doc.lines; n++) {
    const text = doc.line(n).text;
    if (/^#{1,6}\s/.test(text) && text.includes(h.text.slice(0, 40))) {
      v.dispatch({ selection: { anchor: doc.line(n).from }, scrollIntoView: true });
      v.focus();
      break;
    }
  }
  post({ type: "reveal", id: h.id });
}

/* ------------------------------------------------------------ preview */

function post(msg: unknown) {
  frame.value?.contentWindow?.postMessage(msg, "*");
}

async function renderNow() {
  clearTimeout(renderTimer);
  if (!view.value) return;
  try {
    const r = await api.render(view.value.state.doc.toString(), loadedFile.value || null);
    words.value = r.words;
    warnings.value = r.warnings;
    headings.value = r.headings.filter((h) => h.level <= 3);
    lastHtml.value = { html: r.html, cls: r.chapterClass };
    if (frameReady.value) post({ type: "render", html: r.html, cls: r.chapterClass, theme: effectiveTheme.value });
  } catch (e) {
    warnings.value = [errorText(e)];
  }
}

function syncScroll() {
  if (layout.value !== "split" || !view.value) return;
  const s = view.value.scrollDOM;
  const max = s.scrollHeight - s.clientHeight;
  post({ type: "scroll", ratio: max > 0 ? s.scrollTop / max : 0 });
}

function onMessage(e: MessageEvent) {
  if (!frame.value || e.source !== frame.value.contentWindow) return;
  if (e.data?.type === "ready") {
    frameReady.value = true;
    // The theme travels with every render, so a change made while the frame reloaded is not lost.
    post({ type: "render", ...lastHtml.value, theme: effectiveTheme.value });
  }
}

const srcdoc = ref("");
function reloadFrame() {
  frameReady.value = false;
  srcdoc.value = previewDocument(effectiveTheme.value, store.configVersion);
}

function setPreviewTheme(t: string) {
  previewTheme.value = t;
  localStorage.setItem("bc.previewTheme", t);
}

watch(effectiveTheme, (theme) => post({ type: "theme", theme }));

function setLayout(l: "split" | "editor" | "preview") {
  layout.value = l;
  localStorage.setItem("bc.layout", l);
}

/* ------------------------------------------------------------ lifecycle */

watch(() => store.currentChapter, (f) => load(f));
watch(() => store.configVersion, () => {
  reloadFrame();
  renderNow();
});

const stopFlush = onFlush(saveNow);

onMounted(async () => {
  view.value = new EditorView({ parent: editorEl.value!, state: makeState("") });
  window.addEventListener("message", onMessage);
  reloadFrame();
  await nextTick();
  await load(store.currentChapter);
});

onBeforeUnmount(() => {
  stopFlush();
  saveNow();
  window.removeEventListener("message", onMessage);
  view.value?.destroy();
});

const statusText = computed(
  () => ({ saved: "Saved", dirty: "Editing…", saving: "Saving…", error: "Not saved" })[status.value]
);
</script>

<template>
  <header class="topbar">
    <h1 class="ch-title">{{ chapterTitle || "No chapter" }}</h1>
    <span class="status" :class="status">{{ statusText }} · {{ words.toLocaleString() }} words</span>
    <span class="spacer"></span>
    <UiSegmented :model-value="layout" :options="layoutOptions" aria-label="Layout" size="sm" @update:model-value="setLayout($event as any)" />
  </header>

  <div class="toolbar" role="toolbar" aria-label="Formatting">
    <UiIconButton :icon="Heading1" label="Heading 1" size="sm" @click="heading(1)" />
    <UiIconButton :icon="Heading2" label="Heading 2" size="sm" @click="heading(2)" />
    <UiIconButton :icon="Heading3" label="Heading 3" size="sm" @click="heading(3)" />
    <UiMenu :items="moreHeadings" align="start">
      <UiIconButton :icon="ChevronDown" label="More headings" size="sm" aria-haspopup="menu" />
    </UiMenu>
    <span class="sep"></span>
    <UiIconButton :icon="Bold" label="Bold (Ctrl+B)" size="sm" @click="wrap('**')" />
    <UiIconButton :icon="Italic" label="Italic (Ctrl+I)" size="sm" @click="wrap('*')" />
    <UiIconButton :icon="Strikethrough" label="Strikethrough" size="sm" @click="wrap('~~')" />
    <UiIconButton :icon="Highlighter" label="Highlight" size="sm" @click="wrap('==')" />
    <UiIconButton :icon="Code" label="Inline code (Ctrl+E)" size="sm" @click="wrap('`')" />
    <UiIconButton :icon="Link" label="Link (Ctrl+K)" size="sm" @click="link" />
    <span class="sep"></span>
    <UiIconButton :icon="Quote" label="Quote" size="sm" @click="prefixLines('> ')" />
    <UiIconButton :icon="List" label="Bullet list" size="sm" @click="prefixLines('- ')" />
    <UiIconButton :icon="ListOrdered" label="Numbered list" size="sm" @click="prefixLines((i) => `${i + 1}. `)" />
    <UiIconButton :icon="ImageIcon" label="Image" size="sm" :pressed="imagePicker" @click="imagePicker = !imagePicker; insertOpen = false" />
    <UiIconButton :icon="Table" label="Table" size="sm" @click="insertBlock(BLOCKS[0].items[5])" />
    <span class="sep"></span>
    <UiButton variant="primary" size="sm" :icon="Plus" title="Insert a block (Ctrl+Shift+I)" :aria-expanded="insertOpen" @click="insertOpen = !insertOpen; imagePicker = false">Insert</UiButton>
    <UiMenu :items="outlineItems" align="start">
      <UiButton variant="ghost" size="sm" :icon="ListTree" :disabled="!headings.length" aria-haspopup="menu">Outline</UiButton>
    </UiMenu>
    <span class="spacer"></span>
    <UiSegmented v-if="layout !== 'editor'" :model-value="previewTheme" :options="themeOptions" aria-label="Preview theme" size="sm" @update:model-value="setPreviewTheme($event as string)" />
  </div>

  <div v-if="warnings.length" class="warnings notice warn">
    <TriangleAlert class="w-ico" aria-hidden="true" />
    <span>{{ warnings.join(" · ") }}</span>
  </div>

  <div class="workspace" :class="layout">
    <div class="editor-pane" ref="editorEl"></div>
    <div class="preview-pane">
      <iframe ref="frame" class="preview-frame" sandbox="allow-scripts" title="Preview" :srcdoc="srcdoc"></iframe>
    </div>

    <div v-if="insertOpen" class="insert-panel" @keydown.esc="insertOpen = false">
      <div class="insert-head">
        <UiInput v-model="insertFilter" :icon="Search" placeholder="Find a block — quote, chart, quiz…" autofocus aria-label="Find a block" />
        <UiIconButton :icon="X" label="Close" @click="insertOpen = false" />
      </div>
      <div class="insert-groups">
        <section v-for="g in filteredGroups" :key="g.name">
          <h4>{{ g.name }}</h4>
          <button
            v-for="item in g.items"
            :key="item.label"
            type="button"
            class="insert-item"
            :class="{ off: item.module && disabledModules.has(item.module) }"
            :title="item.module && disabledModules.has(item.module) ? 'This module is turned off' : item.snippet.replace(/\$SEL|\$0/g, '')"
            @click="insertBlock(item)"
          >{{ item.label }}</button>
        </section>
      </div>
    </div>

    <div v-if="imagePicker" class="insert-panel image-panel">
      <div class="insert-head">
        <b>Insert an image</b>
        <span class="spacer"></span>
        <UiButton size="sm" :icon="ImageUp" @click="importImage">Import from computer…</UiButton>
        <UiIconButton :icon="X" label="Close" @click="imagePicker = false" />
      </div>
      <div v-if="!images.length" class="empty">No images yet. Import one to add it to the book’s assets.</div>
      <div class="thumbs">
        <button v-for="img in images" :key="img.path" type="button" class="thumb" @click="insertImage(img.path)" :title="img.path">
          <img :src="`${ASSET_BASE}/project/${img.path}`" alt="" loading="lazy" />
          <span>{{ img.path.replace("assets/", "") }}</span>
        </button>
      </div>
    </div>

  </div>
</template>

<style scoped>
.ch-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 40vw; }
.status.dirty, .status.saving { color: var(--muted); }
.status.error { color: var(--danger); }
.w-ico { width: 16px; height: 16px; flex: none; }
.toolbar { display: flex; align-items: center; gap: 2px; padding: 4px 10px; border-bottom: 1px solid var(--line); background: var(--panel); flex-wrap: wrap; flex: none; }
.toolbar .sep { width: 1px; height: 20px; background: var(--line); margin: 0 6px; }
.toolbar .spacer { flex: 1; }
.toolbar .btn { margin-left: 4px; }
.warnings { display: flex; gap: 8px; align-items: flex-start; margin: 8px 12px 0; padding: 8px 12px; font-size: 12px; flex: none; }
/* One row that fills the space; panes keep fixed columns so hiding one never collapses the other. */
.workspace { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); grid-template-rows: minmax(0, 1fr); position: relative; }
.workspace.editor { grid-template-columns: minmax(0, 1fr) 0; }
.workspace.preview { grid-template-columns: 0 minmax(0, 1fr); }
.workspace.editor .preview-pane, .workspace.preview .editor-pane { display: none; }
.editor-pane { grid-column: 1; grid-row: 1; min-height: 0; overflow: hidden; border-right: 1px solid var(--line); background: var(--panel); }
.workspace.editor .editor-pane :deep(.cm-content) { margin: 0 auto; }
.preview-pane { grid-column: 2; grid-row: 1; position: relative; min-height: 0; background: var(--bg); }
.preview-frame { position: absolute; inset: 0; width: 100%; height: 100%; border: 0; display: block; }
.insert-panel { position: absolute; top: 12px; left: 16px; right: 16px; max-height: calc(100% - 24px); z-index: 20; background: var(--panel); border: 1px solid var(--line); border-radius: 14px; box-shadow: 0 18px 50px rgb(0 0 0 / 0.25); display: flex; flex-direction: column; overflow: hidden; }
.insert-head { display: flex; align-items: center; gap: 10px; padding: 14px 16px; border-bottom: 1px solid var(--line); }
.insert-head :deep(.ui-input) { flex: 1; }
.insert-head .spacer { flex: 1; }
/* Groups are small cards packed into columns, with room between them and around the edges. */
.insert-groups { overflow-y: auto; overscroll-behavior: none; padding: 18px; columns: 4 220px; column-gap: 16px; }
.insert-groups section { break-inside: avoid; margin-bottom: 16px; padding: 10px 8px 8px; border: 1px solid var(--line); border-radius: 12px; background: var(--panel-2); }
.insert-groups h4 { margin: 2px 8px 8px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--muted); }
.insert-item { display: block; width: 100%; text-align: left; border: 0; background: none; padding: 7px 10px; margin: 1px 0; border-radius: 8px; cursor: pointer; font-size: 13px; color: var(--ink); }
.insert-item:hover { background: var(--accent-soft); color: var(--accent); }
.insert-item.off { color: var(--muted); }
.insert-item.off::after { content: " (off)"; font-size: 11px; }
.image-panel .empty { padding: 24px; color: var(--muted); text-align: center; }
.thumbs { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 10px; padding: 12px; overflow-y: auto; }
.thumb { border: 1px solid var(--line); border-radius: 8px; background: var(--panel-2); padding: 6px; cursor: pointer; display: flex; flex-direction: column; gap: 6px; font-size: 11px; color: var(--muted); }
.thumb:hover { border-color: var(--accent); }
.thumb img { width: 100%; aspect-ratio: 4 / 3; object-fit: contain; border-radius: 4px; background: var(--panel); }
.thumb span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
