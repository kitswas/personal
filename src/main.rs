#![warn(missing_docs)]

//! # Local-First Personal Finance App
//!
//! This application is designed to be a strictly local, crash-proof double-entry
//! accounting ledger.

mod app;
mod domain;
mod infrastructure;
mod sankey;

use app::FinanceApp;

fn main() -> iced::Result {
	iced::application(FinanceApp::new, FinanceApp::update, FinanceApp::view)
		.theme(FinanceApp::theme)
		.subscription(FinanceApp::subscription)
		.window(iced::window::Settings {
			// we can customize window here if we need
			..Default::default()
		})
		.run()
}
