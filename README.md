# mdbook-compress

<a href="https://crates.io/crates/mdbook-compress" target="_blank"><img src="https://img.shields.io/crates/v/mdbook-compress.svg" alt="crate.io version badge"></a>

An [mdBook](https://github.com/rust-lang/mdBook) backend renderer to generate a single PDF file for a full book.

There are other similar projects, but most rely on chrome in some way to generate a PDF. This project only optionally
requires Node.js to be installed for code block syntax highlighting. If you don't want highlighting you can specify that with `highlight = "none"` in the config (or set `highlight = "no-node"` to use the built-in highlighter).

## Usage

To use this backend, you'll need to add the following to your `book.toml` file:

```toml
[output.compress]
```

and install this project

```bash
cargo install mdbook-compress
```

> If you want to keep the default HTML output, you'll also need to add in `[output.html]` if it's not already there

The resulting PDF will end up in `/book/compress/<book-title>.pdf`. If you want to have a look at an example PDF, you can have a look at [this one](https://github.com/heyitsdoodler/hbml/blob/main/docs/book/compress/HBML%20Tutorial.pdf) which is the whole reason this project exists in the first place.

### Config options

There are a few config options. They're all below and have a few comments to explain things. All the values are the default values.

```toml
[output.compress]
# You can optionally specify a subtitle. If you don't the PDF
# won't include a subtitle
subtitle = ""
# If you want to use custom fonts, specify them here.
# The value is a path relative to 'theme/fonts' under your book root
font.regular = ""
font.bold = ""
font.italic = ""
font.bold-italic = ""
font.monospace = ""
# Font sizes. Any heading after H6 will use the H6 font size
# All font sizes will become a u8
font_size.title = 12
font_size.h1 = 11
font_size.h2 = 10
# H3 is also used for the subtitle
font_size.h3 = 8
font_size.h4 = 7
font_size.h5 = 6
font_size.h6 = 6
font_size.text = 5
# Page configs
# Page size. One of: A4, US letter, US legal (see below for custom sizes)
page.size = "A4"
page.landscape = false
# Insert a page break between chapters (markdown files)
page.new_pages = false
# Line and margin spacing. Both measured in millimeters (f64 internally)
page.spacing.line = 1.5
page.spacing.heading = 2.0
page.spacing.margin = [20.0, 20.0]
# See the highlighting section below
highlight = "all"
```

### Custom page sizes

If you need a custom page size, you can give the width and height (`x` and `y`) dimensions in millimeters like this
```toml
page.size = { x = "width", y = "height" }
```

### Highlighting

Code highlighting with highlight.js (what mdbook uses for the HTML) is pretty slow because it requires calling a node command. To fix this, this project uses syntect to do any highlighting. However, if you specify a custom highlight.js script in the _themes_ directory of your book, the code will use that.

You can change this though. The `highlight` value of the config can be one of:
- `"all"` (default)\
  Use highlight.js file when given otherwise use syntect 
- `"no-node"`\
  Always use syntect even if a highlight.js file is given. In this case you can give `.sublime-syntax` files in your theme folder that will be used for highlighting. This way you can have a faster alternative to Node whilst keeping custom highlighting
- `"none"`\
  Don't do any highlighting

It's worth noting that the highlighting colours for syntect and highlight.js are different because they're different programs

If you use syntect, you can provide a custom `theme.tmtheme` file in your theme directory. If this is a valid theme, that'll get used for highlighting. If not, the theme `base16-ocean.light` is used instead.

## Why does it take so long?

If you're using a custom highlight.js file, this might make the renderer a bit slow. This is due to having to call Node.js for each code block. You should only use this if you require highlighting a language not supported by syntect.

## Things still to add

- Images (This is not possible with `genpdf`... at the moment)
- Custom highlight.js theme application (Can have a custom syntect theme)

## Dependencies

If you want to know what different dependencies are used for, here you go. The descriptions are all a bit general, because anything more specific would make the table too big.

| Dependency                                                        | Version | Use                                                |
|-------------------------------------------------------------------|---------|----------------------------------------------------|
| [`serde`](https://crates.io/crates/serde/1.0.152)                 | 1.0.152 | Config struct deserialisation                      |
| [`mdbook`](https://crates.io/crates/mdbook/0.4.25)                | 0.4.25  | Getting mdbook config and some error printing      |
| [`genpdf`](https://crates.io/crates/genpdf/0.2.0)                 | 0.2.0   | PDF building (really nice library btw)             |
| [`anyhow`](https://crates.io/crates/anyhow/1.0.68)                | 1.0.68  | Error handling                                     |
| [`scraper`](https://crates.io/crates/scraper/0.14.0)              | 0.14.0  | Parsing HTML outputs                               |
| [`ego-tree`](https://crates.io/crates/ego-tree/0.6.2)             | 0.6.2   | Required for function call types when highlighting |
| [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark/0.9.2) | 0.9.2   | Markdown parsing                                   |
| [`syntect`](https://crates.io/crates/syntect/0.5.0)               | 0.5.0   | Built-in code highlighting                         |
