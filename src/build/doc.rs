use html_parser::{Element, Node};
use mdbook::book::Chapter;
use pdf_canvas::graphicsstate::Color;
use pdf_canvas::{BuiltinFont, Canvas, FontRef, Pdf};
use pulldown_cmark::{Event::{self, Start, Text}, Parser};
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::CowStr::Borrowed;
use pulldown_cmark::Tag::CodeBlock;
use crate::build::highlight::highlight;
use crate::build::util::split_width;
use crate::config::{FontSize, PageMargins, PageOpts};

pub fn chapter(chapter: &Chapter, document: &mut Pdf, page: &PageOpts, font_sizes: &FontSize, hl: bool) -> std::io::Result<()> {
	let width = page.size.width(page.landscape);
	let height = page.size.height(page.landscape);
	// preprocessing code blocks
	enum Token<'a> { Event(Event<'a>), Code(Vec<(String, Option<Color>)>) }
	let mut html_raw = String::new();
	pulldown_cmark::html::push_html(&mut html_raw, Parser::new(&*chapter.content));
	let tokens = html_parser::Dom::parse(&*html_raw).unwrap().children;
	// while let Some(e) = raw_tokens.next() {
	// 	if let Start(CodeBlock(Fenced(Borrowed(lang)))) = e {
	// 		let inner = if let Some(Text(Borrowed(inner))) = raw_tokens.next() { inner } else { unreachable!() };
	// 		if hl {
	// 			tokens.push(Token::Code(highlight(lang, inner)));
	// 		} else {
	// 			tokens.push(Token::Code(vec![(inner.to_string(), None)]))
	// 		}
	// 		raw_tokens.next();
	// 	} else {
	// 		tokens.push(Token::Event(e));
	// 	}
	// }
	// // building chapter to pages
	document.render_page(width, height, |c| {
		for token in tokens {
			match token {
				Node::Text(_) => {}
				Node::Element(e) => {
					match &*e.name {
						"p" => paragraph(e.children, c, font_sizes.text, width - 2.0 * page.margin.x, height - 50.0, &page.margin)?,
						_ => {}
					}
				}
				Node::Comment(_) => {}
			}
		}
		Ok(())
	})
}

fn paragraph(children: Vec<Node>, c: &mut Canvas, fsize: f32, width: f32, mut y: f32, margin: &PageMargins) -> std::io::Result<()> {
	let mut text = Vec::new();
	let mut widths = Vec::new();
	let mut fonts = Vec::new();
	fn child_parse(children: Vec<Node>, mut font: BuiltinFont, c: &mut Canvas, size: f32, t: &mut Vec<String>, w: &mut Vec<f32>, f: &mut Vec<(BuiltinFont, f32)>) {
		for i in children {
			match i {
				Node::Text(text) => {
					for word in text.split_whitespace() {
						t.push(word.to_string());
						w.push(c.get_font(font.clone()).get_width(size, word));
						f.push((font.clone(), c.get_font(font.clone()).get_width(size, " ")));
					}
				}
				Node::Element(e) => {
					child_parse(
						e.children,
						match &*e.name {
							"code" => BuiltinFont::Courier,
							"strong" => match font {
								BuiltinFont::Courier => BuiltinFont::Courier_Bold,
								BuiltinFont::Courier_Oblique => BuiltinFont::Courier_BoldOblique,
								BuiltinFont::Helvetica_Oblique => BuiltinFont::Helvetica_BoldOblique,
								_ => BuiltinFont::Helvetica_Bold
							}
							"em" => match font {
								BuiltinFont::Courier => BuiltinFont::Courier_Oblique,
								BuiltinFont::Courier_Bold => BuiltinFont::Courier_BoldOblique,
								BuiltinFont::Helvetica_Bold => BuiltinFont::Helvetica_BoldOblique,
								_ => BuiltinFont::Helvetica_Oblique
							}
							_ => font.clone()
						},
						c, size, t, w, f
					)
				}
				_ => {}
			}
		}
	}
	child_parse(children, BuiltinFont::Helvetica, c, fsize, &mut text, &mut widths, &mut fonts);
	let lines = split_width(width, text, widths, fonts);
	for line in lines {
		let mut w = margin.x;
		for (text, twidth, f) in line {
			c.left_text(w, y, f, fsize, &*text)?;
			w += twidth + c.get_font(f).get_width(fsize, " ");
		}
		y -= fsize + 2.0;
	}
	Ok(())
}
