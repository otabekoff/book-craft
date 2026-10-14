<script setup lang="ts">
// A toggle with an optional label and description. The whole row is clickable.
defineProps<{ label?: string; description?: string; disabled?: boolean; ariaLabel?: string }>();
const model = defineModel<boolean>({ required: true });
</script>

<template>
  <label class="ui-switch" :class="{ disabled, 'no-text': !label }">
    <button
      type="button"
      role="switch"
      class="sw"
      :aria-checked="model"
      :aria-label="ariaLabel ?? label"
      :disabled="disabled"
      @click="model = !model"
    ><span class="knob"></span></button>
    <span v-if="label" class="text" @click.prevent="!disabled && (model = !model)">
      <b>{{ label }}</b>
      <small v-if="description">{{ description }}</small>
      <slot />
    </span>
  </label>
</template>

<style scoped>
.ui-switch { display: flex; align-items: flex-start; gap: 12px; padding: 6px 0; cursor: pointer; }
.ui-switch.no-text { padding: 0; display: inline-flex; }
.ui-switch.disabled { opacity: 0.5; cursor: default; }
.sw {
  position: relative;
  flex: none;
  width: 36px;
  height: 20px;
  margin-top: 1px;
  border-radius: 999px;
  border: 0;
  padding: 0;
  background: color-mix(in srgb, var(--muted) 45%, var(--line));
  cursor: inherit;
  transition: background 0.15s;
}
.sw[aria-checked="true"] { background: var(--accent); }
.knob { position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: white; box-shadow: 0 1px 2px rgb(0 0 0 / 0.3); transition: transform 0.15s; }
.sw[aria-checked="true"] .knob { transform: translateX(16px); }
.text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.text b { font-weight: 500; }
.text small { color: var(--muted); font-size: 12px; line-height: 1.45; }
.text :deep(code) { font-family: var(--mono); font-size: 11.5px; background: var(--panel-2); padding: 0 4px; border-radius: 4px; }
</style>
