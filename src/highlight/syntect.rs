use std::collections::HashSet;

use genpdf::elements::{LinearLayout, Paragraph};
use syntect::{
	html::{ClassStyle, ClassedHTMLGenerator},
	parsing::SyntaxSet,
	util::LinesWithEndings,
};

use crate::highlight::util::{to_block, StyleElement};

pub fn highlight(classes: HashSet<String>, src: String, ss: &SyntaxSet, theme: &StyleElement) -> LinearLayout {
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
				syntax = Some((found_syntax, ss, theme));
				break
			}
		}
	}
	if let Some((syntax, syntax_set, theme)) = syntax {
		let mut parser = ClassedHTMLGenerator::new_with_class_style(syntax, syntax_set, ClassStyle::Spaced);
		for line in LinesWithEndings::from(&*src) {
			parser.parse_html_for_line_which_includes_newline(line).unwrap();
		}
		to_block(parser.finalize(), theme, |t| {
			t.split_whitespace().map(|l| l.to_string()).collect()
		})
	} else {
		for line in src.lines() {
			block.push(Paragraph::new(line))
		}
		block
	}
}
