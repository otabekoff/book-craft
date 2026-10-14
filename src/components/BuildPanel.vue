<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText, type BuildReport, type LogLine, type Target } from "../api";
import { notify, saveConfigNow, store } from "../store";
import AssetPicker from "./AssetPicker.vue";
import Icon from "./Icon.vue";

const b = computed(() => store.config!.build);
const meta = computed(() => store.config!.meta);
const building = ref(false);
const log = ref<LogLine[]>([]);
const report = ref<BuildReport | null>(null);
const password = ref("");
const logEl = ref<HTMLElement>();
const showTools = ref(false);
let unlisten: UnlistenFn | null = null;

const targets = computed(() => [
  { key: "portable" as Target, on: "portable", label: "Portable .exe", desc: "One file that opens straight away. Nothing to install — good for sharing and USB sticks.", tool: null },
  { key: "nsis" as Target, on: "nsis", label: "Setup .exe", desc: "A familiar installer with Start menu and desktop shortcuts and an uninstaller.", tool: "NSIS" },
  { key: "msi" as Target, on: "msi", label: "Windows Installer .msi", desc: "For schools and companies that deploy software with Group Policy or Intune.", tool: "WiX Toolset 3" },
  { key: "msix" as Target, on: "msix", label: "MSIX package", desc: "Modern package for the Microsoft Store and App Installer. Must be signed to install outside the Store.", tool: "MakeAppx" },
]);

const selected = computed(() => targets.value.filter((t) => (b.value as any)[t.on]).map((t) => t.key));
const toolFound = (name: string | null) => !name || !!store.info?.tools.find((t) => t.name === name)?.path;
const ready = computed(() => !!store.info?.readerTemplate);

function size(n: number) {
  return (n / 1048576).toFixed(1) + " MB";
}

async function pickCertificate() {
  const picked = await open({ filters: [{ name: "Certificate", extensions: ["pfx", "p12"] }] });
  if (typeof picked === "string") b.value.signing.certificate = picked;
}

async function pickOutput() {
  const picked = await open({ directory: true, title: "Where should built packages go?" });
  if (typeof picked === "string") b.value.outputDir = picked;
}

async function build() {
  if (!selected.value.length) return;
  await saveConfigNow();
  building.value = true;
  report.value = null;
  log.value = [];
  try {
    report.value = await api.build(selected.value, b.value.signing.enabled ? password.value || null : null);
    const failed = report.value.artifacts.filter((a) => a.error).length;
    notify(failed ? `${failed} package(s) failed — see the log.` : "Your book app is ready.", failed ? "error" : "info");
  } catch (e) {
    log.value.push({ level: "error", message: errorText(e) });
    notify("The build stopped — see the log.", "error");
  } finally {
    building.value = false;
  }
}

async function saveTools() {
  if (!store.info) return;
  try {
    store.info = await api.saveSettings(store.info.settings);
    notify("Tool paths saved.");
  } catch (e) {
    notify(errorText(e), "error");
  }
}

