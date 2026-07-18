use crate::{
	button::ElegantButton,
	theme::{ElegantTheme, Variant},
};
use egui;

pub struct Card<'a> {
	title: &'a str,
	content: &'a str,
}

impl<'a> Card<'a> {
	pub fn new(title: &'a str, content: &'a str) -> Self {
		Self { title, content }
	}
}

impl<'a> egui::Widget for Card<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());
		egui::Frame::new()
			.stroke(egui::Stroke::new(1.0, theme.border))
			.inner_margin(24.0)
			.corner_radius(egui::CornerRadius::same(6))
			.show(ui, |ui| {
				ui.set_width(300.0);
				ui.heading(self.title);
				ui.add_space(8.0);
				ui.label(self.content);
				ui.add_space(16.0);
				ui.horizontal(|ui| {
					ui.add(ElegantButton::new("Cancel").ghost());
					ui.add(ElegantButton::new("Save Changes").variant(Variant::Primary));
				});
			})
			.response
	}
}
