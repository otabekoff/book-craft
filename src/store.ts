// Shared studio state. Config edits are saved automatically after a short pause.
import { reactive, watch } from "vue";
import { api, errorText, type AppInfo, type AssetInfo, type BookConfig, type ChapterInfo, type ProjectData } from "./api";

export type Section = "write" | "details" | "appearance" | "modules" | "assets" | "build";

export const store = reactive({
  info: null as AppInfo | null,
  root: "",
  config: null as BookConfig | null,
  chapters: [] as ChapterInfo[],
  chapterClass: "bc-chapter",
  problems: [] as string[],
  assets: [] as AssetInfo[],
  section: "write" as Section,
  currentChapter: "" as string,
  saving: false,
  configVersion: 0,
  toast: "" as string,
  toastKind: "info" as "info" | "error",
});

let toastTimer = 0;
export function notify(message: string, kind: "info" | "error" = "info") {
  store.toast = message;
  store.toastKind = kind;
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (store.toast = ""), kind === "error" ? 7000 : 3200);
}

let applying = false;
export function applyProject(p: ProjectData) {
  applying = true;
  store.root = p.root;
  store.config = p.config;
  store.chapters = p.chapters;
  store.chapterClass = p.chapterClass;
  store.problems = p.problems;
  if (!store.chapters.some((c) => c.file === store.currentChapter)) {
    store.currentChapter = store.chapters[0]?.file ?? "";
  }
  queueMicrotask(() => (applying = false));
}

export async function loadAssets() {
  try {
    store.assets = await api.assets();
  } catch (e) {
    notify(errorText(e), "error");
  }
}

let saveTimer = 0;
export function saveConfigSoon() {
  clearTimeout(saveTimer);
  saveTimer = window.setTimeout(saveConfigNow, 450);
}

export async function saveConfigNow() {
  clearTimeout(saveTimer);
  if (!store.config) return;
  store.saving = true;
  try {
    const p = await api.saveConfig(JSON.parse(JSON.stringify(store.config)));
    applying = true;
    store.chapters = p.chapters;
    store.chapterClass = p.chapterClass;
    store.problems = p.problems;
    store.configVersion++;
    queueMicrotask(() => (applying = false));
  } catch (e) {
    notify(errorText(e), "error");
  } finally {
    store.saving = false;
  }
}

watch(
  () => store.config,
  () => {
    if (!applying && store.config) saveConfigSoon();
  },
  { deep: true }
);

export async function openProject(dir: string) {
  const p = await api.open(dir);
  applyProject(p);
  store.section = "write";
  await loadAssets();
  store.info = await api.appInfo();
}

export async function closeProject() {
  await saveConfigNow();
  await api.close();
  store.root = "";
  store.config = null;
  store.chapters = [];
  store.currentChapter = "";
  store.info = await api.appInfo();
}
