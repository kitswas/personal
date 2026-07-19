use egui::{
	Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, Id, Margin,
	Stroke, Visuals,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeMode {
	Light,
	Dark,
	System,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Variant {
	Primary,
	Secondary,
	Success,
	Warning,
	Danger,
	Info,
}

impl Default for Variant {
	fn default() -> Self {
		Self::Primary
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MonaspaceFont {
	Argon,
	Krypton,
	Neon,
	Radon,
	Xenon,
}

impl Default for MonaspaceFont {
	fn default() -> Self {
		Self::Neon
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElegantFont {
	Monaspace(MonaspaceFont),
	#[cfg(feature = "noto")]
	Noto,
}

impl Default for ElegantFont {
	fn default() -> Self {
		Self::Monaspace(MonaspaceFont::default())
	}
}

impl From<MonaspaceFont> for ElegantFont {
	fn from(font: MonaspaceFont) -> Self {
		Self::Monaspace(font)
	}
}

pub fn tint_color(color: Color32, factor: f32) -> Color32 {
	let r = color.r() as f32;
	let g = color.g() as f32;
	let b = color.b() as f32;
	let new_r = r + (255.0 - r) * factor;
	let new_g = g + (255.0 - g) * factor;
	let new_b = b + (255.0 - b) * factor;
	Color32::from_rgb(
		new_r.clamp(0.0, 255.0) as u8,
		new_g.clamp(0.0, 255.0) as u8,
		new_b.clamp(0.0, 255.0) as u8,
	)
}

pub fn shade_color(color: Color32, factor: f32) -> Color32 {
	let r = color.r() as f32;
	let g = color.g() as f32;
	let b = color.b() as f32;
	let new_r = r * (1.0 - factor);
	let new_g = g * (1.0 - factor);
	let new_b = b * (1.0 - factor);
	Color32::from_rgb(
		new_r.clamp(0.0, 255.0) as u8,
		new_g.clamp(0.0, 255.0) as u8,
		new_b.clamp(0.0, 255.0) as u8,
	)
}

#[derive(Clone, Debug)]
pub struct SpacingConfig {
	pub button_padding: egui::Vec2,
	pub item_spacing: egui::Vec2,
	pub window_margin: f32,
	pub corner_radius: f32,
	pub badge_inner_margin: egui::Vec2,
	pub badge_corner_radius: f32,
	pub card_inner_margin: f32,
	pub alert_inner_margin: f32,
	pub alert_min_width: f32,
	pub input_inner_margin: egui::Vec2,
	pub border_width: f32,
}

impl Default for SpacingConfig {
	fn default() -> Self {
		Self {
			button_padding: egui::vec2(16.0, 8.0),
			item_spacing: egui::vec2(16.0, 16.0),
			window_margin: 24.0,
			corner_radius: 6.0,
			badge_inner_margin: egui::vec2(8.0, 4.0),
			badge_corner_radius: 12.0,
			card_inner_margin: 24.0,
			alert_inner_margin: 16.0,
			alert_min_width: 240.0,
			input_inner_margin: egui::vec2(12.0, 8.0),
			border_width: 1.0,
		}
	}
}

#[derive(Clone, Debug)]
pub struct ElegantTheme {
	pub primary: Color32,
	pub secondary: Color32,
	pub background: Color32,
	pub foreground: Color32,
	pub border: Color32,
	pub success: Color32,
	pub warning: Color32,
	pub danger: Color32,
	pub info: Color32,
	pub is_dark: bool,
	pub font: ElegantFont,
	pub spacing: SpacingConfig,
}

impl ElegantTheme {
	pub fn get_color(&self, variant: Variant) -> Color32 {
		match variant {
			Variant::Primary => self.primary,
			Variant::Secondary => self.secondary,
			Variant::Success => self.success,
			Variant::Warning => self.warning,
			Variant::Danger => self.danger,
			Variant::Info => self.info,
		}
	}

	pub fn hover_color(&self, color: Color32) -> Color32 {
		if self.is_dark {
			tint_color(color, 0.15)
		} else {
			shade_color(color, 0.1)
		}
	}

	pub fn active_color(&self, color: Color32) -> Color32 {
		if self.is_dark {
			tint_color(color, 0.25)
		} else {
			shade_color(color, 0.2)
		}
	}

	pub fn build(mode: ThemeMode, font: impl Into<ElegantFont>) -> Self {
		let font = font.into();
		let is_dark = match mode {
			ThemeMode::Dark => true,
			ThemeMode::Light => false,
			ThemeMode::System => is_system_dark_mode(),
		};

		let primary = get_os_accent_color().unwrap_or_else(|| {
			if is_dark {
				Color32::from_rgb(203, 166, 247)
			} else {
				Color32::from_rgb(87, 71, 71)
			}
		});

		let spacing = SpacingConfig::default();

		if is_dark {
			Self {
				primary,
				secondary: Color32::from_rgb(49, 50, 68),
				background: Color32::from_rgb(30, 30, 46),
				foreground: Color32::from_rgb(205, 214, 244),
				border: Color32::from_rgb(88, 91, 112),
				success: Color32::from_rgb(166, 227, 161),
				warning: Color32::from_rgb(249, 226, 175),
				danger: Color32::from_rgb(243, 139, 168),
				info: Color32::from_rgb(137, 180, 250),
				is_dark,
				font,
				spacing,
			}
		} else {
			Self {
				primary,
				secondary: Color32::from_rgb(244, 244, 245),
				background: Color32::from_rgb(255, 255, 255),
				foreground: Color32::from_rgb(9, 9, 11),
				border: Color32::from_rgb(212, 212, 216),
				success: Color32::from_rgb(0, 128, 50),
				warning: Color32::from_rgb(204, 153, 0),
				danger: Color32::from_rgb(204, 0, 0),
				info: Color32::from_rgb(0, 102, 204),
				is_dark,
				font,
				spacing,
			}
		}
	}

	pub fn font_definitions(&self) -> FontDefinitions {
		let mut fonts = FontDefinitions::default();

		match self.font {
			ElegantFont::Monaspace(mf) => {
				let font_bytes = match mf {
					MonaspaceFont::Argon => {
						include_bytes!("../assets/argon.ttf").as_slice()
					},
					MonaspaceFont::Krypton => {
						include_bytes!("../assets/krypton.ttf").as_slice()
					},
					MonaspaceFont::Neon => {
						include_bytes!("../assets/neon.ttf").as_slice()
					},
					MonaspaceFont::Radon => {
						include_bytes!("../assets/radon.ttf").as_slice()
					},
					MonaspaceFont::Xenon => {
						include_bytes!("../assets/xenon.ttf").as_slice()
					},
				};
				fonts.font_data.insert(
					"elegant_font".to_owned(),
					std::sync::Arc::new(FontData::from_static(font_bytes)),
				);
				fonts
					.families
					.entry(FontFamily::Proportional)
					.or_default()
					.insert(0, "elegant_font".to_owned());
				fonts
					.families
					.entry(FontFamily::Monospace)
					.or_default()
					.insert(0, "elegant_font".to_owned());
			},
			#[cfg(feature = "noto")]
			ElegantFont::Noto => {
				let downloaded_fonts = noto_fonts_dl::load_fonts();
				for (name, data) in downloaded_fonts {
					fonts.font_data.insert(
						name.clone(),
						std::sync::Arc::new(FontData::from_owned(data.clone())),
					);
					fonts
						.families
						.entry(FontFamily::Proportional)
						.or_default()
						.push(name.clone());
					fonts
						.families
						.entry(FontFamily::Monospace)
						.or_default()
						.push(name.clone());
				}

				// Provide Neon as the monospace fallback when Noto is used
				let neon_bytes = include_bytes!("../assets/neon.ttf").as_slice();
				fonts.font_data.insert(
					"elegant_mono".to_owned(),
					std::sync::Arc::new(FontData::from_static(neon_bytes)),
				);
				fonts
					.families
					.entry(FontFamily::Monospace)
					.or_default()
					.insert(0, "elegant_mono".to_owned());
			},
		}

		fonts
	}

	pub fn apply(&self, ctx: &Context) {
		ctx.set_fonts(self.font_definitions());
		self.apply_visuals(ctx);
	}

	pub fn apply_visuals(&self, ctx: &Context) {
		ctx.data_mut(|d| d.insert_temp(Id::new("elegant_theme"), self.clone()));

		let mut style = (*ctx.style_of(if self.is_dark {
			egui::Theme::Dark
		} else {
			egui::Theme::Light
		}))
		.clone();

		style.spacing.item_spacing = self.spacing.item_spacing;
		style.spacing.button_padding = self.spacing.button_padding;
		style.spacing.window_margin = Margin::same(self.spacing.window_margin as i8);

		let radius = CornerRadius::same(self.spacing.corner_radius as u8);
		style.visuals.window_corner_radius = radius;
		style.visuals.widgets.noninteractive.corner_radius = radius;
		style.visuals.widgets.inactive.corner_radius = radius;
		style.visuals.widgets.hovered.corner_radius = radius;
		style.visuals.widgets.active.corner_radius = radius;

		let mut visuals = if self.is_dark {
			Visuals::dark()
		} else {
			Visuals::light()
		};
		visuals.window_fill = self.background;
		visuals.panel_fill = self.background;

		let border_width = self.spacing.border_width;
		visuals.widgets.noninteractive.bg_fill = self.background;
		visuals.widgets.noninteractive.bg_stroke = Stroke::new(border_width, self.border);
		visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.foreground);

		visuals.widgets.inactive.bg_fill = self.background;
		visuals.widgets.inactive.bg_stroke = Stroke::new(border_width, self.border);
		visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.foreground);

		visuals.widgets.hovered.bg_fill = self.secondary;
		visuals.widgets.hovered.bg_stroke = Stroke::new(border_width, self.border);
		visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.foreground);

		visuals.widgets.active.bg_fill = self.border;
		visuals.widgets.active.bg_stroke = Stroke::new(border_width, self.border);
		visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.foreground);

		style.visuals = visuals;

		let target_theme = if self.is_dark {
			egui::Theme::Dark
		} else {
			egui::Theme::Light
		};
		ctx.options_mut(|o| {
			o.theme_preference = if self.is_dark {
				egui::ThemePreference::Dark
			} else {
				egui::ThemePreference::Light
			}
		});
		ctx.set_style_of(target_theme, style);
	}

	pub fn get(ctx: &Context) -> Self {
		ctx.data_mut(|d| {
			d.get_temp(Id::new("elegant_theme")).unwrap_or_else(|| {
				ElegantTheme::build(ThemeMode::Light, MonaspaceFont::Neon)
			})
		})
	}
}

pub fn is_system_dark_mode() -> bool {
	matches!(dark_light::detect(), Ok(dark_light::Mode::Dark))
}

#[cfg(target_os = "windows")]
pub fn get_os_accent_color() -> Option<Color32> {
	use winreg::{RegKey, enums::*};
	let hkcu = RegKey::predef(HKEY_CURRENT_USER);
	if let Ok(dwm) = hkcu.open_subkey("Software\\Microsoft\\Windows\\DWM") {
		if let Ok(color_val) = dwm.get_value::<u32, _>("ColorizationColor") {
			let r = ((color_val >> 16) & 0xFF) as u8;
			let g = ((color_val >> 8) & 0xFF) as u8;
			let b = (color_val & 0xFF) as u8;
			return Some(Color32::from_rgb(r, g, b));
		}
	}
	None
}

#[cfg(not(target_os = "windows"))]
fn get_os_accent_color() -> Option<Color32> {
	None
}
