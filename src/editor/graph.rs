use egui::Pos2;
use serde::{Deserialize, Serialize};

use circut_lang::prelude::PortRef;

// ─────────────────────────────────────────────────────────────────────────────
//  Serde helper for egui::Pos2
// ─────────────────────────────────────────────────────────────────────────────

pub mod pos2_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(pos: &Pos2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (pos.x, pos.y).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pos2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y) = <(f32, f32)>::deserialize(deserializer)?;
        Ok(Pos2 { x, y })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  EditorNodeKind
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EditorNodeKind {
    Nand,
    /// Name of the gate in App::library, identifying which saved gate this instance represents.
    SavedGate(String),
    /// Name of an imported external gate.
    External(String),
}

// ─────────────────────────────────────────────────────────────────────────────
//  EditorNode — one gate placed on the canvas
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditorNode {
    pub label: String,
    /// Top-left corner in canvas space (not screen space).
    #[serde(with = "pos2_serde")]
    pub pos: Pos2,
    pub input_count: usize,
    pub output_count: usize,
    pub kind: EditorNodeKind,
    /// Names of each input port, in order.  Length always equals `input_count`.
    pub input_labels: Vec<String>,
    /// Names of each output port, in order.  Length always equals `output_count`.
    pub output_labels: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Wire — a directed connection from one output port to one input port
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Wire {
    pub from: PortRef,
    pub to: PortRef,
}

// ─────────────────────────────────────────────────────────────────────────────
//  LibraryGate — a saved circuit that can be re-used as a sub-gate
// ─────────────────────────────────────────────────────────────────────────────

/// A gate that has been saved to the library.
/// Stores both display metadata and the complete editor graph so it can be re-opened.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LibraryGate {
    pub name: String,
    pub input_count: usize,
    pub output_count: usize,
    pub graph: EditorGraph,
}

// ─────────────────────────────────────────────────────────────────────────────
//  EditorGraph — the full graph for one circuit level
// ─────────────────────────────────────────────────────────────────────────────

/// The editor's full representation of one circuit level.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditorGraph {
    /// Names of the external input ports shown on the left rail.
    pub inputs: Vec<String>,
    /// Names of the external output ports shown on the right rail.
    pub outputs: Vec<String>,
    /// Internal gate nodes placed on the canvas.
    pub nodes: Vec<EditorNode>,
    /// All wires connecting ports to each other.
    pub wires: Vec<Wire>,
}

impl Default for EditorGraph {
    fn default() -> Self {
        Self {
            inputs: vec!["I0".into()],
            outputs: vec!["O0".into()],
            nodes: vec![],
            wires: vec![],
        }
    }
}

impl EditorGraph {
    // Node-id layout (flat integer used in PortRef::node):
    //   0 .. inputs.len()-1                              → input pseudo-nodes
    //   inputs.len() .. inputs.len()+outputs.len()-1     → output pseudo-nodes
    //   inputs.len()+outputs.len() ..                    → internal gate nodes

    pub fn input_node_id(&self, input_index: usize) -> usize {
        input_index
    }

    pub fn output_node_id(&self, output_index: usize) -> usize {
        self.inputs.len() + output_index
    }

    pub fn gate_node_id(&self, gate_index: usize) -> usize {
        self.inputs.len() + self.outputs.len() + gate_index
    }

    pub fn is_input_node(&self, node_id: usize) -> bool {
        node_id < self.inputs.len()
    }

    pub fn is_output_node(&self, node_id: usize) -> bool {
        node_id >= self.inputs.len() && node_id < self.inputs.len() + self.outputs.len()
    }

    /// Converts a flat node_id back to an index into `self.nodes`, if it refers to a gate.
    pub fn gate_index_from_node_id(&self, node_id: usize) -> Option<usize> {
        let gate_base = self.inputs.len() + self.outputs.len();
        if node_id >= gate_base && node_id - gate_base < self.nodes.len() {
            Some(node_id - gate_base)
        } else {
            None
        }
    }

    // ── Structural mutations that keep wires consistent ───────────────────────

    pub fn add_input(&mut self, name: String) {
        let new_node_id = self.inputs.len();
        for wire in &mut self.wires {
            if wire.from.node >= new_node_id { wire.from.node += 1; }
            if wire.to.node   >= new_node_id { wire.to.node   += 1; }
        }
        self.inputs.push(name);
    }

    pub fn add_output(&mut self, name: String) {
        let new_node_id = self.inputs.len() + self.outputs.len();
        for wire in &mut self.wires {
            if wire.from.node >= new_node_id { wire.from.node += 1; }
            if wire.to.node   >= new_node_id { wire.to.node   += 1; }
        }
        self.outputs.push(name);
    }

    pub fn remove_input(&mut self, input_index: usize) {
        let removed_node_id = self.input_node_id(input_index);
        self.wires.retain(|wire| wire.from.node != removed_node_id);
        for wire in &mut self.wires {
            if wire.from.node > removed_node_id { wire.from.node -= 1; }
            if wire.to.node   > removed_node_id { wire.to.node   -= 1; }
        }
        self.inputs.remove(input_index);
    }

    pub fn remove_output(&mut self, output_index: usize) {
        let removed_node_id = self.output_node_id(output_index);
        self.wires.retain(|wire| wire.to.node != removed_node_id);
        for wire in &mut self.wires {
            if wire.from.node > removed_node_id { wire.from.node -= 1; }
            if wire.to.node   > removed_node_id { wire.to.node   -= 1; }
        }
        self.outputs.remove(output_index);
    }

