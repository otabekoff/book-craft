<script setup lang="ts">
// Overlay scrollbars for a scrolling element this component does not own (for
// example CodeMirror's scroller). Place it inside a positioned parent that covers
// the target; hide the target's native scrollbar with `scrollbar-width: none`.
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useOverlayScroll } from "./overlayScroll";

const props = withDefaults(defineProps<{ target: HTMLElement | null | undefined; horizontal?: boolean }>(), { horizontal: false });

const hovered = ref(false);
const { y, x, active, dragging, attach, detach, grabThumb, clickTrack } = useOverlayScroll(() => props.horizontal);

const enter = () => (hovered.value = true);
const leave = () => (hovered.value = false);
function bind(el: HTMLElement | null | undefined, old?: HTMLElement | null) {
  old?.removeEventListener("pointerenter", enter);
  old?.removeEventListener("pointerleave", leave);
  if (!el) return detach();
  el.addEventListener("pointerenter", enter);
  el.addEventListener("pointerleave", leave);
  attach(el);
}
onMounted(() => bind(props.target));
watch(() => props.target, (el, old) => bind(el, old));
onBeforeUnmount(() => bind(null, props.target));
</script>

<template>
  <div class="ui-scrollbars" :class="{ active: active || dragging || hovered, dragging }" aria-hidden="true">
    <div v-if="y.show" class="ui-scroll-track y" @pointerdown.self="clickTrack($event, 'y')" @pointerenter="enter">
      <div class="ui-scroll-thumb" :style="{ height: y.size + 'px', transform: `translateY(${y.pos}px)` }" @pointerdown.prevent="grabThumb($event, 'y')"></div>
    </div>
    <div v-if="x.show" class="ui-scroll-track x" @pointerdown.self="clickTrack($event, 'x')" @pointerenter="enter">
      <div class="ui-scroll-thumb" :style="{ width: x.size + 'px', transform: `translateX(${x.pos}px)` }" @pointerdown.prevent="grabThumb($event, 'x')"></div>
    </div>
  </div>
</template>

<style>
.ui-scrollbars { position: absolute; inset: 0; pointer-events: none; z-index: 5; }
.ui-scrollbars .ui-scroll-track { pointer-events: auto; }
.ui-scrollbars.active > .ui-scroll-track, .ui-scrollbars .ui-scroll-track:hover { opacity: 1; }
</style>
