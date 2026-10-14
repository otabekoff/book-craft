<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { markdown } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import { open } from "@tauri-apps/plugin-dialog";
import { api, ASSET_BASE, errorText, type Heading } from "../api";
import { BLOCKS, type BlockItem } from "../blocks";
import { previewDocument } from "../previewDoc";
import { loadAssets, notify, onFlush, store } from "../store";
import {
  Bold, Code, Columns2, Eye, Heading1, Heading2, Heading3, Highlighter, Image as ImageIcon, ImageUp, Italic, Link,
  List, ListOrdered, ListTree, Moon, PenLine, Plus, Quote, Search, Strikethrough, Sun, Coffee, Table, TriangleAlert, X,
} from "@lucide/vue";
import { UiButton, UiIconButton, UiInput, UiSegmented } from "../ui";

const layoutOptions = [
  { value: "editor" as const, icon: PenLine, title: "Editor only" },
  { value: "split" as const, icon: Columns2, title: "Editor and preview" },
  { value: "preview" as const, icon: Eye, title: "Preview only" },
];
const themeOptions = [
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
const previewTheme = ref(localStorage.getItem("bc.previewTheme") || "light");
const insertOpen = ref(false);
const insertFilter = ref("");
const outlineOpen = ref(false);
const imagePicker = ref(false);
const frameReady = ref(false);
const lastHtml = ref({ html: "", cls: "" });

let saveTimer = 0;
let renderTimer = 0;

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
  ".cm-panels": { backgroundColor: "var(--panel-2)", color: "var(--ink)" },
  ".cm-searchMatch": { backgroundColor: "color-mix(in srgb, #facc15 40%, transparent)" },
});

function makeState(doc: string) {
  return EditorState.create({
    doc,
    extensions: [
      basicSetup,
      markdown(),
      syntaxHighlighting(highlight),
      EditorView.lineWrapping,
      theme,
      keymap.of([
        { key: "Mod-s", preventDefault: true, run: () => (saveNow(), true) },
        { key: "Mod-b", run: () => (wrap("**"), true) },
        { key: "Mod-i", run: () => (wrap("*"), true) },
        { key: "Mod-k", run: () => (link(), true) },
        { key: "Mod-e", run: () => (wrap("`"), true) },
        { key: "Mod-Shift-i", run: () => ((insertOpen.value = true), true) },
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
  const { from, to } = v.state.selection.main;
  const first = v.state.doc.lineAt(from).number;
  const last = v.state.doc.lineAt(to).number;
  const changes = [];
  for (let n = first, i = 0; n <= last; n++, i++) {
    const line = v.state.doc.line(n);
    const p = typeof prefix === "string" ? prefix : prefix(i);
    if (typeof prefix === "string" && line.text.startsWith(p)) changes.push({ from: line.from, to: line.from + p.length, insert: "" });
    else changes.push({ from: line.from, insert: p });
  }
  v.dispatch({ changes });
  v.focus();
}

function heading(level: number) {
  const v = view.value!;
  const line = v.state.doc.lineAt(v.state.selection.main.from);
  const stripped = line.text.replace(/^#{1,6}\s+/, "");
  const current = (line.text.match(/^(#{1,6})\s/) || [])[1]?.length;
  const insert = current === level ? stripped : "#".repeat(level) + " " + stripped;
  v.dispatch({ changes: { from: line.from, to: line.to, insert } });
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
  outlineOpen.value = false;
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
    if (frameReady.value) post({ type: "render", html: r.html, cls: r.chapterClass });
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
    post({ type: "render", ...lastHtml.value });
  }
}

const srcdoc = ref("");
function reloadFrame() {
  frameReady.value = false;
  srcdoc.value = previewDocument(previewTheme.value, store.configVersion);
}

function setPreviewTheme(t: string) {
  previewTheme.value = t;
  localStorage.setItem("bc.previewTheme", t);
  post({ type: "theme", theme: t });
}

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
    <UiButton variant="ghost" size="sm" :icon="ListTree" :disabled="!headings.length" @click="outlineOpen = !outlineOpen">Outline</UiButton>
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
      <iframe ref="frame" class="preview" sandbox="allow-scripts" title="Preview" :srcdoc="srcdoc"></iframe>
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

    <div v-if="outlineOpen" class="insert-panel outline-panel">
      <button v-for="h in headings" :key="h.id" type="button" class="outline-item" :style="{ paddingLeft: (h.level - 1) * 14 + 10 + 'px' }" @click="jumpTo(h)">
        {{ h.text }}
      </button>
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
.workspace { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); position: relative; }
.workspace.editor { grid-template-columns: minmax(0, 1fr) 0; }
.workspace.preview { grid-template-columns: 0 minmax(0, 1fr); }
.workspace.editor .preview-pane, .workspace.preview .editor-pane { display: none; }
.editor-pane { min-height: 0; overflow: hidden; border-right: 1px solid var(--line); background: var(--panel); }
.workspace.editor .editor-pane :deep(.cm-content) { margin: 0 auto; }
.preview-pane { min-height: 0; background: var(--bg); }
.preview { width: 100%; height: 100%; border: 0; display: block; }
.insert-panel { position: absolute; top: 8px; left: 12px; right: 12px; max-height: calc(100% - 16px); z-index: 20; background: var(--panel); border: 1px solid var(--line); border-radius: 12px; box-shadow: 0 18px 50px rgb(0 0 0 / 0.2); display: flex; flex-direction: column; overflow: hidden; }
.insert-head { display: flex; align-items: center; gap: 8px; padding: 10px 12px; border-bottom: 1px solid var(--line); }
.insert-head :deep(.ui-input) { flex: 1; }
.insert-head .spacer { flex: 1; }
.insert-groups { overflow-y: auto; padding: 12px; columns: 4 210px; column-gap: 16px; }
.insert-groups section { break-inside: avoid; margin-bottom: 14px; }
.insert-groups h4 { margin: 0 0 6px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); }
.insert-item { display: block; width: 100%; text-align: left; border: 0; background: none; padding: 6px 8px; border-radius: 6px; cursor: pointer; font-size: 13px; }
.insert-item:hover { background: var(--accent-soft); color: var(--accent); }
.insert-item.off { color: var(--muted); }
.insert-item.off::after { content: " (off)"; font-size: 11px; }
.image-panel .empty { padding: 24px; color: var(--muted); text-align: center; }
.thumbs { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 10px; padding: 12px; overflow-y: auto; }
.thumb { border: 1px solid var(--line); border-radius: 8px; background: var(--panel-2); padding: 6px; cursor: pointer; display: flex; flex-direction: column; gap: 6px; font-size: 11px; color: var(--muted); }
.thumb:hover { border-color: var(--accent); }
.thumb img { width: 100%; aspect-ratio: 4 / 3; object-fit: contain; border-radius: 4px; background: var(--panel); }
.thumb span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.outline-panel { left: auto; width: 320px; padding: 6px; overflow-y: auto; }
.outline-item { display: block; width: 100%; text-align: left; border: 0; background: none; padding: 7px 10px; border-radius: 6px; cursor: pointer; font-size: 13px; }
.outline-item:hover { background: var(--panel-2); }
</style>
