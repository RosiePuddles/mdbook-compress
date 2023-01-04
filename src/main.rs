mod config;
mod build;

use anyhow::Error;
use mdbook::book::Book;
use mdbook::BookItem;
use mdbook::renderer::RenderContext;
use crate::config::Config;

fn main() {
    let rc = RenderContext::from_json(&mut std::io::stdin()).unwrap();
    let opts = match rc.config.get_deserialized_opt::<Config, _>("output.compress") {
        Ok(t) => Config::from_rc(t),
        Err(e) => {
            mdbook::utils::log_backtrace(&e);
            std::process::exit(1)
        }
    };
    if let Err(e) = build::Generator::new(rc, opts).build() {
        mdbook::utils::log_backtrace(&Error::new(e));
        std::process::exit(1)
    }
}
