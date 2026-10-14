<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, ASSET_BASE, errorText, type AssetInfo } from "../api";
import { loadAssets, notify, store } from "../store";
import Icon from "./Icon.vue";

const filter = ref("all");
const kinds = ["all", "image", "audio", "video", "font", "pdf", "file"];
const list = computed(() => store.assets.filter((a) => filter.value === "all" || a.kind === filter.value));
const total = computed(() => store.assets.reduce((n, a) => n + a.size, 0));

function size(n: number) {
  return n > 1048576 ? (n / 1048576).toFixed(1) + " MB" : Math.max(1, Math.round(n / 1024)) + " KB";
}

function snippet(a: AssetInfo) {
  const name = a.path.split("/").pop()!.replace(/\.[^.]+$/, "").replace(/[-_]+/g, " ");
  switch (a.kind) {
    case "image":
      return `![${name}](${a.path})`;
    case "audio":
      return `::: audio {src="${a.path}" title="${name}"}\n:::`;
    case "video":
      return `::: video {src="${a.path}"}\n:::`;
    default:
      return `::: attachment {src="${a.path}" label="${name}"}\n:::`;
  }
}

async function copy(a: AssetInfo) {
  try {
    await navigator.clipboard.writeText(snippet(a));
    notify("Copied — paste it into a chapter.");
  } catch {
    notify(snippet(a));
  }
}

async function importFiles() {
  const picked = await open({ multiple: true, title: "Add files to the book" });
  if (!picked) return;
  const files = Array.isArray(picked) ? picked : [picked];
  const byKind = (p: string) => {
    const e = p.split(".").pop()!.toLowerCase();
    if (["png", "jpg", "jpeg", "gif", "webp", "svg", "avif"].includes(e)) return "images";
    if (["mp3", "wav", "ogg", "m4a", "flac", "aac"].includes(e)) return "audio";
    if (["mp4", "webm", "mov", "m4v"].includes(e)) return "video";
    if (["woff", "woff2", "ttf", "otf"].includes(e)) return "fonts";
    return "files";
  };
  try {
    const groups = new Map<string, string[]>();
    for (const f of files) groups.set(byKind(f), [...(groups.get(byKind(f)) ?? []), f]);
    for (const [dir, paths] of groups) await api.importAssets(paths, dir);
    await loadAssets();
    notify(`Added ${files.length} file${files.length === 1 ? "" : "s"}.`);
  } catch (e) {
    notify(errorText(e), "error");
  }
}

async function remove(a: AssetInfo) {
  if (!confirm(`Delete ${a.path}? Chapters that use it will show a broken link.`)) return;
  try {
    store.assets = await api.deleteAsset(a.path);
  } catch (e) {
    notify(errorText(e), "error");
  }
}

function useAs(a: AssetInfo, field: "icon" | "cover") {
  store.config!.meta[field] = a.path;
  notify(field === "icon" ? "Set as the app icon." : "Set as the cover.");
}
</script>

<template>
  <header class="topbar">
    <h1>Assets</h1>
    <span class="status">{{ store.assets.length }} files · {{ size(total) }}</span>
    <span class="spacer"></span>
    <button class="btn primary" type="button" @click="importFiles"><Icon name="plus" />Add files…</button>
  </header>
  <div class="page">
    <div class="page-inner">
      <div class="page-title">
        <div>
          <h2>Assets</h2>
          <p>Images, audio, video, fonts and documents packed into the book. Paths start with <code>assets/</code>.</p>
        </div>
        <div class="seg">
          <button v-for="k in kinds" :key="k" type="button" :aria-pressed="filter === k" @click="filter = k">{{ k === "all" ? "All" : k[0].toUpperCase() + k.slice(1) }}</button>
        </div>
      </div>
      <div v-if="!list.length" class="notice">No files here yet. Use <b>Add files…</b> to bring in pictures, sound or documents.</div>
      <div class="assets">
        <article v-for="a in list" :key="a.path" class="asset">
          <div class="preview">
            <img v-if="a.kind === 'image'" :src="`${ASSET_BASE}/project/${a.path}`" alt="" loading="lazy" />
            <audio v-else-if="a.kind === 'audio'" :src="`${ASSET_BASE}/project/${a.path}`" controls preload="none"></audio>
            <video v-else-if="a.kind === 'video'" :src="`${ASSET_BASE}/project/${a.path}`" preload="metadata" muted></video>
            <span v-else class="kind">{{ a.path.split(".").pop()?.toUpperCase() }}</span>
          </div>
          <div class="meta">
            <div class="name" :title="a.path">{{ a.path.replace("assets/", "") }}</div>
            <div class="sub">{{ a.kind }} · {{ size(a.size) }}</div>
          </div>
          <div class="actions">
            <button class="btn small" type="button" @click="copy(a)">Copy Markdown</button>
            <template v-if="a.kind === 'image'">
              <button class="btn small ghost" type="button" @click="useAs(a, 'icon')" :disabled="store.config?.meta.icon === a.path">Icon</button>
              <button class="btn small ghost" type="button" @click="useAs(a, 'cover')" :disabled="store.config?.meta.cover === a.path">Cover</button>
            </template>
            <button class="icon-btn" type="button" aria-label="Delete" title="Delete" @click="remove(a)"><Icon name="trash" /></button>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<style scoped>
code { font-family: var(--mono); font-size: 12px; }
.assets { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 14px; }
.asset { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; overflow: hidden; display: flex; flex-direction: column; }
.preview { aspect-ratio: 4 / 3; background: var(--panel-2); display: grid; place-items: center; overflow: hidden; }
.preview img, .preview video { width: 100%; height: 100%; object-fit: contain; }
.preview audio { width: 90%; }
.kind { font-weight: 700; color: var(--muted); letter-spacing: 0.06em; }
.meta { padding: 10px 12px 4px; min-width: 0; }
.name { font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.sub { color: var(--muted); font-size: 12px; }
.actions { display: flex; align-items: center; gap: 4px; padding: 6px 8px 10px 12px; flex-wrap: wrap; }
.actions .icon-btn { margin-left: auto; }
</style>
