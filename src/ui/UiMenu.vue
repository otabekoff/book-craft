<script setup lang="ts">
// Dropdown menu anchored to its trigger slot. Items: { label, icon?, danger?, separator?, action }.
import { nextTick, onBeforeUnmount, ref, watch, type Component } from "vue";

export interface MenuItem {
  label?: string;
  icon?: Component;
  danger?: boolean;
  separator?: boolean;
  disabled?: boolean;
  /** Indent level, for outline-like lists. */
  indent?: number;
  action?: () => void;
}

const props = defineProps<{ items: MenuItem[]; align?: "start" | "end" }>();
const open = ref(false);
const anchor = ref<HTMLElement>();
const menu = ref<HTMLElement>();
const pos = ref({ left: 0, top: 0 });

async function toggle() {
  open.value = !open.value;
  if (!open.value) return;
  const r = anchor.value!.getBoundingClientRect();
  pos.value = { left: r.left, top: r.bottom + 4 };
  await nextTick();
  const m = menu.value!.getBoundingClientRect();
  // Open upward only when there is more room above.
  const below = window.innerHeight - r.bottom, above = r.top;
  pos.value = {
    left: Math.max(8, Math.min(props.align === "start" ? r.left : r.right - m.width, window.innerWidth - m.width - 8)),
    top: m.height + 8 > below && above > below ? Math.max(8, r.top - m.height - 4) : r.bottom + 4,
  };
  menu.value?.querySelector<HTMLElement>("button:not(:disabled)")?.focus();
}

function run(item: MenuItem) {
  open.value = false;
  anchor.value?.querySelector<HTMLElement>("button")?.focus();
  item.action?.();
}

function key(e: KeyboardEvent) {
  const buttons = Array.from(menu.value?.querySelectorAll<HTMLElement>("button:not(:disabled)") ?? []);
  const i = buttons.indexOf(document.activeElement as HTMLElement);
  if (e.key === "ArrowDown") buttons[(i + 1) % buttons.length]?.focus();
  else if (e.key === "ArrowUp") buttons[(i - 1 + buttons.length) % buttons.length]?.focus();
  else if (e.key === "Escape" || e.key === "Tab") {
    open.value = false;
    anchor.value?.querySelector<HTMLElement>("button")?.focus();
  } else return;
  e.preventDefault();
}

function outside(e: PointerEvent) {
  const t = e.target as Node;
  if (!menu.value?.contains(t) && !anchor.value?.contains(t)) open.value = false;
}
watch(open, (o) => (o ? document.addEventListener("pointerdown", outside, true) : document.removeEventListener("pointerdown", outside, true)));
onBeforeUnmount(() => document.removeEventListener("pointerdown", outside, true));
</script>

<template>
  <span ref="anchor" class="ui-menu-anchor" @click.stop="toggle">
    <slot :open="open" />
  </span>
  <Teleport to="body">
    <div v-if="open" ref="menu" class="ui-menu" role="menu" :style="{ left: pos.left + 'px', top: pos.top + 'px' }" @keydown="key">
      <template v-for="(item, i) in items" :key="i">
        <hr v-if="item.separator" />
        <button v-else type="button" role="menuitem" :class="{ danger: item.danger }" :disabled="item.disabled" :style="item.indent ? { paddingLeft: 10 + item.indent * 14 + 'px' } : undefined" @click="run(item)">
          <component :is="item.icon" v-if="item.icon" class="ico" aria-hidden="true" />
          <span>{{ item.label }}</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.ui-menu-anchor { display: inline-flex; }
</style>

<style>
.ui-menu { position: fixed; z-index: 180; min-width: 220px; max-width: min(420px, calc(100vw - 16px)); max-height: min(70vh, 520px); overflow-y: auto; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; box-shadow: 0 14px 40px rgb(0 0 0 / 0.2); padding: 4px; animation: ui-pop 0.12s ease-out; }
.ui-menu button { display: flex; width: 100%; align-items: center; gap: 10px; border: 0; background: none; color: var(--ink); text-align: left; padding: 8px 10px; border-radius: 6px; cursor: pointer; font: inherit; font-size: 13px; }
.ui-menu button:hover, .ui-menu button:focus-visible { background: var(--panel-2); outline: none; }
.ui-menu button:disabled { opacity: 0.45; cursor: default; }
.ui-menu button.danger { color: var(--danger); }
.ui-menu button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ui-menu .ico { flex: none; width: 16px; height: 16px; color: var(--muted); }
.ui-menu button.danger .ico { color: var(--danger); }
.ui-menu hr { border: 0; border-top: 1px solid var(--line); margin: 4px 0; }
</style>
