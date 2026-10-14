// Everything the Insert menu can add. `$SEL` is replaced by the selected text,
// `$0` marks where the cursor goes afterwards.

export interface BlockItem {
  label: string;
  hint?: string;
  snippet: string;
  /** Inserted in the middle of a line rather than as its own block. */
  inline?: boolean;
  module?: string;
}

export interface BlockGroup {
  name: string;
  items: BlockItem[];
}

const callout = (type: string, label: string, body = "$SEL$0"): BlockItem => ({
  label,
  snippet: `::: ${type}\n${body}\n:::`,
  module: "callouts",
});

export const BLOCKS: BlockGroup[] = [
  {
    name: "Text",
    items: [
      { label: "Heading 2", snippet: "## $SEL$0" },
      { label: "Heading 3", snippet: "### $SEL$0" },
      { label: "Bullet list", snippet: "- $SEL$0\n- \n- " },
      { label: "Numbered list", snippet: "1. $SEL$0\n2. \n3. " },
      { label: "Task list", snippet: "- [ ] $SEL$0\n- [x] Done item" },
      { label: "Table", snippet: "| Column | Column | Column |\n|:-------|:------:|-------:|\n| $0 |  |  |\n|  |  |  |" },
      { label: "Footnote", snippet: "[^note]$0\n\n[^note]: The footnote text.", inline: false },
      { label: "Scene break", snippet: "***" },
      { label: "Definition list", snippet: "Term\n\n: Definition of the term.$0" },
      { label: "Lead paragraph", snippet: "::: lead\n$SEL$0\n:::" },
      { label: "Centered text", snippet: "::: center\n$SEL$0\n:::" },
      { label: "Small caps", snippet: "::: smallcaps\n$SEL$0\n:::" },
    ],
  },
  {
    name: "Callouts",
    items: [
      callout("note", "Note"),
      callout("tip", "Tip"),
      callout("info", "Info"),
      callout("warning", "Warning"),
      callout("danger", "Danger"),
      callout("important", "Important"),
      callout("success", "Success"),
      callout("question", "Question"),
      { label: "Collapsible callout", snippet: '::: note {collapsible=true title="Click to open"}\n$SEL$0\n:::', module: "callouts" },
      { label: "Custom title & color", snippet: '::: note {title="My title" color=teal}\n$SEL$0\n:::', module: "callouts" },
      callout("summary", "Summary"),
      callout("keypoints", "Key points", "- $0\n- "),
      callout("objectives", "Learning objectives", "- $0\n- "),
      callout("history", "Historical note"),
    ],
  },
  {
    name: "Quotes",
    items: [
      { label: "Block quote", snippet: "> $SEL$0" },
      { label: "Classic quote", snippet: '::: quote {style=classic author="Author" source="Work"}\n$SEL$0\n:::', module: "quotes" },
      { label: "Pull quote", snippet: '::: quote {style=pull author="Author"}\n$SEL$0\n:::', module: "quotes" },
      { label: "Epigraph", snippet: '::: epigraph {author="Author" source="Work"}\n$SEL$0\n:::', module: "quotes" },
      { label: "Quote card", snippet: '::: quote {style=card author="Name" role="Role"}\n$SEL$0\n:::', module: "quotes" },
      { label: "Big quote", snippet: '::: quote {style=big author="Author"}\n$SEL$0\n:::', module: "quotes" },
      { label: "Line quote", snippet: '::: quote {style=line author="Author"}\n$SEL$0\n:::', module: "quotes" },
      { label: "Shaded quote", snippet: '::: quote {style=shaded}\n$SEL$0\n:::', module: "quotes" },
    ],
  },
  {
    name: "Boxes & shapes",
    items: [
      { label: "Framed box", snippet: '::: box {style=framed title="Title"}\n$SEL$0\n:::', module: "boxes" },
      { label: "Shaded box", snippet: "::: box {style=shaded}\n$SEL$0\n:::", module: "boxes" },
      { label: "Tinted box", snippet: "::: box {style=tinted color=blue}\n$SEL$0\n:::", module: "boxes" },
      { label: "Filled box", snippet: "::: box {style=filled color=accent}\n$SEL$0\n:::", module: "boxes" },
      { label: "Outline box", snippet: "::: box {style=outline color=purple}\n$SEL$0\n:::", module: "boxes" },
      { label: "Dashed box", snippet: "::: box {style=dashed}\n$SEL$0\n:::", module: "boxes" },
      { label: "Double border", snippet: "::: box {style=double}\n$SEL$0\n:::", module: "boxes" },
      { label: "Card with shadow", snippet: "::: card\n$SEL$0\n:::", module: "boxes" },
      { label: "Sticky note", snippet: "::: box {style=note}\n$SEL$0\n:::", module: "boxes" },
      { label: "Pill shape", snippet: "::: box {style=filled shape=pill align=center color=teal}\n$SEL$0\n:::", module: "boxes" },
      { label: "Circle shape", snippet: "::: box {style=tinted shape=circle color=orange}\n$SEL$0\n:::", module: "boxes" },
      { label: "Cut corners", snippet: "::: box {style=tinted shape=cut}\n$SEL$0\n:::", module: "boxes" },
      { label: "Ticket shape", snippet: "::: box {style=tinted shape=ticket color=pink}\n$SEL$0\n:::", module: "boxes" },
      { label: "Floating box", snippet: "::: box {style=shaded float=right width=40%}\n$SEL$0\n:::", module: "boxes" },
    ],
  },
  {
    name: "Layout",
    items: [
      { label: "Two columns", snippet: ":::: columns {count=2}\n::: column\n$SEL$0\n:::\n::: column\n\n:::\n::::", module: "layout" },
      { label: "Three columns", snippet: ":::: columns {count=3}\n::: column\n$0\n:::\n::: column\n\n:::\n::: column\n\n:::\n::::", module: "layout" },
      { label: "Newspaper columns", snippet: "::: columns {count=2 flow=true}\n$SEL$0\n:::", module: "layout" },
      { label: "Tabs", snippet: ':::: tabs\n::: tab {title="First"}\n$SEL$0\n:::\n::: tab {title="Second"}\n\n:::\n::::', module: "layout" },
      { label: "Details (expandable)", snippet: '::: details {summary="More"}\n$SEL$0\n:::' },
      { label: "Spoiler", snippet: "::: spoiler\n$SEL$0\n:::" },
      { label: "Inline spoiler", snippet: "||$SEL$0||", inline: true },
      { label: "Aside", snippet: '::: aside {title="Aside"}\n$SEL$0\n:::' },
      { label: "Margin note", snippet: "::: sidenote\n$SEL$0\n:::" },
      { label: "Chapter opener", snippet: '::: opener {label="Part One"}\n$0\n:::' },
      { label: "Page break (print)", snippet: "::: pagebreak\n:::" },
    ],
  },
  {
    name: "Prose forms",
    items: [
      { label: "Poem / verse", snippet: '::: verse {title="Title" author="Poet"}\n$SEL$0\n:::' },
      { label: "Dialogue / script", snippet: "::: dialogue\n**NAME:** $0\n\n**OTHER:** \n:::" },
      { label: "Letter", snippet: "::: letter\n*Place, date*\n\nDear $0,\n\n\nYours,\nName\n:::" },
    ],
  },
  {
    name: "Math & science",
    items: [
      { label: "Inline math", snippet: "$$SEL$0$", inline: true, module: "math" },
      { label: "Display math", snippet: "$$\n$SEL$0\n$$", module: "math" },
      { label: "Numbered equation", snippet: "```math {#eq-name}\n$SEL$0\n```", module: "math" },
      { label: "Aligned equations", snippet: "$$\n\\begin{aligned}\na &= b + c \\\\\n  &= $0\n\\end{aligned}\n$$", module: "math" },
      { label: "Matrix", snippet: "$$\n\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}$0\n$$", module: "math" },
      { label: "Fraction", snippet: "$\\frac{$SEL$0}{}$", inline: true, module: "math" },
      { label: "Units (physics)", snippet: "$\\pu{$SEL$0 m/s^2}$", inline: true, module: "math" },
      { label: "Chemical equation", snippet: "```chem\n$SEL$0 2H2 + O2 -> 2H2O\n```", module: "math" },
      { label: "Inline chemistry", snippet: "$\\ce{$SEL$0H2O}$", inline: true, module: "math" },
      { label: "Theorem", snippet: '::: theorem {title="Name"}\n$SEL$0\n:::', module: "callouts" },
      { label: "Lemma", snippet: "::: lemma\n$SEL$0\n:::", module: "callouts" },
      { label: "Proof", snippet: "::: proof\n$SEL$0 $\\blacksquare$\n:::", module: "callouts" },
      { label: "Definition", snippet: '::: definition {title="Term"}\n$SEL$0\n:::', module: "callouts" },
      { label: "Example", snippet: "::: example\n$SEL$0\n:::", module: "callouts" },
    ],
  },
  {
    name: "Code",
    items: [
      { label: "Inline code", snippet: "`$SEL$0`", inline: true },
      { label: "Code block", snippet: "```python\n$SEL$0\n```", module: "code" },
      { label: "Code with title & lines", snippet: '```rust {title="main.rs" lines=true highlight="2"}\n$SEL$0\n```', module: "code" },
      { label: "Runnable JavaScript", snippet: "```js {run=true}\nconsole.log($SEL$0\"Hello\");\n```", module: "code" },
      { label: "Terminal session", snippet: '```terminal {title="PowerShell"}\nPS C:\\> $0\n```', module: "code" },
      { label: "Diff", snippet: "```diff\n- old line\n+ new line$0\n```", module: "code" },
      { label: "Code in tabs", snippet: ':::: tabs\n::: tab {title="Python"}\n```python\n$0\n```\n:::\n::: tab {title="JavaScript"}\n```js\n\n```\n:::\n::::', module: "layout" },
    ],
  },
  {
    name: "Diagrams & charts",
    items: [
      { label: "Flowchart", snippet: "```mermaid\nflowchart LR\n  A[Start] --> B{Decision}\n  B -->|Yes| C[Do it]\n  B -->|No| D[Skip]$0\n```", module: "diagrams" },
      { label: "Sequence diagram", snippet: "```mermaid\nsequenceDiagram\n  Reader->>Book: Opens\n  Book-->>Reader: Shows chapter$0\n```", module: "diagrams" },
      { label: "Mind map", snippet: "```mermaid\nmindmap\n  root((Topic))\n    Idea A\n    Idea B$0\n```", module: "diagrams" },
      { label: "Timeline diagram", snippet: "```mermaid\ntimeline\n  1450 : Printing press\n  1971 : First e-book$0\n```", module: "diagrams" },
      { label: "Bar chart", snippet: '```chart {type=bar title="Title" y="Units"}\nLabel, Series A, Series B\nOne, 4, 6\nTwo, 7, 5$0\n```', module: "charts" },
      { label: "Line chart", snippet: '```chart {type=line title="Title" x="X" y="Y"}\nx, y\n0, 0\n1, 2\n2, 3$0\n```', module: "charts" },
      { label: "Area chart", snippet: '```chart {type=area}\nx, Value\nA, 3\nB, 5\nC, 4$0\n```', module: "charts" },
      { label: "Pie / donut chart", snippet: '```chart {type=donut title="Share"}\nPart, Value\nA, 40\nB, 35\nC, 25$0\n```', module: "charts" },
      { label: "Horizontal bars", snippet: '```chart {type=hbar}\nItem, Value\nFirst, 12\nSecond, 8$0\n```', module: "charts" },
      { label: "Canvas drawing", snippet: '```canvas {caption="Caption"}\nsize 600 260\nrect 40 60 160 90 fill=soft stroke=accent radius=12 label="Box"\narrow 200 105 340 105\ncircle 420 105 50 fill=accent label="Circle" color=paper$0\n```', module: "canvas" },
      { label: "Physics diagram", snippet: '```canvas {caption="Forces"}\nsize 480 300\ngrid 20\naxes 40 260 400 220 x="x" y="y"\nrect 200 120 80 60 fill=soft stroke=ink label="m"\narrow 240 150 240 250 stroke=red width=3\ntext 250 240 "mg" fill=red\narrow 240 150 360 150 stroke=blue width=3\ntext 340 140 "F" fill=blue$0\n```', module: "canvas" },
    ],
  },
  {
    name: "Media",
    items: [
      { label: "Image", snippet: '![Description](assets/$SEL$0 "Caption")' },
      { label: "Figure with credit", snippet: '::: figure {src="assets/$0" caption="Caption" credit="Photo: Name"}\n:::', module: "media" },
      { label: "Gallery", snippet: "::: gallery {columns=3}\n![](assets/$0) ![](assets/) ![](assets/)\n:::", module: "media" },
      { label: "Before / after", snippet: '::: compare {before="assets/$0" after="assets/"}\nCaption\n:::', module: "media" },
      { label: "Audio", snippet: '::: audio {src="assets/$0" title="Listen"}\nTranscript or caption.\n:::', module: "media" },
      { label: "Video", snippet: '::: video {src="assets/$0" poster="assets/"}\nCaption.\n:::', module: "media" },
      { label: "Attachment / PDF", snippet: '::: attachment {src="assets/$0" label="Open the worksheet"}\n:::', module: "media" },
    ],
  },
  {
    name: "Learning",
    items: [
      { label: "Quiz (one answer)", snippet: '::: quiz {title="Quick check"}\n$0Question?\n\n- [ ] Wrong\n- [x] Right\n- [ ] Wrong\n\n> Explanation shown after answering.\n:::', module: "learning" },
      { label: "Quiz (several answers)", snippet: "::: quiz {multiple=true}\n$0Pick all that apply.\n\n- [x] Right\n- [x] Also right\n- [ ] Wrong\n\n> Explanation.\n:::", module: "learning" },
      { label: "Exercise with solution", snippet: '::: exercise {title="Title" difficulty=medium}\n$0Task.\n\n::: hint\nA hint.\n:::\n\n::: solution\nThe solution.\n:::\n:::', module: "learning" },
      { label: "Flashcards", snippet: "::: flashcards\nTerm\n\n: Meaning.$0\n\nTerm two\n\n: Meaning two.\n:::", module: "learning" },
      { label: "Steps", snippet: "::: steps\n1. $0First step.\n2. Second step.\n3. Third step.\n:::" },
      { label: "Timeline", snippet: "::: timeline\n- **1900** — $0Event\n- **1950** — Event\n:::" },
      { label: "Glossary", snippet: "::: glossary\nTerm\n\n: Definition.$0\n:::" },
    ],
  },
  {
    name: "Advanced",
    items: [
      { label: "Interactive widget (HTML/JS)", snippet: '```widget {height=260 title="Widget"}\n<button onclick="this.textContent=Number(this.textContent)+1">0</button>$0\n```', module: "widgets" },
      { label: "Custom block (style with CSS)", snippet: "::: my-block\n$SEL$0\n:::" },
      { label: "Link to another chapter", snippet: "[$SEL$0link text](02-chapter.md)", inline: true },
      { label: "Heading with custom id", snippet: "## $SEL$0 {#my-id}" },
    ],
  },
];
