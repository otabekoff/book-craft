<script setup lang="ts" generic="T extends string | number">
// Accessible custom select: a combobox button that opens a listbox popover.
// Keyboard: arrows, Home/End, Enter/Space, Escape, type-ahead; search when the list is long.
import { computed, nextTick, onBeforeUnmount, ref, useId, watch, type Component } from "vue";
import { Check, ChevronDown, Search } from "@lucide/vue";

export interface SelectOption<V> {
  value: V;
  label: string;
  description?: string;
  icon?: Component;
  disabled?: boolean;
  group?: string;
  /** Shows the label in this CSS font-family (font pickers). */
  fontFamily?: string;
  /** Small text on the right, e.g. a size. */
  meta?: string;
}

const props = withDefaults(
  defineProps<{
    options: SelectOption<T>[];
    placeholder?: string;
    ariaLabel?: string;
    disabled?: boolean;
    mono?: boolean;
    searchable?: boolean | "auto";
    id?: string;
  }>(),
  { placeholder: "Choose…", searchable: "auto" }
);
const model = defineModel<T>();

const uid = useId();
const listId = `${uid}-list`;
const open = ref(false);
const active = ref(-1);
const query = ref("");
const trigger = ref<HTMLButtonElement>();
const list = ref<HTMLElement>();
const search = ref<HTMLInputElement>();
const pos = ref({ left: 0, top: 0, width: 0, maxHeight: 320, above: false });
let typed = "";
let typedTimer = 0;

const selected = computed(() => props.options.find((o) => o.value === model.value));
const showSearch = computed(() => (props.searchable === "auto" ? props.options.length > 8 : props.searchable));
const visible = computed(() => {
  const q = query.value.trim().toLowerCase();
  return q ? props.options.filter((o) => o.label.toLowerCase().includes(q) || o.description?.toLowerCase().includes(q)) : props.options;
});

function place() {
  const r = trigger.value!.getBoundingClientRect();
  const below = window.innerHeight - r.bottom - 12;
  const above = r.top - 12;
  const wanted = Math.min(340, visible.value.length * 40 + (showSearch.value ? 52 : 8));
  const flip = below < Math.min(wanted, 220) && above > below;
  pos.value = {
    left: Math.min(r.left, window.innerWidth - Math.max(r.width, 220) - 8),
    top: flip ? r.top - 4 : r.bottom + 4,
    width: Math.max(r.width, 220),
    maxHeight: Math.max(160, Math.min(340, flip ? above : below)),
    above: flip,
  };
}

async function show() {
  if (props.disabled || open.value) return;
  query.value = "";
  open.value = true;
  active.value = Math.max(0, visible.value.findIndex((o) => o.value === model.value));
  place();
  await nextTick();
  (showSearch.value ? search.value : list.value)?.focus();
  scrollActive();
}

function hide(refocus = true) {
  open.value = false;
  if (refocus) trigger.value?.focus();
}

function choose(o: SelectOption<T> | undefined) {
  if (!o || o.disabled) return;
  model.value = o.value;
  hide();
}

function move(delta: number) {
  const n = visible.value.length;
  if (!n) return;
  let i = active.value;
  for (let step = 0; step < n; step++) {
    i = (i + delta + n) % n;
    if (!visible.value[i].disabled) break;
  }
  active.value = i;
  scrollActive();
}

function scrollActive() {
  nextTick(() => list.value?.querySelector<HTMLElement>(`[data-index="${active.value}"]`)?.scrollIntoView({ block: "nearest" }));
}

function onTriggerKey(e: KeyboardEvent) {
  if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
    e.preventDefault();
    show();
  }
}

function onListKey(e: KeyboardEvent) {
  switch (e.key) {
    case "ArrowDown":
      e.preventDefault();
      move(1);
      break;
    case "ArrowUp":
      e.preventDefault();
      move(-1);
      break;
    case "Home":
      if (e.target === list.value) {
        e.preventDefault();
        active.value = 0;
        scrollActive();
      }
      break;
    case "End":
      if (e.target === list.value) {
        e.preventDefault();
        active.value = visible.value.length - 1;
        scrollActive();
      }
      break;
    case "Enter":
      e.preventDefault();
      choose(visible.value[active.value]);
      break;
    case "Escape":
      e.preventDefault();
      hide();
      break;
    case "Tab":
      hide(false);
      break;
    default:
      if (!showSearch.value && e.key.length === 1) {
        typed += e.key.toLowerCase();
        clearTimeout(typedTimer);
        typedTimer = window.setTimeout(() => (typed = ""), 600);
        const i = visible.value.findIndex((o) => o.label.toLowerCase().startsWith(typed));
        if (i >= 0) {
          active.value = i;
          scrollActive();
        }
      }
  }
}

function onOutside(e: PointerEvent) {
  const t = e.target as Node;
  if (!list.value?.parentElement?.contains(t) && !trigger.value?.contains(t)) hide(false);
}
function onViewport() {
  if (open.value) place();
}

watch(open, (o) => {
  if (o) {
    document.addEventListener("pointerdown", onOutside, true);
    window.addEventListener("resize", onViewport);
    window.addEventListener("scroll", onViewport, true);
  } else {
    document.removeEventListener("pointerdown", onOutside, true);
    window.removeEventListener("resize", onViewport);
    window.removeEventListener("scroll", onViewport, true);
  }
});
watch(query, () => (active.value = visible.value.findIndex((o) => !o.disabled)));
onBeforeUnmount(() => (open.value = false));
</script>

