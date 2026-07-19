use crate::{theme::ElegantTheme, traits::Elegant};
use egui;

pub struct Progress {
	value: f32, // 0.0 to 1.0
}

impl Progress {
	pub fn new(value: f32) -> Self {
		Self { value }
	}
}

impl egui::Widget for Progress {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());
		let height = 8.0;
		let (rect, response) =
			ui.allocate_at_least(egui::vec2(100.0, height), egui::Sense::hover());

		if ui.is_rect_visible(rect) {
			let painter = ui.painter();
			let radius = height / 2.0;

			// Background track
			painter.rect_filled(rect, radius, theme.secondary);

			// Fill bar
			let mut fill_rect = rect;
			fill_rect.max.x = rect.min.x + (rect.width() * self.value.clamp(0.0, 1.0));
			if fill_rect.width() > 0.0 {
				painter.rect_filled(fill_rect, radius, theme.primary);
			}
		}
		response
	}
}

impl Elegant for Progress {}
crate::impl_flex_widget!(Progress);
