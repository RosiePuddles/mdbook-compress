use std::collections::{BTreeMap, HashSet};
use std::iter::Peekable;
use std::path::PathBuf;
use std::str::Chars;
use anyhow::Error;
use genpdf::Element;
use genpdf::elements::{FramedElement, LinearLayout, PaddedElement, Paragraph, StyledElement};
use genpdf::style::{Color, Style, StyledString};
use crate::build::sections::replace_reserved;

pub fn check_language(name: &str, hl: &String) -> Option<String> {
	match std::process::Command::new("node")
		.arg("-e")
		.arg(format!("{};console.log(!!hljs.getLanguage({:?}))", hl, name))
		.output() {
		Ok(out) => {
			let stdout = String::from_utf8(out.stdout).unwrap();
			if out.status.code() != Some(0) {
				println!("Error in getting highlight.js languages! {}", String::from_utf8(out.stderr).unwrap());
				std::process::exit(1)
			}
			return if stdout.trim() == "true" { Some(name.to_string()) } else { None }
		}
		Err(e) => {
			mdbook::utils::log_backtrace(&Error::new(e));
			std::process::exit(1)
		}
	}
}

pub fn highlight(language: String, src: &str, hl: &String) -> LinearLayout{
	let raw = match std::process::Command::new("node")
		.current_dir(std::env::current_dir().unwrap())
		.args([
			"-e",
			&*format!(
				"{};console.log(hljs.highlight('{}',{{language:'{}'}}).value)", hl,
				src.replace("'", "\\'").replace("\"", "\\\"").replace("\n", "\\n"), language
			)
		]).output() {
		Ok(out) => {
			let stdout = String::from_utf8(out.stdout).unwrap();
			if out.status.code() != Some(0) {
				println!("Error highlighting code! Using un-highlighted code. {}", String::from_utf8(out.stderr).unwrap());
				src.to_string()
			} else { stdout }
		}
		Err(e) => {
			mdbook::utils::log_backtrace(&Error::new(e));
			std::process::exit(1)
		}
	};
	let colour_map = BTreeMap::from([
		("hljs-comment", Color::Greyscale(87)),
		("hljs-quote", Color::Greyscale(87)),
		("hljs-variable", Color::Rgb(215, 0, 37)),
		("hljs-template-variable", Color::Rgb(215, 0, 37)),
		("hljs-tag", Color::Rgb(215, 0, 37)),
		("hljs-attribute", Color::Rgb(215, 0, 37)),
		("hljs-name", Color::Rgb(215, 0, 37)),
		("hljs-regexp", Color::Rgb(215, 0, 37)),
		("hljs-link", Color::Rgb(215, 0, 37)),
		("hljs-name", Color::Rgb(215, 0, 37)),
		("hljs-selector-id", Color::Rgb(215, 0, 37)),
		("hljs-selector-class", Color::Rgb(215, 0, 37)),
		("hljs-number", Color::Rgb(178, 30, 0)),
		("hljs-meta", Color::Rgb(178, 30, 0)),
		("hljs-built_in", Color::Rgb(178, 30, 0)),
		("hljs-builtin-name", Color::Rgb(178, 30, 0)),
		("hljs-literal", Color::Rgb(178, 30, 0)),
		("hljs-type", Color::Rgb(178, 30, 0)),
		("hljs-params", Color::Rgb(178, 30, 0)),
		("hljs-string", Color::Rgb(0, 130, 0)),
		("hljs-symbol", Color::Rgb(0, 130, 0)),
		("hljs-bullet", Color::Rgb(0, 130, 0)),
		("hljs-title", Color::Rgb(0, 48, 242)),
		("hljs-section", Color::Rgb(0, 48, 242)),
		("hljs-keyword", Color::Rgb(157, 0, 236)),
		("hljs-selector-tag", Color::Rgb(157, 0, 236)),
		("hljs-addition", Color::Rgb(34, 134, 58)),
		("hljs-deletion", Color::Rgb(179, 29, 40))
	]);
	let tokens = parse_html(raw);
	let mut out = Vec::new();
	fn expand(elements: Vec<Token>, out_ref: &mut Vec<(String, Style)>, default_colour: Option<Color>, style: Style, colour_map: &BTreeMap<&str, Color>) {
		for i in elements {
			match i {
				Token::Text(t) => {
					let mut new_style = style;
					if let Some(c) = default_colour.clone() { new_style.set_color(c) }
					out_ref.push((t, new_style))
				},
				Token::Element {classes, children} => {
					let mut new_colour = default_colour.clone();
					for class in classes {
						if let Some(c) = colour_map.get(&*class) {
							new_colour = Some(c.clone())
						}
					}
					expand(children, out_ref, new_colour, style, colour_map)
				}
			}
		}
	}
	if &*language == "json" {
		for t in tokens.iter() { println!("{:?}", t) }
	}
	expand(tokens, &mut out, None, Style::new(), &colour_map);
	let mut block = LinearLayout::vertical();
	let mut line = Paragraph::new("");
	if out.is_empty() { return block }
	let mut last = out.pop().unwrap();
	last.0 = last.0.trim_end().to_string();
	for (words, style) in out {
		if words.contains("\n") {
			if &*words == "\n" {
				block.push(line);
				line = Paragraph::new("");
			} else {
				let mut push_end = words.chars().last() == Some('\n');
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

#[derive(Debug)]
enum Token {
	Text(String),
	Element {
		classes: HashSet<String>,
		children: Vec<Token>
	}
}

fn parse_html(src: String) -> Vec<Token> {
	let mut iter = src.chars().peekable();
	fn inner(i: &mut Peekable<Chars>, mut classes: HashSet<String>) -> Vec<Token> {
		let mut out = Vec::new();
		loop {
			match i.next() {
				None => { break }
				Some('<') => {
					// found a <span ...>
					for _ in 0..4 { i.next(); }
					match i.next() {
						Some('>') => out.push(Token::Element {classes: classes.clone(), children: inner(i, classes.clone()) }),
						Some('n') => break,
						_ => {
							// parse classes
							for _ in 0..7 { i.next(); }
							let mut classes_raw = String::new();
							while let Some(c) = i.peek() {
								if c == &'"' { break }
								classes_raw.push(i.next().unwrap())
							}
							// skip closing ">
							for _ in 0..2 { i.next(); }
							let mut new_classes = classes.clone();
							new_classes.extend(classes_raw.split_whitespace().map(|t| t.to_string()));
							out.push(Token::Element {classes: new_classes.clone(), children: inner(i, new_classes) });
						}
					}
					// skip ending >
					i.next();
				}
				Some(c) => {
					let mut text = String::from(c);
					while let Some(c) = i.peek() {
						if c == &'<' { break }
						text.push(i.next().unwrap())
					}
					out.push(Token::Text(replace_reserved(text)))
				}
			}
		}
		out
	}
	inner(&mut iter, HashSet::new())
}
