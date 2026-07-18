use crate::theme::{ElegantTheme, Variant};
use egui;

pub struct ElegantToast<'a> {
	id_salt: egui::Id,
	title: &'a str,
	message: &'a str,
	variant: Variant,
}

impl<'a> ElegantToast<'a> {
	pub fn new(
		id_salt: impl std::hash::Hash + std::fmt::Debug,
		title: &'a str,
		message: &'a str,
	) -> Self {
		Self {
			id_salt: egui::Id::new(id_salt),
			title,
			message,
			variant: Variant::Primary,
		}
	}

	pub fn variant(mut self, variant: Variant) -> Self {
		self.variant = variant;
		self
	}

	pub fn show(self, ctx: &egui::Context) {
		let theme = ElegantTheme::get(ctx);

		let color = match self.variant {
			Variant::Primary => theme.primary,
			Variant::Secondary => theme.foreground.linear_multiply(0.5),
			Variant::Danger => theme.danger,
			Variant::Warning => theme.warning,
			Variant::Success => theme.success,
			Variant::Info => theme.info,
		};

		let bg_color = if theme.is_dark {
			color.linear_multiply(0.2)
		} else {
			color.linear_multiply(0.1)
		};

		let mut frame = egui::Frame::default();
		frame.fill = bg_color;
		frame.stroke = egui::Stroke::new(1.0, color.linear_multiply(0.3));
		frame.inner_margin = egui::Margin::same(16);
		frame.corner_radius = egui::CornerRadius::same(8);

		egui::Window::new(self.title)
			.id(self.id_salt)
			.frame(frame)
			.title_bar(false)
			.collapsible(false)
			.resizable(false)
			.anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
			.show(ctx, |ui| {
				ui.horizontal(|ui| {
					ui.vertical(|ui| {
						ui.label(egui::RichText::new(self.title).strong().color(color));
						ui.add_space(4.0);
						ui.label(
							egui::RichText::new(self.message).color(theme.foreground),
						);
					});
				});
			});
	}
}
