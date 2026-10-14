<script setup lang="ts">
// Modal dialog: Escape and the backdrop close it, focus moves inside and returns afterwards.
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { X } from "@lucide/vue";

defineProps<{ title: string; width?: number }>();
const emit = defineEmits<{ close: [] }>();
const box = ref<HTMLElement>();
const opener = document.activeElement as HTMLElement | null;

function key(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  }
  if (e.key === "Tab" && box.value) {
    const items = box.value.querySelectorAll<HTMLElement>("button, input, textarea, select, [tabindex]:not([tabindex='-1'])");
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last?.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first?.focus();
    }
  }
}

onMounted(async () => {
  await nextTick();
  (box.value?.querySelector<HTMLElement>("[autofocus], input, textarea, button.primary") ?? box.value)?.focus();
});
onBeforeUnmount(() => opener?.focus?.());
</script>

<template>
  <Teleport to="body">
    <div class="ui-overlay" @pointerdown.self="emit('close')">
      <div ref="box" class="ui-dialog" role="dialog" aria-modal="true" :aria-label="title" tabindex="-1" :style="{ width: (width ?? 460) + 'px' }" @keydown="key">
        <header>
          <h2>{{ title }}</h2>
          <button type="button" class="x" aria-label="Close" @click="emit('close')"><X /></button>
        </header>
        <div class="content"><slot /></div>
        <footer v-if="$slots.actions"><slot name="actions" /></footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.ui-overlay { position: fixed; inset: 0; z-index: 150; background: rgb(0 0 0 / 0.38); display: grid; place-items: center; padding: 24px; animation: fade 0.12s; }
.ui-dialog { max-width: 100%; max-height: calc(100vh - 48px); overflow: auto; background: var(--panel); color: var(--ink); border-radius: 14px; box-shadow: 0 24px 60px rgb(0 0 0 / 0.3); display: flex; flex-direction: column; outline: none; animation: rise 0.15s ease-out; }
header { display: flex; align-items: center; gap: 10px; padding: 18px 20px 0; }
h2 { flex: 1; margin: 0; font-size: 17px; font-weight: 600; }
.x { width: 30px; height: 30px; border: 0; background: none; border-radius: 7px; display: grid; place-items: center; color: var(--muted); cursor: pointer; }
.x:hover { background: var(--panel-2); color: var(--ink); }
.x svg { width: 18px; height: 18px; }
.content { padding: 14px 20px 6px; display: flex; flex-direction: column; gap: 14px; line-height: 1.5; }
footer { display: flex; justify-content: flex-end; gap: 8px; padding: 14px 20px 18px; }
@keyframes fade { from { opacity: 0; } }
@keyframes rise { from { opacity: 0; transform: translateY(8px) scale(0.98); } }
</style>
