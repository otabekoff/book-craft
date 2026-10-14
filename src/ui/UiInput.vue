<script setup lang="ts">
import { ref, type Component } from "vue";

const props = withDefaults(
  defineProps<{
    type?: string;
    placeholder?: string;
    icon?: Component;
    mono?: boolean;
    readonly?: boolean;
    disabled?: boolean;
    autofocus?: boolean;
    multiline?: boolean;
    rows?: number;
    min?: number;
    max?: number;
    ariaLabel?: string;
    id?: string;
  }>(),
  { type: "text", rows: 3 }
);
const model = defineModel<string | number>();
const el = ref<HTMLInputElement | HTMLTextAreaElement>();

function onInput(e: Event) {
  const v = (e.target as HTMLInputElement).value;
  model.value = props.type === "number" ? (v === "" ? 0 : Number(v)) : v;
}

defineExpose({ focus: () => el.value?.focus() });
</script>

<template>
  <div class="ui-input" :class="{ mono, disabled, 'has-icon': icon, multiline }">
    <component :is="icon" v-if="icon" class="lead" aria-hidden="true" />
    <textarea
      v-if="multiline"
      ref="el"
      :id="id"
      :rows="rows"
      :value="model"
      :placeholder="placeholder"
      :readonly="readonly"
      :disabled="disabled"
      :aria-label="ariaLabel"
      :autofocus="autofocus"
      @input="onInput"
    ></textarea>
    <input
      v-else
      ref="el"
      :id="id"
      :type="type"
      :value="model"
      :placeholder="placeholder"
      :readonly="readonly"
      :disabled="disabled"
      :min="min"
      :max="max"
      :aria-label="ariaLabel"
      :autofocus="autofocus"
      autocomplete="off"
      spellcheck="false"
      @input="onInput"
    />
    <slot name="end" />
  </div>
</template>

<style scoped>
.ui-input {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 36px;
  width: 100%;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--panel);
  transition: border-color 0.12s, box-shadow 0.12s;
}
.ui-input:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent); }
.ui-input.disabled { opacity: 0.55; }
.ui-input.multiline { align-items: stretch; padding: 0; }
input, textarea { flex: 1; min-width: 0; border: 0; outline: 0; background: transparent; font: inherit; color: var(--ink); padding: 7px 0; }
textarea { padding: 8px 10px; resize: vertical; line-height: 1.5; min-height: 72px; }
input::placeholder, textarea::placeholder { color: color-mix(in srgb, var(--muted) 80%, transparent); }
.mono input, .mono textarea { font-family: var(--mono); font-size: 13px; }
.lead { width: 16px; height: 16px; color: var(--muted); flex: none; }
input[type="number"] { -moz-appearance: textfield; }
input[type="number"]::-webkit-inner-spin-button { opacity: 0.6; }
</style>
