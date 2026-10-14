// Builds the bundled font library from @fontsource packages into
// src-tauri/resources/runtime/fonts:
//
//   fonts.json            manifest (id, name, CSS family, category, licence…)
//   <id>/font.css         @font-face rules with url(__BASE__file.woff2)
//   <id>/*.woff2          Latin + Cyrillic subsets only, to keep books small
//   <id>/LICENSE          the font's licence (OFL requires it to travel with the font)
//
// Run on its own with `node scripts/fonts.mjs`; prepare.mjs also calls it.

import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "src-tauri", "resources", "runtime", "fonts");
const SUBSETS = new Set(["latin", "latin-ext", "cyrillic", "cyrillic-ext"]);

// [id, package, display name, category, CSS files to take, note]
const FONTS = [
  ["literata", "@fontsource-variable/literata", "Literata", "serif", ["wght.css", "wght-italic.css"], "Designed for long reading on screens"],
  ["source-serif-4", "@fontsource-variable/source-serif-4", "Source Serif 4", "serif", ["wght.css", "wght-italic.css"], "Clear, modern book serif"],
  ["merriweather", "@fontsource-variable/merriweather", "Merriweather", "serif", ["wght.css", "wght-italic.css"], "Sturdy and very readable"],
  ["lora", "@fontsource-variable/lora", "Lora", "serif", ["wght.css", "wght-italic.css"], "Calligraphic, good for fiction"],
  ["eb-garamond", "@fontsource-variable/eb-garamond", "EB Garamond", "serif", ["wght.css", "wght-italic.css"], "Classic old-style book face"],
  ["pt-serif", "@fontsource/pt-serif", "PT Serif", "serif", ["400.css", "400-italic.css", "700.css", "700-italic.css"], "Made for Cyrillic and Latin"],
  ["noto-serif", "@fontsource-variable/noto-serif", "Noto Serif", "serif", ["wght.css", "wght-italic.css"], "Wide language coverage"],
  ["playfair-display", "@fontsource-variable/playfair-display", "Playfair Display", "display", ["wght.css", "wght-italic.css"], "High-contrast, for headings"],
  ["inter", "@fontsource-variable/inter", "Inter", "sans", ["wght.css", "wght-italic.css"], "Neutral interface sans"],
  ["source-sans-3", "@fontsource-variable/source-sans-3", "Source Sans 3", "sans", ["wght.css", "wght-italic.css"], "Friendly, open sans"],
  ["ibm-plex-sans", "@fontsource-variable/ibm-plex-sans", "IBM Plex Sans", "sans", ["wght.css", "wght-italic.css"], "Technical, distinctive"],
  ["nunito", "@fontsource-variable/nunito", "Nunito", "sans", ["wght.css", "wght-italic.css"], "Rounded, good for children’s books"],
  ["noto-sans", "@fontsource-variable/noto-sans", "Noto Sans", "sans", ["wght.css", "wght-italic.css"], "Wide language coverage"],
  ["atkinson-hyperlegible-next", "@fontsource-variable/atkinson-hyperlegible-next", "Atkinson Hyperlegible", "sans", ["wght.css", "wght-italic.css"], "For low vision readers (Latin only)"],
  ["opendyslexic", "@fontsource/opendyslexic", "OpenDyslexic", "sans", ["400.css", "400-italic.css", "700.css", "700-italic.css"], "For readers with dyslexia (Latin only)"],
  ["caveat", "@fontsource-variable/caveat", "Caveat", "handwriting", ["wght.css"], "Handwriting, for notes and letters"],
  ["jetbrains-mono", "@fontsource-variable/jetbrains-mono", "JetBrains Mono", "mono", ["wght.css", "wght-italic.css"], "Code font with clear shapes"],
  ["fira-code", "@fontsource-variable/fira-code", "Fira Code", "mono", ["wght.css"], "Code font with ligatures"],
  ["source-code-pro", "@fontsource-variable/source-code-pro", "Source Code Pro", "mono", ["wght.css", "wght-italic.css"], "Calm, classic code font"],
];

export function buildFonts() {
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  const manifest = [];
  for (const [id, pkg, name, category, cssFiles, note] of FONTS) {
    const dir = join(root, "node_modules", ...pkg.split("/"));
    if (!existsSync(dir)) {
      console.warn(`  skipped ${name}: ${pkg} is not installed (bun install)`);
      continue;
    }
    const meta = JSON.parse(readFileSync(join(dir, "metadata.json"), "utf8"));
    const target = join(out, id);
    mkdirSync(target, { recursive: true });
    let css = "";
    let family = "";
    let bytes = 0;
    const subsets = new Set();
    for (const file of cssFiles) {
      const path = join(dir, file);
      if (!existsSync(path)) continue;
      const text = readFileSync(path, "utf8");
      for (const block of text.match(/@font-face\s*{[^}]*}/g) ?? []) {
        const url = block.match(/url\(\.\/files\/([^)]+\.woff2)\)/)?.[1];
        if (!url) continue;
        // File names look like <id>-<subset>-<axis or weight>-<style>.woff2
        const subset = [...SUBSETS].sort((a, b) => b.length - a.length).find((s) => url.startsWith(`${meta.id}-${s}-`));
        if (!subset) continue;
        subsets.add(subset);
        family ||= block.match(/font-family:\s*'([^']+)'/)?.[1] ?? name;
        copyFileSync(join(dir, "files", url), join(target, url));
        bytes += statSync(join(target, url)).size;
        css += block.replace(`url(./files/${url})`, `url(__BASE__${url})`).replace(/format\('woff2-variations'\)/, "format('woff2')") + "\n";
      }
    }
    if (!css) {
      console.warn(`  skipped ${name}: no Latin or Cyrillic files`);
      continue;
    }
    writeFileSync(join(target, "font.css"), css);
    const license = ["LICENSE", "LICENSE.txt", "OFL.txt"].map((f) => join(dir, f)).find(existsSync);
    if (license) copyFileSync(license, join(target, "LICENSE"));
    manifest.push({
      id,
      name,
      family,
      category,
      note,
      italic: cssFiles.some((f) => f.includes("italic")),
      variable: !!meta.variable,
      subsets: [...subsets].sort(),
      license: meta.license?.type ?? "OFL-1.1",
      source: meta.source ?? "",
      kb: Math.round(bytes / 1024),
    });
  }
  writeFileSync(join(out, "fonts.json"), JSON.stringify(manifest, null, 2));
  const total = manifest.reduce((n, f) => n + f.kb, 0);
  console.log(`Font library: ${manifest.length} families, ${(total / 1024).toFixed(1)} MB (only fonts a book uses are packed)`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) buildFonts();
