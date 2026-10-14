<script setup lang="ts">
// Settings card. With `v-model:enabled` the header gets a switch and the body dims when off.
import type { Component } from "vue";
import UiSwitch from "./UiSwitch.vue";

defineProps<{ title?: string; description?: string; icon?: Component }>();
const enabled = defineModel<boolean | undefined>("enabled", { default: undefined });
</script>

<template>
  <section class="ui-card">
    <header v-if="title" class="head">
      <component :is="icon" v-if="icon" class="ico" aria-hidden="true" />
      <div class="titles">
        <h3>{{ title }}</h3>
        <p v-if="description">{{ description }}</p>
      </div>
      <slot name="actions" />
      <UiSwitch v-if="enabled !== undefined" v-model="enabled as boolean" :aria-label="`Turn ${title} on or off`" />
    </header>
    <div class="body" :class="{ off: enabled === false }"><slot /></div>
    <div v-if="$slots.footer" class="foot"><slot name="footer" /></div>
  </section>
</template>

<style scoped>
.ui-card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 18px 20px 20px; display: flex; flex-direction: column; gap: 16px; }
.head { display: flex; align-items: flex-start; gap: 12px; }
.ico { width: 20px; height: 20px; color: var(--accent); flex: none; margin-top: 1px; }
.titles { flex: 1; min-width: 0; }
h3 { margin: 0; font-size: 15px; font-weight: 600; }
p { margin: 4px 0 0; color: var(--muted); font-size: 13px; line-height: 1.5; }
p :deep(code) { font-family: var(--mono); font-size: 12px; }
.body { display: flex; flex-direction: column; gap: 16px; transition: opacity 0.15s; }
.body.off { opacity: 0.45; pointer-events: none; }
.body:empty { display: none; }
.foot { border-top: 1px solid var(--line); padding-top: 14px; }
</style>
