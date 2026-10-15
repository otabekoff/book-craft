// Overlay scrollbars: slim thumbs drawn over a scrolling element whose native
// scrollbar is hidden, so they take no space. Shared by UiScrollArea (which owns
// its scrolling element) and UiScrollbars (which attaches to one, e.g. CodeMirror's).
import { onBeforeUnmount, reactive, ref } from "vue";

export interface Bar {
  show: boolean;
  size: number;
  pos: number;
}

const INSET = 2;
const MIN_THUMB = 28;

export function useOverlayScroll(horizontal: () => boolean) {
  let el: HTMLElement | null = null;
  const active = ref(false);
  const dragging = ref(false);
  const y = reactive<Bar>({ show: false, size: 0, pos: 0 });
  const x = reactive<Bar>({ show: false, size: 0, pos: 0 });
  let hideTimer = 0;
  let pending = false;
  let resize: ResizeObserver | undefined;
  let mutations: MutationObserver | undefined;

  function measure(axis: "y" | "x") {
    if (!el) return;
    const vertical = axis === "y";
    const view = vertical ? el.clientHeight : el.clientWidth;
    const full = vertical ? el.scrollHeight : el.scrollWidth;
    const offset = vertical ? el.scrollTop : el.scrollLeft;
    const bar = vertical ? y : x;
    const track = view - INSET * 2;
    bar.show = full > view + 1 && (vertical || horizontal());
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
    if (!el) return;
    const target = el;
    const vertical = axis === "y";
    const bar = vertical ? y : x;
    const start = vertical ? e.clientY : e.clientX;
    const startScroll = vertical ? target.scrollTop : target.scrollLeft;
    const view = vertical ? target.clientHeight : target.clientWidth;
    const full = vertical ? target.scrollHeight : target.scrollWidth;
    const ratio = (full - view) / Math.max(1, view - INSET * 2 - bar.size);
    const handle = e.currentTarget as HTMLElement;
    handle.setPointerCapture(e.pointerId);
    dragging.value = true;
    const move = (ev: PointerEvent) => {
      const delta = ((vertical ? ev.clientY : ev.clientX) - start) * ratio;
      if (vertical) target.scrollTop = startScroll + delta;
      else target.scrollLeft = startScroll + delta;
    };
    const up = () => {
      dragging.value = false;
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", up);
      handle.removeEventListener("pointercancel", up);
      wake();
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", up);
    handle.addEventListener("pointercancel", up);
  }
  /** Clicking the track moves one view toward the pointer. */
  function clickTrack(e: PointerEvent, axis: "y" | "x") {
    if (!el) return;
    const vertical = axis === "y";
    const bar = vertical ? y : x;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const at = (vertical ? e.clientY - rect.top : e.clientX - rect.left) - INSET;
    const dir = at < bar.pos ? -1 : 1;
    el.scrollBy({ [vertical ? "top" : "left"]: dir * (vertical ? el.clientHeight : el.clientWidth) * 0.9, behavior: "smooth" });
  }

  /** Follow `target`: its scrolling, its size and its children's sizes and content. */
  function attach(target: HTMLElement) {
    detach();
    el = target;
    el.addEventListener("scroll", onScroll, { passive: true });
    resize = new ResizeObserver(update);
    resize.observe(el);
    for (const child of Array.from(el.children)) resize.observe(child);
    mutations = new MutationObserver(update);
    mutations.observe(el, { childList: true, subtree: true, characterData: true });
    update();
  }
  function detach() {
    el?.removeEventListener("scroll", onScroll);
    resize?.disconnect();
    mutations?.disconnect();
    el = null;
  }
  onBeforeUnmount(() => {
    detach();
    clearTimeout(hideTimer);
  });

  return { y, x, active, dragging, attach, detach, update, grabThumb, clickTrack };
}
