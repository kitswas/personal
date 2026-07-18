use eframe::egui;

pub struct Avatar<'a> {
    text: &'a str,
}

impl<'a> Avatar<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> egui::Widget for Avatar<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::vec2(32.0, 32.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let bg_color = egui::Color32::from_rgb(244, 244, 245); // --secondary
            let text_color = egui::Color32::from_rgb(87, 71, 71); // --primary

            painter.circle_filled(rect.center(), size.x / 2.0, bg_color);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.text,
                egui::FontId::proportional(14.0),
                text_color,
            );
        }
        response
    }
}
