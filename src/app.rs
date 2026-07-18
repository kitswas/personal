use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;
use crate::ui::*;

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
		apply_theme(&cc.egui_ctx);
		Self { 
			state: AppState { sample_input: String::new() }, 
			tx, 
			rx 
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
							// ROW 1
							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Buttons").strong());
								ui.add_space(8.0);
								ui.horizontal(|ui| {
									let _ = ui.button("Default");
									let _ = ui.primary_button("Primary");
									let _ = ui.ghost_button("Ghost");
								});
							});

							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Badges & Avatars").strong());
								ui.add_space(8.0);
								ui.horizontal(|ui| {
									let _ = ui.badge("Neutral");
									let _ = ui.badge_success("Success");
									ui.add_space(8.0);
									ui.add(Avatar::new("JD"));
								});
							});
							ui.end_row();

							// ROW 2
							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Cards").strong());
								ui.add_space(8.0);
								ui.add(Card::new(
									"Card Title",
									"Card description goes here.",
								));
							});

							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Alerts").strong());
								ui.add_space(8.0);
								ui.add(Alert::new(
									"Info Alert",
									"This is a default alert message explaining something important.",
								));
							});
							ui.end_row();

							// ROW 3
							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Inputs").strong());
								ui.add_space(8.0);
								ui.set_width(300.0);
								ui.text_input(&mut self.state.sample_input, "Enter text here...");
							});

							ui.vertical(|ui| {
								ui.label(egui::RichText::new("Progress & Spinners").strong());
								ui.add_space(8.0);
								ui.set_width(300.0);
								ui.add(Progress::new(0.65));
								ui.add_space(8.0);
								ui.add(egui::Spinner::new().color(egui::Color32::from_rgb(87, 71, 71)));
							});
							ui.end_row();
						});
				});
			});
		});
	}
}