    pub fn remove_gate(&mut self, gate_index: usize) {
        let removed_node_id = self.gate_node_id(gate_index);
        self.wires.retain(|wire| {
            wire.from.node != removed_node_id && wire.to.node != removed_node_id
        });
        for wire in &mut self.wires {
            if wire.from.node > removed_node_id { wire.from.node -= 1; }
            if wire.to.node   > removed_node_id { wire.to.node   -= 1; }
        }
        self.nodes.remove(gate_index);
    }

    pub fn remove_wire(&mut self, wire_to_remove: &Wire) {
        self.wires.retain(|wire| wire != wire_to_remove);
    }

    pub fn reorder_input(&mut self, old_index: usize, new_index: usize) {
        if old_index == new_index
            || old_index >= self.inputs.len()
            || new_index >= self.inputs.len()
        { return; }

        let permuted_input_node_id = |node_id: usize| -> usize {
            if node_id >= self.inputs.len() { return node_id; }
            if node_id == old_index { return new_index; }
            if old_index < new_index {
                if node_id > old_index && node_id <= new_index { return node_id - 1; }
            } else {
                if node_id >= new_index && node_id < old_index { return node_id + 1; }
            }
            node_id
        };

        for wire in &mut self.wires {
            wire.from.node = permuted_input_node_id(wire.from.node);
            wire.to.node   = permuted_input_node_id(wire.to.node);
        }

        if old_index < new_index {
            self.inputs[old_index..=new_index].rotate_left(1);
        } else {
            self.inputs[new_index..=old_index].rotate_right(1);
        }
    }

    pub fn reorder_output(&mut self, old_index: usize, new_index: usize) {
        if old_index == new_index
            || old_index >= self.outputs.len()
            || new_index >= self.outputs.len()
        { return; }

        let input_count = self.inputs.len();
        let output_count = self.outputs.len();

        let permuted_output_node_id = |node_id: usize| -> usize {
            if node_id < input_count || node_id >= input_count + output_count { return node_id; }
            let output_index = node_id - input_count;
            if output_index == old_index { return input_count + new_index; }
            if old_index < new_index {
                if output_index > old_index && output_index <= new_index { return node_id - 1; }
            } else {
                if output_index >= new_index && output_index < old_index { return node_id + 1; }
            }
            node_id
        };

        for wire in &mut self.wires {
            wire.from.node = permuted_output_node_id(wire.from.node);
            wire.to.node   = permuted_output_node_id(wire.to.node);
        }

        if old_index < new_index {
            self.outputs[old_index..=new_index].rotate_left(1);
        } else {
            self.outputs[new_index..=old_index].rotate_right(1);
        }
    }

    /// Remove all gate nodes whose kind is `SavedGate(library_name)`.
    pub fn remove_all_gates_of_library_name(&mut self, library_name: &str) {
        let gate_indices_to_remove: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(gate_index, node)| {
                if matches!(&node.kind, EditorNodeKind::SavedGate(name) if name == library_name) {
                    Some(gate_index)
                } else {
                    None
                }
            })
            .rev()
            .collect();

        for gate_index in gate_indices_to_remove {
            self.remove_gate(gate_index);
        }
    }

    /// Rename all `SavedGate` references from `old_name` to `new_name`.
    pub fn rename_saved_gate_references(&mut self, old_name: &str, new_name: &str) {
        for node in &mut self.nodes {
            if let EditorNodeKind::SavedGate(ref mut name) = node.kind {
                if name == old_name {
                    *name = new_name.to_string();
                }
            }
        }
    }

    /// Refresh every `SavedGate(gate_name)` node to match the updated definition.
    pub fn update_saved_gate_instances(&mut self, gate_name: &str, updated_library_gate: &LibraryGate) {
        let gate_base = self.inputs.len() + self.outputs.len();

        let matching_node_ids: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(gate_index, node)| {
                if matches!(&node.kind, EditorNodeKind::SavedGate(name) if name == gate_name) {
                    Some(gate_base + gate_index)
                } else {
                    None
                }
            })
            .collect();

        self.wires.retain(|wire| {
            for &node_id in &matching_node_ids {
                if wire.to.node == node_id && wire.to.port >= updated_library_gate.input_count {
                    return false;
                }
                if wire.from.node == node_id && wire.from.port >= updated_library_gate.output_count {
                    return false;
                }
            }
            true
        });

        for node in self.nodes.iter_mut() {
            if !matches!(&node.kind, EditorNodeKind::SavedGate(name) if name == gate_name) {
                continue;
            }
            node.label         = updated_library_gate.name.clone();
            node.input_count   = updated_library_gate.input_count;
            node.output_count  = updated_library_gate.output_count;
            node.input_labels  = updated_library_gate.graph.inputs.clone();
            node.output_labels = updated_library_gate.graph.outputs.clone();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  BulkWireState — state machine for the Shift+drag box-select bulk-wiring feature
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub enum BulkWireState {
    #[default]
    Idle,
    SelectingOutputs {
        drag_start_canvas: Pos2,
        drag_current_canvas: Pos2,
    },
    OutputsChosen {
        selected_output_ports: Vec<PortRef>,
    },
    SelectingInputs {
        selected_output_ports: Vec<PortRef>,
        drag_start_canvas: Pos2,
        drag_current_canvas: Pos2,
    },
}