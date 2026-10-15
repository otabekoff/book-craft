<script setup lang="ts">
// Custom title bar (the window has no native decorations). Empty areas drag the
// window; double-clicking them maximizes, like a native title bar.
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import appIcon from "../../src-tauri/icons/source/icon.svg?url";
import { flushAll, store } from "../store";

const win = getCurrentWindow();
const maximized = ref(false);
const focused = ref(true);
const theme = ref<"system" | "light" | "dark">((localStorage.getItem("bc.theme") as any) || "system");
const unlisten: UnlistenFn[] = [];

const sectionNames: Record<string, string> = {
  write: "Write",
  details: "Book details",
  appearance: "Appearance",
  modules: "Blocks & features",
  assets: "Assets",
  build: "Build",
};

// Only the section: the book's name is in the sidebar and the chapter's in the page header,
// so repeating them here was noise.
const crumbs = computed(() => (store.config ? [sectionNames[store.section]] : []));

function applyTheme() {
  if (theme.value === "system") document.documentElement.removeAttribute("data-theme");
  else document.documentElement.setAttribute("data-theme", theme.value);
}

function cycleTheme() {
  theme.value = theme.value === "system" ? "light" : theme.value === "light" ? "dark" : "system";
  localStorage.setItem("bc.theme", theme.value);
  applyTheme();
}

const themeLabel = computed(() => ({ system: "Theme: match Windows", light: "Theme: light", dark: "Theme: dark" })[theme.value]);

async function refresh() {
  try {
    maximized.value = await win.isMaximized();
  } catch {
    /* not running inside Tauri (mock mode) */
  }
}

onMounted(async () => {
  applyTheme();
  await refresh();
  try {
    unlisten.push(await win.onResized(refresh));
    unlisten.push(await win.onFocusChanged(({ payload }) => (focused.value = payload)));
    // Save pending edits before the window goes away (button, Alt+F4 or taskbar).
    unlisten.push(await win.onCloseRequested(() => flushAll()));
  } catch {
    /* mock mode */
  }
});

onBeforeUnmount(() => unlisten.forEach((u) => u()));
</script>

<template>
  <header class="appbar" :class="{ blurred: !focused }" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <img :src="appIcon" alt="" class="logo" data-tauri-drag-region />
      <span class="name" data-tauri-drag-region>Book Craft</span>
    </div>
    <nav v-if="crumbs.length" class="crumbs" aria-label="Location" data-tauri-drag-region>
      <template v-for="(c, i) in crumbs" :key="i">
        <span class="sep" aria-hidden="true" data-tauri-drag-region>›</span>
        <span class="crumb" :class="{ last: i === crumbs.length - 1 }" data-tauri-drag-region>{{ c }}</span>
      </template>
    </nav>
    <div class="drag" data-tauri-drag-region></div>
    <span v-if="store.saving" class="saving" data-tauri-drag-region>Saving…</span>
    <button type="button" class="tool" :title="themeLabel" :aria-label="themeLabel" @click="cycleTheme">
      <span class="glyph" aria-hidden="true">{{ theme === "dark" ? "" : theme === "light" ? "" : "" }}</span>
    </button>
    <div class="controls">
      <button type="button" class="ctl" aria-label="Minimize" title="Minimize" @click="win.minimize()">
        <span class="glyph" aria-hidden="true">&#xE921;</span>
      </button>
      <button type="button" class="ctl" :aria-label="maximized ? 'Restore' : 'Maximize'" :title="maximized ? 'Restore' : 'Maximize'" @click="win.toggleMaximize()">
        <span class="glyph" aria-hidden="true">{{ maximized ? "" : "" }}</span>
      </button>
      <button type="button" class="ctl close" aria-label="Close" title="Close" @click="win.close()">
        <span class="glyph" aria-hidden="true">&#xE8BB;</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.appbar {
  height: 40px;
  flex: none;
  display: flex;
  align-items: center;
  gap: 4px;
  padding-left: 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
  user-select: none;
  -webkit-user-select: none;
  font-size: 12.5px;
}
.appbar.blurred .name,
.appbar.blurred .crumbs,
.appbar.blurred .ctl { opacity: 0.55; }
.brand { display: flex; align-items: center; gap: 8px; height: 100%; }
.logo { width: 18px; height: 18px; display: block; }
.name { font-weight: 600; letter-spacing: 0.01em; }
.crumbs { display: flex; align-items: center; gap: 6px; min-width: 0; height: 100%; color: var(--muted); padding-left: 2px; }
.crumb { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 260px; }
.crumb.last { color: var(--ink); }
.sep { opacity: 0.6; }
.drag { flex: 1; height: 100%; min-width: 24px; }
.saving { color: var(--muted); margin-right: 6px; }
.glyph { font-family: "Segoe Fluent Icons", "Segoe MDL2 Assets"; font-size: 10px; line-height: 1; }
.tool {
  width: 34px;
  height: 30px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--ink);
  display: grid;
  place-items: center;
  cursor: default;
}
.tool .glyph { font-size: 14px; }
.tool:hover { background: var(--panel-2); }
.controls { display: flex; height: 100%; margin-left: 4px; }
.ctl {
  width: 46px;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--ink);
  display: grid;
  place-items: center;
  cursor: default;
}
.ctl:hover { background: color-mix(in srgb, var(--ink) 8%, transparent); }
.ctl:active { background: color-mix(in srgb, var(--ink) 14%, transparent); }
.ctl.close:hover { background: #c42b1c; color: #fff; }
.ctl.close:active { background: #b22a1b; color: #fff; }
.ctl:focus-visible, .tool:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
</style>
