use std::collections::BTreeMap;
use std::io::Bytes;
use std::path::PathBuf;
use anyhow::Error;
use html_parser::Node;
use pdf_canvas::graphicsstate::Color;

const HLJS: &'static str = include_str!("../theme/hl.js");

pub fn check_language(name: &str, mut theme_path: PathBuf) -> bool {
	theme_path.extend(["theme", "highlight.js"]);
	let hljs = if theme_path.exists() {
		std::fs::read_to_string(theme_path).unwrap_or(HLJS.to_string())
	} else { HLJS.to_string() };
	match std::process::Command::new("node")
		.arg("-e")
		.arg(format!("{};console.log(!!hljs.getLanguage({:?}))", hljs, name))
		.output() {
		Ok(out) => {
			let stdout = String::from_utf8(out.stdout).unwrap();
			if out.status.code() != Some(0) {
				println!("Error in getting highlight.js languages! {}", stdout);
				std::process::exit(1)
			}
			return stdout.trim() == "true"
		}
		Err(e) => {
			mdbook::utils::log_backtrace(&Error::new(e));
			std::process::exit(1)
		}
	}
}

pub fn highlight(language: &str, src: &str, mut theme_path: PathBuf) -> Vec<Vec<(String, Option<Color>)>> {
	theme_path.extend(["theme", "highlight.js"]);
	let hljs = if theme_path.exists() {
		std::fs::read_to_string(theme_path).unwrap_or(HLJS.to_string())
	} else { HLJS.to_string() };
	let raw = match std::process::Command::new("node")
		.current_dir(std::env::current_dir().unwrap())
		.args([
			"-e",
			&*format!(
				"{};console.log(hljs.highlight('{}',{{language:'{}'}}).value)", hljs,
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
	let style = BTreeMap::from([
		("comment".to_string(), Color::gray(87)),
		("quote".to_string(), Color::gray(87)),
		("variable".to_string(), Color::rgb(215, 0, 37)),
		("template-variable".to_string(), Color::rgb(215, 0, 37)),
		("tag".to_string(), Color::rgb(215, 0, 37)),
		("attribute".to_string(), Color::rgb(215, 0, 37)),
		("name".to_string(), Color::rgb(215, 0, 37)),
		("regexp".to_string(), Color::rgb(215, 0, 37)),
		("link".to_string(), Color::rgb(215, 0, 37)),
		("name".to_string(), Color::rgb(215, 0, 37)),
		("selector-id".to_string(), Color::rgb(215, 0, 37)),
		("selector-class".to_string(), Color::rgb(215, 0, 37)),
		("number".to_string(), Color::rgb(178, 30, 0)),
		("meta".to_string(), Color::rgb(178, 30, 0)),
		("built_in".to_string(), Color::rgb(178, 30, 0)),
		("builtin-name".to_string(), Color::rgb(178, 30, 0)),
		("literal".to_string(), Color::rgb(178, 30, 0)),
		("type".to_string(), Color::rgb(178, 30, 0)),
		("params".to_string(), Color::rgb(178, 30, 0)),
		("string".to_string(), Color::rgb(0, 130, 0)),
		("symbol".to_string(), Color::rgb(0, 130, 0)),
		("bullet".to_string(), Color::rgb(0, 130, 0)),
		("title".to_string(), Color::rgb(0, 48, 242)),
		("section".to_string(), Color::rgb(0, 48, 242)),
		("keyword".to_string(), Color::rgb(157, 0, 236)),
		("selector-tag".to_string(), Color::rgb(157, 0, 236)),
		("addition".to_string(), Color::rgb(34, 134, 58)),
		("deletion".to_string(), Color::rgb(179, 29, 40))
	]);
	let dom = html_parser::Dom::parse(&*raw).unwrap();
	let mut out = Vec::new();
	fn expand(elements: Vec<Node>, out_ref: &mut Vec<(String, Option<Color>)>, mut default_colour: Option<Color>, style: &BTreeMap<String, Color>) {
		for i in elements {
			match i {
				Node::Text(t) => out_ref.push((
					t.replace("&amp;", "&")
						.replace("&gt;", ">")
						.replace("&lt;", "<")
						.replace("&quot;", "\"")
						.replace("&#x27;", "'"),
					default_colour)),
				Node::Element(e) => {
					if let Some(c) = e.classes.last() {
						default_colour = match style.get(&c[5..c.len()].to_string()) {
							None => default_colour,
							Some(c) => Some(c.clone())
						}
					}
					expand(e.children, out_ref, default_colour, style)
				}
				_ => {}
			}
		}
	}
	expand(dom.children, &mut out, None, &style);
	let mut lines = Vec::new();
	let mut line_acc = Vec::new();
	for (s, c) in out {
		if s.contains("\n") {
			let mut section_acc = String::new();
			for ch in s.chars() {
				if ch == '\n' {
					line_acc.push((section_acc.clone(), c.clone()));
					lines.push(line_acc.clone());
					line_acc.clear();
					section_acc.clear();
				} else {
					section_acc.push(ch)
				}
			}
			line_acc.push((section_acc, c));
		} else {
			line_acc.push((s, c))
		}
	}
	lines.push(line_acc);
	lines
}
