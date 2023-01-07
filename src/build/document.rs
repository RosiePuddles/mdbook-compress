use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use anyhow::Error;
use genpdf::{Alignment, Document, Element as _, elements, SimplePageDecorator, style};
use genpdf::elements::StyledElement;
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::Style;
use html_parser::Node::Element;
use mdbook::book::{BookItems, Chapter};
use mdbook::BookItem;
use mdbook::renderer::RenderContext;
// use crate::build::doc::chapter;
use crate::config::{Config, PageOpts};

pub struct Generator {
    config: RenderContext,
    pdf_opts: Config,
    document: Document,
    monospace: FontData,
    title: String
}

const HLJS: &'static str = include_str!("../../theme/hl.js");
const OPEN_SANS: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-regular.ttf");
const OPEN_SANS_BOLD: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-700.ttf");
const OPEN_SANS_BOLD_ITALIC: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-700italic.ttf");
const OPEN_SANS_ITALIC: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-italic.ttf");
const SOURCE_CODE_PRO: &[u8] = include_bytes!("../../theme/source-code-pro-v11-all-charsets-500.ttf");

impl Generator {
    /// Create a new PDF generator instance \
    /// This will initialise the PDF with only fonts and configs. [Generator::configure] is then
    /// called which will initialise the first page (title, subtitle, SUMMARY.md) and the page
    /// settings (decorator, size, etc.)
    pub fn new(rc: RenderContext, pdf_opts: Config) -> Result<Self, Error> {
        let fonts = FontFamily {
            regular: FontData::new(OPEN_SANS.to_vec(), None).unwrap(),
            bold: FontData::new(OPEN_SANS_BOLD.to_vec(), None).unwrap(),
            italic: FontData::new(OPEN_SANS_ITALIC.to_vec(), None).unwrap(),
            bold_italic: FontData::new(OPEN_SANS_BOLD_ITALIC.to_vec(), None).unwrap(),
        };
        let title = rc.config.book.title.clone().unwrap_or(String::new());
        Ok(Self {
            config: rc, pdf_opts,
            document: Document::new(fonts),
            monospace: FontData::new(SOURCE_CODE_PRO.to_vec(), None).unwrap(),
            title
        }.configure())
    }
    
