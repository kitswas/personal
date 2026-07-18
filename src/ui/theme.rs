use eframe::egui;

pub fn apply_theme(ctx: &egui::Context) {
    ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::Light);

    let mut style = (*ctx.style_of(egui::Theme::Light)).clone();

    // Spacing
    style.spacing.item_spacing = egui::vec2(16.0, 16.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(24);

    // Rounding
    let radius = egui::CornerRadius::same(6);
    style.visuals.window_corner_radius = radius;
    style.visuals.widgets.noninteractive.corner_radius = radius;
    style.visuals.widgets.inactive.corner_radius = radius;
    style.visuals.widgets.hovered.corner_radius = radius;
    style.visuals.widgets.active.corner_radius = radius;

    let mut visuals = egui::Visuals::light();
    let bg = egui::Color32::from_rgb(255, 255, 255);
    let fg = egui::Color32::from_rgb(9, 9, 11);
    let border = egui::Color32::from_rgb(212, 212, 216);
    let secondary = egui::Color32::from_rgb(244, 244, 245);

    visuals.window_fill = bg;
    visuals.panel_fill = bg;
    
    visuals.widgets.noninteractive.bg_fill = bg;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.inactive.bg_fill = bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.hovered.bg_fill = secondary;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.active.bg_fill = border;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, fg);

    style.visuals = visuals;
    ctx.set_style_of(egui::Theme::Light, style);
}
