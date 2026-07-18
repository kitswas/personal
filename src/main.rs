#![warn(missing_docs)]

//! # Local-First Personal Finance App
//!
//! This application is designed to be a strictly local, crash-proof double-entry
//! accounting ledger.
//!
//! ## Documentation Philosophy
//! Documentation on public items explains **why** a module or function is designed the
//! way it is, avoiding redundant explanations of **what** the code syntax already shows.
//! For system-wide architectural decisions and data-flow diagrams, refer to the
//! `docs/arch/` ADRs and use `cargo-modules` to generate structural graphs.

mod app;
mod state;

use app::FinanceApp;

#[tokio::main]
async fn main() -> eframe::Result<()> {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(
		"Personal Finance",
		native_options,
		Box::new(|cc| Ok(Box::new(FinanceApp::new(cc)))),
	)
}
