use egui::{Pos2, Rect, pos2, vec2};


use circut_lang::prelude::PortRef;

use super::app::App;
use super::constants::{IO_RAIL_STEP, NODE_WIDTH, PORT_RADIUS};
use super::nodes::{ // Updated import
    compute_node_height, input_port_canvas_pos, output_port_canvas_pos,
};
use super::graph::Wire;

impl App {
    pub fn hit_test_port(
        &self,
        screen_pos: Pos2,
        canvas_origin: Pos2,
        canvas_rect: Rect,
    ) -> Option<(PortRef, bool)> {
        let input_count  = self.graph.inputs.len();
        let output_count = self.graph.outputs.len();
        let hit_radius   = (PORT_RADIUS + 6.0) * self.canvas_zoom;
        let center_y     = canvas_rect.center().y;

        let inputs_start_y = center_y - (input_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
        for input_index in 0..input_count {
            let dot_pos = pos2(
                canvas_rect.left() + 14.0,
                inputs_start_y + input_index as f32 * IO_RAIL_STEP,
            );
            if (screen_pos - dot_pos).length() < hit_radius {
                return Some((
                    PortRef { node: self.graph.input_node_id(input_index), port: 0 },
                    true,
                ));
            }
        }

        let outputs_start_y = center_y - (output_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
        for output_index in 0..output_count {
            let dot_pos = pos2(
                canvas_rect.right() - 14.0,
                outputs_start_y + output_index as f32 * IO_RAIL_STEP,
            );
            if (screen_pos - dot_pos).length() < hit_radius {
                return Some((
                    PortRef { node: self.graph.output_node_id(output_index), port: 0 },
                    false,
                ));
            }
        }

        for gate_index in 0..self.graph.nodes.len() {
            let node = &self.graph.nodes[gate_index];

            for port_index in 0..node.input_count {
                let dot_pos = self.canvas_to_screen(
                    input_port_canvas_pos(node, port_index),
                    canvas_origin,
                );
                if (screen_pos - dot_pos).length() < hit_radius {
                    return Some((
                        PortRef { node: self.graph.gate_node_id(gate_index), port: port_index },
                        false,
                    ));
                }
            }

            for port_index in 0..node.output_count {
                let dot_pos = self.canvas_to_screen(
                    output_port_canvas_pos(node, port_index),
                    canvas_origin,
                );
                if (screen_pos - dot_pos).length() < hit_radius {
                    return Some((
                        PortRef { node: self.graph.gate_node_id(gate_index), port: port_index },
                        true,
                    ));
                }
            }
        }

        None
    }

    pub fn hit_test_gate(&self, screen_pos: Pos2, canvas_origin: Pos2) -> Option<usize> {
        for gate_index in (0..self.graph.nodes.len()).rev() {
            let node        = &self.graph.nodes[gate_index];
            let node_height = compute_node_height(node.input_count, node.output_count);
            let top_left_screen = self.canvas_to_screen(node.pos, canvas_origin);
            let node_rect = Rect::from_min_size(
                top_left_screen,
                vec2(NODE_WIDTH * self.canvas_zoom, node_height * self.canvas_zoom),
            );
            if node_rect.contains(screen_pos) {
                return Some(gate_index);
            }
        }
        None
    }

    pub fn hit_test_wire(
        &self,
        screen_pos: Pos2,
        canvas_origin: Pos2,
        canvas_rect: Rect,
    ) -> Option<Wire> {
        let hit_radius = 8.0 * self.canvas_zoom;

        for wire in &self.graph.wires {
            let from_screen = self.port_to_screen_pos(&wire.from, true,  canvas_origin, canvas_rect);
            let to_screen   = self.port_to_screen_pos(&wire.to,   false, canvas_origin, canvas_rect);

            if let (Some(from_pos), Some(to_pos)) = (from_screen, to_screen) {
                let horizontal_ctrl_offset = ((to_pos.x - from_pos.x).abs() * 0.45).max(60.0);
                let ctrl1 = pos2(from_pos.x + horizontal_ctrl_offset, from_pos.y);
                let ctrl2 = pos2(to_pos.x   - horizontal_ctrl_offset, to_pos.y);

                for sample_step in 0..=16usize {
                    let t = sample_step as f32 / 16.0;
                    let u = 1.0 - t;
                    let sample_pos = pos2(
                        u*u*u*from_pos.x + 3.0*u*u*t*ctrl1.x + 3.0*u*t*t*ctrl2.x + t*t*t*to_pos.x,
                        u*u*u*from_pos.y + 3.0*u*u*t*ctrl1.y + 3.0*u*t*t*ctrl2.y + t*t*t*to_pos.y,
                    );
                    if (screen_pos - sample_pos).length() < hit_radius {
                        return Some(wire.clone());
                    }
                }
            }
        }

        None
    }
}