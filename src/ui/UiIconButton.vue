<script setup lang="ts">
import type { Component } from "vue";

withDefaults(defineProps<{ icon: Component; label: string; pressed?: boolean; size?: "sm" | "md"; disabled?: boolean }>(), {
  size: "md",
});
</script>

<template>
  <button
    type="button"
    class="ui-icon-btn"
    :class="`s-${size}`"
    :aria-label="label"
    :title="label"
    :aria-pressed="pressed === undefined ? undefined : pressed"
    :disabled="disabled"
  >
    <component :is="icon" class="ico" aria-hidden="true" />
  </button>
</template>

<style scoped>
.ui-icon-btn {
  display: inline-grid;
  place-items: center;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink);
  cursor: pointer;
  flex: none;
  padding: 0;
}
.s-md { width: 34px; height: 34px; }
.s-sm { width: 28px; height: 28px; }
.ui-icon-btn:not(:disabled):hover { background: var(--panel-2); }
.ui-icon-btn[aria-pressed="true"] { background: var(--accent-soft); color: var(--accent); }
.ui-icon-btn:disabled { opacity: 0.4; cursor: default; }
.ico { width: 18px; height: 18px; }
.s-sm .ico { width: 16px; height: 16px; }
</style>
