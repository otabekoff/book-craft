// Development only: open http://localhost:1420/?mock in a normal browser to try
// the studio UI without the Tauri backend. Never bundled into release builds.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const chapters = [
  { file: "chapters/01-welcome.md", title: "Welcome", hidden: false, kind: "chapter", words: 420, missing: false },
  { file: "chapters/02-stories.md", title: "Stories and Poetry", hidden: false, kind: "chapter", words: 310, missing: false },
  { file: "chapters/03-science.md", title: "Math, Physics and Chemistry", hidden: false, kind: "chapter", words: 280, missing: false },
];

const config = {
  version: 1,
  meta: {
    title: "The Salt Road", subtitle: "", author: "Ann Wren", publisher: "Harbourlight Press", publisherId: "CN=Harbourlight Press",
    description: "A sample book.", language: "en-US", isbn: "", copyright: "© 2026 Ann Wren", rights: "", website: "",
    version: "1.0.0.0", packageId: "com.harbourlight.saltroad", executableName: "", icon: "", cover: "", accent: "#2f5a8a",
  },
  chapters: chapters.map((c) => ({ file: c.file, title: "", hidden: false, kind: "chapter" })),
  typography: { bodyFont: "sitka", headingFont: "same", customFont: "", baseSize: 19, lineHeight: 1.65, maxWidth: 700, justify: false, hyphenate: true, dropCaps: false, smartQuotes: true, sceneBreak: "✻ ✻ ✻", paragraphStyle: "spaced", headingNumbers: false, customCss: "", uiFont: "segoe", codeFont: "cascadia" },
  reader: { defaultTheme: "light", allowThemeChange: true, allowFontSizeChange: true, allowFontChange: true, showToc: true, showProgress: true, rememberPosition: true, showHomeScreen: true, enableSearch: true, enableBookmarks: true, windowWidth: 1200, windowHeight: 820, minWidth: 480, minHeight: 400, allowCopy: true, allowPrint: true, openLinksExternally: true, titleBar: "custom", fontChoices: ["georgia", "segoe"], defaultLayout: "scroll", pageSpread: "auto", paperLook: false, pageTurn: "fade", pageNumbers: true, defaultWidth: "normal" },
  modules: {
    code: { enabled: true, highlight: true, lineNumbers: false, copyButton: true, wrapLines: false, lightTheme: "InspiredGitHub", darkTheme: "OneHalfDark", runnable: true },
    math: { enabled: true, numberEquations: true, chemistry: true },
    diagrams: { enabled: true }, charts: { enabled: true }, canvas: { enabled: true }, callouts: { enabled: true },
    quotes: { enabled: true, defaultStyle: "classic" }, boxes: { enabled: true }, layout: { enabled: true }, media: { enabled: true },
    learning: { enabled: true }, widgets: { enabled: false }, footnotes: { enabled: true }, rawHtml: { enabled: false },
  },
  build: { portable: true, nsis: true, msi: false, msix: false, outputDir: "dist", installMode: "perUser", desktopShortcut: true, startMenuShortcut: true, launchAfterInstall: true, licenseFile: "", webview2: "embed", signing: { enabled: false, certificate: "", timestampUrl: "http://timestamp.digicert.com" }, productName: "" },
};

const project = () => ({ root: "D:\\Books\\The Salt Road", config, chapters, chapterClass: "bc-chapter", problems: [], format: "json" });
const tools = ["NSIS", "WiX Toolset 3", "MakeAppx", "SignTool", "WebView2 bootstrapper"].map((name) => ({ name, neededFor: "", path: "C:\\tools\\" + name, help: "" }));
const info = { version: "0.1.0-mock", readerTemplate: "reader.exe", vendor: "vendor", tools, codeThemes: ["InspiredGitHub", "OneHalfDark"], fonts: [{ key: "sitka", name: "Sitka Text", category: "serif", source: "windows", stack: "Sitka Text, Georgia, serif", note: "Built into Windows", kb: 0, cyrillic: true }, { key: "lib:literata", name: "Literata", category: "serif", source: "library", stack: "Literata Variable, Georgia, serif", note: "Designed for long reading on screens", kb: 263, cyrillic: true }], settings: { recent: ["D:\\Books\\The Salt Road"], tools: { makensis: "", wix: "", makeappx: "", signtool: "", webview2Bootstrapper: "" }, theme: "" } };

export function installMock() {
  mockWindows("main");
  mockIPC((cmd, args: any) => {
    switch (cmd) {
      case "app_info": return info;
      case "project_open": case "project_create": case "project_save_config": case "chapter_remove": return project();
      case "chapter_read": return `# ${chapters.find((c) => c.file === args.file)?.title}\n\nSome **text** to edit.\n\n::: tip\nA callout.\n:::\n`;
      case "chapter_write": return chapters;
      case "chapter_add": return ["chapters/01-welcome.md", project()];
      case "assets_list": case "asset_delete": return [{ path: "assets/icon.png", size: 20480, kind: "image" }];
      case "render_preview": return { html: "<h1 id=\"x\">Preview</h1><p>Rendered by the real backend in the app.</p>", headings: [{ level: 1, id: "x", text: "Preview" }], warnings: [], features: [], words: 6, chapterClass: "bc-chapter" };
      case "project_validate": return [];
      case "plugin:event|listen": return 1;
      default: return null;
    }
  });
}
