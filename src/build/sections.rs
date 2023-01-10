use std::collections::HashSet;

use ego_tree::{iter::Children, NodeRef};
use genpdf::{elements, style::Style, Element};
use pulldown_cmark::Parser;
use scraper::{Html, Node};

use crate::{
	build::{document::HL, Generator},
	highlight,
};

pub fn replace_reserved(s: String) -> String {
	s.replace("&gt;", ">")
		.replace("&lt;", "<")
		.replace("&quot;", "\"")
		.replace("&#39;", "'")
		.replace("&amp;", "&")
}

impl Generator {
	/// Generate the PDF for a book chapter
	pub fn chapter(&mut self, chapter: &str, hl: &Option<HL>) {
		let mut html_raw = String::new();
		pulldown_cmark::html::push_html(
			&mut html_raw,
			Parser::new_ext(chapter, pulldown_cmark::Options::all()),
		);
		let fragment = Html::parse_fragment(&*html_raw);
		let tokens = fragment.root_element();
		let new = self.parse_children(
			tokens.children(),
			Style::new().with_font_size(self.pdf_opts.font_size.text),
			hl,
		);
		self.document.push(elements::PageBreak::new());
		self.document.push(new);
	}

	/// Main caller function
	fn parse_children(
		&mut self, children: Children<Node>, style: Style, hl: &Option<HL>,
	) -> elements::LinearLayout {
		let mut out = elements::LinearLayout::vertical();
		for child in children {
			match child.value() {
				Node::Element(e) => match e.name() {
					"h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
						let mut para = elements::Paragraph::new("");
						self.paragraph(
							child.children(),
							style.with_font_size(self.pdf_opts.font_size.get(e.name())),
							&mut para,
						);
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
					"table" => out.push(self.table(child.children(), style)),
					_ => {}
				},
				_ => {}
			}
		}
		out
	}

	/// Paragraph generation. Uses a [pdfgen::elements::Paragraph]
	fn paragraph(
		&mut self, children: Children<Node>, style: Style, parent: &mut elements::Paragraph,
	) {
		for child in children {
			match child.value() {
				Node::Text(t) => {
					parent.push_styled(replace_reserved(t.to_string().replace("\n", "")), style)
				}
				Node::Element(e) => match e.name() {
					"p" | "a" => self.paragraph(child.children(), style, parent),
					"strong" => self.paragraph(child.children(), style.bold(), parent),
					"em" => self.paragraph(child.children(), style.italic(), parent),
					"code" => self.paragraph(
						child.children(),
						style.with_font_family(self.monospace),
						parent,
					),
					_ => {}
				},
				_ => {}
			}
		}
	}

	/// Block generation. Uses a [pdfgen::elements::LinearLayout]
	fn block(
		&mut self, children: Children<Node>, style: Style, parent: &mut elements::LinearLayout,
	) {
		for child in children {
			macro_rules! para_call {
				($t: expr) => {{
					let mut para = elements::Paragraph::new("");
					self.paragraph(child.children(), $t, &mut para);
					parent.push(para)
				}};
			}
			match child.value() {
				Node::Text(t) => parent.push(
					elements::Paragraph::new(replace_reserved(t.to_string().replace("\n", "")))
						.styled(style),
				),
				Node::Element(e) => match e.name() {
					"p" | "a" => para_call!(style),
					"strong" => para_call!(style.bold()),
					"em" => para_call!(style.italic()),
					"code" => para_call!(style.with_font_family(self.monospace)),
					_ => {}
				},
				_ => {}
			}
		}
	}

