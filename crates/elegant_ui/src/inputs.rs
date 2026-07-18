use crate::theme::ElegantTheme;
use egui;

pub trait InputUiExtensions {
	fn text_input(&mut self, text: &mut String, placeholder: &str) -> egui::Response;
}

impl InputUiExtensions for egui::Ui {
	fn text_input(&mut self, text: &mut String, placeholder: &str) -> egui::Response {
		let theme = ElegantTheme::get(self.ctx());
		let frame = egui::Frame::new()
			.inner_margin(egui::vec2(12.0, 8.0))
			.corner_radius(egui::CornerRadius::same(6))
			.stroke(egui::Stroke::new(1.0, theme.border))
			.fill(theme.background);

		frame
			.show(self, |ui| {
				ui.add_sized(
					egui::vec2(ui.available_width(), 16.0),
					egui::TextEdit::singleline(text).hint_text(placeholder),
				)
			})
			.inner
	}
}
