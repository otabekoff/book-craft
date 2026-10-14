// Builds the reader template and copies it plus the math/diagram libraries
// into src-tauri/resources, where the studio (and its installer) find them.
//
//   node scripts/prepare.mjs            build the reader (release) and copy everything
//   node scripts/prepare.mjs --debug    use a debug reader build (faster to compile)
//   node scripts/prepare.mjs --no-build copy only

import { execSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const args = new Set(process.argv.slice(2));
const profile = args.has("--debug") ? "debug" : "release";
const resources = join(root, "src-tauri", "resources");

function copy(from, to) {
  mkdirSync(dirname(to), { recursive: true });
  cpSync(from, to, { recursive: true });
}

if (!args.has("--no-build")) {
  console.log(`Building the reader template (${profile})…`);
  execSync(`cargo build -p book-reader ${profile === "release" ? "--release" : ""}`, { cwd: root, stdio: "inherit" });
}

const reader = join(root, "target", profile, "book-reader.exe");
if (!existsSync(reader)) {
  console.error(`Reader not found at ${reader}. Run without --no-build.`);
  process.exit(1);
}
copy(reader, join(resources, "reader", "book-reader.exe"));
console.log(`Reader template: ${(statSync(reader).size / 1048576).toFixed(1)} MB`);

const vendor = join(resources, "runtime", "vendor");
rmSync(vendor, { recursive: true, force: true });

const katex = join(root, "node_modules", "katex", "dist");
if (existsSync(katex)) {
  copy(join(katex, "katex.min.js"), join(vendor, "katex", "katex.min.js"));
  copy(join(katex, "katex.min.css"), join(vendor, "katex", "katex.min.css"));
  copy(join(katex, "contrib", "mhchem.min.js"), join(vendor, "katex", "contrib", "mhchem.min.js"));
  for (const f of readdirSync(join(katex, "fonts")).filter((f) => f.endsWith(".woff2"))) {
    copy(join(katex, "fonts", f), join(vendor, "katex", "fonts", f));
  }
  console.log("Copied KaTeX");
} else {
  console.warn("KaTeX is not installed (bun install) — math will show as source.");
}

const mermaid = join(root, "node_modules", "mermaid", "dist", "mermaid.min.js");
if (existsSync(mermaid)) {
  copy(mermaid, join(vendor, "mermaid.min.js"));
  console.log("Copied Mermaid");
} else {
  console.warn("Mermaid is not installed (bun install) — diagrams will show as text.");
}
console.log("Resources ready in src-tauri/resources");
