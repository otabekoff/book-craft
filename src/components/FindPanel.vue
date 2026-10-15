<script setup lang="ts">
// Find & replace for the chapter editor, shown as CodeMirror's search panel.
// One compact row (field with match-case / regex / whole-word toggles, match count,
// previous / next, select all, close) and a Replace row that folds out (Ctrl+H).
import { computed, nextTick, onMounted, ref, watch } from "vue";
import type { EditorView, ViewUpdate } from "@codemirror/view";
import {
  SearchQuery, closeSearchPanel, findNext, findPrevious, getSearchQuery, replaceAll, replaceNext, selectMatches, setSearchQuery,
} from "@codemirror/search";
import { ArrowDown, ArrowUp, CaseSensitive, ChevronRight, Regex, Replace, ReplaceAll, SquareDashedMousePointer, WholeWord, X } from "@lucide/vue";

const props = defineProps<{ view: EditorView; replace: boolean }>();

const start = getSearchQuery(props.view.state);
const find = ref(start.search);
const replaceText = ref(start.replace);
const matchCase = ref(start.caseSensitive);
const regexp = ref(start.regexp);
const wholeWord = ref(start.wholeWord);
const showReplace = ref(props.replace);
const count = ref({ current: 0, total: 0, valid: true });
const findField = ref<HTMLInputElement>();
const replaceField = ref<HTMLInputElement>();

const MAX = 9999;
const status = computed(() => {
  if (!find.value) return "";
  if (!count.value.valid) return "Invalid regex";
  if (!count.value.total) return "No results";
  const total = count.value.total >= MAX ? MAX + "+" : String(count.value.total);
  return count.value.current ? `${count.value.current} of ${total}` : `${total} found`;
});

function commit() {
  const query = new SearchQuery({
    search: find.value,
    replace: replaceText.value,
    caseSensitive: matchCase.value,
    regexp: regexp.value,
    wholeWord: wholeWord.value,
  });
  if (!query.eq(getSearchQuery(props.view.state))) props.view.dispatch({ effects: setSearchQuery.of(query) });
  recount();
}

/** Counts matches and finds which one is selected. */
function recount() {
  const state = props.view.state;
  const query = getSearchQuery(state);
  if (!query.search) return (count.value = { current: 0, total: 0, valid: true });
  if (!query.valid) return (count.value = { current: 0, total: 0, valid: false });
  const sel = state.selection.main;
  const cursor = query.getCursor(state);
  let total = 0;
  let current = 0;
  for (let r = cursor.next(); !r.done && total < MAX; r = cursor.next()) {
    total++;
    if (r.value.from === sel.from && r.value.to === sel.to) current = total;
  }
  count.value = { current, total, valid: true };
}

function toggle(which: "case" | "re" | "word") {
  if (which === "case") matchCase.value = !matchCase.value;
  if (which === "re") regexp.value = !regexp.value;
  if (which === "word") wholeWord.value = !wholeWord.value;
  commit();
  findField.value?.focus();
}
function run(command: (v: EditorView) => boolean) {
  commit();
  command(props.view);
  recount();
}
function close() {
  closeSearchPanel(props.view);
  props.view.focus();
}
/** Ctrl+F / Ctrl+H also work while typing in the panel (it sits outside the editor's text). */
function onPanelKey(e: KeyboardEvent) {
  if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
  const k = e.key.toLowerCase();
  if (k !== "f" && k !== "h") return;
  e.preventDefault();
  e.stopPropagation();
  focus(k === "h");
}
function onFindKey(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    run(e.shiftKey ? findPrevious : findNext);
  } else if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}
function onReplaceKey(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    run(e.ctrlKey || e.metaKey ? replaceAll : replaceNext);
  } else if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}
async function toggleReplace() {
  showReplace.value = !showReplace.value;
  await nextTick();
  (showReplace.value ? replaceField : findField).value?.focus();
}

/** Called by the editor on every update: follow outside query changes and the selection. */
function sync(update: ViewUpdate) {
  const q = getSearchQuery(update.state);
  if (q.search !== find.value || q.replace !== replaceText.value || q.caseSensitive !== matchCase.value || q.regexp !== regexp.value || q.wholeWord !== wholeWord.value) {
    find.value = q.search;
    replaceText.value = q.replace;
    matchCase.value = q.caseSensitive;
    regexp.value = q.regexp;
    wholeWord.value = q.wholeWord;
  }
  if (update.docChanged || update.selectionSet || update.transactions.some((t) => t.effects.some((e) => e.is(setSearchQuery)))) recount();
}
/** Focus the field the user asked for (Ctrl+F → find, Ctrl+H → replace). */
async function focus(replace: boolean) {
  if (replace) showReplace.value = true;
  await nextTick();
  const field = replace ? replaceField.value : findField.value;
  field?.focus();
  field?.select();
}
// Typing in either field updates the search (after v-model has stored the new text).
watch([find, replaceText], commit);
defineExpose({ sync, focus });
onMounted(recount);
</script>

