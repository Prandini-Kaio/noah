use crate::theme::Palette;
use eframe::egui::{self, Align2, FontId, Pos2, Rect, Stroke, Vec2};
use noah_workspace::{
    Scene,
    view::{nice_step, ruler_scale},
};
pub const RULER: f32 = 28.;
fn number(value: f64) -> String {
    if value == 0. {
        "0".into()
    } else if value.abs() >= 1e6 || value.abs() < 0.001 {
        format!("{value:.1e}")
    } else {
        format!("{value:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .into()
    }
}
pub fn draw(
    painter: &egui::Painter,
    scene: &Scene,
    outer: Rect,
    rect: Rect,
    p: Palette,
    pointer: Option<Pos2>,
) {
    let Ok(scale) = ruler_scale(scene, &scene.camera, rect.height() as f64) else {
        return;
    };
    let upp = scale.units_per_pixel;
    if !upp.is_finite() || upp <= 0. {
        return;
    }
    if scene.view.rulers {
        painter.rect_filled(
            Rect::from_min_max(outer.min, Pos2::new(outer.right(), rect.top())),
            0.,
            p.surface,
        );
        painter.rect_filled(
            Rect::from_min_max(outer.min, Pos2::new(rect.left(), outer.bottom())),
            0.,
            p.surface,
        );
        painter.line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0_f32, p.border),
        );
        painter.line_segment(
            [rect.left_top(), rect.left_bottom()],
            Stroke::new(1.0_f32, p.border),
        );
        painter.text(
            outer.min + Vec2::new(4., 8.),
            Align2::LEFT_TOP,
            if scale.unit == "u. cena" {
                "u"
            } else {
                scale.unit
            },
            FontId::proportional(10.),
            if scale.stale { p.warning } else { p.primary },
        );
        let step = nice_step(upp * 85.);
        let minor = step / 5.;
        for axis in 0..2 {
            let span = if axis == 0 {
                rect.width()
            } else {
                rect.height()
            } as f64;
            let low = scale.origin[axis] - span * 0.5 * upp;
            let start = (low / minor).floor();
            if !start.is_finite() || start.abs() > 1e15 {
                continue;
            }
            let count = (span * upp / minor).ceil() as usize + 2;
            for k in 0..count.min(400) {
                let index = start + k as f64;
                let value = index * minor;
                let offset = (value - scale.origin[axis]) / upp + span * 0.5;
                let screen = if axis == 0 {
                    rect.left() + offset as f32
                } else {
                    rect.bottom() - offset as f32
                };
                if screen < if axis == 0 { rect.left() } else { rect.top() }
                    || screen
                        > if axis == 0 {
                            rect.right()
                        } else {
                            rect.bottom()
                        }
                {
                    continue;
                }
                let major = index.rem_euclid(5.).abs() < 0.01;
                let tick = if major { 12. } else { 5. };
                if axis == 0 {
                    painter.line_segment(
                        [
                            Pos2::new(screen, rect.top()),
                            Pos2::new(screen, rect.top() - tick),
                        ],
                        Stroke::new(1.0_f32, p.secondary),
                    );
                    if major {
                        painter.text(
                            Pos2::new(screen + 3., outer.top() + 3.),
                            Align2::LEFT_TOP,
                            number(if value.abs() < step * 1e-9 { 0. } else { value }),
                            FontId::proportional(9.),
                            p.secondary,
                        );
                    }
                } else {
                    painter.line_segment(
                        [
                            Pos2::new(rect.left(), screen),
                            Pos2::new(rect.left() - tick, screen),
                        ],
                        Stroke::new(1.0_f32, p.secondary),
                    );
                    if major {
                        let galley = painter.layout_no_wrap(
                            number(if value.abs() < step * 1e-9 { 0. } else { value }),
                            FontId::proportional(9.),
                            p.secondary,
                        );
                        painter.add(
                            egui::epaint::TextShape::new(
                                Pos2::new(outer.left() + 3., screen - 3.),
                                galley,
                                p.secondary,
                            )
                            .with_angle(-std::f32::consts::FRAC_PI_2),
                        );
                    }
                }
            }
        }
        if let Some(pos) = pointer.filter(|pos| rect.contains(*pos)) {
            painter.line_segment(
                [Pos2::new(pos.x, outer.top()), Pos2::new(pos.x, rect.top())],
                Stroke::new(1.5_f32, p.primary),
            );
            painter.line_segment(
                [
                    Pos2::new(outer.left(), pos.y),
                    Pos2::new(rect.left(), pos.y),
                ],
                Stroke::new(1.5_f32, p.primary),
            );
        }
    }
    let length = nice_step(upp * 100.);
    let px = (length / upp) as f32;
    if px.is_finite() && px < rect.width() * 0.5 {
        let a = rect.left_bottom() + Vec2::new(24., -54.);
        let b = a + Vec2::new(px, 0.);
        painter.line_segment([a, b], Stroke::new(2.0_f32, p.primary));
        for q in [a, b] {
            painter.line_segment(
                [q - Vec2::new(0., 4.), q + Vec2::new(0., 4.)],
                Stroke::new(1.0_f32, p.primary),
            );
        }
        let qualifier = if scale.perspective {
            " · plano do alvo"
        } else {
            " · ortográfica"
        };
        painter.text(
            a - Vec2::new(0., 8.),
            Align2::LEFT_BOTTOM,
            format!("{} {}{}", number(length), scale.unit, qualifier),
            FontId::proportional(10.),
            if scale.stale { p.warning } else { p.secondary },
        );
    }
    if scale.stale {
        painter.text(
            rect.center_top() + Vec2::new(0., 42.),
            Align2::CENTER_TOP,
            "Referência alterada · recalibre a escala",
            FontId::proportional(11.),
            p.warning,
        );
    }
    if scene.view.axes {
        let origin = rect.right_bottom() - Vec2::new(55., 65.);
        let (_, right, up, _) = scene.camera.basis();
        for (axis, name, color) in [
            ([1., 0., 0.], "X", p.error),
            ([0., 1., 0.], "Y", p.success),
            ([0., 0., 1.], "Z", p.primary),
        ] {
            let d = Vec2::new(
                noah_workspace::camera::dot(axis, right) as f32,
                -noah_workspace::camera::dot(axis, up) as f32,
            ) * 30.;
            painter.line_segment([origin, origin + d], Stroke::new(2.0_f32, color));
            painter.text(
                origin + d * 1.3,
                Align2::CENTER_CENTER,
                name,
                FontId::proportional(10.),
                color,
            );
        }
    }
}

