// The studio's effective theme ("light" or "dark"): the app bar's choice
// (html[data-theme]) or, when that is "match Windows", the system setting.
import { readonly, ref } from "vue";

const dark = window.matchMedia("(prefers-color-scheme: dark)");
const current = ref<"light" | "dark">("light");

function update() {
  const chosen = document.documentElement.getAttribute("data-theme");
  current.value = chosen === "dark" || chosen === "light" ? chosen : dark.matches ? "dark" : "light";
}

update();
dark.addEventListener("change", update);
new MutationObserver(update).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });

export const appTheme = readonly(current);
