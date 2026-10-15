<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  BadgeCheck, CircleCheck, CircleX, FileKey, FolderOpen, FolderSearch, Hammer, KeyRound, Package, Play, Settings2,
  ShieldAlert, ShieldCheck, ShieldPlus, Wrench,
} from "@lucide/vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText, type BuildReport, type CertificateInfo, type LogLine, type Target } from "../api";
import { notify, saveConfigNow, store } from "../store";
import { UiBadge, UiButton, UiCard, UiDialog, UiField, UiInput, UiSegmented, UiSelect, UiSwitch, type SelectOption, UiScrollArea } from "../ui";
import AssetPicker from "./AssetPicker.vue";

const b = computed(() => store.config!.build);
const meta = computed(() => store.config!.meta);
const building = ref(false);
const log = ref<LogLine[]>([]);
const report = ref<BuildReport | null>(null);
const password = ref("");
const logEl = ref<HTMLElement>();
const showTools = ref(false);
const cert = ref<CertificateInfo | null>(null);
const certError = ref("");
const checking = ref(false);
const newCert = ref<{ subject: string; password: string; busy: boolean } | null>(null);
const lastCer = ref(localStorage.getItem("bc.lastCer") || "");
let unlisten: UnlistenFn | null = null;

const targets = computed(() => [
  { key: "portable" as Target, on: "portable", label: "Portable .exe", desc: "One file that opens straight away. Nothing to install — good for sharing and USB sticks.", tool: null },
  { key: "nsis" as Target, on: "nsis", label: "Setup .exe", desc: "A familiar installer with Start menu and desktop shortcuts and an uninstaller.", tool: "NSIS" },
  { key: "msi" as Target, on: "msi", label: "Windows Installer .msi", desc: "For schools and companies that deploy software with Group Policy or Intune.", tool: "WiX Toolset 3" },
  { key: "msix" as Target, on: "msix", label: "MSIX package", desc: "For the Microsoft Store and App Installer. Must be signed to install outside the Store.", tool: "MakeAppx" },
]);

const selected = computed(() => targets.value.filter((t) => (b.value as any)[t.on]).map((t) => t.key));
const toolFound = (name: string | null) => !name || !!store.info?.tools.find((t) => t.name === name)?.path;
const ready = computed(() => !!store.info?.readerTemplate);
const installModes = [
  { value: "perUser", label: "Current user" },
  { value: "perMachine", label: "All users (admin)" },
];
const webview2Options: SelectOption<string>[] = [
  { value: "embed", label: "Bundle the bootstrapper", description: "Small; installs WebView2 if a PC lacks it" },
  { value: "download", label: "Download during setup", description: "Needs internet only on PCs without WebView2" },
  { value: "skip", label: "Don’t check", description: "Windows 11 already includes WebView2" },
];

const signingProblems = computed(() => {
  const c = cert.value;
  if (!b.value.signing.enabled || !c) return [];
  const out: string[] = [];
  if (c.expired) out.push(`The certificate expired on ${c.notAfter}.`);
  if (!c.hasPrivateKey) out.push("This file has no private key, so it cannot sign.");
  if (!c.codeSigning) out.push("This certificate is not for code signing.");
  if (b.value.msix && norm(c.subject) !== norm(meta.value.publisherId)) {
    out.push(`For MSIX, the Publisher ID (${meta.value.publisherId || "empty"}) must equal the certificate subject (${c.subject}).`);
  }
  return out;
});
const norm = (s: string) => s.split(",").map((p) => p.trim()).filter(Boolean).sort().join(", ");

function size(n: number) {
  return (n / 1048576).toFixed(1) + " MB";
}

async function pickCertificate() {
  const picked = await open({ filters: [{ name: "Certificate", extensions: ["pfx", "p12"] }] });
  if (typeof picked === "string") b.value.signing.certificate = picked;
}

async function checkCertificate() {
  cert.value = null;
  certError.value = "";
  const path = b.value.signing.certificate.trim();
  if (!path) return;
  checking.value = true;
  try {
    cert.value = await api.certificateInspect(path, password.value || null);
  } catch (e) {
    certError.value = errorText(e);
  } finally {
    checking.value = false;
  }
}