<template>
  <button
    ref="trigger"
    type="button"
    class="ui-select"
    :class="{ open, mono }"
    :id="id"
    role="combobox"
    aria-haspopup="listbox"
    :aria-expanded="open"
    :aria-controls="listId"
    :aria-label="ariaLabel"
    :disabled="disabled"
    @click="open ? hide() : show()"
    @keydown="onTriggerKey"
  >
    <component :is="selected.icon" v-if="selected?.icon" class="opt-icon" aria-hidden="true" />
    <span class="value" :class="{ placeholder: !selected }" :style="selected?.fontFamily ? { fontFamily: selected.fontFamily } : undefined">{{ selected?.label ?? placeholder }}</span>
    <ChevronDown class="chev" aria-hidden="true" />
  </button>
  <Teleport to="body">
    <div
      v-if="open"
      class="ui-select-pop"
      :class="{ above: pos.above }"
      :style="{ left: pos.left + 'px', top: pos.top + 'px', width: pos.width + 'px', maxHeight: pos.maxHeight + 'px' }"
    >
      <div v-if="showSearch" class="search">
        <Search class="s-ico" aria-hidden="true" />
        <input ref="search" v-model="query" placeholder="Search…" aria-label="Search options" @keydown="onListKey" />
      </div>
      <ul
        ref="list"
        :id="listId"
        role="listbox"
        tabindex="-1"
        :aria-activedescendant="active >= 0 ? `${uid}-o${active}` : undefined"
        @keydown="onListKey"
      >
        <template v-for="(o, i) in visible" :key="String(o.value)">
          <li v-if="o.group && o.group !== visible[i - 1]?.group" class="group" role="presentation">{{ o.group }}</li>
          <li
            :id="`${uid}-o${i}`"
            role="option"
            :data-index="i"
            :aria-selected="o.value === model"
            :aria-disabled="o.disabled || undefined"
            :class="{ active: i === active, disabled: o.disabled }"
            @pointermove="active = i"
            @click="choose(o)"
          >
            <component :is="o.icon" v-if="o.icon" class="opt-icon" aria-hidden="true" />
            <span class="text">
              <span :class="{ mono }" :style="o.fontFamily ? { fontFamily: o.fontFamily, fontSize: '15px' } : undefined">{{ o.label }}</span>
              <small v-if="o.description">{{ o.description }}</small>
            </span>
            <small v-if="o.meta" class="meta">{{ o.meta }}</small>
            <Check v-if="o.value === model" class="tick" aria-hidden="true" />
          </li>
        </template>
        <li v-if="!visible.length" class="empty" role="presentation">No matches</li>
      </ul>
    </div>
  </Teleport>
</template>

<style scoped>
.ui-select {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 36px;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--panel);
  color: var(--ink);
  font: inherit;
  text-align: left;
  cursor: pointer;
}
.ui-select:not(:disabled):hover { border-color: color-mix(in srgb, var(--accent) 40%, var(--line)); }
.ui-select.open, .ui-select:focus-visible { border-color: var(--accent); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent); outline: none; }
.ui-select:disabled { opacity: 0.55; cursor: default; }
.value { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.value.placeholder { color: var(--muted); }
.mono .value, .mono { font-family: var(--mono); font-size: 13px; }
.chev { width: 16px; height: 16px; color: var(--muted); transition: transform 0.15s; flex: none; }
.open .chev { transform: rotate(180deg); }
.opt-icon { width: 16px; height: 16px; flex: none; color: var(--muted); }
</style>

<style>
.ui-select-pop {
  position: fixed;
  z-index: 200;
  display: flex;
  flex-direction: column;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 10px;
  box-shadow: 0 14px 40px rgb(0 0 0 / 0.2);
  overflow: hidden;
  animation: ui-pop 0.12s ease-out;
}
.ui-select-pop.above { transform: translateY(-100%); animation-name: ui-pop-up; }
@keyframes ui-pop { from { opacity: 0; translate: 0 -4px; } }
@keyframes ui-pop-up { from { opacity: 0; translate: 0 4px; } }
.ui-select-pop .search { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--line); }
.ui-select-pop .search input { flex: 1; border: 0; outline: 0; background: transparent; font: inherit; color: var(--ink); }
.ui-select-pop .s-ico { width: 15px; height: 15px; color: var(--muted); }
.ui-select-pop ul { list-style: none; margin: 0; padding: 4px; overflow-y: auto; outline: none; }
.ui-select-pop li[role="option"] { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: 6px; cursor: pointer; min-height: 36px; }
.ui-select-pop li.active { background: var(--panel-2); }
.ui-select-pop li[aria-selected="true"] { color: var(--accent); font-weight: 500; }
.ui-select-pop li.disabled { opacity: 0.45; cursor: default; }
.ui-select-pop .text { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.ui-select-pop .text small { color: var(--muted); font-size: 12px; font-weight: 400; }
.ui-select-pop .tick { width: 16px; height: 16px; flex: none; }
.ui-select-pop .meta { color: var(--muted); font-size: 11px; white-space: nowrap; }
.ui-select-pop .group { padding: 8px 10px 4px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); font-weight: 600; }
.ui-select-pop .empty { padding: 12px; color: var(--muted); text-align: center; }
.ui-select-pop .mono { font-family: var(--mono); font-size: 13px; }
</style>
