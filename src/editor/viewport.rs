use egui::{Painter, Pos2, Rect, pos2, Stroke};

use super::app::App;
use super::constants::{COLOR_GRID, GRID_CELL_SIZE};

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    //  Background grid
    // ─────────────────────────────────────────────────────────────────────────

    pub fn draw_grid(&self, painter: &Painter, rect: Rect) {
        let grid_pixel_size = GRID_CELL_SIZE * self.canvas_zoom;
        let pan_remainder_x =
            (self.canvas_pan.x * self.canvas_zoom).rem_euclid(grid_pixel_size);
        let pan_remainder_y =
            (self.canvas_pan.y * self.canvas_zoom).rem_euclid(grid_pixel_size);
        let stroke = Stroke::new(0.5, COLOR_GRID);

        let mut x = rect.left() + pan_remainder_x;
        while x <= rect.right() {
            painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
            x += grid_pixel_size;
        }
        let mut y = rect.top() + pan_remainder_y;
        while y <= rect.bottom() {
            painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], stroke);
            y += grid_pixel_size;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Grid helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Snap a canvas-space position to the nearest grid corner.
pub fn snap_to_grid(pos: Pos2) -> Pos2 {
    Pos2::new(
        (pos.x / GRID_CELL_SIZE).round() * GRID_CELL_SIZE,
        (pos.y / GRID_CELL_SIZE).round() * GRID_CELL_SIZE,
    )
}