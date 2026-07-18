use crate::theme::{ElegantTheme, Variant};
use egui;

pub struct ElegantBadge<'a> {
	text: &'a str,
	variant: Variant,
	outline: bool,
}

impl<'a> ElegantBadge<'a> {
	pub fn new(text: &'a str) -> Self {
		Self {
			text,
			variant: Variant::Primary,
			outline: false,
		}
	}

	pub fn variant(mut self, variant: Variant) -> Self {
		self.variant = variant;
		self
	}

	pub fn outline(mut self) -> Self {
		self.outline = true;
		self
	}
}

impl<'a> egui::Widget for ElegantBadge<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());
		let color = theme.get_color(self.variant);

		let btn = if self.outline {
			egui::Button::new(egui::RichText::new(self.text).color(color).size(12.0))
				.fill(egui::Color32::TRANSPARENT)
				.stroke(egui::Stroke::new(1.0, color))
				.corner_radius(egui::CornerRadius::same(12))
		} else {
			let text_color = if self.variant == Variant::Secondary {
				theme.foreground
			} else {
				theme.background
			};
			egui::Button::new(egui::RichText::new(self.text).color(text_color).size(12.0))
				.fill(color)
				.stroke(egui::Stroke::NONE)
				.corner_radius(egui::CornerRadius::same(12))
		};

		ui.add_enabled(false, btn)
	}
}
