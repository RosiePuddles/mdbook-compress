use std::collections::{BTreeMap, HashSet};

use anyhow::Error;
use genpdf::{
	elements::{LinearLayout, Paragraph},
	style::{Color, Style},
};

use crate::highlight::util::{to_block, StyleElement};

/// Checks if the used highlight_.js script can highlight a specific language. Used before calling
/// [highlight] to check that the Js won't fail because it can't highlight the language
fn check_language(names: HashSet<String>, hl: &String) -> Option<String> {
	for name in names {
		if name.starts_with("language-") {
			match std::process::Command::new("node")
				.arg("-e")
				.arg(format!(
					"{};console.log(!!hljs.getLanguage({:?}))",
					hl,
					&name[9..name.len()]
				))
				.output()
			{
				Ok(out) => {
					let stdout = String::from_utf8(out.stdout).unwrap();
					if out.status.code() != Some(0) {
						println!(
							"Error in getting highlight_.js languages! {}",
							String::from_utf8(out.stderr).unwrap()
						);
						std::process::exit(1)
					}
					if stdout.trim() == "true" {
						return Some(name[9..name.len()].to_string())
					}
				}
				Err(e) => {
					mdbook::utils::log_backtrace(&Error::new(e));
					std::process::exit(1)
				}
			}
		}
	}
	None
}

/// Highlights a section of code using highlight_.js
pub fn highlight(classes: HashSet<String>, hl: &String, src: String) -> LinearLayout {
	if let Some(lang_name) = check_language(classes, hl) {
		let raw = match std::process::Command::new("node")
			.current_dir(std::env::current_dir().unwrap())
			.args([
				"-e",
				&*format!(
					"{};console.log(hljs.highlight('{}',{{language:'{}'}}).value)",
					hl,
					src.replace("'", "\\'")
						.replace("\"", "\\\"")
						.replace("\n", "\\n"),
					lang_name
				),
			])
			.output()
		{
			Ok(out) => {
				let stdout = String::from_utf8(out.stdout).unwrap();
				if out.status.code() != Some(0) {
					println!(
						"Error highlighting code! Using un-highlighted code. {}",
						String::from_utf8(out.stderr).unwrap()
					);
					src.to_string()
				} else {
					stdout
				}
			}
			Err(e) => {
				mdbook::utils::log_backtrace(&Error::new(e));
				std::process::exit(1)
			}
		};
		let colour_map = StyleElement::Parent {
			default: Style::new(),
			children: BTreeMap::from_iter(
				[
					("comment", Color::Greyscale(87)),
					("quote", Color::Greyscale(87)),
					("variable", Color::Rgb(215, 0, 37)),
					("template-variable", Color::Rgb(215, 0, 37)),
					("tag", Color::Rgb(215, 0, 37)),
					("attribute", Color::Rgb(215, 0, 37)),
					("name", Color::Rgb(215, 0, 37)),
					("regexp", Color::Rgb(215, 0, 37)),
					("link", Color::Rgb(215, 0, 37)),
					("name", Color::Rgb(215, 0, 37)),
					("selector-id", Color::Rgb(215, 0, 37)),
					("selector-class", Color::Rgb(215, 0, 37)),
					("number", Color::Rgb(178, 30, 0)),
					("meta", Color::Rgb(178, 30, 0)),
					("built_in", Color::Rgb(178, 30, 0)),
					("builtin-name", Color::Rgb(178, 30, 0)),
					("literal", Color::Rgb(178, 30, 0)),
					("type", Color::Rgb(178, 30, 0)),
					("params", Color::Rgb(178, 30, 0)),
					("string", Color::Rgb(0, 130, 0)),
					("symbol", Color::Rgb(0, 130, 0)),
					("bullet", Color::Rgb(0, 130, 0)),
					("title", Color::Rgb(0, 48, 242)),
					("section", Color::Rgb(0, 48, 242)),
					("keyword", Color::Rgb(157, 0, 236)),
					("selector-tag", Color::Rgb(157, 0, 236)),
					("addition", Color::Rgb(34, 134, 58)),
					("deletion", Color::Rgb(179, 29, 40)),
				]
				.map(|(n, c)| {
					(
						n.to_string(),
						StyleElement::Child(Style::new().with_color(c)),
					)
				}),
			),
		};
		to_block(raw, colour_map, |t| {
			let mut iter = t.split_whitespace().enumerate();
			let mut out = Vec::new();
			if let Some((_, f)) = iter.next() {
				out.push(f[5..f.len()].to_string())
			}
			for (l, f) in iter {
				out.push(f[..f.len() - l].to_string())
			}
			out
		})
	} else {
		let mut block = LinearLayout::vertical();
		for line in src.lines() {
			block.push(Paragraph::new(line))
		}
		block
	}
}