	/// Code block generation
	fn code(
		&mut self, mut children: Children<Node>, parent: &mut elements::LinearLayout,
		hl: &Option<HL>,
	) {
		let mut src = None;
		let mut classes = HashSet::new();
		if let Some(node) = children.next() {
			if let Node::Element(e) = node.value() {
				classes = e
					.classes
					.iter()
					.map(|t| t.to_string())
					.collect::<HashSet<_>>()
			};
			if let Some(node) = node.children().next() {
				if let Node::Text(t) = node.value() {
					src = Some(t.to_string())
				}
			}
		}
		parent.push(
			if let Some(src) = src {
				if let Some(hl) = hl {
					match hl {
						HL::syntect(ss) => highlight::syntect::highlight(classes, src, ss),
						HL::highlight(hl_src) => highlight::node::highlight(classes, hl_src, src),
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
			}
			.styled(
				Style::from(self.monospace)
					.with_line_spacing(0.0)
					.with_font_size(self.pdf_opts.font_size.text),
			),
		);
	}

	/// Table generation
	fn table(
		&mut self, children: Children<Node>, style: Style,
	) -> elements::PaddedElement<elements::TableLayout> {
		let mut rows = Vec::new();
		let mut widths = Vec::new();
		// parse the table based on an expected structure
		for i in children {
			if let Node::Element(e) = i.value() {
				let row_style = if e.name() == "thead" {
					style.bold()
				} else {
					style
				};
				for child in i.children() {
					if let Node::Element(_) = child.value() {
						let mut row: Vec<Box<dyn Element>> = Vec::new();
						let mut row_widths = Vec::new();
						for t in child.children() {
							match t.value() {
								Node::Text(t) => {
									row.push(Box::new(
										elements::Paragraph::new(replace_reserved(t.to_string()))
											.styled(row_style)
											.padded((1, 2)),
									));
									row_widths.push(t.len());
								}
								Node::Element(_) => {
									if is_multiline(t.children().collect()) {
										let mut block = elements::LinearLayout::vertical();
										self.block(t.children(), row_style, &mut block);
										row.push(Box::new(block.padded((1, 2))));
									} else {
										let mut block = elements::Paragraph::new("");
										self.paragraph(t.children(), row_style, &mut block);
										row.push(Box::new(block.padded((1, 2))));
									}
									row_widths.push(count_string_length(t.children().collect()));
								}
								_ => {}
							}
						}
						rows.push(row);
						widths.push(row_widths);
					}
				}
			}
		}
		let width = rows.iter().map(|t| t.len()).max().unwrap_or(0);
		let widths = widths
			.iter()
			.map(|t| {
				let mut out = t.clone();
				out.extend(vec![0; width - t.len()]);
				out
			})
			.fold(vec![Vec::new(); width], |mut acc, elem| {
				for (i, t) in elem.iter().enumerate() {
					acc[i].push(*t)
				}
				acc
			});
		let mean_widths = widths
			.iter()
			.map(|t| t.iter().sum::<usize>() / t.len())
			.collect::<Vec<_>>();
		let max_width = *mean_widths.iter().max().unwrap_or(&1);
		let mut out = elements::TableLayout::new(
			mean_widths
				.iter()
				.map(|w| (3 * *w / max_width).max(1))
				.collect(),
		);
		#[allow(unused_must_use)]
		for mut row in rows {
			for _ in 0..width - row.len() {
				row.push(Box::new(elements::Paragraph::new("")))
			}
			// we don't need to check for an error because we already made sure that all the rows
			// are the same length
			out.push_row(row);
		}
		out.set_cell_decorator(elements::FrameCellDecorator::new(true, false, false));
		out.padded((2, 0))
	}
}

/// Checks if a section is multiline or not. Used by lists and tables when determining is they
/// should use a `LinearElement` or `Paragraph`
fn is_multiline(children: Vec<NodeRef<Node>>) -> bool {
	let mut out = false;
	for i in children {
		match i.value() {
			Node::Text(t) => out = t.to_string().contains("\n"),
			Node::Element(_) => out = is_multiline(i.children().collect()),
			_ => {}
		}
		if out {
			break
		}
	}
	out
}

/// Count the cumulative length of the strings contained under an element
fn count_string_length(children: Vec<NodeRef<Node>>) -> usize {
	let mut out = 0;
	for child in children {
		match child.value() {
			Node::Text(t) => out += t.len(),
			Node::Element(_) => out += count_string_length(child.children().collect()),
			_ => {}
		}
	}
	out
}

/// List macro. Both lists are almost identical, so we use a macro to generate the functions to
/// ensure concurrency between the functions
macro_rules! list {
	($name: ident, $t: ty) => {
		impl Generator {
			fn $name(
				&mut self, children: Children<Node>, style: Style,
				parent: &mut elements::LinearLayout,
			) {
				let mut out = <$t>::new();
				if is_multiline(
					children
						.clone()
						.filter_map(|t| {
							if let Node::Element(_) = t.value() {
								Some(t)
							} else {
								None
							}
						})
						.collect(),
				) {
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
		}
	};
}

list!(unordered_list, elements::UnorderedList);
list!(ordered_list, elements::OrderedList);
