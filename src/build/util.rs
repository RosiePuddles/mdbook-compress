use pdf_canvas::FontRef;

pub fn split_width(width: f32, font: &FontRef, size: f32, src: String) -> Vec<String> {
	let words = src.split_whitespace();
	let words_width = words.clone().map(|c| font.get_width(size, c)).collect::<Vec<f32>>();
	let space_width = font.get_width(size, " ");
	let mut width_acc = 0f32;
	let mut lines = Vec::new();
	let mut line_acc = Vec::new();
	for (word, word_width) in words.zip(words_width) {
		if width_acc + word_width > width {
			lines.push(line_acc.join(" "));
			line_acc = vec![word];
			width_acc = word_width;
		} else {
			line_acc.push(word);
			width_acc += word_width + space_width;
		}
	}
	lines
}
