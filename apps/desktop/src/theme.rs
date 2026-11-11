use eframe::egui::{self, Color32, CornerRadius, Stroke};
use noah_workspace::view::ThemeMode;
#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub elevated: Color32,
    pub primary: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub border: Color32,
    pub viewport: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub selection: Color32,
}
const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}
impl Palette {
    pub fn new(theme: ThemeMode) -> Self {
        match theme {
            ThemeMode::ArcticDark => Self {
                background: rgb(0x121C26),
                surface: rgb(0x1B2A36),
                elevated: rgb(0x273A49),
                primary: rgb(0x8EBBD2),
                text: rgb(0xE3ECF2),
                secondary: rgb(0xA3B8C6),
                border: rgb(0x354B5C),
                viewport: rgb(0x202D38),
                success: rgb(0x8AC1A9),
                warning: rgb(0xE4B779),
                error: rgb(0xD48F8F),
                selection: rgb(0x324E61),
            },
            ThemeMode::ArcticLight => Self {
                background: rgb(0xECF1F3),
                surface: rgb(0xD1DEE4),
                elevated: rgb(0xF7FAFB),
                primary: rgb(0x587E95),
                text: rgb(0x324D60),
                secondary: rgb(0x536B79),
                border: rgb(0xBACBD4),
                viewport: rgb(0xDCE5E9),
                success: rgb(0x3B7963),
                warning: rgb(0x97651E),
                error: rgb(0xA35050),
                selection: rgb(0xBACBD4),
            },
        }
    }
}
pub fn apply(ctx: &egui::Context, theme: ThemeMode) {
    let p = Palette::new(theme);
    let mut style = (*ctx.style()).clone();
    style.visuals = if theme == ThemeMode::ArcticDark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    style.visuals.panel_fill = p.surface;
    style.visuals.window_fill = p.elevated;
    style.visuals.extreme_bg_color = p.background;
    style.visuals.faint_bg_color = p.surface;
    style.visuals.override_text_color = Some(p.text);
    style.visuals.weak_text_color = Some(p.secondary);
    style.visuals.hyperlink_color = p.primary;
    style.visuals.selection.bg_fill = p.selection;
    style.visuals.selection.stroke = Stroke::new(1.0_f32, p.primary);
    style.visuals.error_fg_color = p.error;
    style.visuals.warn_fg_color = p.warning;
    style.visuals.window_stroke = Stroke::new(1.0_f32, p.border);
    style.visuals.window_corner_radius = CornerRadius::same(12);
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.noninteractive,
    ] {
        widget.bg_fill = p.surface;
        widget.weak_bg_fill = p.surface;
        widget.bg_stroke = Stroke::new(1.0_f32, p.border);
        widget.fg_stroke = Stroke::new(1.0_f32, p.text);
        widget.corner_radius = CornerRadius::same(7);
    }
    style.visuals.widgets.hovered.bg_fill = p.elevated;
    style.visuals.widgets.hovered.weak_bg_fill = p.elevated;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.primary);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(7);
    style.visuals.widgets.active.bg_fill = p.selection;
    style.visuals.widgets.active.weak_bg_fill = p.selection;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, p.primary);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(7);
    style.spacing.item_spacing = egui::vec2(10., 10.);
    style.spacing.button_padding = egui::vec2(14., 8.);
    style.animation_time = 0.18;
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(13.));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(13.));
    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(11.));
    ctx.set_style(style);
}
pub fn action(ui: &mut egui::Ui, label: &str, p: Palette, primary: bool) -> egui::Response {
    let text = egui::RichText::new(label).color(if primary { p.background } else { p.text });
    let response = ui.add(if primary {
        egui::Button::new(text).fill(p.primary)
    } else {
        egui::Button::new(text)
    });
    let glow = ui.ctx().animate_bool(
        response.id.with("arctic_glow"),
        response.hovered() || response.has_focus(),
    );
    if glow > 0.001 {
        ui.painter().rect_stroke(
            response.rect.expand(glow * 1.5),
            CornerRadius::same(8),
            Stroke::new(1.0_f32, p.primary.gamma_multiply(glow)),
            egui::StrokeKind::Outside,
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palettes_follow_spec() {
        let d = Palette::new(ThemeMode::ArcticDark);
        assert_eq!(d.background, rgb(0x121C26));
        assert_eq!(d.viewport, rgb(0x202D38));
        assert_eq!(d.primary, rgb(0x8EBBD2));
        assert_eq!(d.selection, rgb(0x324E61));
        let l = Palette::new(ThemeMode::ArcticLight);
        assert_eq!(l.background, rgb(0xECF1F3));
        assert_eq!(l.text, rgb(0x324D60));
        assert_eq!(l.elevated, rgb(0xF7FAFB));
    }
}
