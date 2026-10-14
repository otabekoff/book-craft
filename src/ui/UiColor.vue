<script setup lang="ts">
// Color field: preset swatches, a native picker for anything else, and the hex value.
import { Check, Pipette } from "@lucide/vue";

withDefaults(defineProps<{ swatches?: string[] }>(), {
  swatches: () => ["#2f5a8a", "#1d6fa5", "#0f766e", "#2e7d4f", "#7a3b2e", "#b45309", "#b0306a", "#5b3f86", "#374151"],
});
const model = defineModel<string>({ required: true });

function onHex(e: Event) {
  const v = (e.target as HTMLInputElement).value.trim();
  if (/^#?[0-9a-f]{6}$/i.test(v)) model.value = v.startsWith("#") ? v : "#" + v;
}
</script>

<template>
  <div class="ui-color">
    <div class="swatches" role="radiogroup" aria-label="Preset colors">
      <button
        v-for="s in swatches"
        :key="s"
        type="button"
        role="radio"
        class="sw"
        :style="{ background: s }"
        :aria-checked="model.toLowerCase() === s"
        :aria-label="s"
        :title="s"
        @click="model = s"
      ><Check v-if="model.toLowerCase() === s" class="tick" /></button>
      <label class="sw custom" title="Pick any color">
        <Pipette class="pip" />
        <input type="color" :value="model" @input="model = ($event.target as HTMLInputElement).value" aria-label="Pick any color" />
      </label>
    </div>
    <div class="hex">
      <span class="preview" :style="{ background: model }"></span>
      <input :value="model" @change="onHex" aria-label="Hex color" spellcheck="false" />
    </div>
  </div>
</template>

<style scoped>
.ui-color { display: flex; flex-direction: column; gap: 10px; }
.swatches { display: flex; flex-wrap: wrap; gap: 8px; }
.sw { width: 28px; height: 28px; border-radius: 50%; border: 0; display: grid; place-items: center; cursor: pointer; padding: 0; position: relative; }
.sw[aria-checked="true"] { box-shadow: 0 0 0 2px var(--panel), 0 0 0 4px var(--ink); }
.tick { width: 15px; height: 15px; color: white; stroke-width: 3; }
.custom { background: conic-gradient(red, yellow, lime, cyan, blue, magenta, red); overflow: hidden; }
.pip { width: 14px; height: 14px; color: white; filter: drop-shadow(0 0 1px rgb(0 0 0 / 0.6)); }
.custom input { position: absolute; inset: 0; opacity: 0; cursor: pointer; }
.hex { display: flex; align-items: center; gap: 8px; width: 150px; min-height: 34px; padding: 0 10px; border: 1px solid var(--line); border-radius: 8px; background: var(--panel); }
.hex:focus-within { border-color: var(--accent); }
.preview { width: 16px; height: 16px; border-radius: 4px; flex: none; border: 1px solid rgb(0 0 0 / 0.1); }
.hex input { border: 0; outline: 0; background: none; font-family: var(--mono); font-size: 13px; color: var(--ink); width: 100%; }
</style>
