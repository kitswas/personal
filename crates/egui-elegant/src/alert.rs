use crate::{
	theme::{ElegantTheme, Variant},
	traits::Elegant,
};
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

		let frame = egui::Frame::new()
			.fill(theme.secondary)
			.stroke(egui::Stroke::new(theme.spacing.border_width, theme.border))
			.inner_margin(theme.spacing.alert_inner_margin)
			.corner_radius(egui::CornerRadius::same(theme.spacing.corner_radius as u8));

		let response = frame
			.show(ui, |ui| {
				ui.set_min_width(theme.spacing.alert_min_width);
				ui.horizontal_wrapped(|ui| {
					ui.label(egui::RichText::new(self.title).strong().color(color));
					ui.add_space(4.0);
					ui.label(egui::RichText::new(self.content).color(theme.foreground));
				});
			})
			.response;

		let painter = ui.painter();
		let mut left_border = response.rect;
		left_border.max.x = left_border.min.x + theme.spacing.border_width * 4.0;
		painter.rect_filled(
			left_border,
			egui::CornerRadius {
				nw: theme.spacing.corner_radius as u8,
				sw: theme.spacing.corner_radius as u8,
				ne: 0,
				se: 0,
			},
			color,
		);

		response
	}
}

impl<'a> Elegant for Alert<'a> {
	/// Alerts should have a minimum width so they don't collapse too narrow.
	fn flex_default_min_width() -> Option<f32> {
		Some(240.0)
	}
}
crate::impl_flex_widget!(Alert<'a>);
