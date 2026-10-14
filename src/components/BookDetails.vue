<script setup lang="ts">
import { computed } from "vue";
import { BookText, FileCog, Fingerprint, Palette, TriangleAlert, Wand } from "@lucide/vue";
import { api, errorText } from "../api";
import { applyProject, notify, saveConfigNow, store } from "../store";
import { UiButton, UiCard, UiColor, UiField, UiInput, UiSegmented, UiSelect, type SelectOption } from "../ui";
import AssetPicker from "./AssetPicker.vue";

const meta = computed(() => store.config!.meta);
const build = computed(() => store.config!.build);
const exeName = computed(() => (meta.value.executableName.trim() || build.value.productName.trim() || meta.value.title.trim() || "Book") + ".exe");

const languages: SelectOption<string>[] = [
  ["en-US", "English (US)"],
  ["en-GB", "English (UK)"],
  ["uz-UZ", "Oʻzbekcha"],
  ["uz-Cyrl-UZ", "Ўзбекча"],
  ["ru-RU", "Русский"],
  ["tr-TR", "Türkçe"],
  ["kk-KZ", "Қазақша"],
  ["de-DE", "Deutsch"],
  ["fr-FR", "Français"],
  ["es-ES", "Español"],
  ["it-IT", "Italiano"],
  ["pt-BR", "Português (Brasil)"],
  ["ar-SA", "العربية"],
  ["hi-IN", "हिन्दी"],
  ["zh-CN", "中文 (简体)"],
  ["ja-JP", "日本語"],
  ["ko-KR", "한국어"],
].map(([value, label]) => ({ value, label, description: value }));

const languageOptions = computed(() =>
  languages.some((l) => l.value === meta.value.language)
    ? languages
    : [...languages, { value: meta.value.language, label: meta.value.language }]
);

function suggestId() {
  const part = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, "").replace(/^[0-9]+/, "").slice(0, 20);
  meta.value.packageId = `com.${part(meta.value.publisher) || "bookcraft"}.${part(meta.value.title) || "book"}`;
}

const formatOptions = [
  { value: "json" as const, label: "book.json" },
  { value: "toml" as const, label: "book.toml" },
];
async function setFormat(f: "json" | "toml") {
  if (f === store.format) return;
  try {
    await saveConfigNow();
    applyProject(await api.setFormat(f));
    notify(`Settings are now stored in book.${f}.`);
  } catch (e) {
    notify(errorText(e), "error");
  }
}

function suggestPublisherId() {
  meta.value.publisherId = "CN=" + (meta.value.publisher.trim() || meta.value.author.trim() || "Publisher");
}
</script>

