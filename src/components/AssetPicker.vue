<script setup lang="ts">
// A path field with a dropdown of the project's assets and an Import button.
import { computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, ASSET_BASE, errorText } from "../api";
import { loadAssets, notify, store } from "../store";

const props = defineProps<{ kind?: string; extensions?: string[]; subdir?: string; placeholder?: string; preview?: boolean }>();
const model = defineModel<string>({ required: true });

const options = computed(() =>
  store.assets.filter((a) => {
    if (props.extensions) return props.extensions.some((e) => a.path.toLowerCase().endsWith("." + e));
    return !props.kind || a.kind === props.kind;
  })
);

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
    </div>
    <div class="col">
      <div class="input-row">
        <select class="input" v-model="model">
          <option value="">{{ placeholder ?? "None" }}</option>
          <option v-for="a in options" :key="a.path" :value="a.path">{{ a.path }}</option>
          <option v-if="model && !options.some((o) => o.path === model)" :value="model">{{ model }}</option>
        </select>
        <button type="button" class="btn" @click="importFile">Import…</button>
      </div>
      <slot />
    </div>
  </div>
</template>

<style scoped>
.picker { display: flex; gap: 12px; align-items: flex-start; }
.col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.thumb { width: 64px; height: 64px; border-radius: 12px; background: var(--panel-2); border: 1px solid var(--line); overflow: hidden; flex: none; display: grid; place-items: center; }
.thumb img { width: 100%; height: 100%; object-fit: cover; }
</style>
