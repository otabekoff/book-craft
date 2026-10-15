<script setup lang="ts">
import { computed, ref } from "vue";
import { Ellipsis, FolderOpen, Library, Trash, TriangleAlert, X } from "@lucide/vue";
import appIcon from "../../src-tauri/icons/source/icon.svg?url";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText } from "../api";
import { applyProject, loadAssets, notify, openProject, store } from "../store";
import { UiButton, UiCard, UiDialog, UiField, UiIconButton, UiInput, UiMenu, type MenuItem } from "../ui";

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
  const picked = dir ?? (await open({ directory: true, title: "Open a book folder (it contains book.json or book.toml)" }));
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

/* Recent books: open, forget, or move to the Recycle Bin. */
const toDelete = ref<string | null>(null);
const deleting = ref(false);
function recentMenu(path: string): MenuItem[] {
  return [
    { label: "Open", icon: FolderOpen, action: () => openExisting(path) },
    { label: "Remove from recent", icon: X, action: () => removeRecent(path) },
    { separator: true },
    { label: "Delete from disk…", icon: Trash, danger: true, action: () => (toDelete.value = path) },
  ];
}
async function removeRecent(path: string) {
  try {
    store.info = await api.removeRecent(path);
  } catch (e) {
    notify(errorText(e), "error");
  }
}
async function trashBook() {
  if (!toDelete.value) return;
  deleting.value = true;
  try {
    store.info = await api.trashProject(toDelete.value);
    notify(`Moved “${shortName(toDelete.value)}” to the Recycle Bin.`);
    toDelete.value = null;
  } catch (e) {
    notify(errorText(e), "error");
  } finally {
    deleting.value = false;
  }
}

function shortName(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}
</script>

<template>
  <div class="welcome">
    <section class="intro">
      <img :src="appIcon" class="logo" alt="" />
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
      <UiCard title="Start a new book">
        <form class="form" @submit.prevent="create">
          <UiField label="Title" for="new-title">
            <UiInput id="new-title" v-model="title" placeholder="The Salt Road" autofocus />
          </UiField>
          <UiField label="Author" for="new-author">
            <UiInput id="new-author" v-model="author" placeholder="Your name" />
          </UiField>
          <UiField label="Location" hint="A new folder named after the book is created there, with sample chapters you can edit.">
            <div class="row">
              <UiInput :model-value="target" readonly placeholder="Choose a folder…" aria-label="Location" @click="pickParent" />
              <UiButton :icon="FolderOpen" @click="pickParent">Choose</UiButton>
            </div>
          </UiField>
          <UiButton variant="primary" size="lg" type="submit" :loading="busy" :disabled="!title.trim() || !parent">Create book</UiButton>
        </form>
      </UiCard>

      <UiCard title="Open a book">
        <UiButton :icon="FolderOpen" :disabled="busy" @click="openExisting()">Open folder…</UiButton>
        <div v-if="store.info?.settings.recent.length" class="recent">
          <div class="recent-head">Recent</div>
          <div v-for="r in store.info.settings.recent" :key="r" class="recent-row">
            <button type="button" class="recent-item" :title="r" @click="openExisting(r)">
              <Library class="r-ico" aria-hidden="true" />
              <span><b>{{ shortName(r) }}</b><small>{{ r }}</small></span>
            </button>
            <UiMenu :items="recentMenu(r)">
              <UiIconButton :icon="Ellipsis" :label="`Options for ${shortName(r)}`" size="sm" class="recent-menu" />
            </UiMenu>
          </div>
        </div>
      </UiCard>

      <UiDialog v-if="toDelete" title="Delete this book from disk?" @close="toDelete = null">
        <p>The folder <b>{{ shortName(toDelete) }}</b> and everything in it (chapters, images, settings and builds) is moved to the Recycle Bin. You can restore it from there.</p>
        <p class="muted path">{{ toDelete }}</p>
        <template #actions>
          <UiButton @click="toDelete = null">Cancel</UiButton>
          <UiButton variant="danger" :icon="Trash" :loading="deleting" @click="trashBook">Move to Recycle Bin</UiButton>
        </template>
      </UiDialog>

      <div v-if="store.info && !store.info.readerTemplate" class="notice warn">
        <TriangleAlert class="r-ico" aria-hidden="true" />
        The reader template is missing, so books can be written but not built yet. Run <code>bun run prepare:resources</code> and restart.
      </div>
    </section>
  </div>
</template>

<style scoped>
.welcome { min-height: 100%; display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: 56px; padding: 48px; }
.intro { flex: 0 1 420px; display: flex; flex-direction: column; gap: 14px; }
.logo { width: 64px; height: 64px; }
.intro h1 { font-size: 40px; letter-spacing: -0.01em; }
.lead { font-size: 17px; line-height: 1.5; color: var(--muted); margin: 0; }
.features { margin: 4px 0 0; padding-left: 20px; line-height: 1.9; }
.version { color: var(--muted); font-size: 12px; margin: 0; }
.actions { flex: 0 1 460px; display: flex; flex-direction: column; gap: 16px; }
.form { display: flex; flex-direction: column; gap: 14px; }
.row { display: flex; gap: 8px; }
.row > :first-child { flex: 1; }
.recent { display: flex; flex-direction: column; gap: 2px; }
.recent-head { font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); font-weight: 600; margin: 4px 0; }
.recent-row { display: flex; align-items: center; gap: 2px; border-radius: 8px; }
.recent-row:hover { background: var(--panel-2); }
.recent-row .recent-item { flex: 1; }
.recent-row .recent-item:hover { background: none; }
.recent-menu { opacity: 0; }
.recent-row:hover .recent-menu, .recent-row:focus-within .recent-menu { opacity: 1; }
.muted { color: var(--muted); }
.path { font-family: var(--mono); font-size: 12px; overflow-wrap: anywhere; }
.recent-item { display: flex; gap: 10px; align-items: center; border: 0; background: none; color: var(--ink); text-align: left; padding: 8px; border-radius: 8px; cursor: pointer; min-width: 0; font: inherit; }
.recent-item:hover { background: var(--panel-2); }
.recent-item span { display: flex; flex-direction: column; min-width: 0; }
.recent-item small { color: var(--muted); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.r-ico { width: 18px; height: 18px; flex: none; color: var(--muted); }
.notice { display: flex; gap: 8px; align-items: flex-start; }
code { font-family: var(--mono); font-size: 12px; }
</style>