<template>
  <header class="topbar"><h1>Book details</h1><span class="spacer"></span><span class="status">{{ store.saving ? "Saving…" : "Saved automatically" }}</span></header>
  <div class="page">
    <div class="page-inner">
      <div class="page-title">
        <div>
          <h2>Book details</h2>
          <p>These go into the app itself: its window, icon, version information and installers.</p>
        </div>
      </div>

      <div v-if="store.problems.length" class="notice warn problems">
        <TriangleAlert class="w-ico" aria-hidden="true" />
        <div><b>Before you build</b><ul><li v-for="p in store.problems" :key="p">{{ p }}</li></ul></div>
      </div>

      <UiCard title="About the book" :icon="BookText">
        <div class="grid two">
          <UiField label="Title" for="m-title"><UiInput id="m-title" v-model="meta.title" /></UiField>
          <UiField label="Subtitle" for="m-sub"><UiInput id="m-sub" v-model="meta.subtitle" /></UiField>
          <UiField label="Author" for="m-author"><UiInput id="m-author" v-model="meta.author" /></UiField>
          <UiField label="Publisher" for="m-pub" hint="Shown as the company in Windows."><UiInput id="m-pub" v-model="meta.publisher" /></UiField>
          <UiField label="Language" for="m-lang"><UiSelect id="m-lang" v-model="meta.language" :options="languageOptions" /></UiField>
          <UiField label="ISBN" for="m-isbn"><UiInput id="m-isbn" v-model="meta.isbn" placeholder="Optional" /></UiField>
          <UiField label="Website" for="m-web"><UiInput id="m-web" v-model="meta.website" placeholder="https://" /></UiField>
          <UiField label="Copyright line" for="m-copy"><UiInput id="m-copy" v-model="meta.copyright" placeholder="© 2026 Your Name" /></UiField>
        </div>
        <UiField label="Description" for="m-desc" hint="Shown on the book's home screen and in the MSIX package.">
          <UiInput id="m-desc" v-model="meta.description" multiline :rows="3" />
        </UiField>
        <UiField label="Rights statement" for="m-rights">
          <UiInput id="m-rights" v-model="meta.rights" multiline :rows="2" placeholder="All rights reserved. No part of this book may be reproduced…" />
        </UiField>
      </UiCard>

      <UiCard title="Look" :icon="Palette">
        <div class="grid two">
          <UiField label="App icon">
            <AssetPicker v-model="meta.icon" :extensions="['png', 'jpg', 'jpeg', 'webp', 'ico']" placeholder="Generated icon" preview>
              <small>A square PNG of 512×512 or larger works best. Every size Windows needs is made from it.</small>
            </AssetPicker>
          </UiField>
          <UiField label="Cover image">
            <AssetPicker v-model="meta.cover" kind="image" placeholder="Typographic cover" preview>
              <small>Portrait, about 2:3. Without one, a cover is drawn from the title and accent color.</small>
            </AssetPicker>
          </UiField>
          <UiField label="Accent color" hint="Links, buttons, the cover and highlights.">
            <UiColor v-model="meta.accent" />
          </UiField>
        </div>
      </UiCard>

      <UiCard title="App identity" :icon="Fingerprint" description="Windows uses these to tell your book apart from other apps, to upgrade it, and to match it to your signing certificate.">
        <div class="grid two">
          <UiField label="Package ID" for="m-id" hint="Reverse-domain style. Keep it the same for every edition so updates replace the old version.">
            <div class="row">
              <UiInput id="m-id" v-model="meta.packageId" mono placeholder="com.publisher.book" />
              <UiButton size="sm" :icon="Wand" @click="suggestId">Suggest</UiButton>
            </div>
          </UiField>
          <UiField label="Version" for="m-ver" hint="Up to four numbers. Raise it for every new edition.">
            <UiInput id="m-ver" v-model="meta.version" mono placeholder="1.0.0.0" />
          </UiField>
          <UiField label="App name in Windows" for="m-prod" hint="Start menu, Add/Remove Programs and window title. Defaults to the book title.">
            <UiInput id="m-prod" v-model="build.productName" :placeholder="meta.title" />
          </UiField>
          <UiField label="Executable file name" for="m-exe">
            <UiInput id="m-exe" v-model="meta.executableName" :placeholder="exeName.replace(/\.exe$/, '')" />
            <template #hint>Result: <b>{{ exeName }}</b></template>
          </UiField>
          <UiField label="Publisher ID (for MSIX)" for="m-pid" hint="Must exactly match the subject of the certificate you sign the MSIX with (or the one Partner Center gives you).">
            <div class="row">
              <UiInput id="m-pid" v-model="meta.publisherId" mono placeholder="CN=Your Publisher" />
              <UiButton size="sm" :icon="Wand" @click="suggestPublisherId">Suggest</UiButton>
            </div>
          </UiField>
        </div>
      </UiCard>
      <UiCard title="Settings file" :icon="FileCog" description="Where this book’s settings are stored inside its folder. Both hold the same data; TOML is easier to read and edit by hand.">
        <UiSegmented :model-value="store.format" :options="formatOptions" aria-label="Settings file format" @update:model-value="setFormat($event as any)" />
      </UiCard>
    </div>
  </div>
</template>


<style scoped>
.row { display: flex; gap: 8px; align-items: center; }
.row > :first-child { flex: 1; min-width: 0; }
.problems { display: flex; gap: 10px; }
.w-ico { width: 18px; height: 18px; flex: none; color: var(--warn); margin-top: 1px; }
</style>
