use eframe::egui;

pub struct Progress {
    value: f32, // 0.0 to 1.0
}

impl Progress {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl egui::Widget for Progress {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let height = 8.0;
        let width = ui.available_width().max(100.0);
        let size = egui::vec2(width, height);
        
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let bg_color = egui::Color32::from_rgb(244, 244, 245); // --secondary
            let fill_color = egui::Color32::from_rgb(87, 71, 71); // --primary

            let radius = height / 2.0;

            // Background track
            painter.rect_filled(rect, radius, bg_color);

            // Fill bar
            let mut fill_rect = rect;
            fill_rect.max.x = rect.min.x + (rect.width() * self.value.clamp(0.0, 1.0));
            if fill_rect.width() > 0.0 {
                painter.rect_filled(fill_rect, radius, fill_color);
            }
        }
        response
    }
}
