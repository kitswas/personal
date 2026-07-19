use eframe::egui;
use egui_elegant::{
	egui_flex::{Flex, item},
	*,
};
use std::{
	sync::mpsc::{Receiver, channel},
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
					egui::RichText::new("UI Component Showcase (Flex)").size(32.0),
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
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Buttons").strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									ui.horizontal_wrapped(|ui| {
										ui.spacing_mut().item_spacing =
											egui::vec2(8.0, 8.0);
										ui.add(
											ElegantButton::new("Primary")
												.variant(Variant::Primary),
										);
										ui.add(
											ElegantButton::new("Secondary")
												.variant(Variant::Secondary),
										);
										ui.add(
											ElegantButton::new("Danger")
												.variant(Variant::Danger),
										);
										ui.add(ElegantButton::new("Outline").outline());
										ui.add(
											ElegantButton::new("Danger Outline")
												.variant(Variant::Danger)
												.outline(),
										);
										ui.add(ElegantButton::new("Ghost").ghost());
									});
								});
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Badges & Avatars").strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									ui.horizontal_wrapped(|ui| {
										ui.spacing_mut().item_spacing =
											egui::vec2(8.0, 8.0);
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
											ElegantBadge::new("Danger")
												.variant(Variant::Danger),
										);
										ui.add_space(8.0);
										ui.add(Avatar::new("JD"));
									});
								});
							});
						}),
						Box::new(move |ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Alerts").strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									ui.add_sized(
										egui::vec2(inner_width, 0.0),
										Alert::new(
											"Success!",
											"Your changes have been saved.",
										)
										.variant(Variant::Success),
									);
								});
								flex.add_ui(item(), |ui| {
									ui.add_sized(
										egui::vec2(inner_width, 0.0),
										Alert::new(
											"Warning!",
											"Please review before continuing.",
										)
										.variant(Variant::Warning),
									);
								});
								flex.add_ui(item(), |ui| {
									ui.add_sized(
										egui::vec2(inner_width, 0.0),
										Alert::new(
											"Info",
											"This is a default alert message.",
										)
										.variant(Variant::Info),
									);
								});
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Cards & Accordion").strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									Card::new().show(ui, |ui| {
										Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(
											ui,
											|flex| {
												flex.add(
													item(),
													egui::Label::new(
														egui::RichText::new("Card Title")
															.heading(),
													),
												);
												flex.add(
													item(),
													egui::Label::new(
														"Card description goes here.",
													),
												);
												flex.add_ui(item(), |ui| {
													ui.horizontal_wrapped(|ui| {
														ui.spacing_mut().item_spacing =
															egui::vec2(8.0, 8.0);
														ui.add(
															ElegantButton::new("Cancel")
																.ghost(),
														);
														ui.add(
															ElegantButton::new(
																"Save Changes",
															)
															.variant(Variant::Primary),
														);
													});
												});
											},
										);
									});
								});
								flex.add_ui(item(), |ui| {
									ElegantAccordion::new("acc1", "Advanced Options")
										.show(ui, |ui| {
											ui.label(
												"Hidden content inside the accordion.",
											);
										});
								});
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Inputs & Dropdowns")
											.strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									ui.text_input(
										&mut self.state.sample_input,
										"Enter text here...",
									);
								});
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Tags").strong(),
									),
								);
								flex.add(
									item(),
									ElegantTagInput::new(
										&mut self.state.tags,
										&mut self.state.new_tag,
									),
								);
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Dropdown").strong(),
									),
								);
								flex.add_ui(item(), |ui| {
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
								});
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Progress & Skeleton")
											.strong(),
									),
								);
								flex.add(item(), Progress::new(0.65));
								flex.add_ui(item(), |ui| {
									let theme = ElegantTheme::get(ui.ctx());
									ui.add(egui::Spinner::new().color(theme.primary));
								});
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Skeleton").strong(),
									),
								);
								flex.add(item(), Skeleton::new(200.0, 24.0));
								flex.add(item(), Skeleton::new(150.0, 16.0));
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Tabs").strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									egui::ScrollArea::horizontal().show(ui, |ui| {
										ui.add(ElegantTabs::new(
											&["Overview", "Transactions", "Settings"],
											&mut self.state.selected_tab,
										));
									});
								});
								flex.add(
									item(),
									egui::Label::new(format!(
										"Selected tab: {}",
										self.state.selected_tab
									)),
								);
							});
						}),
						Box::new(|ui: &mut egui::Ui| {
							Flex::vertical().gap(egui::vec2(0.0, 8.0)).show(ui, |flex| {
								flex.add(
									item(),
									egui::Label::new(
										egui::RichText::new("Toast Notification")
											.strong(),
									),
								);
								flex.add_ui(item(), |ui| {
									if ui
										.add(
											ElegantButton::new("Show Toast")
												.variant(Variant::Success),
										)
										.clicked()
									{
										self.state.show_toast = true;
									}
								});
							});
						}),
					];

					Flex::horizontal()
						.wrap(true)
						.gap(egui::vec2(GRID_GAP, GRID_GAP))
						.show(ui, |flex| {
							for card_fn in &mut cards {
								Card::new().show_flex(flex, item(), |ui| {
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
		"egui-elegant flex showcase",
		native_options,
		Box::new(|cc| Ok(Box::new(ShowcaseApp::new(cc)))),
	)
}
