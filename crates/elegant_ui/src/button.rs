use crate::theme::{ElegantTheme, Variant};
use egui;

pub struct ElegantButton<'a> {
	text: &'a str,
	variant: Variant,
	outline: bool,
	ghost: bool,
}

impl<'a> ElegantButton<'a> {
	pub fn new(text: &'a str) -> Self {
		Self {
			text,
			variant: Variant::Primary,
			outline: false,
			ghost: false,
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

	pub fn ghost(mut self) -> Self {
		self.ghost = true;
		self
	}
}

impl<'a> egui::Widget for ElegantButton<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());
		let color = theme.get_color(self.variant);

		let btn = if self.ghost {
			egui::Button::new(egui::RichText::new(self.text).color(color))
				.fill(egui::Color32::TRANSPARENT)
				.stroke(egui::Stroke::NONE)
		} else if self.outline {
			egui::Button::new(egui::RichText::new(self.text).color(color))
				.fill(egui::Color32::TRANSPARENT)
				.stroke(egui::Stroke::new(1.0, color))
		} else {
			let text_color = if self.variant == Variant::Secondary {
				theme.foreground
			} else {
				theme.background
			};
			egui::Button::new(egui::RichText::new(self.text).color(text_color))
				.fill(color)
				.stroke(egui::Stroke::NONE)
		};

		ui.add(btn)
	}
}
