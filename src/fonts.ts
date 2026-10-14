import { ASSET_BASE } from "./api";

/** Loads @font-face rules for every bundled font once, so pickers can preview them. */
export function useFontPreviews() {
  if (document.getElementById("bc-font-previews")) return;
  const link = document.createElement("link");
  link.id = "bc-font-previews";
  link.rel = "stylesheet";
  link.href = `${ASSET_BASE}/_runtime/fonts/all.css`;
  document.head.appendChild(link);
}
