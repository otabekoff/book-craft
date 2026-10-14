import { createApp } from "vue";
import App from "./App.vue";
import "./styles.css";

async function start() {
  if (import.meta.env.DEV && new URLSearchParams(location.search).has("mock")) {
    (await import("./devMock")).installMock();
  }
  createApp(App).mount("#app");
}

start();
