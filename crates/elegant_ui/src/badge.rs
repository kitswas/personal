use crate::{
	theme::{ElegantTheme, Variant},
	traits::Elegant,
};
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

		let (bg_fill, fg_fill, stroke) = if self.outline {
			let (text_c, stroke_c) = if self.variant == Variant::Secondary {
				(theme.foreground, theme.border)
			} else {
				(color, color)
			};
			(
				egui::Color32::TRANSPARENT,
				text_c,
				egui::Stroke::new(theme.spacing.border_width, stroke_c),
			)
		} else {
			let text_color = if self.variant == Variant::Secondary {
				theme.foreground
			} else {
				theme.background
			};
			(color, text_color, egui::Stroke::NONE)
		};

		egui::Frame::new()
			.fill(bg_fill)
			.stroke(stroke)
			.inner_margin(theme.spacing.badge_inner_margin)
			.corner_radius(egui::CornerRadius::same(
				theme.spacing.badge_corner_radius as u8,
			))
			.show(ui, |ui| {
				ui.add(
					egui::Label::new(
						egui::RichText::new(self.text).color(fg_fill).size(12.0),
					)
					.wrap_mode(egui::TextWrapMode::Extend),
				);
			})
			.response
	}
}

impl<'a> Elegant for ElegantBadge<'a> {}
crate::impl_flex_widget!(ElegantBadge<'a>);
