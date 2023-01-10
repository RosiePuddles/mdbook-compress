mod build;
mod config;
mod highlight;

use anyhow::Error;
use mdbook::renderer::RenderContext;

use crate::{build::Generator, config::Config};

fn main() {
	let rc = RenderContext::from_json(&mut std::io::stdin()).unwrap();
	let opts = match rc
		.config
		.get_deserialized_opt::<Config, _>("output.compress")
	{
		Ok(t) => Config::from_rc(t),
		Err(e) => {
			let err = Error::msg(format!("Unable to parse config config file: {}", e));
			mdbook::utils::log_backtrace(&err);
			return
		}
	};
	if let Err(e) = Generator::new(rc, opts).build() {
		mdbook::utils::log_backtrace(&Error::new(e));
	}
}
