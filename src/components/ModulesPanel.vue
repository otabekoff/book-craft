<script setup lang="ts">
import { computed, type Component } from "vue";
import {
  AppWindow, BarChart3, Boxes, Brush, Code, Columns3, FileCode, GraduationCap, Images, Megaphone, MessageSquareQuote,
  Sigma, StickyNote, Workflow,
} from "@lucide/vue";
import { store } from "../store";
import { UiCard, UiField, UiSelect, UiSwitch, type SelectOption } from "../ui";

const m = computed(() => store.config!.modules);
const themes = computed<SelectOption<string>[]>(() => (store.info?.codeThemes ?? []).map((t) => ({ value: t, label: t })));

const simple: { key: keyof typeof m.value; name: string; desc: string; example: string; icon: Component }[] = [
  { key: "callouts", name: "Callouts", desc: "Note, tip, warning, danger, theorem, proof, definition and more. Also GitHub-style alerts.", example: "::: tip", icon: Megaphone },
  { key: "quotes", name: "Quotes", desc: "Classic, pull, epigraph, card, big, line and shaded quote styles with attribution.", example: "::: quote {style=pull}", icon: MessageSquareQuote },
  { key: "boxes", name: "Boxes & shapes", desc: "Framed, shaded, tinted, filled, dashed and double boxes; rounded, pill, circle, cut and ticket shapes.", example: "::: box {style=shaded}", icon: Boxes },
  { key: "layout", name: "Layout", desc: "Columns, tabs and other arrangements.", example: ":::: columns", icon: Columns3 },
  { key: "media", name: "Media", desc: "Figures with credits, galleries, before/after sliders, audio, video and attachments.", example: "::: figure", icon: Images },
  { key: "learning", name: "Learning", desc: "Quizzes, exercises with hints and solutions, and flashcards.", example: "::: quiz", icon: GraduationCap },
  { key: "diagrams", name: "Diagrams", desc: "Flowcharts, sequence diagrams, mind maps and timelines with Mermaid. Adds about 3 MB to books that use them.", example: "```mermaid", icon: Workflow },
  { key: "charts", name: "Charts", desc: "Bar, line, area, pie, donut and horizontal bar charts from CSV or JSON. Drawn at build time.", example: "```chart", icon: BarChart3 },
  { key: "canvas", name: "Canvas drawings", desc: "Shapes, arrows, axes, text and images from a simple drawing language.", example: "```canvas", icon: Brush },
  { key: "footnotes", name: "Footnotes", desc: "Numbered notes collected at the end of each chapter.", example: "[^1]", icon: StickyNote },
];

const quoteStyles: SelectOption<string>[] = [
  { value: "classic", label: "Classic", description: "Italic with a quote mark" },
  { value: "line", label: "Line on the left" },
  { value: "shaded", label: "Shaded" },
  { value: "minimal", label: "Minimal" },
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

      <UiCard title="Code" :icon="Code" description="Syntax highlighting for over 200 languages, done at build time." v-model:enabled="m.code.enabled">
        <div class="grid">
          <UiSwitch v-model="m.code.highlight" label="Syntax highlighting" />
          <UiSwitch v-model="m.code.lineNumbers" label="Line numbers by default" description="Override per block with {lines=false}." />
          <UiSwitch v-model="m.code.copyButton" label="Copy button" />
          <UiSwitch v-model="m.code.wrapLines" label="Wrap long lines" />
          <UiSwitch v-model="m.code.runnable" label="Runnable JavaScript" description="```js {run=true} runs in a sandbox with no access to the reader’s computer." />
        </div>
        <div class="grid">
          <UiField label="Light theme" for="c-light"><UiSelect id="c-light" v-model="m.code.lightTheme" :options="themes" /></UiField>
          <UiField label="Dark theme" for="c-dark"><UiSelect id="c-dark" v-model="m.code.darkTheme" :options="themes" /></UiField>
        </div>
      </UiCard>

      <UiCard title="Math & science" :icon="Sigma" v-model:enabled="m.math.enabled" description="LaTeX math with KaTeX: $x^2$ inline, $$…$$ or ```math for display. Adds about 1 MB to books that use it.">
        <div class="grid">
          <UiSwitch v-model="m.math.numberEquations" label="Number equations with an id" description="```math {#eq-name} gets (1), (2)…" />
          <UiSwitch v-model="m.math.chemistry" label="Chemistry and units" description="\ce{H2O}, \pu{9.8 m/s^2} and ```chem blocks." />
        </div>
      </UiCard>

      <UiCard title="Block types">
        <div class="modules">
          <div v-for="s in simple" :key="s.key" class="module" :class="{ on: (m[s.key] as any).enabled }">
            <component :is="s.icon" class="m-ico" aria-hidden="true" />
            <div class="m-body">
              <b>{{ s.name }}</b>
              <small>{{ s.desc }}</small>
              <code>{{ s.example }}</code>
            </div>
            <UiSwitch v-model="(m[s.key] as any).enabled" :aria-label="s.name" />
          </div>
        </div>
        <UiField v-if="m.quotes.enabled" label="Style for plain > quotes" for="q-style" style="max-width: 320px">
          <UiSelect id="q-style" v-model="m.quotes.defaultStyle" :options="quoteStyles" />
        </UiField>
      </UiCard>

      <UiCard title="Advanced" :icon="FileCode">
        <UiSwitch v-model="m.widgets.enabled" label="Interactive widgets" description="```widget blocks run your own HTML and JavaScript in a sandboxed frame — for simulations and custom interactions. They cannot reach the reader’s files." />
        <UiSwitch v-model="m.rawHtml.enabled" label="Raw HTML in Markdown" description="Off: HTML tags are shown as text. On: they are passed into the page as-is. Only turn on for HTML you wrote yourself." />
      </UiCard>
      <p class="foot"><AppWindow class="f-ico" aria-hidden="true" /> Changes apply to the preview immediately and to the app on the next build.</p>
    </div>
  </div>
</template>

<style scoped>
.modules { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 10px; }
.module { display: flex; gap: 12px; align-items: flex-start; padding: 12px; border: 1px solid var(--line); border-radius: 10px; transition: border-color 0.15s, background 0.15s; }
.module.on { border-color: color-mix(in srgb, var(--accent) 40%, var(--line)); background: color-mix(in srgb, var(--accent) 5%, var(--panel)); }
.m-ico { width: 20px; height: 20px; color: var(--muted); flex: none; margin-top: 1px; }
.module.on .m-ico { color: var(--accent); }
.m-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
.m-body small { color: var(--muted); font-size: 12px; line-height: 1.45; }
.m-body code { align-self: flex-start; font-family: var(--mono); font-size: 11.5px; background: var(--panel-2); padding: 1px 6px; border-radius: 4px; }
.foot { display: flex; align-items: center; gap: 8px; color: var(--muted); font-size: 12px; margin: 0; }
.f-ico { width: 16px; height: 16px; }
</style>
