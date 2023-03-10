use std::{fs::File, io::BufReader};

use genpdf::{
	elements,
	error::Error,
	fonts::{Font, FontData, FontFamily},
	style::Style,
	Alignment, Document, Element as _, SimplePageDecorator,
};
use mdbook::{book::Chapter, renderer::RenderContext, BookItem};
use syntect::{
	highlighting::ThemeSet,
	parsing::{SyntaxSet, SyntaxSetBuilder},
};

use crate::{
	config::{Config, Highlight},
	highlight::util::StyleElement,
};

/// Main struct used for PDF generation
pub struct Generator {
	/// mdbook given config
	pub config: RenderContext,
	/// PDF options. See [Config]
	pub pdf_opts: Config,
	/// Document struct. Will eventually generate a PDF
	pub document: Document,
	/// Monospace font family. More for convenience than anything else
	pub monospace: FontFamily<Font>,
	/// Document title. Included in other places, but I'm lazy and made it easier to access
	pub title: String,
}

// Required file contents
const OPEN_SANS: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-regular.ttf");
const OPEN_SANS_BOLD: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-700.ttf");
const OPEN_SANS_BOLD_ITALIC: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-700italic.ttf");
const OPEN_SANS_ITALIC: &[u8] = include_bytes!("../../theme/open-sans-v17-all-charsets-italic.ttf");
const SOURCE_CODE_PRO: &[u8] = include_bytes!("../../theme/SourceCodePro-Regular.ttf");

impl Generator {
	/// Create a new PDF generator instance \
	/// This will initialise the PDF with only fonts and configs. [Generator::configure] is then
	/// called which will initialise the first page (title, subtitle, SUMMARY.md) and the page
	/// settings (decorator, size, etc.)
	pub fn new(rc: RenderContext, pdf_opts: Config) -> Self {
		let mut fonts = FontFamily {
			regular: FontData::new(OPEN_SANS.to_vec(), None).unwrap(),
			bold: FontData::new(OPEN_SANS_BOLD.to_vec(), None).unwrap(),
			italic: FontData::new(OPEN_SANS_ITALIC.to_vec(), None).unwrap(),
			bold_italic: FontData::new(OPEN_SANS_BOLD_ITALIC.to_vec(), None).unwrap(),
		};
		let mut monospace_raw = FontData::new(SOURCE_CODE_PRO.to_vec(), None).unwrap();
		if let Some(given_fonts) = &pdf_opts.font {
			macro_rules! parse_font_path {
				($t: expr, $o: expr, $l: expr) => {
					if let Some(p) = $t {
						match std::fs::read(rc.root.join("theme").join("fonts").join(p)) {
							Ok(t) => match FontData::new(t, None) {
								Ok(f) => $o = f,
								Err(e) => println!("Unable to parse font for {}: {}", $l, e),
							},
							Err(e) => println!("Unable to parse {} font ({}): {}", $l, p, e),
						}
					}
				};
			}
			parse_font_path!(&given_fonts.regular, fonts.regular, "regular");
			parse_font_path!(&given_fonts.bold, fonts.bold, "bold");
			parse_font_path!(&given_fonts.italic, fonts.italic, "italic");
			parse_font_path!(&given_fonts.bold_italic, fonts.bold_italic, "bold-italic");
			if let Some(p) = &given_fonts.monospace {
				if let Ok(t) = std::fs::read_to_string(rc.root.join("theme").join("fonts").join(p)) {
					match FontData::new(t.bytes().collect(), None) {
						Ok(f) => monospace_raw = f,
						Err(e) => println!("Unable to parse font for monospace: {}", e),
					}
				}
			}
		}
		let title = rc.config.book.title.clone().unwrap_or(String::new());
		let mut document = Document::new(fonts);
		let monospace = document.add_font_family(FontFamily {
			regular: monospace_raw.clone(),
			bold: monospace_raw.clone(),
			italic: monospace_raw.clone(),
			bold_italic: monospace_raw.clone(),
		});
		Self {
			config: rc,
			pdf_opts,
			document,
			monospace,
			title,
		}
		.configure()
	}

