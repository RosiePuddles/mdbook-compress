use std::collections::{BTreeMap, HashSet};
use genpdf::elements::{LinearLayout, Paragraph};
use genpdf::style::{Color, Style};
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassedHTMLGenerator, ClassStyle};
use syntect::util::LinesWithEndings;
use crate::highlight::util::{StyleElement, to_block};

pub fn highlight(classes: HashSet<String>, src: String) -> LinearLayout {
	let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
	let mut syntax = None;
	let mut block = LinearLayout::vertical();
	for class in classes.iter() {
		if class.starts_with("language-") {
			syntax = ss.syntaxes().iter().rev().find(|&s| class[9..class.len()] == s.name.to_lowercase());
			if syntax.is_some() { break }
		}
	}
	if let Some(syntax) = syntax {
		let mut parser = ClassedHTMLGenerator::new_with_class_style(syntax, &ss, ClassStyle::Spaced);
		for line in LinesWithEndings::from(&*src) {
			parser.parse_html_for_line_which_includes_newline(line).unwrap();
		}
		let mut colour_map = StyleElement::Parent { default: Style::new(), children: BTreeMap::new() };
		let theme = ThemeSet::load_defaults();
		for i in theme.themes["InspiredGitHub"].clone().scopes {
			let mut style = Style::new();
			if let Some(c) = i.style.foreground { style.set_color(Color::Rgb(c.r, c.g, c.b)) }
			if let Some(fs) = i.style.font_style {
				if (fs.bits() & 1) == 1 { style.set_bold() }
				if (fs.bits() & 4) == 4 { style.set_italic() }
			}
			for selector in i.scope.selectors {
				for scope in selector.path.scopes {
					let scope = scope.build_string();
					colour_map = colour_map.insert(scope.split(".").collect::<Vec<_>>().iter(), style);
				}
			}
		}
		to_block(parser.finalize(), colour_map, |t| t.split_whitespace().map(|l| l.to_string()).collect())
	} else {
		for line in src.lines() {
			block.push(Paragraph::new(line))
		}
		block
	}
}