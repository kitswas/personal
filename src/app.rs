use eframe::egui;

pub struct FinanceApp {}

impl FinanceApp {
	pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		Self {}
	}
}

impl eframe::App for FinanceApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		ui.heading("Local-First Personal Finance");
		ui.label("Work in progress...");
	}
}
