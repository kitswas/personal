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
	let font_bytes = include_bytes!(
		"../monaspace/Variable Fonts/Monaspace Neon/Monaspace Neon Var.ttf"
	);

	iced::application(FinanceApp::new, FinanceApp::update, FinanceApp::view)
		.font(font_bytes)
		.default_font(iced::Font {
			family: iced::font::Family::Name("Monaspace Neon Var"),
			weight: iced::font::Weight::Normal,
			stretch: iced::font::Stretch::Normal,
			style: iced::font::Style::Normal,
		})
		.theme(FinanceApp::theme)
		.subscription(FinanceApp::subscription)
		.window(iced::window::Settings {
			// we can customize window here if we need
			..Default::default()
		})
		.run()
}
