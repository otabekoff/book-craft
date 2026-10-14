<script setup lang="ts">
// Asset chooser: a select of the project's matching assets plus an Import button.
import { computed } from "vue";
import { Upload, File as FileIcon, Image as ImageIcon, Music, Type, Video } from "@lucide/vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, ASSET_BASE, errorText } from "../api";
import { loadAssets, notify, store } from "../store";
import { UiButton, UiSelect, type SelectOption } from "../ui";

const props = defineProps<{ kind?: string; extensions?: string[]; subdir?: string; placeholder?: string; preview?: boolean; id?: string }>();
const model = defineModel<string>({ required: true });

const icons: Record<string, any> = { image: ImageIcon, audio: Music, video: Video, font: Type };

const options = computed<SelectOption<string>[]>(() => {
  const list: SelectOption<string>[] = [{ value: "", label: props.placeholder ?? "None" }];
  for (const a of store.assets) {
    const ok = props.extensions
      ? props.extensions.some((e) => a.path.toLowerCase().endsWith("." + e))
      : !props.kind || a.kind === props.kind;
    if (ok) list.push({ value: a.path, label: a.path.replace(/^assets\//, ""), icon: icons[a.kind] ?? FileIcon });
  }
  if (model.value && !list.some((o) => o.value === model.value)) list.push({ value: model.value, label: model.value + " (missing)" });
  return list;
});

async function importFile() {
  const picked = await open({
    multiple: false,
    filters: props.extensions ? [{ name: "Files", extensions: props.extensions }] : undefined,
  });
  if (typeof picked !== "string") return;
  try {
    const [path] = await api.importAssets([picked], props.subdir ?? "");
    await loadAssets();
    model.value = path;
  } catch (e) {
    notify(errorText(e), "error");
  }
}
</script>

<template>
  <div class="picker">
    <div v-if="preview" class="thumb">
      <img v-if="model" :src="`${ASSET_BASE}/project/${model}?v=${store.configVersion}`" alt="" />
      <ImageIcon v-else class="none" aria-hidden="true" />
    </div>
    <div class="col">
      <div class="row">
        <UiSelect :id="id" v-model="model" :options="options" :aria-label="placeholder ?? 'Choose a file'" />
        <UiButton :icon="Upload" @click="importFile">Import…</UiButton>
      </div>
      <slot />
    </div>
  </div>
</template>

<style scoped>
.picker { display: flex; gap: 12px; align-items: flex-start; }
.col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.row { display: flex; gap: 8px; }
.row > :first-child { flex: 1; min-width: 0; }
.thumb { width: 64px; height: 64px; border-radius: 12px; background: var(--panel-2); border: 1px solid var(--line); overflow: hidden; flex: none; display: grid; place-items: center; }
.thumb img { width: 100%; height: 100%; object-fit: cover; }
.none { width: 22px; height: 22px; color: var(--muted); }
.col :deep(small) { color: var(--muted); font-size: 12px; line-height: 1.45; }
</style>