pub fn ortho_grid(painter: &egui::Painter, scene: &Scene, rect: Rect, p: Palette) {
    let Ok(scale) = ruler_scale(scene, &scene.camera, rect.height() as f64) else {
        return;
    };
    let upp = scale.units_per_pixel;
    if !upp.is_finite() || upp <= 0. {
        return;
    }
    let step = nice_step(upp * 85.);
    for axis in 0..2 {
        let span = if axis == 0 {
            rect.width()
        } else {
            rect.height()
        } as f64;
        let first = ((scale.origin[axis] - span * 0.5 * upp) / step).floor();
        if !first.is_finite() || first.abs() > 1e15 {
            continue;
        }
        for i in 0..((span * upp / step).ceil() as usize + 2).min(100) {
            let value = (first + i as f64) * step;
            let offset = (value - scale.origin[axis]) / upp + span * 0.5;
            let line = if axis == 0 {
                let x = rect.left() + offset as f32;
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())]
            } else {
                let y = rect.bottom() - offset as f32;
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)]
            };
            let zero = value.abs() < step * 1e-9;
            painter.line_segment(
                line,
                Stroke::new(
                    if zero { 1.2_f32 } else { 1.0_f32 },
                    p.border.gamma_multiply(if zero { 0.9 } else { 0.45 }),
                ),
            );
        }
    }
}
