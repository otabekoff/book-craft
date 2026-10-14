<script setup lang="ts">
defineProps<{ tone?: "neutral" | "ok" | "warn" | "bad" | "accent" }>();
</script>

<template>
  <span class="ui-badge" :class="tone ?? 'neutral'"><slot /></span>
</template>

<style scoped>
.ui-badge { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; font-weight: 600; padding: 2px 8px; border-radius: 999px; white-space: nowrap; }
.neutral { background: var(--panel-2); color: var(--muted); }
.ok { background: color-mix(in srgb, var(--ok) 14%, var(--panel)); color: var(--ok); }
.warn { background: color-mix(in srgb, var(--warn) 16%, var(--panel)); color: var(--warn); }
.bad { background: color-mix(in srgb, var(--danger) 12%, var(--panel)); color: var(--danger); }
.accent { background: var(--accent-soft); color: var(--accent); }
</style>
