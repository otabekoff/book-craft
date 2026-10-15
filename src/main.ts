import { createApp } from "vue";
import App from "./App.vue";
import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { flushAll } from "./store";

// No browser menu (Back, Reload, Inspect…) on right-click. Text fields and the editor keep
// theirs for cut, copy and paste.
document.addEventListener("contextmenu", (e) => {
  const target = e.target as HTMLElement | null;
  if (target?.closest("input, textarea, [contenteditable=true], .cm-content")) return;
  e.preventDefault();
});

async function start() {
  if (import.meta.env.DEV && new URLSearchParams(location.search).has("mock")) {
    (await import("./devMock")).installMock();
  }
  createApp(App).mount("#app");
  // Ctrl+C in the terminal: save pending edits, then let the app exit cleanly.
  listen("app-exit-requested", async () => {
    try {
      await flushAll();
    } finally {
      await invoke("quit");
    }
  }).catch(() => {
    /* not running inside Tauri (mock mode) */
  });
}

start();
