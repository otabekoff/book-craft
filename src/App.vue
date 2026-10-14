<script setup lang="ts">
import { onMounted } from "vue";
import { api, errorText } from "./api";
import { notify, store } from "./store";
import AppBar from "./components/AppBar.vue";
import Welcome from "./components/Welcome.vue";
import Sidebar from "./components/Sidebar.vue";
import ChapterEditor from "./components/ChapterEditor.vue";
import BookDetails from "./components/BookDetails.vue";
import Appearance from "./components/Appearance.vue";
import ModulesPanel from "./components/ModulesPanel.vue";
import AssetsPanel from "./components/AssetsPanel.vue";
import BuildPanel from "./components/BuildPanel.vue";

onMounted(async () => {
  try {
    store.info = await api.appInfo();
  } catch (e) {
    notify(errorText(e), "error");
  }
});
</script>

<template>
  <div class="window">
    <AppBar />
    <div class="window-body">
      <Welcome v-if="!store.root || !store.config" />
      <div v-else class="app-shell">
        <Sidebar />
        <main class="main">
          <ChapterEditor v-if="store.section === 'write'" />
          <BookDetails v-else-if="store.section === 'details'" />
          <Appearance v-else-if="store.section === 'appearance'" />
          <ModulesPanel v-else-if="store.section === 'modules'" />
          <AssetsPanel v-else-if="store.section === 'assets'" />
          <BuildPanel v-else-if="store.section === 'build'" />
        </main>
      </div>
    </div>
  </div>
  <div v-if="store.toast" class="toast" :class="store.toastKind" role="status">{{ store.toast }}</div>
</template>
