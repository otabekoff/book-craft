<script setup lang="ts">
import { computed, ref } from "vue";
import { api, ASSET_BASE, errorText } from "../api";
import { applyProject, closeProject, notify, saveConfigNow, store, type Section } from "../store";
import Icon from "./Icon.vue";

const sections: { id: Section; label: string; icon: string }[] = [
  { id: "write", label: "Write", icon: "write" },
  { id: "details", label: "Book details", icon: "details" },
  { id: "appearance", label: "Appearance", icon: "appearance" },
  { id: "modules", label: "Blocks & features", icon: "modules" },
  { id: "assets", label: "Assets", icon: "assets" },
  { id: "build", label: "Build", icon: "build" },
];

const menu = ref<{ file: string; x: number; y: number } | null>(null);
const dialog = ref<{ mode: "add" | "rename"; file?: string; value: string } | null>(null);
const dragFrom = ref<number | null>(null);
const dropAt = ref<number | null>(null);

const iconUrl = computed(() => {
  const icon = store.config?.meta.icon;
  return icon ? `${ASSET_BASE}/project/${icon}?v=${store.configVersion}` : "";
});

// Order and flags come from the config (edited live); titles and word counts from disk.
const items = computed(() =>
  (store.config?.chapters ?? []).map((ref) => {
    const info = store.chapters.find((c) => c.file === ref.file);
    return {
      file: ref.file,
      title: ref.title.trim() || info?.title || ref.file,
      hidden: ref.hidden,
      kind: ref.kind || "chapter",
      words: info?.words ?? 0,
      missing: info?.missing ?? false,
    };
  })
);

function select(file: string) {
  store.currentChapter = file;
  store.section = "write";
}

function openMenu(e: MouseEvent, file: string) {
  const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
  menu.value = { file, x: Math.min(r.left, window.innerWidth - 220), y: r.bottom + 4 };
}

function chapterRef(file: string) {
  return store.config!.chapters.find((c) => c.file === file)!;
}

async function addChapter() {
  if (!dialog.value) return;
  try {
    await saveConfigNow();
    const [file, p] = await api.addChapter(dialog.value.value.trim() || "New chapter", store.currentChapter || null);
    applyProject(p);
    select(file);
  } catch (e) {
    notify(errorText(e), "error");
  }
  dialog.value = null;
}

function rename() {
  if (!dialog.value?.file) return;
  chapterRef(dialog.value.file).title = dialog.value.value.trim();
  dialog.value = null;
}

function move(file: string, delta: number) {
  const list = store.config!.chapters;
  const i = list.findIndex((c) => c.file === file);
  const j = i + delta;
  if (i < 0 || j < 0 || j >= list.length) return;
  const [item] = list.splice(i, 1);
  list.splice(j, 0, item);
}

function setKind(file: string, kind: string) {
  chapterRef(file).kind = kind;
}

function toggleHidden(file: string) {
  const c = chapterRef(file);
  c.hidden = !c.hidden;
}

async function remove(file: string) {
  const info = items.value.find((c) => c.file === file);
  if (!confirm(`Remove “${info?.title ?? file}” from the book?\n\nThe file is moved to chapters/.trash, not deleted.`)) return;
  try {
    await saveConfigNow();
    applyProject(await api.removeChapter(file));
  } catch (e) {
    notify(errorText(e), "error");
  }
}

function onDrop(index: number) {
  const from = dragFrom.value;
  dragFrom.value = dropAt.value = null;
  if (from == null || from === index) return;
  const list = store.config!.chapters;
  const [item] = list.splice(from, 1);
  list.splice(from < index ? index - 1 : index, 0, item);
}

function numberFor(i: number) {
  let n = 0;
  for (let j = 0; j <= i; j++) {
    const c = items.value[j];
    if (c && c.kind === "chapter" && !c.hidden) n++;
  }
  const c = items.value[i];
  return c && c.kind === "chapter" && !c.hidden ? String(n) : "·";
}
</script>

