use crate::theme::ElegantTheme;
use egui;

pub trait BadgeUiExtensions {
	fn badge(&mut self, text: &str) -> egui::Response;
	fn badge_success(&mut self, text: &str) -> egui::Response;
}

impl BadgeUiExtensions for egui::Ui {
	fn badge(&mut self, text: &str) -> egui::Response {
		let theme = ElegantTheme::get(self.ctx());
		let btn = egui::Button::new(
			egui::RichText::new(text).size(12.0).color(theme.foreground),
		)
		.fill(theme.secondary)
		.stroke(egui::Stroke::NONE)
		.corner_radius(egui::CornerRadius::same(12));
		self.add_enabled(false, btn)
	}

	fn badge_success(&mut self, text: &str) -> egui::Response {
		let theme = ElegantTheme::get(self.ctx());
		let btn = egui::Button::new(
			egui::RichText::new(text).color(theme.background).size(12.0),
		)
		.fill(theme.success)
		.stroke(egui::Stroke::NONE)
		.corner_radius(egui::CornerRadius::same(12));
		self.add_enabled(false, btn)
	}
}
