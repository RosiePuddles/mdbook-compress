use std::collections::{BTreeMap, HashSet};

use genpdf::{
	elements::{LinearLayout, Paragraph},
	style::{Color, Style},
};
use syntect::{
	highlighting::{Theme, ThemeSet},
	html::{ClassStyle, ClassedHTMLGenerator},
	parsing::SyntaxSet,
	util::LinesWithEndings,
};

use crate::highlight::util::{to_block, StyleElement};

pub fn highlight(
	classes: HashSet<String>, src: String, provided_syntaxes: &SyntaxSet, given_theme: &Option<Theme>,
) -> LinearLayout {
	let ss = SyntaxSet::load_defaults_newlines();
	let mut syntax = None;
	let mut block = LinearLayout::vertical();
	for class in classes.iter() {
		if class.starts_with("language-") {
			if let Some(found_syntax) = ss
				.syntaxes()
				.iter()
				.rev()
				.find(|&s| class[9..class.len()] == s.name.to_lowercase())
			{
				syntax = Some((found_syntax, &ss));
				break
			}
			if let Some(found_syntax) = provided_syntaxes
				.syntaxes()
				.iter()
				.rev()
				.find(|&s| class[9..class.len()] == s.name.to_lowercase())
			{
				syntax = Some((found_syntax, provided_syntaxes));
				break
			}
		}
	}
	if let Some((syntax, syntax_set)) = syntax {
		let mut parser = ClassedHTMLGenerator::new_with_class_style(syntax, syntax_set, ClassStyle::Spaced);
		for line in LinesWithEndings::from(&*src) {
			parser.parse_html_for_line_which_includes_newline(line).unwrap();
		}
		let mut colour_map = StyleElement::Parent {
			default: Style::new(),
			children: BTreeMap::new(),
		};
		let theme = ThemeSet::load_defaults();
		let theme = if let Some(thm) = given_theme {
			thm.clone()
		} else {
			theme.themes["base16-ocean.light"].clone()
		};
		for i in theme.scopes {
			let mut style = Style::new();
			if let Some(c) = i.style.foreground {
				style.set_color(Color::Rgb(c.r, c.g, c.b))
			}
			if let Some(fs) = i.style.font_style {
				if (fs.bits() & 1) == 1 {
					style.set_bold()
				}
				if (fs.bits() & 4) == 4 {
					style.set_italic()
				}
			}
			for selector in i.scope.selectors {
				for scope in selector.path.scopes {
					let scope = scope.build_string();
					colour_map = colour_map.insert(scope.split(".").collect::<Vec<_>>().iter(), style);
				}
			}
		}
		to_block(parser.finalize(), colour_map, |t| {
			t.split_whitespace().map(|l| l.to_string()).collect()
		})
	} else {
		for line in src.lines() {
			block.push(Paragraph::new(line))
		}
		block
	}
}