<template>
  <aside class="sidebar" @click="menu = null">
    <div class="sidebar-head">
      <div class="book-badge"><img v-if="iconUrl" :src="iconUrl" alt="" /></div>
      <div style="min-width: 0">
        <div class="book-name">{{ store.config?.meta.title }}</div>
        <div class="book-path" :title="store.root">{{ store.root }}</div>
      </div>
    </div>
    <nav class="nav" aria-label="Sections">
      <button
        v-for="s in sections"
        :key="s.id"
        type="button"
        :aria-current="store.section === s.id ? 'page' : undefined"
        @click="store.section = s.id"
      >
        <Icon :name="s.icon" />{{ s.label }}
        <span v-if="s.id === 'build' && store.problems.length" class="dot" :title="store.problems.join('\n')"></span>
        <span v-if="s.id === 'assets'" class="count">{{ store.assets.length }}</span>
      </button>
    </nav>
    <div class="chapters">
      <div class="chapters-head">
        <span>Chapters</span>
        <button class="icon-btn" type="button" title="Add chapter" aria-label="Add chapter" @click.stop="dialog = { mode: 'add', value: '' }">
          <Icon name="plus" />
        </button>
      </div>
      <div
        v-for="(c, i) in items"
        :key="c.file"
        class="chapter-item"
        :class="{ 'hidden-ch': c.hidden, dragging: dragFrom === i, 'drop-before': dropAt === i && dragFrom !== i }"
        :aria-current="store.section === 'write' && store.currentChapter === c.file"
        draggable="true"
        role="button"
        tabindex="0"
        :title="`${c.file} · ${c.words} words${c.missing ? ' · file missing' : ''}`"
        @click="select(c.file)"
        @keydown.enter="select(c.file)"
        @dragstart="dragFrom = i"
        @dragover.prevent="dropAt = i"
        @dragleave="dropAt = null"
        @drop.prevent="onDrop(i)"
        @dragend="dragFrom = dropAt = null"
      >
        <span class="num">{{ numberFor(i) }}</span>
        <span class="title">{{ c.title }}</span>
        <Icon v-if="c.missing" name="warn" style="color: var(--danger)" />
        <button class="icon-btn menu-btn" type="button" aria-label="Chapter options" @click.stop="openMenu($event, c.file)">
          <Icon name="more" />
        </button>
      </div>
      <div
        class="chapter-item"
        style="min-height: 12px"
        :class="{ 'drop-before': dropAt === items.length }"
        @dragover.prevent="dropAt = items.length"
        @drop.prevent="onDrop(items.length)"
      ></div>
    </div>
    <div class="sidebar-foot">
      <button class="btn small ghost" type="button" @click="api.testRead().catch((e) => notify(errorText(e), 'error'))" title="Open the book in the real reader without building packages">
        <Icon name="play" />Try in reader
      </button>
      <span style="flex: 1"></span>
      <button class="btn small ghost" type="button" @click="closeProject">Close</button>
    </div>

    <div v-if="menu" class="menu" :style="{ left: menu.x + 'px', top: menu.y + 'px' }" @click.stop>
      <button type="button" @click="dialog = { mode: 'rename', file: menu.file, value: chapterRef(menu.file).title }; menu = null">Set title…</button>
      <button type="button" @click="move(menu.file, -1); menu = null"><Icon name="up" />Move up</button>
      <button type="button" @click="move(menu.file, 1); menu = null"><Icon name="down" />Move down</button>
      <hr />
      <button type="button" @click="setKind(menu.file, 'chapter'); menu = null">Numbered chapter</button>
      <button type="button" @click="setKind(menu.file, 'front'); menu = null">Front matter (preface, dedication)</button>
      <button type="button" @click="setKind(menu.file, 'part'); menu = null">Part divider</button>
      <button type="button" @click="setKind(menu.file, 'back'); menu = null">Back matter (appendix)</button>
      <hr />
      <button type="button" @click="toggleHidden(menu.file); menu = null">
        {{ chapterRef(menu.file).hidden ? "Include in book" : "Leave out of book" }}
      </button>
      <button type="button" class="danger" style="color: var(--danger)" @click="remove(menu.file); menu = null"><Icon name="trash" />Remove…</button>
    </div>

    <div v-if="dialog" class="overlay" @click.self="dialog = null">
      <form class="dialog" @submit.prevent="dialog.mode === 'add' ? addChapter() : rename()">
        <h2>{{ dialog.mode === "add" ? "New chapter" : "Chapter title" }}</h2>
        <label class="field">
          <span>Title</span>
          <input class="input" v-model="dialog.value" autofocus :placeholder="dialog.mode === 'rename' ? 'Leave empty to use the chapter’s first heading' : 'Chapter title'" />
        </label>
        <div class="dialog-actions">
          <button type="button" class="btn" @click="dialog = null">Cancel</button>
          <button type="submit" class="btn primary">{{ dialog.mode === "add" ? "Add chapter" : "Save" }}</button>
        </div>
      </form>
    </div>
  </aside>
</template>
