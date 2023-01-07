# mdbook-compress

> **Work In Progress**
>
> Currently non-functional

An [mdBook](https://github.com/rust-lang/mdBook) backend renderer to generate a single PDF file for a full book.

There are other similar projects, but most rely on chrome in some way to generate a PDF. This project only optionally
requires Node.js to be installed for code block syntax highlighting. If you don't want highlighting or have Node.js
installed, you can specify in the config settings.

## Usage

To use this backend, you'll need to add the following to your `book.toml` file:

```toml
[output.compress]
```

> If you want to keep the default HTML output, you'll also need to add in `[output.html]` if it's not already there

The resulting PDF will end up in `/book/compress/<book-title>.pdf`.

### Config options

This is simply a filled out book config section for this backend with comments for annotation. All the values are the default values used if not otherwise specified.

```toml
[output.compress]
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
page.margin.x = 12
page.margin.y = 20
# Use Node.js to try and highlight code (true)
# or leave it unhighlighted (false)
highlight = true
```

### Custom page sizes

If you need a custom page size, you can give the width and height (`x` and `y`) dimensions in millimeters like this
```toml
page.size = { x = "width", y = "height" }
```

## Why does it take so long?

Because there's no way (that I know of) to call JS scripts quickly other that running a node command, building the PDF can take a bit of time.

If you know a faster way to do this, please open an issue or a PR.

## Dependencies

If you want to know what different dependencies are used for, here you go

| Dependency       | Version | Use                                           |
|------------------|---------|-----------------------------------------------|
| `serde`          | 1.0.152 | Config struct deserialisation                 |
| `mdbook`         | 0.4.25  | Getting mdbook config and some error printing |
| `genpdf`         | 0.2.0   | PDF building (really nice library btw)        |
| `anyhow`         | 1.0.68  | Error handling                                |
| `html_parser`    | 0.6.3   | Parsing HTML output from highlight.js         |
| `pulldown-cmark` | 0.9.2   | Markdown parsing                              |
