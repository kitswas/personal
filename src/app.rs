use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::egui;
use egui_elegant::*;
use std::time::Duration;

pub enum Message {
	ThemeChanged(bool),
}

pub struct AppState {
	// Add app state fields here
}

pub struct FinanceApp {
	state: AppState,
	tx: Sender<Message>,
	rx: Receiver<Message>,
	theme_mode: ThemeMode,
	is_dark: bool,
}

impl FinanceApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let (tx, rx) = unbounded();
		let theme_mode = ThemeMode::System;
		let theme = ElegantTheme::build(theme_mode, MonaspaceFont::Xenon);
		let is_dark = theme.is_dark;
		theme.apply(&cc.egui_ctx);

		let tx_clone = tx.clone();
		let ctx_clone = cc.egui_ctx.clone();
		std::thread::spawn(move || {
			let mut last_is_dark = is_dark;
			loop {
				std::thread::sleep(Duration::from_secs(1));
				let current_is_dark = is_system_dark_mode();
				if current_is_dark != last_is_dark {
					last_is_dark = current_is_dark;
					let _ = tx_clone.send(Message::ThemeChanged(current_is_dark));
					ctx_clone.request_repaint();
				}
			}
		});

		Self {
			state: AppState {},
			tx,
			rx,
			theme_mode,
			is_dark,
		}
	}
}

impl eframe::App for FinanceApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		while let Ok(msg) = self.rx.try_recv() {
			match msg {
				Message::ThemeChanged(is_dark) => {
					if self.theme_mode == ThemeMode::System {
						self.is_dark = is_dark;
						let theme =
							ElegantTheme::build(ThemeMode::System, MonaspaceFont::Neon);
						theme.apply(ui.ctx());
					}
				},
			}
		}

		// In egui 0.35, eframe gives you a central Ui.
		// If you want a left panel layout, you can use ui.horizontal.
		// For a true SidePanel, you'd need the context, but let's just draw on the
		// provided ui.
		ui.horizontal(|ui| {
			// Left side panel mockup
			ui.vertical(|ui| {
				ui.set_width(200.0);
				ui.add_space(24.0);
				ui.heading("Personal Finance");
				ui.add_space(16.0);
				ui.add(ElegantButton::new("Dashboard").ghost());
				ui.add(ElegantButton::new("Transactions").ghost());
				ui.add(ElegantButton::new("Accounts").ghost());
			});

			ui.separator();

			// Main content mockup
			ui.vertical(|ui| {
				ui.add_space(24.0);
				ui.heading("Dashboard");
				ui.add_space(16.0);
				ui.label("Welcome to your local-first personal finance app.");
			});
		});
	}
}
