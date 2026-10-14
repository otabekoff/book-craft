<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText } from "../api";
import { applyProject, loadAssets, notify, openProject, store } from "../store";
import Icon from "./Icon.vue";

const title = ref("");
const author = ref("");
const parent = ref("");
const busy = ref(false);

const folderName = computed(() =>
  (title.value.trim() || "My Book").replace(/[\\/:*?"<>|]/g, " ").replace(/\s+/g, " ").trim()
);
const target = computed(() => (parent.value ? parent.value.replace(/[\\/]+$/, "") + "\\" + folderName.value : ""));

async function pickParent() {
  const dir = await open({ directory: true, title: "Where should the book folder go?" });
  if (typeof dir === "string") parent.value = dir;
}

async function create() {
  if (!title.value.trim() || !target.value) return;
  busy.value = true;
  try {
    const p = await api.create(target.value, title.value.trim(), author.value.trim());
    applyProject(p);
    store.section = "write";
    await loadAssets();
    store.info = await api.appInfo();
  } catch (e) {
    notify(errorText(e), "error");
  } finally {
    busy.value = false;
  }
}

async function openExisting(dir?: string) {
  const picked = dir ?? (await open({ directory: true, title: "Open a book folder (it contains book.json)" }));
  if (typeof picked !== "string") return;
  busy.value = true;
  try {
    await openProject(picked);
  } catch (e) {
    notify(errorText(e), "error");
  } finally {
    busy.value = false;
  }
}

function shortName(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}
</script>

<template>
  <div class="welcome">
    <section class="intro">
      <div class="logo"><Icon name="book" /></div>
      <h1>Book Craft</h1>
      <p class="lead">Write your book in Markdown. Get a Windows app — portable .exe, setup .exe, .msi or .msix — with one click.</p>
      <ul class="features">
        <li>Callouts, quotes, boxes, columns and tabs</li>
        <li>Math, chemistry, charts, diagrams and drawings</li>
        <li>Highlighted and runnable code</li>
        <li>Quizzes, exercises and flashcards</li>
        <li>Your icon, publisher and package name in every build</li>
      </ul>
      <p class="version" v-if="store.info">Version {{ store.info.version }}</p>
    </section>

    <section class="actions">
      <form class="card" @submit.prevent="create">
        <h3>Start a new book</h3>
        <label class="field">
          <span>Title</span>
          <input class="input" v-model="title" placeholder="The Salt Road" required autofocus />
        </label>
        <label class="field">
          <span>Author</span>
          <input class="input" v-model="author" placeholder="Your name" />
        </label>
        <div class="field">
          <span>Location</span>
          <div class="input-row">
            <input class="input" :value="target" readonly placeholder="Choose a folder…" @click="pickParent" />
            <button type="button" class="btn" @click="pickParent"><Icon name="folder" />Choose</button>
          </div>
          <small>A new folder named after the book is created there, with sample chapters you can edit.</small>
        </div>
        <button class="btn primary big" type="submit" :disabled="busy || !title.trim() || !parent">Create book</button>
      </form>

      <div class="card">
        <h3>Open a book</h3>
        <button class="btn" type="button" @click="openExisting()" :disabled="busy"><Icon name="folder" />Open folder…</button>
        <div v-if="store.info?.settings.recent.length" class="recent">
          <div class="recent-head">Recent</div>
          <button v-for="r in store.info.settings.recent" :key="r" type="button" class="recent-item" @click="openExisting(r)">
            <Icon name="book" />
            <span><b>{{ shortName(r) }}</b><small>{{ r }}</small></span>
          </button>
        </div>
      </div>

      <div v-if="store.info && !store.info.readerTemplate" class="notice warn">
        The reader template is missing, so books can be written but not built yet. Run <code>bun run prepare:resources</code> and restart.
      </div>
    </section>
  </div>
</template>

<style scoped>
.welcome { min-height: 100%; display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: 56px; padding: 48px; }
.intro { flex: 0 1 420px; display: flex; flex-direction: column; gap: 14px; }
.logo { width: 56px; height: 56px; border-radius: 14px; background: var(--accent); color: var(--accent-ink); display: grid; place-items: center; }
.logo .icon { width: 28px; height: 28px; }
.intro h1 { font-size: 40px; letter-spacing: -0.01em; }
.lead { font-size: 17px; line-height: 1.5; color: var(--muted); margin: 0; }
.features { margin: 4px 0 0; padding-left: 20px; line-height: 1.9; }
.version { color: var(--muted); font-size: 12px; margin: 0; }
.actions { flex: 0 1 460px; display: flex; flex-direction: column; gap: 16px; }
.recent { display: flex; flex-direction: column; gap: 2px; }
.recent-head { font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); font-weight: 600; margin: 4px 0; }
.recent-item { display: flex; gap: 10px; align-items: center; border: 0; background: none; text-align: left; padding: 8px; border-radius: 8px; cursor: pointer; min-width: 0; }
.recent-item:hover { background: var(--panel-2); }
.recent-item span { display: flex; flex-direction: column; min-width: 0; }
.recent-item small { color: var(--muted); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
code { font-family: var(--mono); font-size: 12px; }
</style>
