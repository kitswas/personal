use crate::theme::ElegantTheme;
use egui;

pub trait InputUiExtensions {
	fn text_input(&mut self, text: &mut String, placeholder: &str) -> egui::Response;
}

impl InputUiExtensions for egui::Ui {
	fn text_input(&mut self, text: &mut String, placeholder: &str) -> egui::Response {
		let theme = ElegantTheme::get(self.ctx());
		let frame = egui::Frame::new()
			.inner_margin(theme.spacing.input_inner_margin)
			.corner_radius(egui::CornerRadius::same(theme.spacing.corner_radius as u8))
			.stroke(egui::Stroke::new(theme.spacing.border_width, theme.border))
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
