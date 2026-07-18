use crate::theme::ElegantTheme;
use egui;

pub trait ButtonUiExtensions {
	fn primary_button(&mut self, text: &str) -> egui::Response;
	fn ghost_button(&mut self, text: &str) -> egui::Response;
}

impl ButtonUiExtensions for egui::Ui {
	fn primary_button(&mut self, text: &str) -> egui::Response {
		let theme = ElegantTheme::get(self.ctx());
		let btn = egui::Button::new(egui::RichText::new(text).color(theme.background))
			.fill(theme.primary)
			.stroke(egui::Stroke::NONE);
		self.add(btn)
	}

	fn ghost_button(&mut self, text: &str) -> egui::Response {
		let btn = egui::Button::new(text)
			.fill(egui::Color32::TRANSPARENT)
			.stroke(egui::Stroke::NONE);
		self.add(btn)
	}
}
