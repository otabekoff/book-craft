// Full build: resources, checks, tests, the studio's installers and the command-line builder.
// Run: bun run build:all   (add --skip-tests to go faster)
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const skipTests = process.argv.includes("--skip-tests");

const steps = [
  ["Prepare resources (reader template, fonts, KaTeX, Mermaid)", "bun run prepare:resources"],
  ["Type-check the studio UI", "bun run check"],
  ...(skipTests ? [] : [["Rust tests", "cargo test --workspace"]]),
  // Resources are already prepared, so skip tauri's own prepare step.
  ["Build and package the studio (setup .exe, .msi)", `bun run tauri build --config "{\\"build\\":{\\"beforeBuildCommand\\":\\"bun run build\\"}}"`],
  ["Build the command-line builder", "cargo build --release -p bookcraft"],
];

const started = Date.now();
for (const [i, [title, command]] of steps.entries()) {
  console.log(`\n\x1b[1m[${i + 1}/${steps.length}] ${title}\x1b[0m\n> ${command}`);
  const t = Date.now();
  const result = spawnSync(command, { cwd: root, shell: true, stdio: "inherit" });
  if (result.status !== 0) {
    console.error(`\n\x1b[31mFailed: ${title} (exit ${result.status ?? result.signal})\x1b[0m`);
    process.exit(result.status || 1);
  }
  console.log(`\x1b[32m✓ ${title}\x1b[0m (${((Date.now() - t) / 1000).toFixed(0)} s)`);
}

const outputs = [];
const bundle = join(root, "target", "release", "bundle");
for (const kind of ["nsis", "msi"]) {
  const dir = join(bundle, kind);
  if (existsSync(dir)) for (const f of readdirSync(dir)) if (/\.(exe|msi)$/i.test(f)) outputs.push(join(dir, f));
}
const cli = join(root, "target", "release", "bookcraft.exe");
if (existsSync(cli)) outputs.push(cli);

console.log(`\n\x1b[1mDone in ${((Date.now() - started) / 60000).toFixed(1)} min\x1b[0m`);
for (const f of outputs) console.log(`  ${(statSync(f).size / 1048576).toFixed(1).padStart(6)} MB  ${f}`);
