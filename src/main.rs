#![warn(missing_docs)]

//! # Local-First Personal Finance App
//!
//! This application is designed to be a strictly local, crash-proof double-entry
//! accounting ledger.

mod app;

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
