<script setup lang="ts">
// Custom slider: drag, click on the track, or use the keyboard
// (arrows = one step, Page Up/Down = ten steps, Home/End = ends).
import { computed, ref } from "vue";

const props = withDefaults(
  defineProps<{
    min: number;
    max: number;
    step?: number;
    ariaLabel?: string;
    format?: (v: number) => string;
    disabled?: boolean;
    marks?: number[];
    showValue?: boolean;
  }>(),
  { step: 1, showValue: true }
);
const model = defineModel<number>({ required: true });
const track = ref<HTMLElement>();
const dragging = ref(false);

const decimals = computed(() => (String(props.step).split(".")[1] ?? "").length);
const pct = computed(() => ((clamp(model.value) - props.min) / (props.max - props.min)) * 100);
const label = computed(() => (props.format ? props.format(model.value) : String(model.value)));

function clamp(v: number) {
  return Math.min(props.max, Math.max(props.min, v));
}
function snap(v: number) {
  const s = Math.round((v - props.min) / props.step) * props.step + props.min;
  return Number(clamp(s).toFixed(decimals.value));
}
function setFromX(x: number) {
  const r = track.value!.getBoundingClientRect();
  model.value = snap(props.min + ((x - r.left) / r.width) * (props.max - props.min));
}

function down(e: PointerEvent) {
  if (props.disabled) return;
  dragging.value = true;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  setFromX(e.clientX);
}
function moveP(e: PointerEvent) {
  if (dragging.value) setFromX(e.clientX);
}
function up() {
  dragging.value = false;
}
function key(e: KeyboardEvent) {
  const big = props.step * 10;
  const map: Record<string, number> = {
    ArrowRight: props.step,
    ArrowUp: props.step,
    ArrowLeft: -props.step,
    ArrowDown: -props.step,
    PageUp: big,
    PageDown: -big,
  };
  if (e.key in map) model.value = snap(model.value + map[e.key]);
  else if (e.key === "Home") model.value = props.min;
  else if (e.key === "End") model.value = props.max;
  else return;
  e.preventDefault();
}
</script>

<template>
  <div class="ui-range" :class="{ disabled, dragging }">
    <div
      ref="track"
      class="track"
      role="slider"
      :tabindex="disabled ? -1 : 0"
      :aria-label="ariaLabel"
      :aria-valuemin="min"
      :aria-valuemax="max"
      :aria-valuenow="model"
      :aria-valuetext="label"
      :aria-disabled="disabled || undefined"
      @pointerdown="down"
      @pointermove="moveP"
      @pointerup="up"
      @pointercancel="up"
      @keydown="key"
    >
      <div class="rail"></div>
      <div class="fill" :style="{ width: pct + '%' }"></div>
      <span
        v-for="m in marks ?? []"
        :key="m"
        class="mark"
        :style="{ left: ((m - min) / (max - min)) * 100 + '%' }"
      ></span>
      <div class="thumb" :style="{ left: pct + '%' }"><span v-if="dragging" class="bubble">{{ label }}</span></div>
    </div>
    <output v-if="showValue" class="out">{{ label }}</output>
  </div>
</template>

<style scoped>
.ui-range { display: flex; align-items: center; gap: 12px; min-height: 36px; }
.ui-range.disabled { opacity: 0.5; }
.track { position: relative; flex: 1; height: 28px; cursor: pointer; touch-action: none; outline: none; border-radius: 6px; }
.disabled .track { cursor: default; }
.rail, .fill { position: absolute; top: 50%; height: 4px; margin-top: -2px; border-radius: 2px; left: 0; }
.rail { right: 0; background: var(--line); }
.fill { background: var(--accent); }
.mark { position: absolute; top: 50%; width: 2px; height: 8px; margin: -4px 0 0 -1px; background: var(--muted); opacity: 0.5; border-radius: 1px; }
.thumb {
  position: absolute;
  top: 50%;
  width: 18px;
  height: 18px;
  margin: -9px 0 0 -9px;
  border-radius: 50%;
  background: var(--panel);
  border: 2px solid var(--accent);
  box-shadow: 0 1px 3px rgb(0 0 0 / 0.25);
  transition: transform 0.1s;
}
.track:hover .thumb, .dragging .thumb { transform: scale(1.12); }
.track:focus-visible .thumb { box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 30%, transparent); }
.bubble {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--ink);
  color: var(--bg);
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 5px;
  white-space: nowrap;
  pointer-events: none;
}
.out { min-width: 56px; text-align: right; font-size: 13px; font-variant-numeric: tabular-nums; color: var(--muted); }
</style>
