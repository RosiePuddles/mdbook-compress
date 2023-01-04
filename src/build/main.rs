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

const MARGIN : f32 = 10.0;

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
                self.pdf_opts.page_size.width(self.pdf_opts.landscape),
                self.pdf_opts.page_size.height(self.pdf_opts.landscape)
            )? }
        }
        self.document.render_page(
            self.pdf_opts.page_size.width(self.pdf_opts.landscape),
            self.pdf_opts.page_size.height(self.pdf_opts.landscape),
            |c| {
                let a = c.get_font(BuiltinFont::Helvetica);
                c.center_text(
                    self.pdf_opts.page_size.width(self.pdf_opts.landscape) / 2.0,
                    self.pdf_opts.page_size.height(self.pdf_opts.landscape) - 20.0,
                    BuiltinFont::Helvetica,
                    self.pdf_opts.font_size.title,
                    &*self.config.config.book.title.clone().unwrap_or(String::new())
                )?;
                let check = split_width(
                    self.pdf_opts.page_size.width(self.pdf_opts.landscape) - MARGIN * 2.0,
                    &a, self.pdf_opts.font_size.text,
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Blandit volutpat maecenas volutpat blandit. Risus feugiat in ante metus dictum at tempor. Egestas integer eget aliquet nibh praesent tristique magna sit amet. Vestibulum sed arcu non odio euismod. Condimentum id venenatis a condimentum vitae. Vitae elementum curabitur vitae nunc sed. Egestas diam in arcu cursus euismod quis. Faucibus vitae aliquet nec ullamcorper sit amet risus. In tellus integer feugiat scelerisque varius morbi enim nunc. Parturient montes nascetur ridiculus mus mauris vitae ultricies leo integer. Sagittis aliquam malesuada bibendum arcu vitae elementum curabitur vitae.".to_string()
                );
                let mut check_iter = check.iter();
                c.text(|t| {
                    t.set_font(&a, self.pdf_opts.font_size.text)?;
                    t.set_leading(self.pdf_opts.font_size.text + 2.0)?;
                    t.pos(MARGIN, 200.0)?;
                    t.show(&*check_iter.next().unwrap())?;
                    for i in check_iter { t.show_line(&*i)?; }
                    Ok(())
                })?;
                Ok(())
            }
        )?;
        self.document.finish()
    }
}