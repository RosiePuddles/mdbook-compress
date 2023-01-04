use mdbook::book::Book;
use mdbook::BookItem::Chapter;
use mdbook::renderer::RenderContext;
use mdbook::utils::take_rustdoc_include_anchored_lines;
use pdf_canvas::graphicsstate::Color;
use pdf_canvas::{BuiltinFont, Canvas, Pdf};
use crate::build::doc::chapter;
use crate::build::util::split_width;
use crate::config::{Config, PageSize};

pub struct Generator {
    config: RenderContext,
    pdf_opts: Config,
    document: Pdf
}

impl Generator {
    pub fn new(rc: RenderContext, pdf_opts: Config) -> Self {
        let title = &*rc.config.book.title.clone().unwrap_or(String::new());
        let mut document = Pdf::create(&*format!("{}.pdf", title)).unwrap();
        document.set_title(title);
        Self { config: rc, pdf_opts, document }
    }
    
    pub fn build(mut self) -> Result<(), std::io::Error> {
        for i in self.config.book.iter() {
            if let Chapter(c) = i { chapter(
                c, &mut self.document,
                &self.pdf_opts.page, &self.pdf_opts.font_size,
                self.pdf_opts.highlight, self.config.root.clone()
            )? }
        }
        self.document.render_page(
            self.pdf_opts.page.size.width(self.pdf_opts.page.landscape),
            self.pdf_opts.page.size.height(self.pdf_opts.page.landscape),
            |c| {
                let a = c.get_font(BuiltinFont::Helvetica);
                c.center_text(
                    self.pdf_opts.page.size.width(self.pdf_opts.page.landscape) / 2.0,
                    self.pdf_opts.page.size.height(self.pdf_opts.page.landscape) - 20.0,
                    BuiltinFont::Helvetica,
                    self.pdf_opts.font_size.title,
                    &*self.config.config.book.title.clone().unwrap_or(String::new())
                )?;
                Ok(())
            }
        )?;
        self.document.finish()
    }
}