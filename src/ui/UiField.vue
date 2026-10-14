<script setup lang="ts">
// Label + control + hint. Pass `for` to link the label to a control with that id.
defineProps<{ label: string; hint?: string; error?: string; for?: string }>();
</script>

<template>
  <div class="ui-field">
    <label v-if="$props.for" class="lbl" :for="$props.for">{{ label }}</label>
    <span v-else class="lbl">{{ label }}</span>
    <slot />
    <small v-if="error" class="err">{{ error }}</small>
    <small v-else-if="hint || $slots.hint" class="hint"><slot name="hint">{{ hint }}</slot></small>
  </div>
</template>

<style scoped>
.ui-field { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
.lbl { font-size: 13px; color: var(--muted); font-weight: 500; }
.hint { color: var(--muted); font-size: 12px; line-height: 1.45; }
.hint :deep(code), .hint :deep(b) { font-family: var(--mono); font-size: 11.5px; }
.err { color: var(--danger); font-size: 12px; }
</style>
