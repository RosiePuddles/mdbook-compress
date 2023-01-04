use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
	#[serde(default = "FontSize::default")]
	pub font_size: FontSize,
	#[serde(default = "PageOpts::default")]
	pub page: PageOpts,
	#[serde(default = "default_hl")]
	pub highlight: bool
}

#[derive(Deserialize, Debug)]
pub struct FontSize {
	#[serde(default = "default_title")]
	pub title: f32,
	#[serde(default = "default_h1")]
	pub h1: f32,
	#[serde(default = "default_h2")]
	pub h2: f32,
	#[serde(default = "default_h3")]
	pub h3: f32,
	#[serde(default = "default_h4")]
	pub h4: f32,
	#[serde(default = "default_h5")]
	pub h5: f32,
	#[serde(default = "default_h6")]
	pub h6: f32,
	#[serde(default = "default_text")]
	pub text: f32,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
pub enum PageSize {
	A4,
	#[serde(rename = "US letter")]
	US_letter,
	#[serde(rename = "US legal")]
	US_legal
}

#[derive(Deserialize, Debug)]
pub struct PageMargins {
	#[serde(default = "default_margin_x")]
	pub x: f32,
	#[serde(default = "default_margin_y")]
	pub y: f32
}

#[derive(Deserialize, Debug)]
pub struct PageOpts {
	#[serde(default = "PageSize::default")]
	pub size: PageSize,
	#[serde(default = "default_landscape")]
	pub landscape: bool,
	#[serde(default = "PageMargins::default")]
	pub margin: PageMargins
}

fn default_title() -> f32 { 12.5 }
fn default_h1() -> f32 { 11.0 }
fn default_h2() -> f32 { 10.0 }
fn default_h3() -> f32 { 8.5 }
fn default_h4() -> f32 { 7.0 }
fn default_h5() -> f32 { 6.0 }
fn default_h6() -> f32 { 6.0 }
fn default_text() -> f32 { 5.0 }
fn default_margin_x() -> f32 { 12.0 }
fn default_margin_y() -> f32 { 20.0 }
fn default_landscape() -> bool { false }
fn default_hl() -> bool { true }

impl Config {
	pub fn from_rc(rc: Option<Self>) -> Self { rc.unwrap_or(Self::default()) }
}

impl PageSize {
	pub fn width(&self, landscape: bool) -> f32 {
		if landscape { return self.height(false) }
		match self {
			PageSize::A4 => 210.0,
			PageSize::US_letter => 215.9,
			PageSize::US_legal => 215.9,
		}
	}
	pub fn height(&self, landscape: bool) -> f32 {
		if landscape { return self.width(false) }
		match self {
			PageSize::A4 => 297.0,
			PageSize::US_letter => 279.4,
			PageSize::US_legal => 355.6,
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			font_size: Default::default(),
			page: Default::default(),
			highlight: true
		}
	}
}

impl Default for FontSize {
	fn default() -> Self {
		Self {
			title: 12.5,
			h1: 11.0,
			h2: 10.0,
			h3: 8.5,
			h4: 7.0,
			h5: 6.0,
			h6: 6.0,
			text: 5.0,
		}
	}
}

impl Default for PageSize {
	fn default() -> Self {
		PageSize::A4
	}
}

impl Default for PageMargins {
	fn default() -> Self {
		Self {
			x: 12.0,
			y: 20.0,
		}
	}
}

impl Default for PageOpts {
	fn default() -> Self {
		Self {
			size: Default::default(),
			landscape: false,
			margin: Default::default(),
		}
	}
}
