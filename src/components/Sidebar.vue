<script setup lang="ts">
import { computed, ref, type Component } from "vue";
import {
  ArrowDown, ArrowUp, BookMarked, BookOpen, Ellipsis, Eye, EyeOff, FileText, FolderOutput, Hammer, Images,
  LayoutGrid, ListOrdered, Palette, PenLine, Play, Plus, TextCursorInput, Trash, TriangleAlert,
} from "@lucide/vue";
import { api, ASSET_BASE, errorText } from "../api";
import { applyProject, closeProject, notify, saveConfigNow, store, type Section } from "../store";
import { UiButton, UiDialog, UiField, UiIconButton, UiInput, UiMenu, UiScrollArea, type MenuItem } from "../ui";

const sections: { id: Section; label: string; icon: Component }[] = [
  { id: "write", label: "Write", icon: PenLine },
  { id: "details", label: "Book details", icon: FileText },
  { id: "appearance", label: "Appearance", icon: Palette },
  { id: "modules", label: "Blocks & features", icon: LayoutGrid },
  { id: "assets", label: "Assets", icon: Images },
  { id: "build", label: "Build", icon: Hammer },
];

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

function menuFor(file: string, index: number): MenuItem[] {
  const c = chapterRef(file);
  const kind = (k: string, label: string, icon: Component): MenuItem => ({
    label,
    checked: (c.kind || "chapter") === k,
    icon,
    action: () => (c.kind = k),
  });
  return [
    { label: "Set title…", icon: TextCursorInput, action: () => (dialog.value = { mode: "rename", file, value: c.title }) },
    { label: "Move up", icon: ArrowUp, disabled: index === 0, action: () => move(file, -1) },
    { label: "Move down", icon: ArrowDown, disabled: index === items.value.length - 1, action: () => move(file, 1) },
    { separator: true },
    kind("chapter", "Numbered chapter", ListOrdered),
    kind("front", "Front matter", BookOpen),
    kind("part", "Part divider", BookMarked),
    kind("back", "Back matter", FolderOutput),
    { separator: true },
    { label: c.hidden ? "Include in book" : "Leave out of book", icon: c.hidden ? Eye : EyeOff, action: () => (c.hidden = !c.hidden) },
    { label: "Remove…", icon: Trash, danger: true, action: () => remove(file) },
  ];
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

const trying = ref(false);
/** Builds a quick test copy of the book and opens it; the button waits until the reader is up. */
async function tryInReader() {
  if (trying.value) return;
  trying.value = true;
  try {
    await api.testRead();
  } catch (e) {
    notify(errorText(e), "error");
  } finally {
    trying.value = false;
  }
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-head">
      <div class="book-badge"><img v-if="iconUrl" :src="iconUrl" alt="" /><BookOpen v-else class="fallback" /></div>
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
        <component :is="s.icon" class="nav-ico" aria-hidden="true" />{{ s.label }}
        <span v-if="s.id === 'build' && store.problems.length" class="dot" :title="store.problems.join('\n')"></span>
        <span v-if="s.id === 'assets'" class="count">{{ store.assets.length }}</span>
      </button>
    </nav>
    <UiScrollArea class="chapters-area" viewport-class="chapters">
      <div class="chapters-head">
        <span>Chapters</span>
        <UiIconButton :icon="Plus" label="Add chapter" size="sm" @click="dialog = { mode: 'add', value: '' }" />
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
        <TriangleAlert v-if="c.missing" class="warn-ico" aria-label="File missing" />
        <UiMenu :items="menuFor(c.file, i)">
          <UiIconButton :icon="Ellipsis" label="Chapter options" size="sm" class="menu-btn" />
        </UiMenu>
      </div>
      <div
        class="chapter-item drop-tail"
        :class="{ 'drop-before': dropAt === items.length }"
        @dragover.prevent="dropAt = items.length"
        @drop.prevent="onDrop(items.length)"
      ></div>
    </UiScrollArea>
    <div class="sidebar-foot">
      <UiButton variant="ghost" size="sm" :icon="Play" :loading="trying" :disabled="trying" title="Open the book in the real reader without building packages" @click="tryInReader">
        {{ trying ? "Opening…" : "Try in reader" }}
      </UiButton>
      <span style="flex: 1"></span>
      <UiButton variant="ghost" size="sm" @click="closeProject">Close</UiButton>
    </div>

    <UiDialog v-if="dialog" :title="dialog.mode === 'add' ? 'New chapter' : 'Chapter title'" @close="dialog = null">
      <form id="chapter-form" @submit.prevent="dialog.mode === 'add' ? addChapter() : rename()">
        <UiField label="Title" for="chapter-title" :hint="dialog.mode === 'rename' ? 'Leave empty to use the chapter’s first heading.' : undefined">
          <UiInput id="chapter-title" v-model="dialog.value" autofocus placeholder="Chapter title" />
        </UiField>
      </form>
      <template #actions>
        <UiButton @click="dialog = null">Cancel</UiButton>
        <UiButton variant="primary" type="submit" form="chapter-form">{{ dialog.mode === "add" ? "Add chapter" : "Save" }}</UiButton>
      </template>
    </UiDialog>
  </aside>
</template>

<style scoped>
.nav-ico { width: 18px; height: 18px; flex: none; }
.fallback { width: 18px; height: 18px; margin: 7px; color: var(--muted); }
.warn-ico { width: 16px; height: 16px; color: var(--danger); flex: none; }
.chapter-item :deep(.menu-btn) { opacity: 0; }
.chapter-item:hover :deep(.menu-btn), .chapter-item[aria-current="true"] :deep(.menu-btn), .chapter-item :deep(.menu-btn:focus-visible) { opacity: 1; }
.drop-tail { min-height: 12px; }
</style>