async function createCertificate() {
  if (!newCert.value) return;
  newCert.value.busy = true;
  try {
    const c = await api.certificateCreate(newCert.value.subject, newCert.value.password);
    b.value.signing.enabled = true;
    b.value.signing.certificate = c.pfx;
    password.value = newCert.value.password;
    if (!meta.value.publisherId.trim()) meta.value.publisherId = c.info.subject;
    lastCer.value = c.cer;
    localStorage.setItem("bc.lastCer", c.cer);
    cert.value = c.info;
    certError.value = "";
    newCert.value = null;
    notify("Test certificate created and selected.");
  } catch (e) {
    notify(errorText(e), "error");
    if (newCert.value) newCert.value.busy = false;
  }
}

async function trust() {
  if (!lastCer.value) return;
  if (!confirm("Windows will ask for administrator permission to trust this test certificate on this PC, so MSIX packages signed with it can be installed here.\n\nOnly do this for certificates you made yourself. Continue?")) return;
  try {
    await api.certificateTrust(lastCer.value);
    notify("This PC now trusts the test certificate for MSIX installs.");
  } catch (e) {
    notify(errorText(e), "error");
  }
}

async function pickOutput() {
  const picked = await open({ directory: true, title: "Where should built packages go?" });
  if (typeof picked === "string") b.value.outputDir = picked;
}

const longPath = computed(() => {
  const dir = b.value.outputDir.trim() || "dist";
  const full = /^[a-zA-Z]:[\\/]/.test(dir) ? dir : `${store.root}\\${dir}`;
  return full.length > 200 ? full : "";
});

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

watch(() => [b.value.signing.enabled, b.value.signing.certificate], () => {
  if (b.value.signing.enabled) checkCertificate();
});

