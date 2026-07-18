use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::egui;
use elegant_ui::*;

pub enum Message {
	Noop,
}

pub struct AppState {
	pub sample_input: String,
}

pub struct FinanceApp {
	state: AppState,
	tx: Sender<Message>,
	rx: Receiver<Message>,
}

impl FinanceApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let (tx, rx) = unbounded();
		let theme = ElegantTheme::build(ThemeMode::System); // Reads OS dark/light mode and accent color
		theme.apply(&cc.egui_ctx);
		Self {
			state: AppState {
				sample_input: String::new(),
			},
			tx,
			rx,
		}
	}
}

impl eframe::App for FinanceApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		while let Ok(_msg) = self.rx.try_recv() {}

		egui::CentralPanel::default().show(ui, |ui| {
			ui.vertical_centered(|ui| {
				ui.add_space(40.0);
				ui.heading(egui::RichText::new("UI Component Showcase").size(32.0));
				ui.add_space(40.0);

				egui::ScrollArea::vertical().show(ui, |ui| {
					egui::Grid::new("showcase_grid")
						.spacing(egui::vec2(40.0, 40.0))
						.show(ui, |ui| {
							// ROW 1: Buttons
							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Buttons").strong());
								ui.add_space(8.0);
								ui.horizontal(|ui| {
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

							ui.vertical(|ui| {
								ui.label(
									egui::RichText::new("Badges & Avatars").strong(),
								);
								ui.add_space(8.0);
								ui.horizontal(|ui| {
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
									ui.add_space(16.0);
									ui.add(Avatar::new("JD"));
								});
							});
							ui.end_row();

							// ROW 2: Alerts
							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Alerts").strong());
								ui.add_space(8.0);
								ui.add(
									Alert::new(
										"Success!",
										"Your changes have been saved.",
									)
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
									Alert::new(
										"Info",
										"This is a default alert message.",
									)
									.variant(Variant::Info),
								);

								ui.add_space(8.0);
								ui.add(
									Alert::new("Error!", "Something went wrong.")
										.variant(Variant::Danger),
								);
							});

							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Cards").strong());
								ui.add_space(8.0);
								ui.add(Card::new(
									"Card Title",
									"Card description goes here.",
								));

								ui.add_space(16.0);
								ui.label(egui::RichText::new("Inputs").strong());
								ui.add_space(8.0);
								ui.set_width(300.0);
								ui.text_input(
									&mut self.state.sample_input,
									"Enter text here...",
								);

								ui.add_space(16.0);
								ui.label(
									egui::RichText::new("Progress & Spinners").strong(),
								);
								ui.add_space(8.0);
								ui.set_width(300.0);
								ui.add(Progress::new(0.65));
								ui.add_space(8.0);
								let theme = ElegantTheme::get(ui.ctx());
								ui.add(egui::Spinner::new().color(theme.primary));
							});
							ui.end_row();
						});
				});
			});
		});
	}
}