	/// Sets up the document\
	/// Generates the title, optional subtitle, and contents, along with setting fonts, paper size,
	/// and other important things
	fn configure(mut self) -> Self {
		self.document.set_title(self.title.clone());
		self.document.set_minimal_conformance();
		self.document.set_line_spacing(self.pdf_opts.page.spacing.line);
		self.document
			.set_paper_size(self.pdf_opts.page.size.size(self.pdf_opts.page.landscape));
		let mut decorator = SimplePageDecorator::new();
		decorator.set_header(|p| {
			let mut layout = elements::LinearLayout::vertical();
			if p > 1 {
				layout.push(elements::Paragraph::new(p.to_string()).aligned(Alignment::Center));
				layout.push(elements::Break::new(1));
			}
			layout.styled(Style::new().with_font_size(10))
		});
		decorator.set_margins(self.pdf_opts.page.spacing.margin);
		self.document.set_page_decorator(decorator);
		self.document.push(
			elements::Paragraph::new(self.title.clone())
				.aligned(Alignment::Center)
				.styled(Style::new().bold().with_font_size(self.pdf_opts.font_size.title)),
		);
		if let Some(subtitle) = &self.pdf_opts.subtitle {
			self.document.push(
				elements::Paragraph::new(subtitle)
					.aligned(Alignment::Center)
					.styled(Style::new().with_font_size(self.pdf_opts.font_size.h4)),
			);
		}
		let mut chapter_map = Vec::new();
		let mut contents = elements::OrderedList::new();
		for i in self.config.book.sections.iter() {
			match i {
				BookItem::Chapter(c) => {
					chapter_map.push(ChapterMap::new(c));
				}
				_ => {}
			}
		}
		for chapter in chapter_map {
			chapter.to_list(self.pdf_opts.font_size.text, &mut contents)
		}
		self.document
			.push(contents.styled(Style::new().with_font_size(self.pdf_opts.font_size.text)));
		self
	}

	/// Build the PDF\
	/// Appends PDF elements to the document, then writes the generated document, optionally
	/// returning an error that's handled in the main function
	pub fn build(mut self) -> Result<(), Error> {
		// check for highlighting, and custom a highlight_.js file
		let default_theme = ThemeSet::load_defaults().themes["base16-ocean.light"].clone();
		let default_theme = default_theme.scopes.into();
		let mut hl = match self.pdf_opts.highlight {
			Highlight::all => {
				if let Ok(custom) = std::fs::read_to_string(self.config.root.join("theme").join("highlight.js")) {
					Some(HL::highlight(custom))
				} else {
					let mut ss = SyntaxSetBuilder::new();
					if let Err(e) = ss.add_from_folder(self.config.root.join("theme"), true) {
						println!("Unable to load syntax files from theme folder: {}", e)
					};
					Some(HL::syntect((ss.build(), default_theme)))
				}
			}
			Highlight::no_node => {
				let mut ss = SyntaxSet::load_defaults_newlines().into_builder();
				if let Err(e) = ss.add_from_folder(self.config.root.join("theme"), true) {
					println!("Unable to load syntax files from theme folder: {}", e)
				};
				Some(HL::syntect((ss.build(), default_theme)))
			}
			Highlight::none => None,
		};
		if let Some(HL::syntect((ss, _))) = &hl {
			if let Ok(theme) = File::open(self.config.root.join("theme").join("theme.tmtheme")) {
				match ThemeSet::load_from_reader(&mut BufReader::new(theme)) {
					Ok(theme) => hl = Some(HL::syntect((ss.clone(), theme.scopes.into()))),
					Err(e) => {
						println!("Error loading custom ththeme: {}", e)
					}
				}
			}
		}
		for chapter in self.config.clone().book.iter() {
			if let BookItem::Chapter(chapter) = chapter {
				self.chapter(&*chapter.content, &hl)
			}
		}
		match File::create(format!("{}.pdf", self.title)) {
			Ok(f) => match self.document.render(f) {
				Ok(_) => Ok(()),
				Err(e) => Err(e),
			},
			Err(e) => Err(Error::new(format!("Unable to write document: {}", e), e)),
		}
	}
}

/// Highlighting struct. Will be wrapped in an `Option` when passed to the chapter builder
#[allow(non_camel_case_types)]
pub enum HL {
	/// Use syntect highlighting (bundled and in Rust so faster)
	syntect((SyntaxSet, StyleElement)),
	/// Use highlight_.js highlighting (much slower. Called through Node.js)
	highlight(String),
}

/// Local chapter map representation
enum ChapterMap {
	Branch(String, Vec<ChapterMap>),
	Leaf(String),
}

impl ChapterMap {
	/// Make a new chapter map
	pub fn new(c: &Chapter) -> Self {
		if c.sub_items.is_empty() {
			ChapterMap::Leaf(c.name.clone())
		} else {
			ChapterMap::Branch(
				c.name.clone(),
				c.sub_items
					.iter()
					.filter_map(|sc| {
						if let BookItem::Chapter(sc) = sc {
							Some(ChapterMap::new(sc))
						} else {
							None
						}
					})
					.collect(),
			)
		}
	}

	/// Write the map to a set of nested lists
	pub fn to_list(self, fs: u8, parent: &mut elements::OrderedList) {
		match self {
			ChapterMap::Branch(name, children) => {
				let mut child_list = elements::OrderedList::new();
				for c in children {
					c.to_list(fs, &mut child_list)
				}
				let mut block = elements::LinearLayout::vertical();
				block.push(elements::Paragraph::new(name).styled(Style::new().with_font_size(fs)));
				block.push(child_list.styled(Style::new().with_font_size(fs)));
				parent.push(block.styled(Style::new().with_font_size(fs)));
			}
			ChapterMap::Leaf(name) => {
				parent.push(elements::Paragraph::new(name).styled(Style::new().with_font_size(fs)))
			}
		}
	}
}
