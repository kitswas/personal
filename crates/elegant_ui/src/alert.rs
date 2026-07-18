use crate::theme::{ElegantTheme, Variant};
use egui;

pub struct Alert<'a> {
	title: &'a str,
	content: &'a str,
	variant: Variant,
}

impl<'a> Alert<'a> {
	pub fn new(title: &'a str, content: &'a str) -> Self {
		Self {
			title,
			content,
			variant: Variant::Info,
		}
	}

	pub fn variant(mut self, variant: Variant) -> Self {
		self.variant = variant;
		self
	}
}

impl<'a> egui::Widget for Alert<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());
		let color = theme.get_color(self.variant);

		// For alerts, background is very light (or dark in dark mode), border and text is
		// the variant color. To approximate "light variant background", we can just use
		// the color with reduced alpha, or standard secondary bg with colored border.
		// Let's use secondary background but color the left border heavily.

		let frame = egui::Frame::new()
			.fill(theme.secondary)
			.stroke(egui::Stroke::new(1.0, theme.border))
			.inner_margin(16.0)
			.corner_radius(egui::CornerRadius::same(6));

		let response = frame
			.show(ui, |ui| {
				ui.set_width(300.0);
				ui.horizontal(|ui| {
					// Colored side bar or icon could go here. Let's just color the title.
					ui.label(egui::RichText::new(self.title).strong().color(color));
					ui.add_space(4.0);
					ui.label(egui::RichText::new(self.content).color(theme.foreground));
				});
			})
			.response;

		// Custom thick left border for the alert
		let painter = ui.painter();
		let mut left_border = response.rect;
		left_border.max.x = left_border.min.x + 4.0;
		painter.rect_filled(
			left_border,
			egui::CornerRadius {
				nw: 6,
				sw: 6,
				ne: 0,
				se: 0,
			},
			color,
		);

		response
	}
}
