<script setup lang="ts">
import { Check } from "@lucide/vue";

defineProps<{ label?: string; description?: string; disabled?: boolean }>();
const model = defineModel<boolean>({ required: true });
</script>

<template>
  <label class="ui-check" :class="{ disabled }">
    <input type="checkbox" v-model="model" :disabled="disabled" class="native" />
    <span class="box" aria-hidden="true"><Check class="tick" /></span>
    <span v-if="label || $slots.default" class="text">
      <b v-if="label">{{ label }}</b>
      <small v-if="description">{{ description }}</small>
      <slot />
    </span>
  </label>
</template>

<style scoped>
.ui-check { display: flex; align-items: flex-start; gap: 10px; cursor: pointer; padding: 4px 0; position: relative; }
.ui-check.disabled { opacity: 0.5; cursor: default; }
.native { position: absolute; opacity: 0; width: 1px; height: 1px; margin: 0; }
.box {
  flex: none;
  width: 18px;
  height: 18px;
  margin-top: 1px;
  border-radius: 5px;
  border: 1.5px solid color-mix(in srgb, var(--muted) 70%, var(--line));
  background: var(--panel);
  display: grid;
  place-items: center;
  transition: background 0.12s, border-color 0.12s;
}
.tick { width: 13px; height: 13px; color: var(--accent-ink); stroke-width: 3; opacity: 0; transform: scale(0.6); transition: all 0.12s; }
.native:checked + .box { background: var(--accent); border-color: var(--accent); }
.native:checked + .box .tick { opacity: 1; transform: none; }
.native:focus-visible + .box { box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 30%, transparent); }
.text { display: flex; flex-direction: column; gap: 2px; }
.text b { font-weight: 500; }
.text small { color: var(--muted); font-size: 12px; line-height: 1.45; }
</style>
