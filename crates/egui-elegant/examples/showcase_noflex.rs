use eframe::egui;
use egui_elegant::*;
use std::{
	sync::mpsc::{Receiver, Sender, channel},
	time::Duration,
};

pub enum Message {
	ThemeChanged(bool),
}

pub struct AppState {
	pub sample_input: String,
	pub selected_tab: usize,
	pub tags: Vec<String>,
	pub new_tag: String,
	pub selected_dropdown: String,
	pub show_toast: bool,
}

pub struct ShowcaseApp {
	state: AppState,
	tx: Sender<Message>,
	rx: Receiver<Message>,
	theme_mode: ThemeMode,
	is_dark: bool,
}

impl ShowcaseApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let (tx, rx) = channel();
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
			state: AppState {
				sample_input: String::new(),
				selected_tab: 0,
				tags: vec!["finance".to_string(), "rust".to_string()],
				new_tag: String::new(),
				selected_dropdown: "option1".to_string(),
				show_toast: false,
			},
			tx,
			rx,
			theme_mode,
			is_dark,
		}
	}
}

impl eframe::App for ShowcaseApp {
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

		egui::CentralPanel::default().show(ui, |ui| {
			ui.vertical_centered(|ui| {
				ui.add_space(40.0);
				ui.heading(
					egui::RichText::new("UI Component Showcase (No Flex)").size(32.0),
				);
				ui.add_space(40.0);
			});

			const CARD_OUTER_WIDTH: f32 = 360.0;
			const GRID_GAP: f32 = 24.0;
			const CARD_INNER_PADDING: f32 = 24.0;

			egui::ScrollArea::vertical()
				.auto_shrink([false, false])
				.show(ui, |ui| {
					ui.add_space(GRID_GAP);

					let inner_width = CARD_OUTER_WIDTH - (2.0 * CARD_INNER_PADDING);

					let mut cards: Vec<Box<dyn FnMut(&mut egui::Ui)>> = vec![
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Buttons").strong());
							ui.add_space(8.0);
							ui.horizontal_wrapped(|ui| {
								ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
								ui.add(
									ElegantButton::new("Primary")
										.variant(Variant::Primary),
								);
								ui.add(
									ElegantButton::new("Secondary")
										.variant(Variant::Secondary),
								);
								ui.add(
									ElegantButton::new("Danger").variant(Variant::Danger),
								);
								ui.add(ElegantButton::new("Outline").outline());
								ui.add(
									ElegantButton::new("Danger Outline")
										.variant(Variant::Danger)
										.outline(),
								);
								ui.add(ElegantButton::new("Ghost").ghost());
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Badges & Avatars").strong());
							ui.add_space(8.0);
							ui.horizontal_wrapped(|ui| {
								ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
								ui.add(ElegantBadge::new("Default"));
								ui.add(
									ElegantBadge::new("Secondary")
										.variant(Variant::Secondary),
								);
								ui.add(ElegantBadge::new("Outline").outline());
								ui.add(
									ElegantBadge::new("Success")
										.variant(Variant::Success),
								);
								ui.add(
									ElegantBadge::new("Warning")
										.variant(Variant::Warning),
								);
								ui.add(
									ElegantBadge::new("Danger").variant(Variant::Danger),
								);
								ui.add_space(8.0);
								ui.add(Avatar::new("JD"));
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Alerts").strong());
							ui.add_space(8.0);
							ui.add(
								Alert::new("Success!", "Your changes have been saved.")
									.variant(Variant::Success),
							);
							ui.add_space(8.0);
							ui.add(
								Alert::new(
									"Warning!",
									"Please review before continuing.",
								)
								.variant(Variant::Warning),
							);
							ui.add_space(8.0);
							ui.add(
								Alert::new("Info", "This is a default alert message.")
									.variant(Variant::Info),
							);
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Cards & Accordion").strong());
							ui.add_space(8.0);
							Card::new().show(ui, |ui| {
								ui.heading("Card Title");
								ui.add_space(8.0);
								ui.label("Card description goes here.");
								ui.add_space(16.0);
								ui.horizontal_wrapped(|ui| {
									ui.add(ElegantButton::new("Cancel").ghost());
									ui.add(
										ElegantButton::new("Save Changes")
											.variant(Variant::Primary),
									);
								});
							});
							ui.add_space(16.0);
							ElegantAccordion::new("acc1", "Advanced Options").show(
								ui,
								|ui| {
									ui.label("Hidden content inside the accordion.");
								},
							);
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Inputs & Dropdowns").strong());
							ui.add_space(8.0);
							ui.text_input(
								&mut self.state.sample_input,
								"Enter text here...",
							);
							ui.add_space(16.0);
							ui.label(egui::RichText::new("Tags").strong());
							ui.add_space(8.0);
							ui.add(ElegantTagInput::new(
								&mut self.state.tags,
								&mut self.state.new_tag,
							));
							ui.add_space(16.0);
							ui.label(egui::RichText::new("Dropdown").strong());
							ui.add_space(8.0);
							ElegantDropdown::new(
								"dropdown1",
								&mut self.state.selected_dropdown,
							)
							.options(vec![
								("option1".to_string(), "Option 1".to_string()),
								("option2".to_string(), "Option 2".to_string()),
								("option3".to_string(), "Option 3".to_string()),
							])
							.show(ui);
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Progress & Skeleton").strong());
							ui.add_space(8.0);
							ui.add(Progress::new(0.65));
							ui.add_space(8.0);
							let theme = ElegantTheme::get(ui.ctx());
							ui.add(egui::Spinner::new().color(theme.primary));
							ui.add_space(16.0);
							ui.label(egui::RichText::new("Skeleton").strong());
							ui.add_space(8.0);
							ui.add(Skeleton::new(200.0, 24.0));
							ui.add_space(4.0);
							ui.add(Skeleton::new(150.0, 16.0));
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Tabs").strong());
							ui.add_space(8.0);
							egui::ScrollArea::horizontal().show(ui, |ui| {
								ui.add(ElegantTabs::new(
									&["Overview", "Transactions", "Settings"],
									&mut self.state.selected_tab,
								));
							});
							ui.add_space(16.0);
							ui.label(format!(
								"Selected tab: {}",
								self.state.selected_tab
							));
						}),
						Box::new(|ui: &mut egui::Ui| {
							ui.label(egui::RichText::new("Toast Notification").strong());
							ui.add_space(8.0);
							if ui
								.add(
									ElegantButton::new("Show Toast")
										.variant(Variant::Success),
								)
								.clicked()
							{
								self.state.show_toast = true;
							}
						}),
					];

					// Native egui vertical layout fallback
					ui.vertical(|ui| {
						ui.spacing_mut().item_spacing = egui::vec2(GRID_GAP, GRID_GAP);
						for card_fn in &mut cards {
							Card::new().show(ui, |ui| {
								ui.set_min_width(inner_width);
								ui.set_max_width(inner_width);
								card_fn(ui);
							});
						}
					});

					ui.add_space(32.0);
				});
		});

		if self.state.show_toast {
			ElegantToast::new("toast1", "Notification", "This is a toast message!")
				.variant(Variant::Success)
				.show(ui.ctx());
		}
	}
}

fn main() -> eframe::Result<()> {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(
		"egui-elegant non-flex showcase",
		native_options,
		Box::new(|cc| Ok(Box::new(ShowcaseApp::new(cc)))),
	)
}