<template>
  <div class="find" role="search" aria-label="Find and replace" @keydown="onPanelKey">
    <button type="button" class="find-fold" :class="{ 'find-open': showReplace }" :aria-expanded="showReplace" :title="showReplace ? 'Hide replace' : 'Show replace (Ctrl+H)'" @click="toggleReplace">
      <ChevronRight class="ico" aria-hidden="true" />
    </button>
    <div class="find-rows">
      <div class="find-row">
        <div class="find-field" :class="{ 'find-bad': find && !count.valid }">
          <input ref="findField" v-model="find" type="text" placeholder="Find" aria-label="Find" main-field="true" spellcheck="false" @keydown="onFindKey" />
          <button type="button" class="find-opt" :aria-pressed="matchCase" title="Match case" @click="toggle('case')"><CaseSensitive class="ico" aria-hidden="true" /></button>
          <button type="button" class="find-opt" :aria-pressed="wholeWord" title="Whole word" @click="toggle('word')"><WholeWord class="ico" aria-hidden="true" /></button>
          <button type="button" class="find-opt" :aria-pressed="regexp" title="Regular expression" @click="toggle('re')"><Regex class="ico" aria-hidden="true" /></button>
        </div>
        <span class="find-count" :class="{ 'find-none': find && !count.total }" aria-live="polite">{{ status }}</span>
        <div class="find-actions">
          <button type="button" class="find-act" title="Previous (Shift+Enter)" :disabled="!count.total" @click="run(findPrevious)"><ArrowUp class="ico" aria-hidden="true" /></button>
          <button type="button" class="find-act" title="Next (Enter)" :disabled="!count.total" @click="run(findNext)"><ArrowDown class="ico" aria-hidden="true" /></button>
          <button type="button" class="find-act" title="Select all matches" :disabled="!count.total" @click="run(selectMatches)"><SquareDashedMousePointer class="ico" aria-hidden="true" /></button>
        </div>
      </div>
      <div v-if="showReplace" class="find-row">
        <div class="find-field">
          <input ref="replaceField" v-model="replaceText" type="text" placeholder="Replace" aria-label="Replace with" spellcheck="false" @keydown="onReplaceKey" />
        </div>
        <div class="find-actions">
          <button type="button" class="find-act" title="Replace (Enter)" :disabled="!count.total" @click="run(replaceNext)"><Replace class="ico" aria-hidden="true" /></button>
          <button type="button" class="find-act" title="Replace all (Ctrl+Enter)" :disabled="!count.total" @click="run(replaceAll)"><ReplaceAll class="ico" aria-hidden="true" /></button>
        </div>
      </div>
    </div>
    <button type="button" class="find-act find-close" title="Close (Esc)" @click="close"><X class="ico" aria-hidden="true" /></button>
  </div>
</template>

<style scoped>
.find { display: flex; align-items: flex-start; gap: 6px; padding: 8px 10px 8px 6px; font-family: var(--font); font-size: 13px; color: var(--ink); }
.find-rows { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.find-row { display: flex; align-items: center; gap: 8px; min-width: 0; }
.ico { width: 16px; height: 16px; flex: none; }
.find-fold, .find-act, .find-opt { display: grid; place-items: center; border: 0; background: none; color: var(--muted); cursor: pointer; padding: 0; }
.find-fold { width: 24px; height: 30px; border-radius: 6px; }
.find-fold .ico { transition: transform 0.15s; }
.find-fold.find-open .ico { transform: rotate(90deg); }
.find-fold:hover, .find-act:hover:not(:disabled) { background: var(--panel-2); color: var(--ink); }
.find-field {
  flex: 0 1 360px;
  min-width: 140px;
  display: flex;
  align-items: center;
  gap: 2px;
  height: 30px;
  padding: 0 3px 0 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--panel-2);
}
.find-field:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.find-field.find-bad { border-color: var(--danger); }
.find-field input { flex: 1; min-width: 0; height: 100%; border: 0; background: none; color: var(--ink); font: inherit; outline: none; }
.find-field input::placeholder { color: var(--muted); }
.find-opt { width: 24px; height: 22px; border-radius: 5px; }
.find-opt:hover { background: color-mix(in srgb, var(--ink) 8%, transparent); color: var(--ink); }
.find-opt[aria-pressed="true"] { background: var(--accent-soft); color: var(--accent); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 55%, transparent); }
.find-count { min-width: 74px; font-size: 12px; color: var(--muted); white-space: nowrap; font-variant-numeric: tabular-nums; }
.find-count.find-none { color: var(--danger); }
.find-actions { display: flex; gap: 2px; }
.find-act { width: 30px; height: 30px; border-radius: 7px; }
.find-act:disabled { opacity: 0.35; cursor: default; }
.find-close { margin-left: auto; }
</style>