    fn configure(mut self) -> Self {
        self.document.set_title(self.title.clone());
        self.document.set_minimal_conformance();
        self.document.set_line_spacing(self.pdf_opts.page.spacing.line);
        self.document.set_paper_size(self.pdf_opts.page.size.size(self.pdf_opts.page.landscape));
        let mut decorator = SimplePageDecorator::new();
        decorator.set_header(|p| {
            let mut layout = elements::LinearLayout::vertical();
            if p > 1 {
                layout.push(
                    elements::Paragraph::new(p.to_string()).aligned(Alignment::Center),
                );
                layout.push(elements::Break::new(1));
            }
            layout.styled(Style::new().with_font_size(10))
        });
        decorator.set_margins(self.pdf_opts.page.spacing.margin);
        self.document.set_page_decorator(decorator);
        self.document.push(
            elements::Paragraph::new(self.title.clone())
                .aligned(Alignment::Center)
                .styled(Style::new().bold().with_font_size(self.pdf_opts.font_size.title))
        );
        if let Some(subtitle) = &self.pdf_opts.subtitle {
            self.document.push(
                elements::Paragraph::new(subtitle)
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_font_size(self.pdf_opts.font_size.h4))
            );
        }
        let mut chapter_map = Vec::new();
        let mut contents = elements::OrderedList::new();
        for i in self.config.book.sections.iter() {
            match i {
                BookItem::Chapter(c) => {
                    chapter_map.push(ChapterMap::new(c));
                }
                _ => {}
            }
        }
        for chapter in chapter_map { chapter.to_list(self.pdf_opts.font_size.text, &mut contents)}
        self.document.push(contents.styled(Style::new().with_font_size(self.pdf_opts.font_size.text)));
        self
    }
    
    pub fn build(mut self) -> Result<(), genpdf::error::Error> {
        let mut hl = None;
        if self.pdf_opts.highlight {
            hl = if let Ok(custom) = std::fs::read_to_string(self.config.root.join("theme").join("highlight.js")) {
                Some(custom)
            } else {
                Some(HLJS.to_string())
            }
        }
        // let font = self.document.add_external_font(OPEN_SANS)?;
        // for i in self.config.book.iter() {
        //     if let Chapter(c) = i { chapter(
        //         c, &mut self.document,
        //         &self.pdf_opts.page, &self.pdf_opts.font_size, &hl
        //     )? }
        // }
        // self.document.render_page(
        //     self.pdf_opts.page.size.width(self.pdf_opts.page.landscape),
        //     self.pdf_opts.page.size.height(self.pdf_opts.page.landscape),
        //     |c| {
        //         // c.center_text(
        //         //     self.pdf_opts.page.size.width(self.pdf_opts.page.landscape) / 2.0,
        //         //     self.pdf_opts.page.size.height(self.pdf_opts.page.landscape) - 20.0,
        //         //     BuiltinFont::Helvetica,
        //         //     self.pdf_opts.font_size.title,
        //         //     &*self.config.config.book.title.clone().unwrap_or(String::new())
        //         // )?;
        //         let style = BTreeMap::from([
        //             ("hljs-comment", Color::gray(87)),
        //             ("hljs-quote", Color::gray(87)),
        //             ("hljs-variable", Color::rgb(215, 0, 37)),
        //             ("hljs-template-variable", Color::rgb(215, 0, 37)),
        //             ("hljs-tag", Color::rgb(215, 0, 37)),
        //             ("hljs-attribute", Color::rgb(215, 0, 37)),
        //             ("hljs-name", Color::rgb(215, 0, 37)),
        //             ("hljs-regexp", Color::rgb(215, 0, 37)),
        //             ("hljs-link", Color::rgb(215, 0, 37)),
        //             ("hljs-name", Color::rgb(215, 0, 37)),
        //             ("hljs-selector-id", Color::rgb(215, 0, 37)),
        //             ("hljs-selector-class", Color::rgb(215, 0, 37)),
        //             ("hljs-number", Color::rgb(178, 30, 0)),
        //             ("hljs-meta", Color::rgb(178, 30, 0)),
        //             ("hljs-built_in", Color::rgb(178, 30, 0)),
        //             ("hljs-builtin-name", Color::rgb(178, 30, 0)),
        //             ("hljs-literal", Color::rgb(178, 30, 0)),
        //             ("hljs-type", Color::rgb(178, 30, 0)),
        //             ("hljs-params", Color::rgb(178, 30, 0)),
        //             ("hljs-string", Color::rgb(0, 130, 0)),
        //             ("hljs-symbol", Color::rgb(0, 130, 0)),
        //             ("hljs-bullet", Color::rgb(0, 130, 0)),
        //             ("hljs-title", Color::rgb(0, 48, 242)),
        //             ("hljs-section", Color::rgb(0, 48, 242)),
        //             ("hljs-keyword", Color::rgb(157, 0, 236)),
        //             ("hljs-selector-tag", Color::rgb(157, 0, 236)),
        //             ("hljs-addition", Color::rgb(34, 134, 58)),
        //             ("hljs-deletion", Color::rgb(179, 29, 40))
        //         ]);
        //         let mut w = self.pdf_opts.page.size.height(self.pdf_opts.page.landscape) - 30.0;
        //         let font_ref = c.get_font(BuiltinFont::Helvetica);
        //         for (k, v) in style {
        //             c.text(|f| {
        //                 f.set_fill_color(v.clone())?;
        //                 f.set_font(&font_ref, self.pdf_opts.font_size.text)?;
        //                 f.pos(self.pdf_opts.page.margin.x, w)?;
        //                 f.show(k)
        //             })?;
        //             c.text(|f| {
        //                 // f.set_fill_color(Color::gray(0))?;
        //                 f.set_font(&font_ref, self.pdf_opts.font_size.text)?;
        //                 f.pos(self.pdf_opts.page.margin.x + 50.0, w)?;
        //                 f.show(k)
        //             })?;
        //             c.text(|f| {
        //                 f.set_font(&font_ref, self.pdf_opts.font_size.text)?;
        //                 f.pos(self.pdf_opts.page.margin.x + 100.0, w)?;
        //                 f.show(&*format!("{:?}", v))
        //             })?;
        //             w -= self.pdf_opts.font_size.text + 2.0;
        //         }
        //         Ok(())
        //     }
        // )?;
        self.document.render(File::create(format!("{}.pdf", self.title)).unwrap())
    }
}

/// Local chapter map representation
enum ChapterMap {
    Branch (String, Vec<ChapterMap>),
    Leaf (String)
}

impl ChapterMap {
    /// Make a new chapter map
    pub fn new(c: &Chapter) -> Self {
        if c.sub_items.is_empty() { ChapterMap::Leaf(c.name.clone()) }
        else {
            ChapterMap::Branch(c.name.clone(), c.sub_items.iter().filter_map(|sc| {
                if let BookItem::Chapter(sc) = sc { Some(ChapterMap::new(sc)) } else { None }
            }).collect())
        }
    }
    
    /// Write the map to a set of nested lists
    pub fn to_list(self, fs: u8, parent: &mut elements::OrderedList) {
        match self {
            ChapterMap::Branch(name, children) => {
                let mut child_list = elements::OrderedList::new();
                for c in children { c.to_list(fs, &mut child_list)}
                let mut block = elements::LinearLayout::vertical();
                block.push(elements::Paragraph::new(name).styled(Style::new().with_font_size(fs)));
                block.push(child_list.styled(Style::new().with_font_size(fs)));
                parent.push(block.styled(Style::new().with_font_size(fs)));
            }
            ChapterMap::Leaf(name) => parent.push(
                elements::Paragraph::new(name)
                    .styled(Style::new().with_font_size(fs))
            )
        }
    }
}
