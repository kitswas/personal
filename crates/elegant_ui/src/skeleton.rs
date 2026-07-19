use crate::{theme::ElegantTheme, traits::Elegant};
use egui;

pub struct Skeleton {
	width: f32,
	height: f32,
	rounding: f32,
}

impl Skeleton {
	pub fn new(width: f32, height: f32) -> Self {
		Self {
			width,
			height,
			rounding: 4.0,
		}
	}

	pub fn rounding(mut self, rounding: f32) -> Self {
		self.rounding = rounding;
		self
	}
}

impl egui::Widget for Skeleton {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let (rect, response) = ui.allocate_exact_size(
			egui::vec2(self.width, self.height),
			egui::Sense::hover(),
		);

		let theme = ElegantTheme::get(ui.ctx());

		// Simple pulsation effect
		let time = ui.input(|i| i.time);
		let t = (time * 3.0).sin() as f32 * 0.5 + 0.5; // 0.0 to 1.0

		let base_color = theme.background.linear_multiply(0.8);
		let highlight_color = theme.background.linear_multiply(0.9);

		let color = if theme.is_dark {
			egui::Color32::from_rgb(
				(base_color.r() as f32 * (1.0 - t) + highlight_color.r() as f32 * t)
					as u8,
				(base_color.g() as f32 * (1.0 - t) + highlight_color.g() as f32 * t)
					as u8,
				(base_color.b() as f32 * (1.0 - t) + highlight_color.b() as f32 * t)
					as u8,
			)
		} else {
			let base_color = egui::Color32::from_gray(230);
			let highlight_color = egui::Color32::from_gray(245);
			egui::Color32::from_rgb(
				(base_color.r() as f32 * (1.0 - t) + highlight_color.r() as f32 * t)
					as u8,
				(base_color.g() as f32 * (1.0 - t) + highlight_color.g() as f32 * t)
					as u8,
				(base_color.b() as f32 * (1.0 - t) + highlight_color.b() as f32 * t)
					as u8,
			)
		};

		ui.painter().rect_filled(rect, self.rounding, color);
		ui.ctx().request_repaint(); // Keep animating

		response
	}
}

impl Elegant for Skeleton {}
crate::impl_flex_widget!(Skeleton);
