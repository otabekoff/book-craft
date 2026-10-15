<script setup lang="ts">
import { computed, onMounted } from "vue";
import { BookOpenText, CaseSensitive, Type } from "@lucide/vue";
import { useFontPreviews } from "../fonts";
import { store } from "../store";
import { UiBadge, UiCard, UiCheckbox, UiField, UiInput, UiRange, UiSegmented, UiSelect, UiSwitch, type SelectOption, UiScrollArea } from "../ui";
import AssetPicker from "./AssetPicker.vue";
import FontPicker from "./FontPicker.vue";

const t = computed(() => store.config!.typography);
const r = computed(() => store.config!.reader);

const fontList = computed(() => store.info?.fonts ?? []);
const stackOf = (key: string): string =>
  key === "same" ? stackOf(t.value.bodyFont) : fontList.value.find((f) => f.key === key)?.stack ?? "inherit";
const readerFontOptions = computed(() => fontList.value.filter((f) => !["mono", "handwriting"].includes(f.category) && f.key !== "custom"));
const bundledKb = computed(() => {
  const keys = new Set([t.value.bodyFont, t.value.headingFont, t.value.uiFont, t.value.codeFont, ...r.value.fontChoices]);
  return fontList.value.filter((f) => f.source === "library" && keys.has(f.key)).reduce((n, f) => n + f.kb, 0);
});
function toggleChoice(key: string, on: boolean) {
  const list = r.value.fontChoices.filter((k) => k !== key);
  if (on) list.push(key);
  r.value.fontChoices = list;
}
onMounted(useFontPreviews);
const paragraphOptions: SelectOption<string>[] = [
  { value: "spaced", label: "Space between paragraphs", description: "Easy to scan; common on screens" },
  { value: "indent", label: "Indented", description: "First-line indent, like a printed novel" },
];
const layoutOptions = [
  { value: "scroll", label: "Scroll" },
  { value: "pages", label: "Pages" },
];
const spreadOptions = [
  { value: "auto", label: "Auto" },
  { value: "one", label: "One" },
  { value: "two", label: "Two" },
];
const turnOptions = [
  { value: "fade", label: "Fade" },
  { value: "slide", label: "Slide" },
  { value: "flip", label: "Flip" },
  { value: "none", label: "Instant" },
];
const widthOptions = [
  { value: "narrow", label: "Narrow" },
  { value: "normal", label: "Normal" },
  { value: "wide", label: "Wide" },
  { value: "full", label: "Full" },
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
  <UiScrollArea class="page-area" viewport-class="page">
    <div class="page-inner">
      <div class="page-title">
        <div>
          <h2>Appearance</h2>
          <p>Typography and the reading experience. Readers can still adjust size and theme if you allow it.</p>
        </div>
      </div>

      <UiCard title="Fonts" :icon="CaseSensitive" description="Bundled fonts look the same on every PC and are packed into the book only when used. Windows fonts add nothing to the size.">
        <template #actions>
          <UiBadge :tone="bundledKb ? 'accent' : 'neutral'">{{ bundledKb ? `+${bundledKb} KB of fonts` : "No bundled fonts" }}</UiBadge>
        </template>
        <div class="grid two">
          <UiField label="Body text" for="f-body">
            <FontPicker id="f-body" v-model="t.bodyFont" :categories="['serif', 'sans', 'display']" />
          </UiField>
          <UiField label="Headings" for="f-head">
            <FontPicker id="f-head" v-model="t.headingFont" :categories="['serif', 'sans', 'display', 'handwriting']" :extra="{ value: 'same', label: 'Same as body text' }" />
          </UiField>
          <UiField label="Labels, captions and controls" for="f-ui" hint="Callout titles, tables, captions and the reader’s own buttons.">
            <FontPicker id="f-ui" v-model="t.uiFont" :categories="['sans']" />
          </UiField>
          <UiField label="Code" for="f-code">
            <FontPicker id="f-code" v-model="t.codeFont" :categories="['mono']" />
          </UiField>
          <UiField v-if="t.bodyFont === 'custom'" label="Custom font file">
            <AssetPicker v-model="t.customFont" :extensions="['woff2', 'woff', 'ttf', 'otf']" subdir="fonts" placeholder="Choose a font file" />
          </UiField>
        </div>
        <div class="specimen" aria-hidden="true">
          <div class="sp-head" :style="{ fontFamily: stackOf(t.headingFont) }">The Harbour at Dawn</div>
          <p class="sp-body" :style="{ fontFamily: stackOf(t.bodyFont) }">The fishing boats came in before the light did — Мара услышала их с чердака. <em>Italic</em>, <b>bold</b> and 0123456789.</p>
          <div class="sp-row">
            <span class="sp-label" :style="{ fontFamily: stackOf(t.uiFont) }">Tip · Figure 1 · Chapter 2 of 12</span>
            <code class="sp-code" :style="{ fontFamily: stackOf(t.codeFont) }">let fish = boats.len();</code>
          </div>
        </div>
      </UiCard>

      <UiCard title="Typography" :icon="Type">
        <div class="grid">
          <UiField label="Paragraphs" for="a-par"><UiSelect id="a-par" v-model="t.paragraphStyle" :options="paragraphOptions" /></UiField>
        </div>
        <div class="grid">
          <UiField label="Text size">
            <UiRange v-model="t.baseSize" :min="14" :max="26" :marks="[19]" aria-label="Text size" :format="(v) => v + ' px'" />
          </UiField>
          <UiField label="Line spacing">
            <UiRange v-model="t.lineHeight" :min="1.3" :max="2.1" :step="0.05" aria-label="Line spacing" :format="(v) => v.toFixed(2)" />
          </UiField>
          <UiField label="Page turn" hint="Flip curls the page like a real book; readers can drag its corner.">
            <UiSegmented v-model="r.pageTurn" :options="turnOptions" aria-label="Page turn" />
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
          <UiField label="Reading layout" hint="Readers can switch any time. Pages turn with the arrow keys, Space or a click near the edge.">
            <UiSegmented v-model="r.defaultLayout" :options="layoutOptions" aria-label="Reading layout" />
          </UiField>
          <UiField label="Pages per screen" hint="Auto shows two pages side by side when the window is wide enough.">
            <UiSegmented v-model="r.pageSpread" :options="spreadOptions" aria-label="Pages per screen" />
          </UiField>
          <UiField label="Text width">
            <UiSegmented v-model="r.defaultWidth" :options="widthOptions" aria-label="Text width" />
          </UiField>
          <UiField label="Page numbers" hint="A number under each page in page mode.">
            <UiSwitch v-model="r.pageNumbers" label="Show page numbers" />
          </UiField>
          <UiField label="Paper look" hint="Text sits on a paper sheet over a darker desk.">
            <UiSwitch v-model="r.paperLook" label="Show the page as paper" />
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
        <UiField v-if="r.allowFontChange" label="Fonts readers can switch to" hint="The book’s own body font is always offered first. Bundled fonts here are packed into the book.">
          <div class="choices">
            <UiCheckbox
              v-for="f in readerFontOptions"
              :key="f.key"
              :model-value="r.fontChoices.includes(f.key)"
              @update:model-value="toggleChoice(f.key, $event)"
            >
              <span class="choice"><span :style="{ fontFamily: f.stack }">{{ f.name }}</span><small>{{ f.kb ? `+${f.kb} KB` : "Windows" }}</small></span>
            </UiCheckbox>
          </div>
        </UiField>
      </UiCard>
    </div>
  </UiScrollArea>
</template>

<style scoped>
.pair { display: flex; align-items: center; gap: 8px; color: var(--muted); }
.specimen { border: 1px solid var(--line); border-radius: 10px; padding: 18px 20px; background: var(--panel-2); display: flex; flex-direction: column; gap: 8px; }
.sp-head { font-size: 24px; font-weight: 650; line-height: 1.2; }
.sp-body { margin: 0; font-size: 17px; line-height: 1.6; }
.sp-row { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px; }
.sp-label { font-size: 13px; font-weight: 600; color: var(--accent); }
.sp-code { font-size: 13px; background: var(--panel); border: 1px solid var(--line); border-radius: 5px; padding: 2px 6px; }
.choices { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 2px 16px; }
.choice { display: flex; align-items: baseline; gap: 8px; }
.choice small { color: var(--muted); font-size: 11px; }
code { font-family: var(--mono); font-size: 11.5px; }
</style>
