<script setup lang="ts">
import type { Component } from "vue";
import { LoaderCircle } from "@lucide/vue";

withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md" | "lg";
    icon?: Component;
    iconRight?: Component;
    loading?: boolean;
    disabled?: boolean;
    type?: "button" | "submit";
    block?: boolean;
  }>(),
  { variant: "secondary", size: "md", type: "button" }
);
</script>

<template>
  <button
    :type="type"
    class="ui-btn"
    :class="[`v-${variant}`, `s-${size}`, { block }]"
    :disabled="disabled || loading"
    :aria-busy="loading || undefined"
  >
    <LoaderCircle v-if="loading" class="ui-ico spin" aria-hidden="true" />
    <component :is="icon" v-else-if="icon" class="ui-ico" aria-hidden="true" />
    <span v-if="$slots.default" class="label"><slot /></span>
    <component :is="iconRight" v-if="iconRight" class="ui-ico" aria-hidden="true" />
  </button>
</template>

<style scoped>
.ui-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: 8px;
  border: 1px solid transparent;
  font: inherit;
  font-weight: 500;
  white-space: nowrap;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, filter 0.12s;
  color: var(--ink);
}
.ui-btn:disabled { opacity: 0.5; cursor: default; }
.s-sm { min-height: 30px; padding: 0 10px; font-size: 13px; }
.s-md { min-height: 36px; padding: 0 14px; }
.s-lg { min-height: 44px; padding: 0 20px; font-size: 15px; }
.block { width: 100%; }
.v-primary { background: var(--accent); color: var(--accent-ink); }
.v-primary:not(:disabled):hover { filter: brightness(1.08); }
.v-secondary { background: var(--panel); border-color: var(--line); }
.v-secondary:not(:disabled):hover { background: var(--panel-2); }
.v-ghost { background: transparent; }
.v-ghost:not(:disabled):hover { background: var(--panel-2); }
.v-danger { background: var(--panel); border-color: color-mix(in srgb, var(--danger) 40%, var(--line)); color: var(--danger); }
.v-danger:not(:disabled):hover { background: color-mix(in srgb, var(--danger) 10%, var(--panel)); }
.ui-ico { width: 16px; height: 16px; flex: none; }
.s-lg .ui-ico { width: 18px; height: 18px; }
.spin { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
