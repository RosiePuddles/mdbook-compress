use std::{
	collections::BTreeMap,
	fmt::{Debug, Formatter},
	iter::Peekable,
	slice::Iter,
	str::Chars,
};

use genpdf::{
	elements::{LinearLayout, Paragraph},
	style::{Style, StyledString},
};

use crate::build::sections::replace_reserved;

/// Simplified HTML token. Can either be raw text, or an element with children and classes
#[derive(Debug)]
pub(crate) enum Token {
	/// Raw text
	Text(String),
	/// Element (span) containing classes and children
	Element { classes: Vec<String>, children: Vec<Token> },
}

impl Token {
	/// Expands a token into a string and style which is pushed to an output vector
	pub(crate) fn expand(self, out_ref: &mut Vec<(String, Style)>, style: Style, style_map: &StyleElement) {
		match self {
			Token::Text(t) => out_ref.push((t, style)),
			Token::Element { classes, children } => {
				for child in children {
					child.expand(
						out_ref,
						style_map.get_style(classes.iter().map(|c| c.as_str())),
						style_map,
					)
				}
			}
		}
	}
}

/// Style selector struct. Mimics a tree structure with each branch having a default style
pub enum StyleElement {
	Parent {
		default: Style,
		children: BTreeMap<String, StyleElement>,
	},
	Child(Style),
}

impl StyleElement {
	/// Get a style for a given path
	pub fn get_style<'a, T: Iterator<Item = &'a str>>(&self, mut classes: T) -> Style {
		match self {
			StyleElement::Parent { default, children } => {
				if let Some(class) = classes.next() {
					if let Some(child) = children.get(class) {
						child.get_style(classes)
					} else {
						default.clone()
					}
				} else {
					default.clone()
				}
			}
			StyleElement::Child(s) => s.clone(),
		}
	}

	/// Insert a key into the style map
	pub fn insert(self, mut path: Iter<&str>, style: Style) -> Self {
		if let Some(next) = path.next() {
			if path.len() == 0 {
				match self {
					StyleElement::Parent { mut children, default } => {
						children.insert(next.to_string(), Self::Child(style));
						StyleElement::Parent { children, default }
					}
					StyleElement::Child(_) => Self::Child(style),
				}
			} else {
				match self {
					StyleElement::Parent { mut children, default } => {
						if let Some(continuation) = children.remove(next.clone()) {
							children.insert(next.to_string(), continuation.insert(path, style));
						} else {
							let mut new_child = Self::Child(style);
							new_child = new_child.insert(path, style);
							children.insert(next.to_string(), new_child);
						}
						StyleElement::Parent { children, default }
					}
					StyleElement::Child(s) => {
						let mut new_child = Self::Child(s.clone());
						new_child = new_child.insert(path, style);
						Self::Parent {
							default: s.clone(),
							children: BTreeMap::from([(next.to_string(), new_child)]),
						}
					}
				}
			}
		} else {
			self
		}
	}
}

impl Debug for StyleElement {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let w = f.width().unwrap_or(0) + 1;
		match self {
			StyleElement::Parent { default, children } => {
				writeln!(f, "{:?}", default)?;
				for (name, child) in children {
					write!(f, "{}{}: {:>w$?}", " ".repeat(w - 1), name, child)?
				}
				Ok(())
			}
			StyleElement::Child(s) => writeln!(f, "{:?}", s),
		}
	}
}

/// A bad HTML parser that makes a lot of assumptions. Assumes only <span> tags or text
fn parse_html(src: String, map: fn(String) -> Vec<String>) -> Vec<Token> {
	let mut iter = src.chars().peekable();
	fn inner(i: &mut Peekable<Chars>, map: fn(String) -> Vec<String>) -> Vec<Token> {
		let mut out = Vec::new();
		loop {
			match i.next() {
				None => break,
				Some('<') => {
					// found a <span ...>
					for _ in 0..4 {
						i.next();
					}
					match i.next() {
						Some('>') => out.push(Token::Element {
							classes: Vec::new(),
							children: inner(i, map),
						}),
						Some('n') => break,
						_ => {
							// parse classes
							for _ in 0..7 {
								i.next();
							}
							let mut classes_raw = String::new();
							while let Some(c) = i.peek() {
								if c == &'"' {
									break
								}
								classes_raw.push(i.next().unwrap())
							}
							// skip closing ">
							for _ in 0..2 {
								i.next();
							}
							out.push(Token::Element {
								classes: map(classes_raw),
								children: inner(i, map),
							});
						}
					}
					// skip ending >
					i.next();
				}
				Some(c) => {
					let mut text = String::from(c);
					while let Some(c) = i.peek() {
						if c == &'<' {
							break
						}
						text.push(i.next().unwrap())
					}
					out.push(Token::Text(replace_reserved(text)))
				}
			}
		}
		out
	}
	inner(&mut iter, map)
}

pub fn to_block(raw: String, colour_map: StyleElement, f: fn(String) -> Vec<String>) -> LinearLayout {
	let tokens = parse_html(raw, f);
	let mut out = Vec::new();
	for child in tokens {
		child.expand(&mut out, Style::new(), &colour_map)
	}
	let mut block = LinearLayout::vertical();
	let mut line = Paragraph::new("");
	if out.is_empty() {
		return block
	}
	let mut last = out.pop().unwrap();
	last.0 = last.0.trim_end().to_string();
	for (words, style) in out {
		if words.contains("\n") {
			if &*words == "\n" {
				block.push(line);
				line = Paragraph::new("");
			} else {
				let push_end = words.chars().last() == Some('\n');
				let mut lines = words.split("\n");
				line.push(StyledString::new(lines.next().unwrap(), style));
				for section in lines {
					block.push(line);
					line = Paragraph::new(StyledString::new(section, style));
				}
				if push_end {
					block.push(line);
					line = Paragraph::new("");
				}
			}
		} else {
			line.push(StyledString::new(words, style))
		}
	}
	if last.0.contains("\n") {
		let mut lines = last.0.split("\n");
		line.push(StyledString::new(lines.next().unwrap(), last.1));
		for section in lines {
			block.push(line);
			line = Paragraph::new(StyledString::new(section, last.1));
		}
	} else {
		line.push(StyledString::new(last.0, last.1))
	}
	block.push(line);
	block
}
