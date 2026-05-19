use egui::{Pos2, Rect, pos2};

use circut_lang::prelude::PortRef;

use super::app::App;
use super::constants::IO_RAIL_STEP;
use super::nodes::{input_port_canvas_pos, output_port_canvas_pos}; // Updated import

impl App {
    pub fn canvas_to_screen(&self, canvas_pos: Pos2, canvas_origin: Pos2) -> Pos2 {
        canvas_origin + (canvas_pos.to_vec2() + self.canvas_pan) * self.canvas_zoom
    }

    pub fn screen_to_canvas(&self, screen_pos: Pos2, canvas_origin: Pos2) -> Pos2 {
        ((screen_pos - canvas_origin) / self.canvas_zoom - self.canvas_pan).to_pos2()
    }

    pub fn port_to_screen_pos(
        &self,
        port: &PortRef,
        is_output_port: bool,
        canvas_origin: Pos2,
        canvas_rect: Rect,
    ) -> Option<Pos2> {
        let input_count  = self.graph.inputs.len();
        let output_count = self.graph.outputs.len();
        let center_y = canvas_rect.center().y;

        if self.graph.is_input_node(port.node) {
            let start_y = center_y - (input_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
            return Some(pos2(
                canvas_rect.left() + 14.0,
                start_y + port.node as f32 * IO_RAIL_STEP,
            ));
        }

        if self.graph.is_output_node(port.node) {
            let output_index = port.node - input_count;
            let start_y = center_y - (output_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
            return Some(pos2(
                canvas_rect.right() - 14.0,
                start_y + output_index as f32 * IO_RAIL_STEP,
            ));
        }

        if let Some(gate_index) = self.graph.gate_index_from_node_id(port.node) {
            let node = &self.graph.nodes[gate_index];
            let canvas_pos = if is_output_port {
                output_port_canvas_pos(node, port.port)
            } else {
                input_port_canvas_pos(node, port.port)
            };
            return Some(self.canvas_to_screen(canvas_pos, canvas_origin));
        }

        None
    }
}