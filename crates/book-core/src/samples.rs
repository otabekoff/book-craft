//! Starter content for new projects: one chapter per block family, so authors
//! can see (and copy) every feature.

use anyhow::Result;
use image::{ImageBuffer, Rgba};

pub fn chapters(title: &str) -> Vec<(String, String)> {
    vec![
        ("chapters/01-welcome.md".into(), welcome(title)),
        ("chapters/02-stories.md".into(), STORIES.into()),
        ("chapters/03-science.md".into(), SCIENCE.into()),
        ("chapters/04-code.md".into(), CODE.into()),
        ("chapters/05-learning.md".into(), LEARNING.into()),
    ]
}

fn welcome(title: &str) -> String {
    WELCOME.replace("@@TITLE@@", title)
}

const WELCOME: &str = r#"# Welcome to @@TITLE@@

This first chapter shows the everyday formatting tools. Edit or delete anything — it is your book now.

Write **bold**, *italic*, ~~struck~~, ==highlighted==, H~2~O and x^2^ text. Smart quotes turn "this" into “this”. Footnotes work too.[^1]

[^1]: Footnotes appear at the end of the chapter.

## Lists and tables

1. Numbered lists
2. With **formatting**
   - and nested items

| Format | Opens with | Best for |
|:-------|:----------:|---------:|
| Portable `.exe` | Double-click | Sharing a single file |
| Setup `.exe` | Installer wizard | Most readers |
| `.msi` | Windows Installer | Schools and companies |
| `.msix` | App Installer / Store | The Microsoft Store |

## Callouts

::: tip
Callouts highlight something the reader should not miss.
:::

::: warning {title="Before you build"}
Fill in **Book details** — the title, publisher and package ID end up inside the app.
:::

::: note {collapsible=true title="Click to expand"}
Any callout can be collapsible.
:::

> [!IMPORTANT]
> GitHub-style alerts work as well.

## Quotes

> A plain block quote uses the book's default quote style.

::: quote {style=pull author="Ursula K. Le Guin"}
The creative adult is the child who survived.
:::

::: quote {style=card author="A happy reader" role="Early reviewer"}
I read it in one sitting.
:::

## Boxes and shapes

::: box {style=shaded title="Shaded box"}
Use boxes for side material.
:::

:::: columns {count=3}
::: box {style=framed shape=rounded align=center}
Rounded
:::
::: box {style=dashed shape=square align=center}
Dashed
:::
::: box {style=filled shape=pill align=center color=teal}
Pill
:::
::::

## Tabs and details

:::: tabs
::: tab {title="Windows"}
Press **Win + R**.
:::
::: tab {title="Keyboard"}
Press **Ctrl + Shift + Esc**.
:::
::::

::: details {summary="What is a directive?"}
A block that starts with `:::` and a name. Add options in `{ }`.
:::
"#;

const STORIES: &str = r#"# Stories and Poetry

::: epigraph {author="Anonymous" source="Harbour song"}
The road goes on, and so must we.
:::

The fishing boats came in before the light did. Mara heard them from the attic — the knock of hulls against the stone, the low voices of people who had been awake all night.

She dressed without lighting the lamp. Her grandmother's ledger lay on the table, its cover warped by years of sea air.

***

By the time she reached the quay, the gulls had already found the morning's catch.

## Verse

::: verse {title="Low Tide" author="M. Wren"}
The water leaves the harbour slow,
as if it had somewhere else to go;
the boats lie down upon their sides
and wait for the returning tides.
:::

## Dialogue

::: dialogue
**MARA:** You knew her?

**INNKEEPER:** Everyone on this road knew her. She owed me a story.

**MARA:** Then I suppose I'll have to do.
:::

## A letter

::: letter
*Wren Harbour, 3 March*

Dear Mara,

If you are reading this, the ledger has found you. Take the inland road.

With love,
Gran
:::

## Asides

::: aside {title="About the salt road"}
Salt was carried inland on foot and by cart for centuries. Many routes are now walking trails.
:::
"#;

