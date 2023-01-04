use mdbook::book::Chapter;
use pdf_canvas::graphicsstate::Color;
use pdf_canvas::Pdf;
use pulldown_cmark::{Event::{self, Start, Text}, Parser};
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::CowStr::Borrowed;
use pulldown_cmark::Tag::CodeBlock;
use crate::build::highlight::highlight;

pub fn chapter(chapter: &Chapter, document: &mut Pdf, width: f32, height: f32) -> std::io::Result<()> {
	// preprocessing code blocks
	enum Token<'a> { Event(Event<'a>), Code(Vec<(String, Option<Color>)>) }
	let mut raw_tokens = Parser::new(&*chapter.content);
	let mut tokens = Vec::new();
	while let Some(e) = raw_tokens.next() {
		if let Start(CodeBlock(Fenced(Borrowed(lang)))) = e {
			let inner = if let Some(Text(Borrowed(inner))) = raw_tokens.next() { inner } else { unreachable!() };
			tokens.push(Token::Code(highlight(lang, inner)));
			raw_tokens.next();
		} else {
			tokens.push(Token::Event(e));
		}
	}
	// building chapter to pages
	document.render_page(width, height, |c| {
		for token in tokens {
			match token {
				Token::Event(e) => {
					println!("{:?}", e)
				}
				Token::Code(c) => {}
			}
		}
		Ok(())
	})
}
