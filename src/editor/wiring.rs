use egui::{Color32, Painter, Pos2, Rect, Stroke, pos2};

use crate::editor::constants::COLOR_WIRE_PENDING;

use super::app::App;
use super::constants::{COLOR_WIRE, COLOR_WIRE_HIGH, COLOR_WIRE_LOW};
use super::graph::Wire;

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    //  Wires
    // ─────────────────────────────────────────────────────────────────────────

    pub fn draw_wires(
        &self,
        painter: &Painter,
        canvas_origin: Pos2,
        canvas_rect: Rect,
        hovered_wire: &Option<Wire>,
    ) {
        for wire in &self.graph.wires {
            let from_screen = self.port_to_screen_pos(&wire.from, true,  canvas_origin, canvas_rect);
            let to_screen   = self.port_to_screen_pos(&wire.to,   false, canvas_origin, canvas_rect);

            if let (Some(from_pos), Some(to_pos)) = (from_screen, to_screen) {
                let wire_color = if self.sim_runtime.is_some() {
                    let wire_index = self.port_to_wire_index
                        .get(&(wire.from.node, wire.from.port, true))
                        .copied();
                    match wire_index {
                        Some(idx) if self.live_wire_signals.get(&idx).copied().unwrap_or(false) => {
                            COLOR_WIRE_HIGH
                        }
                        Some(_) => COLOR_WIRE_LOW,
                        None    => COLOR_WIRE,
                    }
                } else {
                    COLOR_WIRE
                };

                let is_hovered = hovered_wire.as_ref() == Some(wire);
                let line_width = if is_hovered { 4.5 } else { 2.5 };
                let color      = if is_hovered { Color32::from_rgb(255, 80, 80) } else { wire_color };
                draw_bezier_wire(painter, from_pos, to_pos, color, line_width);
            }
        }
    }


    pub fn draw_pending_wires(
        &self,
        painter: &Painter,
        canvas_origin: Pos2,
        canvas_rect: Rect,
        pointer_screen_pos: Option<Pos2>,
    ) {
        if let Some(wire_start_port) = &self.pending_wire_start.clone() {
            let start_screen =
                self.port_to_screen_pos(wire_start_port, true, canvas_origin, canvas_rect);
            if let (Some(start_pos), Some(mouse_pos)) = (start_screen, pointer_screen_pos) {
                draw_bezier_wire(&painter, start_pos, mouse_pos, COLOR_WIRE_PENDING, 2.0);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Rect construction & Bezier wire rendering
// ─────────────────────────────────────────────────────────────────────────────

/// Build a canvas-space `Rect` from any two opposite corners (order does not matter).
pub fn canvas_rect_from_two_points(corner_a: Pos2, corner_b: Pos2) -> Rect {
    Rect::from_min_max(
        Pos2::new(corner_a.x.min(corner_b.x), corner_a.y.min(corner_b.y)),
        Pos2::new(corner_a.x.max(corner_b.x), corner_a.y.max(corner_b.y)),
    )
}

/// Draw a smooth cubic Bezier wire between two screen-space points.
pub fn draw_bezier_wire(painter: &Painter, from: Pos2, to: Pos2, color: Color32, line_width: f32) {
    let horizontal_ctrl_offset = ((to.x - from.x).abs() * 0.45).max(60.0);
    let ctrl1 = pos2(from.x + horizontal_ctrl_offset, from.y);
    let ctrl2 = pos2(to.x   - horizontal_ctrl_offset, to.y);
    let mut prev_point = from;
    for step in 1..=32usize {
        let t = step as f32 / 32.0;
        let u = 1.0 - t;
        let next_point = pos2(
            u*u*u*from.x + 3.0*u*u*t*ctrl1.x + 3.0*u*t*t*ctrl2.x + t*t*t*to.x,
            u*u*u*from.y + 3.0*u*u*t*ctrl1.y + 3.0*u*t*t*ctrl2.y + t*t*t*to.y,
        );
        painter.line_segment([prev_point, next_point], Stroke::new(line_width, color));
        prev_point = next_point;
    }
}