const SCIENCE: &str = r#"# Math, Physics and Chemistry

Inline math sits in text: the energy of a photon is $E = h\nu$, and $\sqrt{a^2 + b^2}$ is a hypotenuse.

Display math stands on its own line:

$$
\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}
$$

Give an equation an id to number it:

```math {#eq-newton}
\vec{F} = m\,\vec{a}
```

Equation (see [Newton's second law](#eq-newton)) is numbered automatically.

## Theorems and proofs

::: theorem {title="Pythagoras"}
In a right triangle, $a^2 + b^2 = c^2$.
:::

::: proof
Arrange four copies of the triangle inside a square of side $a + b$ and compare areas. $\blacksquare$
:::

::: definition {title="Velocity"}
Velocity is the rate of change of position: $v = \dfrac{dx}{dt}$.
:::

## Units and chemistry

Units: $\pu{9.81 m/s^2}$ and $\pu{6.022e23 mol^-1}$.

```chem
2H2 + O2 -> 2H2O
CO2 + C -> 2CO
```

## A force diagram

```canvas {caption="Forces on a block on a ramp"}
size 560 300
grid 20
polygon "60,260 500,260 500,80" fill=soft stroke=muted
rect 300 150 70 50 fill=accent stroke=ink radius=4 rotate=-22 label="m" color=paper
arrow 335 175 335 265 stroke=red width=3
text 345 250 "mg" fill=red size=16 weight=bold
arrow 335 175 300 95 stroke=blue width=3
text 270 100 "N" fill=blue size=16 weight=bold
arrow 335 175 400 150 stroke=green width=3
text 405 145 "f" fill=green size=16 weight=bold
```

## Charts

```chart {type=line title="Distance over time" x="Time (s)" y="Distance (m)"}
t, Ball A, Ball B
0, 0, 0
1, 4.9, 3
2, 19.6, 12
3, 44.1, 27
4, 78.4, 48
```

```chart {type=bar title="Readers by format" y="Readers"}
Format, 2025, 2026
Portable, 120, 180
Setup, 340, 410
MSI, 80, 95
MSIX, 40, 130
```

```chart {type=donut title="Time spent per chapter"}
Chapter, Minutes
Stories, 18
Science, 32
Code, 25
Learning, 12
```
"#;

const CODE: &str = r#"# Programming

Inline code looks like `let x = 5;`. Code blocks are highlighted for over 200 languages.

```rust {title="src/main.rs" lines=true highlight="2"}
fn main() {
    let greeting = "Hello, reader!";
    println!("{greeting}");
}
```

```python {caption="A list comprehension"}
squares = [n * n for n in range(10) if n % 2 == 0]
print(squares)
```

## Runnable JavaScript

Readers can press **Run** — the code runs in a sandbox without access to their computer.

```js {run=true}
const words = ["salt", "road", "harbour"];
console.log(words.map(w => w.toUpperCase()).join(" · "));
console.log("Sum:", [1, 2, 3, 4].reduce((a, b) => a + b, 0));
```

## Tabs for several languages

:::: tabs
::: tab {title="Rust"}
```rust
let v: Vec<i32> = (1..=5).collect();
```
:::
::: tab {title="TypeScript"}
```ts
const v: number[] = [1, 2, 3, 4, 5];
```
:::
::: tab {title="C#"}
```csharp
var v = Enumerable.Range(1, 5).ToList();
```
:::
::::

## Terminal sessions

```terminal {title="PowerShell"}
PS C:\book> cargo build --release
   Compiling book-reader v0.1.0
    Finished release [optimized] target(s)
PS C:\book> .\target\release\book-reader.exe
```

## Diffs

```diff
- let total = price * count;
+ let total = price * count as f64;
```

## Diagrams

```mermaid {caption="How a book is built"}
flowchart LR
  A[Markdown chapters] --> B[Book Craft]
  B --> C[book.pak]
  C --> D[Portable .exe]
  C --> E[Setup .exe]
  C --> F[.msi]
  C --> G[.msix]
```
"#;

const LEARNING: &str = r#"# Learning Tools

::: objectives
- Check your understanding with quizzes
- Practise with exercises and hidden solutions
- Revise with flashcards
:::

## Quiz

::: quiz {title="Quick check"}
Which format is best for the Microsoft Store?

- [ ] Portable .exe
- [ ] .msi
- [x] .msix

> MSIX packages are what the Store distributes and updates.
:::

## Exercise

::: exercise {title="Falling ball" difficulty=easy}
A ball falls freely for $2\,\text{s}$. How far does it fall? Use $g = 9.8\,\text{m/s}^2$.

::: hint
Use $d = \tfrac{1}{2} g t^2$.
:::

::: solution
$d = \tfrac{1}{2} \cdot 9.8 \cdot 2^2 = 19.6\,\text{m}$.
:::
:::

## Flashcards

::: flashcards
Velocity

: The rate of change of position.

Acceleration

: The rate of change of velocity.

Momentum

: Mass times velocity, $p = mv$.
:::

## Steps

::: steps
1. Write your chapters in Markdown.
2. Fill in the book details and pick an icon.
3. Choose the packages you want and press **Build**.
:::

## Timeline

::: timeline
- **1450** — Movable type printing in Europe
- **1971** — The first e-book is typed up
- **2026** — You build your own book app
:::

::: summary
Quizzes, exercises, flashcards, steps and timelines are all plain Markdown inside a `:::` block.
:::
"#;

/// A simple book glyph on the accent colour, 512×512 PNG.
pub fn default_icon_png(accent: &str) -> Result<Vec<u8>> {
    let (r, g, b) = parse_hex(accent).unwrap_or((0x2f, 0x5a, 0x8a));
    let size = 512u32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(size, size);
    let radius = 96.0;
    for (x, y, px) in img.enumerate_pixels_mut() {
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        let inside = rounded_rect_alpha(fx, fy, 16.0, 16.0, 496.0, 496.0, radius);
        if inside <= 0.0 {
            *px = Rgba([0, 0, 0, 0]);
            continue;
        }
        let mut c = [r, g, b];
        // Two open pages.
        let page_l = rounded_rect_alpha(fx, fy, 112.0, 150.0, 252.0, 370.0, 18.0);
        let page_r = rounded_rect_alpha(fx, fy, 260.0, 150.0, 400.0, 370.0, 18.0);
        let page = page_l.max(page_r);
        if page > 0.0 {
            for (i, ch) in c.iter_mut().enumerate() {
                let white = [250u8, 249, 245][i] as f32;
                *ch = (*ch as f32 * (1.0 - page) + white * page) as u8;
            }
            // Text lines on the pages.
            let on_line = ((fy - 190.0) % 34.0) < 8.0 && fy > 186.0 && fy < 340.0;
            let in_l = fx > 140.0 && fx < 228.0;
            let in_r = fx > 284.0 && fx < 372.0;
            if on_line && (in_l || in_r) {
                c = [
                    ((r as u16 + 250) / 2) as u8,
                    ((g as u16 + 249) / 2) as u8,
                    ((b as u16 + 245) / 2) as u8,
                ];
            }
        }
        *px = Rgba([c[0], c[1], c[2], (inside * 255.0) as u8]);
    }
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)?;
    Ok(out)
}

fn rounded_rect_alpha(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> f32 {
    let cx = x.clamp(x0 + r, x1 - r);
    let cy = y.clamp(y0 + r, y1 - r);
    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    (r - d + 0.5).clamp(0.0, 1.0)
}

pub fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let h = s.trim().trim_start_matches('#');
    let h = if h.len() == 3 { h.chars().flat_map(|c| [c, c]).collect::<String>() } else { h.to_string() };
    if h.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&h[0..2], 16).ok()?,
        u8::from_str_radix(&h[2..4], 16).ok()?,
        u8::from_str_radix(&h[4..6], 16).ok()?,
    ))
}
