import { createApp } from "vue";
import App from "./App.vue";
import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { flushAll } from "./store";

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
