<script setup lang="ts">
import { computed } from "vue";
import { store } from "../store";

const m = computed(() => store.config!.modules);
const themes = computed(() => store.info?.codeThemes ?? []);

const simple: { key: keyof typeof m.value; name: string; desc: string; example: string }[] = [
  { key: "callouts", name: "Callouts", desc: "Note, tip, warning, danger, theorem, proof, definition and more. Also GitHub-style alerts.", example: "::: tip" },
  { key: "quotes", name: "Quotes", desc: "Classic, pull, epigraph, card, big, line and shaded quote styles with attribution.", example: "::: quote {style=pull}" },
  { key: "boxes", name: "Boxes & shapes", desc: "Framed, shaded, tinted, filled, dashed and double boxes; rounded, pill, circle, cut and ticket shapes.", example: "::: box {style=shaded}" },
  { key: "layout", name: "Layout", desc: "Columns, tabs and other arrangements.", example: ":::: columns" },
  { key: "media", name: "Media", desc: "Figures with credits, galleries, before/after sliders, audio, video and attachments.", example: "::: figure" },
  { key: "learning", name: "Learning", desc: "Quizzes, exercises with hints and solutions, and flashcards.", example: "::: quiz" },
  { key: "diagrams", name: "Diagrams", desc: "Flowcharts, sequence diagrams, mind maps and timelines with Mermaid. Adds about 3 MB to books that use them.", example: "```mermaid" },
  { key: "charts", name: "Charts", desc: "Bar, line, area, pie, donut and horizontal bar charts from CSV or JSON. Drawn at build time, no extra size.", example: "```chart" },
  { key: "canvas", name: "Canvas drawings", desc: "Shapes, arrows, axes, text and images from a simple drawing language — good for physics and geometry.", example: "```canvas" },
  { key: "footnotes", name: "Footnotes", desc: "Numbered notes collected at the end of each chapter.", example: "[^1]" },
];
</script>

<template>
  <header class="topbar"><h1>Blocks &amp; features</h1><span class="spacer"></span><span class="status">{{ store.saving ? "Saving…" : "Saved automatically" }}</span></header>
  <div class="page">
    <div class="page-inner">
      <div class="page-title">
        <div>
          <h2>Blocks &amp; features</h2>
          <p>Turn block types on or off and set their options. Libraries are only packed into books that use them.</p>
        </div>
      </div>

      <section class="card">
        <h3><input type="checkbox" v-model="m.code.enabled" aria-label="Code blocks" /> Code</h3>
        <p class="card-sub">Syntax highlighting for over 200 languages, done at build time.</p>
        <fieldset :disabled="!m.code.enabled" class="grid">
          <label class="check"><input type="checkbox" v-model="m.code.highlight" /><span><b>Syntax highlighting</b></span></label>
          <label class="check"><input type="checkbox" v-model="m.code.lineNumbers" /><span><b>Line numbers by default</b><small>Override per block with <code>{lines=false}</code>.</small></span></label>
          <label class="check"><input type="checkbox" v-model="m.code.copyButton" /><span><b>Copy button</b></span></label>
          <label class="check"><input type="checkbox" v-model="m.code.wrapLines" /><span><b>Wrap long lines</b></span></label>
          <label class="check"><input type="checkbox" v-model="m.code.runnable" /><span><b>Runnable JavaScript</b><small><code>```js {run=true}</code> runs in a sandbox with no access to the reader’s computer.</small></span></label>
          <label class="field">
            <span>Light theme</span>
            <select class="input" v-model="m.code.lightTheme"><option v-for="th in themes" :key="th">{{ th }}</option></select>
          </label>
          <label class="field">
            <span>Dark theme</span>
            <select class="input" v-model="m.code.darkTheme"><option v-for="th in themes" :key="th">{{ th }}</option></select>
          </label>
        </fieldset>
      </section>

      <section class="card">
        <h3><input type="checkbox" v-model="m.math.enabled" aria-label="Math" /> Math &amp; science</h3>
        <p class="card-sub">LaTeX math with KaTeX: <code>$x^2$</code> inline, <code>$$…$$</code> or <code>```math</code> for display. Adds about 1 MB to books that use it.</p>
        <fieldset :disabled="!m.math.enabled" class="grid">
          <label class="check"><input type="checkbox" v-model="m.math.numberEquations" /><span><b>Number equations with an id</b><small><code>```math {#eq-name}</code> gets (1), (2)…</small></span></label>
          <label class="check"><input type="checkbox" v-model="m.math.chemistry" /><span><b>Chemistry and units</b><small><code>\ce{H2O}</code>, <code>\pu{9.8 m/s^2}</code> and <code>```chem</code> blocks.</small></span></label>
        </fieldset>
      </section>

      <section class="card">
        <h3>Block types</h3>
        <div class="modules">
          <label v-for="s in simple" :key="s.key" class="module">
            <input type="checkbox" v-model="(m[s.key] as any).enabled" />
            <span>
              <b>{{ s.name }}</b>
              <small>{{ s.desc }}</small>
              <code>{{ s.example }}</code>
            </span>
          </label>
        </div>
        <label class="field" v-if="m.quotes.enabled" style="max-width: 320px">
          <span>Style for plain <code>&gt;</code> quotes</span>
          <select class="input" v-model="m.quotes.defaultStyle">
            <option value="classic">Classic (italic with quote mark)</option>
            <option value="line">Line on the left</option>
            <option value="shaded">Shaded</option>
            <option value="minimal">Minimal</option>
          </select>
        </label>
      </section>

      <section class="card">
        <h3>Advanced</h3>
        <label class="check">
          <input type="checkbox" v-model="m.widgets.enabled" />
          <span><b>Interactive widgets</b><small><code>```widget</code> blocks run your own HTML and JavaScript in a sandboxed frame — for simulations and custom interactions. They cannot reach the reader’s files.</small></span>
        </label>
        <label class="check">
          <input type="checkbox" v-model="m.rawHtml.enabled" />
          <span><b>Raw HTML in Markdown</b><small>Off: HTML tags are shown as text. On: they are passed into the page as-is. Only turn on for HTML you wrote yourself.</small></span>
        </label>
      </section>
    </div>
  </div>
</template>

<style scoped>
fieldset { border: 0; padding: 0; margin: 0; }
fieldset:disabled { opacity: 0.5; }
h3 input { width: 16px; height: 16px; accent-color: var(--accent); }
code { font-family: var(--mono); font-size: 12px; background: var(--panel-2); padding: 1px 5px; border-radius: 4px; }
.modules { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px; }
.module { display: flex; gap: 10px; align-items: flex-start; padding: 12px; border: 1px solid var(--line); border-radius: 10px; cursor: pointer; }
.module:has(input:checked) { border-color: color-mix(in srgb, var(--accent) 45%, var(--line)); background: color-mix(in srgb, var(--accent) 5%, var(--panel)); }
.module input { margin-top: 3px; width: 16px; height: 16px; accent-color: var(--accent); flex: none; }
.module span { display: flex; flex-direction: column; gap: 4px; }
.module small { color: var(--muted); font-size: 12px; line-height: 1.45; }
.module code { align-self: flex-start; }
</style>
