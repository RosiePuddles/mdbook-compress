use serde::Deserialize;

/// Root config struct
#[derive(Deserialize, Debug)]
pub struct Config {
	#[serde(default = "FontSize::default")]
	pub font_size: FontSize,
	#[serde(default = "PageOpts::default")]
	pub page: PageOpts,
	#[serde(default = "Highlight::default")]
	pub highlight: Highlight,
	pub subtitle: Option<String>,
}

/// Highlighting settings
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
pub enum Highlight {
	all,
	#[serde(rename = "no-node")]
	no_node,
	none,
}

/// Font sizes for title, H1 to H6, and text
#[derive(Deserialize, Debug)]
pub struct FontSize {
	#[serde(default = "default_title")]
	pub title: u8,
	#[serde(default = "default_h1")]
	pub h1: u8,
	#[serde(default = "default_h2")]
	pub h2: u8,
	#[serde(default = "default_h3")]
	pub h3: u8,
	#[serde(default = "default_h4")]
	pub h4: u8,
	#[serde(default = "default_h5")]
	pub h5: u8,
	#[serde(default = "default_h6")]
	pub h6: u8,
	#[serde(default = "default_text")]
	pub text: u8,
}

/// Page size enum
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum PageSize {
	A4,
	#[serde(rename = "US letter")]
	US_letter,
	#[serde(rename = "US legal")]
	US_legal,
	Custom {
		x: f64,
		y: f64,
	},
}

/// Page spacing values (line spacing and margins)
#[derive(Deserialize, Debug)]
pub struct PageSpaces {
	#[serde(default = "default_line_space")]
	pub line: f64,
	#[serde(default = "default_margin")]
	pub margin: (f64, f64),
}

/// Page option configs (size and spacing)
#[derive(Deserialize, Debug)]
pub struct PageOpts {
	#[serde(default = "PageSize::default")]
	pub size: PageSize,
	#[serde(default = "default_landscape")]
	pub landscape: bool,
	#[serde(default = "PageSpaces::default")]
	pub spacing: PageSpaces,
}

fn default_title() -> u8 { 25 }
fn default_h1() -> u8 { 22 }
fn default_h2() -> u8 { 20 }
fn default_h3() -> u8 { 17 }
fn default_h4() -> u8 { 14 }
fn default_h5() -> u8 { 12 }
fn default_h6() -> u8 { 12 }
fn default_text() -> u8 { 10 }
fn default_line_space() -> f64 { 1.5 }
fn default_margin() -> (f64, f64) { (20.0, 20.0) }
fn default_landscape() -> bool { false }

impl Config {
	pub fn from_rc(rc: Option<Self>) -> Self { rc.unwrap_or(Self::default()) }
}

impl PageSize {
	/// Get page size. Requires landscape bool
	pub fn size(&self, landscape: bool) -> (f64, f64) {
		let (x, y) = match self {
			PageSize::A4 => (210.0, 297.0),
			PageSize::US_letter => (215.9, 279.4),
			PageSize::US_legal => (215.9, 355.6),
			PageSize::Custom { x, y } => (*x, *y),
		};
		if landscape {
			(y, x)
		} else {
			(x, y)
		}
	}
}

impl FontSize {
	/// Get the text size for a given ID (from HTML tags)
	pub fn get(&self, section: &str) -> u8 {
		match section {
			"h1" => self.h1,
			"h2" => self.h2,
			"h3" => self.h3,
			"h4" => self.h4,
			"h5" => self.h5,
			"h6" => self.h6,
			_ => self.text,
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			font_size: Default::default(),
			page: Default::default(),
			highlight: Default::default(),
			subtitle: None,
		}
	}
}

impl Default for Highlight {
	fn default() -> Self { Self::all }
}

impl Default for FontSize {
	fn default() -> Self {
		Self {
			title: default_title(),
			h1: default_h1(),
			h2: default_h2(),
			h3: default_h3(),
			h4: default_h4(),
			h5: default_h5(),
			h6: default_h6(),
			text: default_text(),
		}
	}
}

impl Default for PageSize {
	fn default() -> Self { PageSize::A4 }
}

impl Default for PageSpaces {
	fn default() -> Self {
		Self {
			line: default_line_space(),
			margin: default_margin(),
		}
	}
}

impl Default for PageOpts {
	fn default() -> Self {
		Self {
			size: Default::default(),
			landscape: default_landscape(),
			spacing: Default::default(),
		}
	}
}
