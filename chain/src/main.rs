//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use sc_cli::{VersionInfo, IntoExit, error};

fn main() -> Result<(), cli::error::Error> {
	let version = VersionInfo {
		name: "Weather Forecast",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "weather-forecast",
		author: "Yaoqi Jia",
		description: "Weather Forecast",
		support_url: "https://github.com/jiayaoqijia/Weather-Forecast",
	};

	cli::run(std::env::args(), cli::Exit, version)
}
