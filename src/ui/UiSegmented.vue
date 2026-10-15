<script setup lang="ts" generic="T extends string | number">
// One-of-several picker shown as a row of buttons (radio group semantics).
import type { Component } from "vue";

defineProps<{
  options: { value: T; label?: string; icon?: Component; title?: string; disabled?: boolean }[];
  ariaLabel?: string;
  size?: "sm" | "md";
}>();
const model = defineModel<T>();

function key(e: KeyboardEvent, options: { value: T; disabled?: boolean }[], i: number) {
  const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
  if (!d) return;
  e.preventDefault();
  let next = (i + d + options.length) % options.length;
  while (options[next].disabled && next !== i) next = (next + d + options.length) % options.length;
  model.value = options[next].value;
  ((e.currentTarget as HTMLElement).parentElement?.children[next] as HTMLElement)?.focus();
}
</script>

<template>
  <div class="ui-seg" :class="size === 'sm' ? 's-sm' : ''" role="radiogroup" :aria-label="ariaLabel">
    <button
      v-for="(o, i) in options"
      :key="String(o.value)"
      type="button"
      role="radio"
      :aria-checked="model === o.value"
      :tabindex="model === o.value ? 0 : -1"
      :title="o.title ?? o.label"
      :aria-label="o.label ? undefined : o.title"
      :disabled="o.disabled"
      @click="model = o.value"
      @keydown="key($event, options, i)"
    >
      <component :is="o.icon" v-if="o.icon" class="ico" aria-hidden="true" />
      <span v-if="o.label">{{ o.label }}</span>
    </button>
  </div>
</template>

<style scoped>
.ui-seg { display: inline-flex; flex-wrap: wrap; gap: 2px; padding: 3px; border-radius: 9px; background: var(--panel-2); }
button { display: inline-flex; align-items: center; gap: 6px; min-height: 30px; padding: 0 12px; border: 0; border-radius: 7px; background: transparent; color: var(--muted); font: inherit; font-size: 13px; cursor: pointer; }
button:hover { color: var(--ink); }
button[aria-checked="true"] { background: var(--panel); color: var(--ink); font-weight: 600; box-shadow: 0 1px 2px rgb(0 0 0 / 0.14); }
.s-sm button { min-height: 26px; padding: 0 8px; }
button:disabled { opacity: 0.4; cursor: not-allowed; }
button:disabled:hover { color: var(--muted); }
.ico { width: 16px; height: 16px; }
</style>