onMounted(async () => {
  unlisten = await listen<LogLine>("build-log", async (e) => {
    log.value.push(e.payload);
    await nextTick();
    logEl.value?.scrollTo({ top: logEl.value.scrollHeight });
  });
  store.info = await api.appInfo();
  if (b.value.signing.enabled) checkCertificate();
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <header class="topbar">
    <h1>Build</h1>
    <span class="spacer"></span>
    <UiButton :icon="Play" :disabled="!ready" @click="api.testRead().catch((e) => notify(errorText(e), 'error'))">Try in reader</UiButton>
    <UiButton
      variant="primary"
      :icon="Hammer"
      :loading="building"
      :disabled="!selected.length || !ready || store.problems.length > 0 || signingProblems.length > 0"
      @click="build"
    >
      {{ building ? "Building…" : "Build " + selected.length + " package" + (selected.length === 1 ? "" : "s") }}
    </UiButton>
  </header>
  <UiScrollArea class="page-area" viewport-class="page">
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

      <UiCard title="Packages" :icon="Package">
        <div class="targets">
          <div v-for="t in targets" :key="t.key" class="target" :class="{ on: (b as any)[t.on] }" @click="(b as any)[t.on] = !(b as any)[t.on]">
            <div class="t-head">
              <b>{{ t.label }}</b>
              <UiSwitch v-model="(b as any)[t.on]" :aria-label="t.label" @click.stop />
            </div>
            <small>{{ t.desc }}</small>
            <UiBadge v-if="t.tool" :tone="toolFound(t.tool) ? 'ok' : 'bad'">{{ t.tool }} {{ toolFound(t.tool) ? "found" : "missing" }}</UiBadge>
          </div>
        </div>
      </UiCard>

      <UiCard title="Installation" :icon="Settings2">
        <div class="grid">
          <UiField label="Install for" hint="Current user needs no administrator rights.">
            <UiSegmented v-model="b.installMode" :options="installModes" aria-label="Install for" />
          </UiField>
          <UiField label="WebView2 runtime" for="b-wv2"><UiSelect id="b-wv2" v-model="b.webview2" :options="webview2Options" /></UiField>
          <UiField label="License shown by installers">
            <AssetPicker v-model="b.licenseFile" :extensions="['txt', 'md']" subdir="legal" placeholder="No license page" />
          </UiField>
        </div>
        <div class="grid">
          <UiSwitch v-model="b.startMenuShortcut" label="Start menu shortcut" />
          <UiSwitch v-model="b.desktopShortcut" label="Desktop shortcut" />
          <UiSwitch v-model="b.launchAfterInstall" label="Open the book after setup" description="Setup .exe only." />
        </div>
        <UiField label="Output folder" for="b-out" hint="Relative paths are inside the book folder." :error="longPath ? `This path is long (${longPath.length} characters). Builds still work, but Windows tools may struggle to open files there.` : undefined">
          <div class="row">
            <UiInput id="b-out" v-model="b.outputDir" placeholder="dist" />
            <UiButton :icon="FolderOpen" @click="pickOutput">Choose…</UiButton>
          </div>
        </UiField>
      </UiCard>

      <UiCard title="Code signing" :icon="ShieldCheck" v-model:enabled="b.signing.enabled" description="Signed apps show your name as publisher. MSIX must be signed to install outside the Store, with a certificate whose subject equals the Publisher ID.">
        <div class="grid">
          <UiField label="Certificate (.pfx)" for="s-cert">
            <div class="row">
              <UiInput id="s-cert" v-model="b.signing.certificate" placeholder="C:\certs\publisher.pfx" :icon="FileKey" />
              <UiButton :icon="FolderSearch" @click="pickCertificate">Choose…</UiButton>
            </div>
          </UiField>
          <UiField label="Certificate password" for="s-pw" hint="Used for this session only, never saved. Command-line builds read BOOKCRAFT_CERT_PASSWORD.">
            <UiInput id="s-pw" type="password" v-model="password" :icon="KeyRound" @change="checkCertificate" />
          </UiField>
          <UiField label="Timestamp server" for="s-ts" hint="Keeps signatures valid after the certificate expires. Needs internet while building.">
            <UiInput id="s-ts" v-model="b.signing.timestampUrl" />
          </UiField>
        </div>

        <div v-if="checking" class="cert muted">Checking the certificate…</div>
        <div v-else-if="certError" class="cert bad"><CircleX class="c-ico" /> {{ certError }}</div>
        <div v-else-if="cert" class="cert" :class="signingProblems.length ? 'bad' : 'ok'">
          <component :is="signingProblems.length ? ShieldAlert : BadgeCheck" class="c-ico" />
          <div>
            <b>{{ cert.subject }}</b>
            <small>Valid until {{ cert.notAfter }} · {{ cert.selfSigned ? "self-signed test certificate" : "issued by " + cert.issuer }}</small>
            <ul v-if="signingProblems.length"><li v-for="p in signingProblems" :key="p">{{ p }}</li></ul>
          </div>
        </div>

        <template #footer>
        <div class="test-cert">
          <div>
            <b>No certificate yet?</b>
            <small>Make a self-signed test certificate to try signing and MSIX installs on your own PC. Readers’ PCs won’t trust it — buy one from a certificate authority or use Azure Trusted Signing for real releases.</small>
          </div>
          <div class="row-end">
            <UiButton :icon="ShieldPlus" @click="newCert = { subject: meta.publisherId || 'CN=' + (meta.publisher || meta.author || 'Test Publisher'), password: '', busy: false }">Create test certificate…</UiButton>
            <UiButton v-if="lastCer" variant="ghost" :icon="Wrench" @click="trust" title="Needed only to install MSIX packages signed with a test certificate">Trust on this PC…</UiButton>
          </div>
        </div>
        </template>
      </UiCard>

      <UiCard v-if="log.length || report" title="Result">
        <div v-if="report" class="artifacts">
          <div v-for="a in report.artifacts" :key="a.target" class="artifact" :class="{ failed: a.error }">
            <component :is="a.error ? CircleX : CircleCheck" class="a-ico" aria-hidden="true" />
            <div class="a-body">
              <b>{{ a.label }} <UiBadge v-if="a.signed" tone="ok">signed</UiBadge></b>
              <small v-if="a.path">{{ a.path }} · {{ size(a.size) }}</small>
              <small v-else class="err">{{ a.error }}</small>
            </div>
            <template v-if="a.path">
              <UiButton size="sm" @click="api.launch(a.path!).catch((e) => notify(errorText(e), 'error'))">Open</UiButton>
              <UiButton size="sm" variant="ghost" @click="api.reveal(a.path!)">Show in folder</UiButton>
            </template>
          </div>
          <p class="sub">Built in {{ report.seconds.toFixed(1) }} s · book data {{ size(report.pakSize) }}</p>
        </div>
        <div class="log" ref="logEl" role="log">
          <div v-for="(l, i) in log" :key="i" :class="l.level">{{ l.message }}</div>
        </div>
      </UiCard>

      <UiCard title="Packaging tools" :icon="Wrench">
        <template #actions>
          <UiButton size="sm" variant="ghost" @click="showTools = !showTools">{{ showTools ? "Hide paths" : "Set paths…" }}</UiButton>
        </template>
        <div class="tools">
          <div v-for="t in store.info?.tools ?? []" :key="t.name" class="tool">
            <UiBadge :tone="t.path ? 'ok' : 'bad'">{{ t.path ? "found" : "missing" }}</UiBadge>
            <b>{{ t.name }}</b>
            <small>{{ t.neededFor }}</small>
            <small class="path">{{ t.path || t.help }}</small>
          </div>
        </div>
        <div v-if="showTools && store.info" class="grid two">
          <UiField label="makensis.exe" for="t-nsis"><UiInput id="t-nsis" v-model="store.info.settings.tools.makensis" placeholder="Found automatically" /></UiField>
          <UiField label="WiX bin folder" for="t-wix"><UiInput id="t-wix" v-model="store.info.settings.tools.wix" placeholder="Found automatically" /></UiField>
          <UiField label="makeappx.exe" for="t-appx"><UiInput id="t-appx" v-model="store.info.settings.tools.makeappx" placeholder="Found automatically" /></UiField>
          <UiField label="signtool.exe" for="t-sign"><UiInput id="t-sign" v-model="store.info.settings.tools.signtool" placeholder="Found automatically" /></UiField>
          <UiField label="MicrosoftEdgeWebview2Setup.exe" for="t-wv"><UiInput id="t-wv" v-model="store.info.settings.tools.webview2Bootstrapper" placeholder="Found automatically" /></UiField>
          <div class="row-end"><UiButton @click="saveTools">Save paths</UiButton></div>
        </div>
      </UiCard>
    </div>
  </UiScrollArea>

  <UiDialog v-if="newCert" title="Create a test certificate" @close="newCert = null">
    <p class="muted">A self-signed code-signing certificate, saved in Book Craft’s data folder (not in your book). It is valid for two years.</p>
    <form id="cert-form" class="form" @submit.prevent="createCertificate">
      <UiField label="Subject" for="nc-subject" hint="Starts with CN=. It also becomes the Publisher ID for MSIX if that is empty.">
        <UiInput id="nc-subject" v-model="newCert.subject" mono />
      </UiField>
      <UiField label="Password for the .pfx file" for="nc-pw" hint="At least 4 characters. You’ll need it when building.">
        <UiInput id="nc-pw" type="password" v-model="newCert.password" autofocus />
      </UiField>
    </form>
    <template #actions>
      <UiButton @click="newCert = null">Cancel</UiButton>
      <UiButton variant="primary" type="submit" form="cert-form" :loading="newCert.busy" :disabled="!newCert.subject.startsWith('CN=') || newCert.password.length < 4">Create</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
code { font-family: var(--mono); font-size: 12px; }
.row { display: flex; gap: 8px; }
.row > :first-child { flex: 1; min-width: 0; }
.row-end { display: flex; gap: 8px; justify-content: flex-end; align-items: flex-end; flex-wrap: wrap; }
.form { display: flex; flex-direction: column; gap: 14px; }
.muted { color: var(--muted); margin: 0; }
.targets { display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 12px; }
.target { display: flex; flex-direction: column; gap: 8px; padding: 14px; border: 1px solid var(--line); border-radius: 12px; cursor: pointer; transition: border-color 0.15s, background 0.15s; }
.target.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 6%, var(--panel)); }
.t-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.target small { color: var(--muted); font-size: 12px; line-height: 1.45; }
.target .ui-badge { align-self: flex-start; }
.cert { display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border-radius: 10px; background: var(--panel-2); }
.cert b { display: block; }
.cert small { color: var(--muted); font-size: 12px; }
.cert ul { margin: 6px 0 0; padding-left: 18px; font-size: 13px; }
.cert.ok { background: color-mix(in srgb, var(--ok) 9%, var(--panel)); }
.cert.ok .c-ico { color: var(--ok); }
.cert.bad { background: color-mix(in srgb, var(--danger) 8%, var(--panel)); }
.cert.bad .c-ico { color: var(--danger); }
.cert.muted { color: var(--muted); }
.c-ico { width: 20px; height: 20px; flex: none; }
.test-cert { display: flex; gap: 16px; align-items: flex-end; justify-content: space-between; flex-wrap: wrap; }
.test-cert > div:first-child { flex: 1 1 320px; display: flex; flex-direction: column; gap: 4px; }
.test-cert small { color: var(--muted); font-size: 12px; line-height: 1.5; }
.artifacts { display: flex; flex-direction: column; gap: 8px; }
.artifact { display: flex; align-items: center; gap: 12px; padding: 10px 12px; border-radius: 10px; background: color-mix(in srgb, var(--ok) 8%, var(--panel)); }
.a-ico { width: 20px; height: 20px; color: var(--ok); flex: none; }
.artifact.failed { background: color-mix(in srgb, var(--danger) 8%, var(--panel)); }
.artifact.failed .a-ico { color: var(--danger); }
.a-body { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.a-body b { display: flex; align-items: center; gap: 8px; }
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
