<script setup lang="ts">
import { computed } from "vue";
import { BookOpenText, Type } from "@lucide/vue";
import { store } from "../store";
import { UiCard, UiField, UiInput, UiRange, UiSegmented, UiSelect, UiSwitch, type SelectOption } from "../ui";
import AssetPicker from "./AssetPicker.vue";

const t = computed(() => store.config!.typography);
const r = computed(() => store.config!.reader);

const fonts = computed<SelectOption<string>[]>(() => (store.info?.fonts ?? []).map(([value, label]) => ({ value, label })));
const headingFonts = computed<SelectOption<string>[]>(() => [
  { value: "same", label: "Same as body" },
  ...fonts.value.filter((f) => f.value !== "custom"),
]);
const paragraphOptions: SelectOption<string>[] = [
  { value: "spaced", label: "Space between paragraphs", description: "Easy to scan; common on screens" },
  { value: "indent", label: "Indented", description: "First-line indent, like a printed novel" },
];
const titleBarOptions = [
  { value: "custom", label: "Book’s own" },
  { value: "native", label: "Windows" },
];
const themeOptions = [
  { value: "light", label: "Light" },
  { value: "sepia", label: "Sepia" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "Match Windows" },
];

const toggles: { key: keyof typeof r.value; label: string; hint: string }[] = [
  { key: "showHomeScreen", label: "Home screen", hint: "Cover, description and “Continue reading” when the app opens." },
  { key: "showToc", label: "Contents sidebar", hint: "Chapter list beside the text." },
  { key: "showProgress", label: "Reading progress", hint: "Progress bar and percentage." },
  { key: "rememberPosition", label: "Remember the reader’s place", hint: "Reopen at the last page read." },
  { key: "enableSearch", label: "Search", hint: "Full-text search with highlights (Ctrl+F)." },
  { key: "enableBookmarks", label: "Bookmarks", hint: "Readers can save places (Ctrl+D)." },
  { key: "allowThemeChange", label: "Theme choice", hint: "Light, sepia, dark and “Match Windows”." },
  { key: "allowFontSizeChange", label: "Text size control", hint: "Ctrl + / Ctrl − and buttons." },
  { key: "allowFontChange", label: "Typeface choice", hint: "Readers can pick another font." },
  { key: "allowCopy", label: "Allow copying text", hint: "Off discourages copy and paste (code stays copyable)." },
  { key: "allowPrint", label: "Allow printing", hint: "Ctrl+P prints the current chapter." },
  { key: "openLinksExternally", label: "Open web links in the browser", hint: "Otherwise web links are ignored." },
];
</script>

<template>
  <header class="topbar"><h1>Appearance</h1><span class="spacer"></span><span class="status">{{ store.saving ? "Saving…" : "Saved automatically" }}</span></header>
  <div class="page">
    <div class="page-inner">
      <div class="page-title">
        <div>
          <h2>Appearance</h2>
          <p>Typography and the reading experience. Readers can still adjust size and theme if you allow it.</p>
        </div>
      </div>

      <UiCard title="Typography" :icon="Type">
        <div class="grid">
          <UiField label="Body typeface" for="a-body"><UiSelect id="a-body" v-model="t.bodyFont" :options="fonts" /></UiField>
          <UiField label="Heading typeface" for="a-head"><UiSelect id="a-head" v-model="t.headingFont" :options="headingFonts" /></UiField>
          <UiField v-if="t.bodyFont === 'custom'" label="Font file">
            <AssetPicker v-model="t.customFont" :extensions="['woff2', 'woff', 'ttf', 'otf']" subdir="fonts" placeholder="Choose a font file" />
          </UiField>
          <UiField label="Paragraphs" for="a-par"><UiSelect id="a-par" v-model="t.paragraphStyle" :options="paragraphOptions" /></UiField>
        </div>
        <div class="grid">
          <UiField label="Text size">
            <UiRange v-model="t.baseSize" :min="14" :max="26" :marks="[19]" aria-label="Text size" :format="(v) => v + ' px'" />
          </UiField>
          <UiField label="Line spacing">
            <UiRange v-model="t.lineHeight" :min="1.3" :max="2.1" :step="0.05" aria-label="Line spacing" :format="(v) => v.toFixed(2)" />
          </UiField>
          <UiField label="Text width">
            <UiRange v-model="t.maxWidth" :min="520" :max="1000" :step="10" aria-label="Text width" :format="(v) => v + ' px'" />
          </UiField>
        </div>
        <div class="grid">
          <UiField label="Scene break (***)" for="a-scene" hint="Empty = a thin line.">
            <UiInput id="a-scene" v-model="t.sceneBreak" />
          </UiField>
          <UiField label="Custom stylesheet">
            <AssetPicker v-model="t.customCss" :extensions="['css']" subdir="styles" placeholder="None" />
            <template #hint>Style your own <code>::: my-block</code> blocks or override anything (classes start with <code>bc-</code>).</template>
          </UiField>
        </div>
        <div class="grid">
          <UiSwitch v-model="t.dropCaps" label="Drop caps" description="Large first letter in each chapter." />
          <UiSwitch v-model="t.justify" label="Justified text" description="Even right edge." />
          <UiSwitch v-model="t.hyphenate" label="Hyphenation" description="Breaks long words at line ends." />
          <UiSwitch v-model="t.smartQuotes" label="Smart punctuation" description="“Curly” quotes, dashes and ellipses." />
          <UiSwitch v-model="t.headingNumbers" label="Numbered sections" description="1. 1.1 1.2 …" />
        </div>
      </UiCard>

      <UiCard title="Reader" :icon="BookOpenText">
        <div class="grid">
          <UiField label="Default theme">
            <UiSegmented v-model="r.defaultTheme" :options="themeOptions" aria-label="Default theme" />
          </UiField>
          <UiField label="Title bar" hint="The book’s own bar follows its theme and shows the chapter.">
            <UiSegmented v-model="r.titleBar" :options="titleBarOptions" aria-label="Title bar" />
          </UiField>
          <UiField label="Window size">
            <div class="pair">
              <UiInput type="number" v-model="r.windowWidth" :min="400" :max="4000" aria-label="Window width" />
              <span>×</span>
              <UiInput type="number" v-model="r.windowHeight" :min="300" :max="3000" aria-label="Window height" />
            </div>
          </UiField>
          <UiField label="Smallest window">
            <div class="pair">
              <UiInput type="number" v-model="r.minWidth" :min="320" :max="2000" aria-label="Minimum width" />
              <span>×</span>
              <UiInput type="number" v-model="r.minHeight" :min="240" :max="2000" aria-label="Minimum height" />
            </div>
          </UiField>
        </div>
        <div class="grid">
          <UiSwitch v-for="item in toggles" :key="item.key" v-model="(r as any)[item.key]" :label="item.label" :description="item.hint" />
        </div>
      </UiCard>
    </div>
  </div>
</template>

<style scoped>
.pair { display: flex; align-items: center; gap: 8px; color: var(--muted); }
code { font-family: var(--mono); font-size: 11.5px; }
</style>
