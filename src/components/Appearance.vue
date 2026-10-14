<script setup lang="ts">
import { computed } from "vue";
import { store } from "../store";
import AssetPicker from "./AssetPicker.vue";

const t = computed(() => store.config!.typography);
const r = computed(() => store.config!.reader);
const fonts = computed(() => store.info?.fonts ?? []);

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
  { key: "allowCopy", label: "Allow copying text", hint: "Turn off to discourage copy and paste (code blocks stay copyable)." },
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

      <section class="card">
        <h3>Typography</h3>
        <div class="grid">
          <label class="field">
            <span>Body typeface</span>
            <select class="input" v-model="t.bodyFont">
              <option v-for="[k, label] in fonts" :key="k" :value="k">{{ label }}</option>
            </select>
          </label>
          <label class="field">
            <span>Heading typeface</span>
            <select class="input" v-model="t.headingFont">
              <option value="same">Same as body</option>
              <option v-for="[k, label] in fonts.filter(([k]) => k !== 'custom')" :key="k" :value="k">{{ label }}</option>
            </select>
          </label>
          <div class="field" v-if="t.bodyFont === 'custom'">
            <span>Font file</span>
            <AssetPicker v-model="t.customFont" :extensions="['woff2', 'woff', 'ttf', 'otf']" subdir="fonts" placeholder="Choose a font file" />
          </div>
          <label class="field">
            <span>Text size — {{ t.baseSize }} px</span>
            <input type="range" min="14" max="26" v-model.number="t.baseSize" />
          </label>
          <label class="field">
            <span>Line spacing — {{ t.lineHeight.toFixed(2) }}</span>
            <input type="range" min="1.3" max="2.1" step="0.05" v-model.number="t.lineHeight" />
          </label>
          <label class="field">
            <span>Text width — {{ t.maxWidth }} px</span>
            <input type="range" min="520" max="1000" step="10" v-model.number="t.maxWidth" />
          </label>
          <label class="field">
            <span>Paragraphs</span>
            <select class="input" v-model="t.paragraphStyle">
              <option value="spaced">Space between paragraphs</option>
              <option value="indent">Indented, like a printed book</option>
            </select>
          </label>
          <label class="field">
            <span>Scene break (<code>***</code>)</span>
            <input class="input" v-model="t.sceneBreak" placeholder="Empty = a thin line" />
          </label>
          <div class="field">
            <span>Custom stylesheet</span>
            <AssetPicker v-model="t.customCss" :extensions="['css']" subdir="styles" placeholder="None" />
            <small>Style your own <code>::: my-block</code> blocks or override anything (classes start with <code>bc-</code>).</small>
          </div>
        </div>
        <div class="grid">
          <label class="check"><input type="checkbox" v-model="t.dropCaps" /><span><b>Drop caps</b><small>Large first letter in each chapter.</small></span></label>
          <label class="check"><input type="checkbox" v-model="t.justify" /><span><b>Justified text</b><small>Even right edge.</small></span></label>
          <label class="check"><input type="checkbox" v-model="t.hyphenate" /><span><b>Hyphenation</b><small>Breaks long words at line ends.</small></span></label>
          <label class="check"><input type="checkbox" v-model="t.smartQuotes" /><span><b>Smart punctuation</b><small>“Curly” quotes, en and em dashes, ellipses.</small></span></label>
          <label class="check"><input type="checkbox" v-model="t.headingNumbers" /><span><b>Numbered sections</b><small>1. 1.1 1.2 …</small></span></label>
        </div>
      </section>

      <section class="card">
        <h3>Reader</h3>
        <div class="grid">
          <div class="field">
            <span>Default theme</span>
            <div class="seg">
              <button v-for="th in ['light', 'sepia', 'dark', 'system']" :key="th" type="button" :aria-pressed="r.defaultTheme === th" @click="r.defaultTheme = th">
                {{ th === "system" ? "Match Windows" : th[0].toUpperCase() + th.slice(1) }}
              </button>
            </div>
          </div>
          <label class="field">
            <span>Window size</span>
            <div class="input-row">
              <input class="input" type="number" min="400" max="4000" v-model.number="r.windowWidth" aria-label="Window width" />
              ×
              <input class="input" type="number" min="300" max="3000" v-model.number="r.windowHeight" aria-label="Window height" />
            </div>
          </label>
          <label class="field">
            <span>Smallest window</span>
            <div class="input-row">
              <input class="input" type="number" min="320" max="2000" v-model.number="r.minWidth" aria-label="Minimum width" />
              ×
              <input class="input" type="number" min="240" max="2000" v-model.number="r.minHeight" aria-label="Minimum height" />
            </div>
          </label>
        </div>
        <div class="grid">
          <label v-for="item in toggles" :key="item.key" class="check">
            <input type="checkbox" v-model="(r as any)[item.key]" />
            <span><b>{{ item.label }}</b><small>{{ item.hint }}</small></span>
          </label>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
code { font-family: var(--mono); font-size: 12px; }
input[type="range"] { accent-color: var(--accent); }
</style>
