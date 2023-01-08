use std::collections::HashSet;
use ego_tree::iter::Children;
use ego_tree::NodeRef;
use genpdf::{Document, Element, elements};
use genpdf::fonts::{Font, FontCache, FontFamily};
use genpdf::style::{Color, Style, StyledString};
use pulldown_cmark::Parser;
use scraper::{Html, Node, Selector};
use serde::Serialize;
use crate::build::Generator;
use crate::build::highlight::{check_language, highlight};

pub fn replace_reserved(s: String) -> String {
	s.replace("&gt;", ">")
		.replace("&lt;", "<")
		.replace("&quot;", "\"")
		.replace("&#x27;", "'")
		.replace("&amp;", "&")
}

impl Generator {
	pub fn chapter(&mut self, chapter: &str, hl: &Option<String>) {
		let mut html_raw = String::new();
		pulldown_cmark::html::push_html(&mut html_raw, Parser::new_ext(chapter, pulldown_cmark::Options::all()));
		let fragment = Html::parse_fragment(&*html_raw);
		let tokens = fragment.root_element();
		let new = self.parse_children(tokens.children(), Style::new().with_font_size(self.pdf_opts.font_size.text), hl);
		self.document.push(elements::PageBreak::new());
		self.document.push(new);
	}
	
	fn parse_children(&mut self, children: Children<Node>, style: Style, hl: &Option<String>) -> elements::LinearLayout {
		let mut out = elements::LinearLayout::vertical();
		for child in children {
			match child.value() {
				Node::Element(e) => {
					match e.name() {
						"h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
							let mut para = elements::Paragraph::new("");
							self.paragraph(child.children(), style.with_font_size(self.pdf_opts.font_size.get(e.name())), &mut para);
							out.push(para.padded((0, 0, 3, 0)))
						}
						"p" => {
							let mut para = elements::Paragraph::new("");
							self.paragraph(child.children(), style, &mut para);
							out.push(para.padded((0, 0, 1, 0)))
						}
						"ol" => {
							let mut block = elements::LinearLayout::vertical();
							self.ordered_list(child.children(), style, &mut block);
							out.push(block)
						}
						"ul" => {
							let mut block = elements::LinearLayout::vertical();
							self.unordered_list(child.children(), style, &mut block);
							out.push(block)
						}
						"pre" => {
							let mut block = elements::LinearLayout::vertical();
							self.code(child.children(), &mut block, hl);
							out.push(block.padded((0, 0, 1, 0)))
						}
						_ => {}
					}
				}
				_ => {}
			}
		}
		out
	}
	
	fn paragraph(&mut self, children: Children<Node>, style: Style, parent: &mut elements::Paragraph) {
		for child in children {
			match child.value() {
				Node::Text(t) => parent.push_styled(replace_reserved(t.to_string().replace("\n", "")), style),
				Node::Element(e) => {
					match e.name() {
						"p" | "a" => self.paragraph(child.children(), style, parent),
						"strong" => self.paragraph(child.children(), style.bold(), parent),
						"em" => self.paragraph(child.children(), style.italic(), parent),
						"code" => self.paragraph(child.children(), style.with_font_family(self.monospace), parent),
						_ => {}
					}
				}
				_ => {}
			}
		}
	}
	
	fn block(&mut self, children: Children<Node>, style: Style, parent: &mut elements::LinearLayout) {
		for child in children {
			macro_rules! para_call {
				($t: expr) => {{
					let mut para = elements::Paragraph::new("");
					self.paragraph(child.children(), $t, &mut para);
					parent.push(para)
				}};
			}
			match child.value() {
				Node::Text(t) => parent.push(elements::Paragraph::new(replace_reserved(t.to_string().replace("\n", ""))).styled(style)),
				Node::Element(e) => {
					match e.name() {
						"p" | "a" => para_call!(style),
						"strong" => para_call!(style.bold()),
						"em" => para_call!(style.italic()),
						"code" => para_call!(style.with_font_family(self.monospace)),
						_ => {}
					}
				}
				_ => {}
			}
		}
	}
	
	fn code(&mut self, mut children: Children<Node>, parent: &mut elements::LinearLayout, hl: &Option<String>) {
		let mut src = None;
		let mut classes = HashSet::new();
		if let Some(mut node) = children.next() {
			if let Node::Element(e) = node.value() {
				classes = e.classes.clone()
			};
			if let Some(node) = node.children().next() {
				if let Node::Text(t) = node.value() { src = Some(t.to_string()) }
			}
		}
		parent.push(if let Some(src) = src {
			if let Some(hl) = hl {
				let mut highlightable = None;
				for class in classes {
					let class = class.to_string();
					if class.starts_with("language-") {
						highlightable = check_language(&class[9..class.len()], hl)
					}
					if highlightable.is_some() { break }
				}
				if let Some(lang) = highlightable {
					highlight(lang, &*replace_reserved(src), hl)
				} else {
					let mut block = elements::LinearLayout::vertical();
					let mut lines = src.lines();
					if let Some(line) = lines.next() {
						block.push(elements::Paragraph::new(line))
					}
					for line in lines {
						block.push(elements::Paragraph::new(line))
					}
					block
				}
			} else {
				let mut block = elements::LinearLayout::vertical();
				for line in src.lines() {
					block.push(elements::Paragraph::new(line))
				}
				block
			}
		} else {
			elements::LinearLayout::vertical()
		}.styled(
			Style::from(self.monospace)
				.with_line_spacing(0.0)
				.with_font_size(self.pdf_opts.font_size.text)
		));
	}
}

fn is_multiline(children: Vec<NodeRef<Node>>) -> bool {
	let mut out = false;
	for i in children {
		match i.value() {
			Node::Text(t) => out = t.to_string().contains("\n"),
			Node::Element(_) => out = is_multiline(i.children().collect()),
			_ => {}
		}
		if out { break }
	}
	out
}

macro_rules! list {
	($name: ident, $t: ty) => {
impl Generator {
	fn $name(&mut self, children: Children<Node>, style: Style, parent: &mut elements::LinearLayout) {
		let mut out = <$t>::new();
		if is_multiline(children.clone().filter_map(|t| if let Node::Element(_) = t.value() { Some(t) } else { None }).collect()) {
			for child in children {
				match child.value() {
					Node::Element(_) => {
						let mut block = elements::LinearLayout::vertical();
						self.block(child.children(), style, &mut block);
						out.push(block)
					}
					_ => {}
				}
			}
		} else {
			for child in children {
				match child.value() {
					Node::Element(_) => {
						let mut block = elements::Paragraph::new("");
						self.paragraph(child.children(), style, &mut block);
						out.push(block)
					}
					_ => {}
				}
			}
		}
		parent.push(out.styled(style))
	}
}};}

list!(unordered_list, elements::UnorderedList);
list!(ordered_list, elements::OrderedList);
