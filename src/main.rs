mod config;
mod build;

use anyhow::Error;
use mdbook::renderer::RenderContext;
use crate::build::Generator;
use crate::config::Config;

fn main() {
    let rc = RenderContext::from_json(&mut std::io::stdin()).unwrap();
    let opts = match rc.config.get_deserialized_opt::<Config, _>("output.compress") {
        Ok(t) => Config::from_rc(t),
        Err(e) => {
            let err = Error::msg(format!("Unable to parse config config file: {}", e));
            mdbook::utils::log_backtrace(&err);
            return
        }
    };
    let generator = match Generator::new(rc, opts) {
        Ok(g) => g,
        Err(e) => {
            return
        }
    };
    if let Err(e) = generator.build() {
        mdbook::utils::log_backtrace(&Error::new(e));
    }
}
