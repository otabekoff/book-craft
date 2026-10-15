<script setup lang="ts">
// Scroll container with slim overlay scrollbars that show on hover and while scrolling.
// The native scrollbar is hidden; wheel, touch and keyboard scrolling stay native.
// Thumbs can be dragged, and clicking a track jumps a page toward the pointer.
import { onBeforeUnmount, onMounted, reactive, ref } from "vue";

const props = withDefaults(defineProps<{ viewportClass?: string; horizontal?: boolean }>(), { viewportClass: "", horizontal: false });

const viewport = ref<HTMLElement>();
const content = ref<HTMLElement>();
const active = ref(false);
const dragging = ref(false);
const y = reactive({ show: false, size: 0, pos: 0 });
const x = reactive({ show: false, size: 0, pos: 0 });
defineExpose({ viewport });

const INSET = 2;
const MIN_THUMB = 28;
let hideTimer = 0;
let pending = false;
let resize: ResizeObserver | undefined;
let mutations: MutationObserver | undefined;

function measure(axis: "y" | "x") {
  const el = viewport.value!;
  const vertical = axis === "y";
  const view = vertical ? el.clientHeight : el.clientWidth;
  const full = vertical ? el.scrollHeight : el.scrollWidth;
  const offset = vertical ? el.scrollTop : el.scrollLeft;
  const bar = vertical ? y : x;
  const track = view - INSET * 2;
  bar.show = full > view + 1 && (vertical || props.horizontal);
  if (!bar.show) return;
  bar.size = Math.max(MIN_THUMB, (track * view) / full);
  bar.pos = ((track - bar.size) * offset) / (full - view);
}
/** Measure once per burst of scroll, resize and DOM changes. */
function update() {
  if (pending) return;
  pending = true;
  queueMicrotask(() => {
    pending = false;
    if (!viewport.value) return;
    measure("y");
    measure("x");
  });
}
function wake() {
  active.value = true;
  clearTimeout(hideTimer);
  hideTimer = window.setTimeout(() => (active.value = false), 900);
}
function onScroll() {
  update();
  wake();
}

/** Scroll so the thumb follows the pointer. */
function grabThumb(e: PointerEvent, axis: "y" | "x") {
  const el = viewport.value!;
  const vertical = axis === "y";
  const bar = vertical ? y : x;
  const start = vertical ? e.clientY : e.clientX;
  const startScroll = vertical ? el.scrollTop : el.scrollLeft;
  const view = vertical ? el.clientHeight : el.clientWidth;
  const full = vertical ? el.scrollHeight : el.scrollWidth;
  const ratio = (full - view) / Math.max(1, view - INSET * 2 - bar.size);
  const target = e.currentTarget as HTMLElement;
  target.setPointerCapture(e.pointerId);
  dragging.value = true;
  const move = (ev: PointerEvent) => {
    const delta = ((vertical ? ev.clientY : ev.clientX) - start) * ratio;
    if (vertical) el.scrollTop = startScroll + delta;
    else el.scrollLeft = startScroll + delta;
  };
  const up = () => {
    dragging.value = false;
    target.removeEventListener("pointermove", move);
    target.removeEventListener("pointerup", up);
    target.removeEventListener("pointercancel", up);
    wake();
  };
  target.addEventListener("pointermove", move);
  target.addEventListener("pointerup", up);
  target.addEventListener("pointercancel", up);
}
/** Clicking the track moves one view toward the pointer. */
function clickTrack(e: PointerEvent, axis: "y" | "x") {
  const el = viewport.value!;
  const vertical = axis === "y";
  const bar = vertical ? y : x;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const at = (vertical ? e.clientY - rect.top : e.clientX - rect.left) - INSET;
  const dir = at < bar.pos ? -1 : 1;
  el.scrollBy({ [vertical ? "top" : "left"]: dir * (vertical ? el.clientHeight : el.clientWidth) * 0.9, behavior: "smooth" });
}

onMounted(() => {
  update();
  resize = new ResizeObserver(update);
  resize.observe(viewport.value!);
  if (content.value) resize.observe(content.value);
  mutations = new MutationObserver(update);
  mutations.observe(viewport.value!, { childList: true, subtree: true, characterData: true });
});
onBeforeUnmount(() => {
  resize?.disconnect();
  mutations?.disconnect();
  clearTimeout(hideTimer);
});
</script>

<template>
  <div class="ui-scroll" :class="{ active: active || dragging, dragging }">
    <div ref="viewport" class="ui-scroll-viewport" :class="viewportClass" @scroll.passive="onScroll">
      <div ref="content" class="ui-scroll-content"><slot /></div>
    </div>
    <div v-if="y.show" class="ui-scroll-track y" aria-hidden="true" @pointerdown.self="clickTrack($event, 'y')">
      <div class="ui-scroll-thumb" :style="{ height: y.size + 'px', transform: `translateY(${y.pos}px)` }" @pointerdown.prevent="grabThumb($event, 'y')"></div>
    </div>
    <div v-if="x.show" class="ui-scroll-track x" aria-hidden="true" @pointerdown.self="clickTrack($event, 'x')">
      <div class="ui-scroll-thumb" :style="{ width: x.size + 'px', transform: `translateX(${x.pos}px)` }" @pointerdown.prevent="grabThumb($event, 'x')"></div>
    </div>
  </div>
</template>

<style>
.ui-scroll { position: relative; min-height: 0; min-width: 0; overflow: hidden; display: flex; flex-direction: column; }
.ui-scroll-viewport { flex: 1; min-height: 0; overflow: auto; scrollbar-width: none; overscroll-behavior: none; }
.ui-scroll-viewport::-webkit-scrollbar { display: none; }
.ui-scroll-content { min-width: 0; }
.ui-scroll-track { position: absolute; z-index: 30; opacity: 0; transition: opacity 0.25s; }
.ui-scroll-track.y { top: 2px; bottom: 2px; right: 1px; width: 12px; }
.ui-scroll-track.x { left: 2px; right: 2px; bottom: 1px; height: 12px; }
.ui-scroll:hover > .ui-scroll-track, .ui-scroll.active > .ui-scroll-track { opacity: 1; }
.ui-scroll-thumb { position: absolute; border-radius: 999px; background: color-mix(in srgb, var(--ink) 26%, transparent); transition: background 0.15s, width 0.15s, height 0.15s; }
.ui-scroll-track.y .ui-scroll-thumb { top: 0; right: 2px; width: 6px; }
.ui-scroll-track.x .ui-scroll-thumb { left: 0; bottom: 2px; height: 6px; }
.ui-scroll-track:hover .ui-scroll-thumb, .ui-scroll.dragging .ui-scroll-thumb { background: color-mix(in srgb, var(--ink) 42%, transparent); }
.ui-scroll-track.y:hover .ui-scroll-thumb, .ui-scroll.dragging > .y .ui-scroll-thumb { width: 8px; }
.ui-scroll-track.x:hover .ui-scroll-thumb, .ui-scroll.dragging > .x .ui-scroll-thumb { height: 8px; }
</style>