onMounted(async () => {
  unlisten = await listen<LogLine>("build-log", async (e) => {
    log.value.push(e.payload);
    await nextTick();
    logEl.value?.scrollTo({ top: logEl.value.scrollHeight });
  });
  store.info = await api.appInfo();
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <header class="topbar">
    <h1>Build</h1>
    <span class="spacer"></span>
    <button class="btn" type="button" @click="api.testRead().catch((e) => notify(errorText(e), 'error'))" :disabled="!ready"><Icon name="play" />Try in reader</button>
    <button class="btn primary" type="button" @click="build" :disabled="building || !selected.length || !ready || store.problems.length > 0">
      <Icon name="build" />{{ building ? "Building…" : "Build " + selected.length + " package" + (selected.length === 1 ? "" : "s") }}
    </button>
  </header>
  <div class="page">
    <div class="page-inner">
      <div class="page-title">
        <div>
          <h2>Build your book app</h2>
          <p>Choose the packages you want. Each one contains the same reader with your book, icon and details.</p>
        </div>
      </div>

      <div v-if="!ready" class="notice error">The reader template is missing. Run <code>bun run prepare:resources</code> in the project folder, then restart Book Craft.</div>
      <div v-if="store.problems.length" class="notice warn">
        <b>Fix these in Book details first</b>
        <ul><li v-for="p in store.problems" :key="p">{{ p }}</li></ul>
      </div>

      <section class="card">
        <h3>Packages</h3>
        <div class="targets">
          <label v-for="t in targets" :key="t.key" class="target" :class="{ on: (b as any)[t.on] }">
            <input type="checkbox" v-model="(b as any)[t.on]" />
            <span class="t-body">
              <b>{{ t.label }}</b>
              <small>{{ t.desc }}</small>
              <span v-if="t.tool" class="tag" :class="toolFound(t.tool) ? 'ok' : 'bad'">{{ t.tool }} {{ toolFound(t.tool) ? "found" : "missing" }}</span>
            </span>
          </label>
        </div>
      </section>

      <section class="card">
        <h3>Installation</h3>
        <div class="grid">
          <div class="field">
            <span>Install for</span>
            <div class="seg">
              <button type="button" :aria-pressed="b.installMode === 'perUser'" @click="b.installMode = 'perUser'">Current user</button>
              <button type="button" :aria-pressed="b.installMode === 'perMachine'" @click="b.installMode = 'perMachine'">All users (admin)</button>
            </div>
            <small>Current user needs no administrator rights.</small>
          </div>
          <label class="field">
            <span>WebView2 runtime</span>
            <select class="input" v-model="b.webview2">
              <option value="embed">Bundle the bootstrapper (works offline to start)</option>
              <option value="download">Download it during setup if missing</option>
              <option value="skip">Don’t check (Windows 11 already has it)</option>
            </select>
          </label>
          <div class="field">
            <span>License shown by installers</span>
            <AssetPicker v-model="b.licenseFile" :extensions="['txt', 'md']" subdir="legal" placeholder="No license page" />
          </div>
        </div>
        <div class="grid">
          <label class="check"><input type="checkbox" v-model="b.startMenuShortcut" /><span><b>Start menu shortcut</b></span></label>
          <label class="check"><input type="checkbox" v-model="b.desktopShortcut" /><span><b>Desktop shortcut</b></span></label>
          <label class="check"><input type="checkbox" v-model="b.launchAfterInstall" /><span><b>Open the book after setup</b><small>Setup .exe only.</small></span></label>
        </div>
        <label class="field">
          <span>Output folder</span>
          <div class="input-row">
            <input class="input" v-model="b.outputDir" placeholder="dist" />
            <button class="btn" type="button" @click="pickOutput"><Icon name="folder" />Choose…</button>
          </div>
          <small>Relative paths are inside the book folder.</small>
        </label>
      </section>

      <section class="card">
        <h3><input type="checkbox" v-model="b.signing.enabled" aria-label="Sign packages" /> Code signing</h3>
        <p class="card-sub">Signed apps open without SmartScreen warnings once your certificate has a reputation. MSIX must be signed to install outside the Store; its certificate subject must equal the Publisher ID (<code>{{ meta.publisherId || "not set" }}</code>).</p>
        <fieldset class="grid" :disabled="!b.signing.enabled">
          <label class="field">
            <span>Certificate (.pfx)</span>
            <div class="input-row">
              <input class="input" v-model="b.signing.certificate" placeholder="C:\certs\publisher.pfx" />
              <button class="btn" type="button" @click="pickCertificate">Choose…</button>
            </div>
          </label>
          <label class="field">
            <span>Certificate password</span>
            <input class="input" type="password" v-model="password" autocomplete="off" placeholder="Used for this build only" />
            <small>Never saved. Builds from the command line can use BOOKCRAFT_CERT_PASSWORD.</small>
          </label>
          <label class="field">
            <span>Timestamp server</span>
            <input class="input" v-model="b.signing.timestampUrl" />
          </label>
        </fieldset>
      </section>

      <section class="card" v-if="log.length || report">
        <h3>Result</h3>
        <div v-if="report" class="artifacts">
          <div v-for="a in report.artifacts" :key="a.target" class="artifact" :class="{ failed: a.error }">
            <Icon :name="a.error ? 'warn' : 'check'" />
            <div class="a-body">
              <b>{{ a.label }}</b>
              <small v-if="a.path">{{ a.path }} · {{ size(a.size) }}{{ a.signed ? " · signed" : "" }}</small>
              <small v-else class="err">{{ a.error }}</small>
            </div>
            <template v-if="a.path">
              <button class="btn small" type="button" @click="api.launch(a.path!).catch((e) => notify(errorText(e), 'error'))">Open</button>
              <button class="btn small ghost" type="button" @click="api.reveal(a.path!)">Show in folder</button>
            </template>
          </div>
          <p class="sub">Built in {{ report.seconds.toFixed(1) }} s · book data {{ size(report.pakSize) }}</p>
        </div>
        <div class="log" ref="logEl" role="log">
          <div v-for="(l, i) in log" :key="i" :class="l.level">{{ l.message }}</div>
        </div>
      </section>

      <section class="card">
        <h3>
          Packaging tools
          <span class="spacer" style="flex: 1"></span>
          <button class="btn small ghost" type="button" @click="showTools = !showTools">{{ showTools ? "Hide paths" : "Set paths…" }}</button>
        </h3>
        <div class="tools">
          <div v-for="t in store.info?.tools ?? []" :key="t.name" class="tool">
            <span class="tag" :class="t.path ? 'ok' : 'bad'">{{ t.path ? "found" : "missing" }}</span>
            <b>{{ t.name }}</b>
            <small>{{ t.neededFor }}</small>
            <small class="path">{{ t.path || t.help }}</small>
          </div>
        </div>
        <div v-if="showTools && store.info" class="grid two">
          <label class="field"><span>makensis.exe</span><input class="input" v-model="store.info.settings.tools.makensis" placeholder="Found automatically" /></label>
          <label class="field"><span>WiX bin folder</span><input class="input" v-model="store.info.settings.tools.wix" placeholder="Found automatically" /></label>
          <label class="field"><span>makeappx.exe</span><input class="input" v-model="store.info.settings.tools.makeappx" placeholder="Found automatically" /></label>
          <label class="field"><span>signtool.exe</span><input class="input" v-model="store.info.settings.tools.signtool" placeholder="Found automatically" /></label>
          <label class="field"><span>MicrosoftEdgeWebview2Setup.exe</span><input class="input" v-model="store.info.settings.tools.webview2Bootstrapper" placeholder="Found automatically" /></label>
          <div class="field" style="justify-content: flex-end"><button class="btn" type="button" @click="saveTools">Save paths</button></div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
code { font-family: var(--mono); font-size: 12px; }
fieldset { border: 0; padding: 0; margin: 0; }
fieldset:disabled { opacity: 0.5; }
h3 input { width: 16px; height: 16px; accent-color: var(--accent); }
.targets { display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 12px; }
.target { display: flex; gap: 10px; padding: 14px; border: 1px solid var(--line); border-radius: 12px; cursor: pointer; align-items: flex-start; }
.target.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 6%, var(--panel)); }
.target input { margin-top: 3px; width: 16px; height: 16px; accent-color: var(--accent); flex: none; }
.t-body { display: flex; flex-direction: column; gap: 6px; }
.t-body small { color: var(--muted); font-size: 12px; line-height: 1.45; }
.t-body .tag { align-self: flex-start; }
.artifacts { display: flex; flex-direction: column; gap: 8px; }
.artifact { display: flex; align-items: center; gap: 12px; padding: 10px 12px; border-radius: 10px; background: color-mix(in srgb, var(--ok) 8%, var(--panel)); }
.artifact > .icon { color: var(--ok); }
.artifact.failed { background: color-mix(in srgb, var(--danger) 8%, var(--panel)); }
.artifact.failed > .icon { color: var(--danger); }
.a-body { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.a-body small { color: var(--muted); overflow-wrap: anywhere; font-size: 12px; }
.a-body .err { color: var(--danger); white-space: pre-wrap; }
.sub { color: var(--muted); font-size: 12px; margin: 0; }
.log { max-height: 280px; overflow-y: auto; background: var(--panel-2); border-radius: 8px; padding: 10px 12px; font-family: var(--mono); font-size: 12px; line-height: 1.6; }
.log .warn { color: var(--warn); }
.log .error { color: var(--danger); white-space: pre-wrap; }
.tools { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px; }
.tool { display: grid; grid-template-columns: auto 1fr; gap: 2px 10px; align-items: center; padding: 10px; border: 1px solid var(--line); border-radius: 10px; }
.tool small { grid-column: 2; color: var(--muted); font-size: 12px; }
.tool .path { overflow-wrap: anywhere; }
</style>
