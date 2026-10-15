<script setup lang="ts">
// Scroll container with slim overlay scrollbars that show on hover and while scrolling.
// The native scrollbar is hidden; wheel, touch and keyboard scrolling stay native.
// Thumbs can be dragged, and clicking a track jumps a page toward the pointer.
import { onMounted, ref } from "vue";
import { useOverlayScroll } from "./overlayScroll";

const props = withDefaults(defineProps<{ viewportClass?: string; horizontal?: boolean }>(), { viewportClass: "", horizontal: false });

const viewport = ref<HTMLElement>();
const { y, x, active, dragging, attach, grabThumb, clickTrack } = useOverlayScroll(() => props.horizontal);
defineExpose({ viewport });
onMounted(() => attach(viewport.value!));
</script>

<template>
  <div class="ui-scroll" :class="{ active: active || dragging, dragging }">
    <div ref="viewport" class="ui-scroll-viewport" :class="viewportClass">
      <div class="ui-scroll-content"><slot /></div>
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
/* Tracks and thumbs, shared with UiScrollbars. */
.ui-scroll-track { position: absolute; z-index: 30; opacity: 0; transition: opacity 0.25s; }
.ui-scroll-track.y { top: 2px; bottom: 2px; right: 1px; width: 12px; }
.ui-scroll-track.x { left: 2px; right: 2px; bottom: 1px; height: 12px; }
.ui-scroll:hover > .ui-scroll-track, .ui-scroll.active > .ui-scroll-track { opacity: 1; }
.ui-scroll-thumb { position: absolute; border-radius: 999px; background: color-mix(in srgb, var(--ink) 26%, transparent); transition: background 0.15s, width 0.15s, height 0.15s; }
.ui-scroll-track.y .ui-scroll-thumb { top: 0; right: 2px; width: 6px; }
.ui-scroll-track.x .ui-scroll-thumb { left: 0; bottom: 2px; height: 6px; }
.ui-scroll-track:hover .ui-scroll-thumb, .dragging > .ui-scroll-track .ui-scroll-thumb { background: color-mix(in srgb, var(--ink) 42%, transparent); }
.ui-scroll-track.y:hover .ui-scroll-thumb, .dragging > .y .ui-scroll-thumb { width: 8px; }
.ui-scroll-track.x:hover .ui-scroll-thumb, .dragging > .x .ui-scroll-thumb { height: 8px; }
</style>
