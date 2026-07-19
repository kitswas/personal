use crate::{
	theme::{ElegantTheme, Variant},
	traits::Elegant,
};
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

		ui.scope(|ui| {
			if self.ghost {
				ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
				ui.visuals_mut().widgets.inactive.weak_bg_fill =
					egui::Color32::TRANSPARENT;
				ui.visuals_mut().widgets.hovered.bg_fill = theme.secondary;
				ui.visuals_mut().widgets.hovered.weak_bg_fill = theme.secondary;
				ui.visuals_mut().widgets.active.bg_fill = theme.border;
				ui.visuals_mut().widgets.active.weak_bg_fill = theme.border;

				ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
				ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
				ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::NONE;

				ui.visuals_mut().widgets.inactive.fg_stroke.color = color;
				ui.visuals_mut().widgets.hovered.fg_stroke.color =
					theme.hover_color(color);
				ui.visuals_mut().widgets.active.fg_stroke.color =
					theme.active_color(color);
			} else if self.outline {
				ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
				ui.visuals_mut().widgets.inactive.weak_bg_fill =
					egui::Color32::TRANSPARENT;
				ui.visuals_mut().widgets.hovered.bg_fill = theme.secondary;
				ui.visuals_mut().widgets.hovered.weak_bg_fill = theme.secondary;
				ui.visuals_mut().widgets.active.bg_fill = theme.border;
				ui.visuals_mut().widgets.active.weak_bg_fill = theme.border;

				ui.visuals_mut().widgets.inactive.bg_stroke =
					egui::Stroke::new(1.0, color);
				ui.visuals_mut().widgets.hovered.bg_stroke =
					egui::Stroke::new(1.0, theme.hover_color(color));
				ui.visuals_mut().widgets.active.bg_stroke =
					egui::Stroke::new(1.0, theme.active_color(color));

				ui.visuals_mut().widgets.inactive.fg_stroke.color = color;
				ui.visuals_mut().widgets.hovered.fg_stroke.color =
					theme.hover_color(color);
				ui.visuals_mut().widgets.active.fg_stroke.color =
					theme.active_color(color);
			} else {
				let text_color = if self.variant == Variant::Secondary {
					theme.foreground
				} else {
					theme.background
				};

				ui.visuals_mut().widgets.inactive.bg_fill = color;
				ui.visuals_mut().widgets.inactive.weak_bg_fill = color;
				ui.visuals_mut().widgets.hovered.bg_fill = theme.hover_color(color);
				ui.visuals_mut().widgets.hovered.weak_bg_fill = theme.hover_color(color);
				ui.visuals_mut().widgets.active.bg_fill = theme.active_color(color);
				ui.visuals_mut().widgets.active.weak_bg_fill = theme.active_color(color);

				ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
				ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
				ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::NONE;

				ui.visuals_mut().widgets.inactive.fg_stroke.color = text_color;
				ui.visuals_mut().widgets.hovered.fg_stroke.color = text_color;
				ui.visuals_mut().widgets.active.fg_stroke.color = text_color;
			}

			ui.add(egui::Button::new(self.text).wrap_mode(egui::TextWrapMode::Extend))
		})
		.inner
	}
}

impl<'a> Elegant for ElegantButton<'a> {}
crate::impl_flex_widget!(ElegantButton<'a>);
