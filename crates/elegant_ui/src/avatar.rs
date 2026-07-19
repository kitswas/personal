use crate::{theme::ElegantTheme, traits::Elegant};
use egui;

pub struct Avatar<'a> {
	text: &'a str,
}

impl<'a> Avatar<'a> {
	pub fn new(text: &'a str) -> Self {
		Self { text }
	}
}

impl<'a> egui::Widget for Avatar<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());
		let size = egui::vec2(32.0, 32.0);
		let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

		if ui.is_rect_visible(rect) {
			let painter = ui.painter();
			painter.circle_filled(rect.center(), size.x / 2.0, theme.secondary);
			painter.text(
				rect.center(),
				egui::Align2::CENTER_CENTER,
				self.text,
				egui::FontId::proportional(14.0),
				theme.primary,
			);
		}
		response
	}
}

impl<'a> Elegant for Avatar<'a> {}
crate::impl_flex_widget!(Avatar<'a>);
