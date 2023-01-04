use std::str::SplitWhitespace;
use pdf_canvas::{BuiltinFont, FontRef};

pub fn split_width(width: f32, src: Vec<String>, widths: Vec<f32>, fonts: Vec<(BuiltinFont, f32)>) -> Vec<Vec<(String, f32, BuiltinFont)>> {
	let mut width_acc = 0f32;
	let mut lines = Vec::new();
	let mut line_acc = Vec::new();
	for ((word, word_width), (font, space_width)) in src.iter().zip(widths).zip(fonts) {
		if width_acc + word_width > width {
			lines.push(line_acc.clone());
			line_acc = vec![(word.clone(), word_width, font)];
			width_acc = word_width;
		} else {
			line_acc.push((word.clone(), word_width, font));
			width_acc += word_width + space_width;
		}
	}
	lines
}
