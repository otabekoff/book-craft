// Typed wrappers around the studio's Tauri commands.
import { invoke } from "@tauri-apps/api/core";

export interface Meta {
  title: string;
  subtitle: string;
  author: string;
  publisher: string;
  publisherId: string;
  description: string;
  language: string;
  isbn: string;
  copyright: string;
  rights: string;
  website: string;
  version: string;
  packageId: string;
  executableName: string;
  icon: string;
  cover: string;
  accent: string;
}

export interface ChapterRef {
  file: string;
  title: string;
  hidden: boolean;
  kind: string;
}

export interface Typography {
  bodyFont: string;
  headingFont: string;
  customFont: string;
  baseSize: number;
  lineHeight: number;
  maxWidth: number;
  justify: boolean;
  hyphenate: boolean;
  dropCaps: boolean;
  smartQuotes: boolean;
  sceneBreak: string;
  paragraphStyle: string;
  headingNumbers: boolean;
  customCss: string;
}

export interface ReaderSettings {
  defaultTheme: string;
  allowThemeChange: boolean;
  allowFontSizeChange: boolean;
  allowFontChange: boolean;
  showToc: boolean;
  showProgress: boolean;
  rememberPosition: boolean;
  showHomeScreen: boolean;
  enableSearch: boolean;
  enableBookmarks: boolean;
  windowWidth: number;
  windowHeight: number;
  minWidth: number;
  minHeight: number;
  allowCopy: boolean;
  allowPrint: boolean;
  openLinksExternally: boolean;
  titleBar: string;
}

export interface Toggle {
  enabled: boolean;
}

export interface Modules {
  code: Toggle & {
    highlight: boolean;
    lineNumbers: boolean;
    copyButton: boolean;
    wrapLines: boolean;
    lightTheme: string;
    darkTheme: string;
    runnable: boolean;
  };
  math: Toggle & { numberEquations: boolean; chemistry: boolean };
  diagrams: Toggle;
  charts: Toggle;
  canvas: Toggle;
  callouts: Toggle;
  quotes: Toggle & { defaultStyle: string };
  boxes: Toggle;
  layout: Toggle;
  media: Toggle;
  learning: Toggle;
  widgets: Toggle;
  footnotes: Toggle;
  rawHtml: Toggle;
}

export interface BuildSettings {
  portable: boolean;
  nsis: boolean;
  msi: boolean;
  msix: boolean;
  outputDir: string;
  installMode: string;
  desktopShortcut: boolean;
  startMenuShortcut: boolean;
  launchAfterInstall: boolean;
  licenseFile: string;
  webview2: string;
  signing: { enabled: boolean; certificate: string; timestampUrl: string };
  productName: string;
}

export interface BookConfig {
  version: number;
  meta: Meta;
  chapters: ChapterRef[];
  typography: Typography;
  reader: ReaderSettings;
  modules: Modules;
  build: BuildSettings;
}

export interface ChapterInfo {
  file: string;
  title: string;
  hidden: boolean;
  kind: string;
  words: number;
  missing: boolean;
}

export interface ProjectData {
  root: string;
  config: BookConfig;
  chapters: ChapterInfo[];
  chapterClass: string;
  problems: string[];
  format: "json" | "toml";
}

export interface AssetInfo {
  path: string;
  size: number;
  kind: string;
}

export interface ToolStatus {
  name: string;
  neededFor: string;
  path: string | null;
  help: string;
}

export interface ToolPaths {
  makensis: string;
  wix: string;
  makeappx: string;
  signtool: string;
  webview2Bootstrapper: string;
}

export interface StudioSettings {
  recent: string[];
  tools: ToolPaths;
  theme: string;
}

export interface AppInfo {
  version: string;
  readerTemplate: string | null;
  vendor: string | null;
  tools: ToolStatus[];
  codeThemes: string[];
  fonts: [string, string][];
  settings: StudioSettings;
}

export interface Heading {
  level: number;
  id: string;
  text: string;
}

export interface Preview {
  html: string;
  headings: Heading[];
  warnings: string[];
  features: string[];
  words: number;
  chapterClass: string;
}

export type Target = "portable" | "nsis" | "msi" | "msix";

export interface Artifact {
  target: Target;
  label: string;
  path: string | null;
  size: number;
  signed: boolean;
  error: string | null;
}

export interface BuildReport {
  outputDir: string;
  artifacts: Artifact[];
  warnings: string[];
  seconds: number;
  pakSize: number;
}

export interface LogLine {
  level: "info" | "warn" | "error";
  message: string;
}

export interface CertificateInfo {
  subject: string;
  issuer: string;
  thumbprint: string;
  notAfter: string;
  expired: boolean;
  selfSigned: boolean;
  codeSigning: boolean;
  hasPrivateKey: boolean;
}

export interface TestCertificate {
  pfx: string;
  cer: string;
  info: CertificateInfo;
}

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  saveSettings: (settings: StudioSettings) => invoke<AppInfo>("save_settings", { settings }),
  create: (dir: string, title: string, author: string) => invoke<ProjectData>("project_create", { dir, title, author }),
  open: (dir: string) => invoke<ProjectData>("project_open", { dir }),
  close: () => invoke<void>("project_close"),
  saveConfig: (config: BookConfig) => invoke<ProjectData>("project_save_config", { config }),
  validate: () => invoke<string[]>("project_validate"),
  setFormat: (format: "json" | "toml") => invoke<ProjectData>("project_set_format", { format }),
  readChapter: (file: string) => invoke<string>("chapter_read", { file }),
  writeChapter: (file: string, content: string) => invoke<ChapterInfo[]>("chapter_write", { file, content }),
  addChapter: (title: string, after: string | null) => invoke<[string, ProjectData]>("chapter_add", { title, after }),
  removeChapter: (file: string) => invoke<ProjectData>("chapter_remove", { file }),
  assets: () => invoke<AssetInfo[]>("assets_list"),
  importAssets: (paths: string[], subdir: string) => invoke<string[]>("assets_import", { paths, subdir }),
  deleteAsset: (path: string) => invoke<AssetInfo[]>("asset_delete", { path }),
  render: (markdown: string, file: string | null) => invoke<Preview>("render_preview", { markdown, file }),
  build: (targets: Target[], password: string | null) => invoke<BuildReport>("build_book", { args: { targets, password } }),
  testRead: () => invoke<void>("test_read"),
  reveal: (path: string) => invoke<void>("reveal", { path }),
  launch: (path: string) => invoke<void>("launch", { path }),
  certificateInspect: (path: string, password: string | null) => invoke<CertificateInfo>("certificate_inspect", { path, password }),
  certificateCreate: (subject: string, password: string) => invoke<TestCertificate>("certificate_create", { subject, password }),
  certificateTrust: (cer: string) => invoke<void>("certificate_trust", { cer }),
};

export const ASSET_BASE = "http://bookasset.localhost";

export function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e);
}
