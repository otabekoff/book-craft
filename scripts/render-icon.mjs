// Renders src-tauri/icons/source/icon.svg to a transparent 1024×1024 PNG with
// headless Microsoft Edge. `bun run icons` then makes every size with `tauri icon`.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "src-tauri", "icons", "source", "icon.svg");
const out = join(root, "src-tauri", "icons", "source", "icon.png");

const edge = [
  "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
].find(existsSync);
if (!edge) {
  console.error("Microsoft Edge was not found; export icon.svg to a 1024×1024 PNG another way.");
  process.exit(1);
}

const work = mkdtempSync(join(tmpdir(), "bc-icon-"));
const page = join(work, "icon.html");
writeFileSync(
  page,
  `<!doctype html><html><head><style>html,body{margin:0;background:transparent}svg{display:block;width:1024px;height:1024px}</style></head><body>${readFileSync(src, "utf8")}</body></html>`
);
execFileSync(edge, [
  "--headless=new",
  "--disable-gpu",
  "--hide-scrollbars",
  "--force-device-scale-factor=1",
  "--default-background-color=00000000",
  "--window-size=1024,1024",
  `--user-data-dir=${join(work, "profile")}`,
  `--screenshot=${out}`,
  pathToFileURL(page).href,
], { stdio: "ignore" });
rmSync(work, { recursive: true, force: true });
console.log(`Rendered ${out}`);
