<script setup lang="ts">
import { computed } from "vue";
import { store } from "../store";
import AssetPicker from "./AssetPicker.vue";

const meta = computed(() => store.config!.meta);
const build = computed(() => store.config!.build);
const exeName = computed(() => (meta.value.executableName.trim() || build.value.productName.trim() || meta.value.title.trim() || "Book") + ".exe");

const languages = [
  ["en-US", "English (US)"],
  ["en-GB", "English (UK)"],
  ["uz-UZ", "Oʻzbekcha"],
  ["uz-Cyrl-UZ", "Ўзбекча"],
  ["ru-RU", "Русский"],
  ["tr-TR", "Türkçe"],
  ["de-DE", "Deutsch"],
  ["fr-FR", "Français"],
  ["es-ES", "Español"],
  ["it-IT", "Italiano"],
  ["pt-BR", "Português (Brasil)"],
  ["ar-SA", "العربية"],
  ["zh-CN", "中文 (简体)"],
  ["ja-JP", "日本語"],
  ["ko-KR", "한국어"],
  ["hi-IN", "हिन्दी"],
  ["kk-KZ", "Қазақша"],
];

function suggestId() {
  const part = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, "").replace(/^[0-9]+/, "").slice(0, 20);
  const pub = part(meta.value.publisher) || "bookcraft";
  const book = part(meta.value.title) || "book";
  meta.value.packageId = `com.${pub}.${book}`;
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

      <div v-if="store.problems.length" class="notice warn">
        <b>Before you build</b>
        <ul><li v-for="p in store.problems" :key="p">{{ p }}</li></ul>
      </div>

      <section class="card">
        <h3>About the book</h3>
        <div class="grid two">
          <label class="field"><span>Title</span><input class="input" v-model="meta.title" /></label>
          <label class="field"><span>Subtitle</span><input class="input" v-model="meta.subtitle" /></label>
          <label class="field"><span>Author</span><input class="input" v-model="meta.author" /></label>
          <label class="field"><span>Publisher</span><input class="input" v-model="meta.publisher" placeholder="Shown as the company in Windows" /></label>
          <label class="field">
            <span>Language</span>
            <select class="input" v-model="meta.language">
              <option v-for="[code, name] in languages" :key="code" :value="code">{{ name }} — {{ code }}</option>
              <option v-if="!languages.some(([c]) => c === meta.language)" :value="meta.language">{{ meta.language }}</option>
            </select>
          </label>
          <label class="field"><span>ISBN</span><input class="input" v-model="meta.isbn" placeholder="Optional" /></label>
          <label class="field"><span>Website</span><input class="input" v-model="meta.website" placeholder="https://" /></label>
          <label class="field"><span>Copyright line</span><input class="input" v-model="meta.copyright" placeholder="© 2026 Your Name" /></label>
        </div>
        <label class="field">
          <span>Description</span>
          <textarea class="input" v-model="meta.description" rows="3" placeholder="Shown on the book's home screen and in the MSIX package."></textarea>
        </label>
        <label class="field">
          <span>Rights statement</span>
          <textarea class="input" v-model="meta.rights" rows="2" placeholder="All rights reserved. No part of this book may be reproduced…"></textarea>
        </label>
      </section>

      <section class="card">
        <h3>Look</h3>
        <div class="grid two">
          <div class="field">
            <span>App icon</span>
            <AssetPicker v-model="meta.icon" :extensions="['png', 'jpg', 'jpeg', 'webp', 'ico']" placeholder="Generated icon" preview>
              <small>A square PNG of 512×512 or larger works best. Every size Windows needs is made from it.</small>
            </AssetPicker>
          </div>
          <div class="field">
            <span>Cover image</span>
            <AssetPicker v-model="meta.cover" kind="image" placeholder="Typographic cover" preview>
              <small>Portrait, about 2:3. Without one, a cover is drawn from the title and accent color.</small>
            </AssetPicker>
          </div>
          <label class="field">
            <span>Accent color</span>
            <div class="input-row">
              <input type="color" class="color-input" v-model="meta.accent" />
              <input class="input mono" v-model="meta.accent" />
            </div>
            <small>Used for links, buttons, the cover and highlights.</small>
          </label>
        </div>
      </section>

      <section class="card">
        <h3>App identity</h3>
        <p class="card-sub">Windows uses these to tell your book apart from other apps, to upgrade it, and to match it to your signing certificate.</p>
        <div class="grid two">
          <label class="field">
            <span>Package ID</span>
            <div class="input-row">
              <input class="input mono" v-model="meta.packageId" placeholder="com.publisher.book" />
              <button type="button" class="btn small" @click="suggestId">Suggest</button>
            </div>
            <small>Reverse-domain style. Keep it the same for every edition so updates replace the old version.</small>
          </label>
          <label class="field">
            <span>Version</span>
            <input class="input mono" v-model="meta.version" placeholder="1.0.0.0" />
            <small>Up to four numbers. Raise it for every new edition.</small>
          </label>
          <label class="field">
            <span>App name in Windows</span>
            <input class="input" v-model="build.productName" :placeholder="meta.title" />
            <small>Start menu, Add/Remove Programs and window title. Defaults to the book title.</small>
          </label>
          <label class="field">
            <span>Executable file name</span>
            <input class="input" v-model="meta.executableName" :placeholder="exeName.replace(/\.exe$/, '')" />
            <small>Result: <b>{{ exeName }}</b></small>
          </label>
          <label class="field">
            <span>Publisher ID (for MSIX)</span>
            <div class="input-row">
              <input class="input mono" v-model="meta.publisherId" placeholder="CN=Your Publisher" />
              <button type="button" class="btn small" @click="suggestPublisherId">Suggest</button>
            </div>
            <small>Must exactly match the subject of the certificate you sign the MSIX with (or the one Partner Center gives you).</small>
          </label>
        </div>
      </section>
    </div>
  </div>
</template>
