use crate::theme::ElegantTheme;
use egui;

pub struct Alert<'a> {
	title: &'a str,
	content: &'a str,
}

impl<'a> Alert<'a> {
	pub fn new(title: &'a str, content: &'a str) -> Self {
		Self { title, content }
	}
}

impl<'a> egui::Widget for Alert<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());

		egui::Frame::new()
			.fill(theme.secondary)
			.stroke(egui::Stroke::new(1.0, theme.border))
			.inner_margin(16.0)
			.corner_radius(egui::CornerRadius::same(6))
			.show(ui, |ui| {
				ui.set_width(300.0);
				ui.label(
					egui::RichText::new(self.title)
						.strong()
						.color(theme.foreground),
				);
				ui.add_space(4.0);
				ui.label(egui::RichText::new(self.content).color(theme.foreground));
			})
			.response
	}
}
