use std::collections::HashMap;
use std::iter::once;
use circut_lang::prelude::*;
use super::app::App;
use super::graph::{EditorGraph, EditorNodeKind};

impl App {

    pub fn create_script_from_graph(&mut self, entry_point_name: impl Into<String> + Clone) -> CircutLangScript {
        let entry_point_desc = editor_graph_to_desc(&self.graph);
        let gates: HashMap<String, GraphDesc> = self
            .library
            .iter()
            .map(|(name, saved_gate)| (name.clone(), editor_graph_to_desc(&saved_gate.graph)))
            .chain(
                once((entry_point_name.clone().into(), entry_point_desc))
            )
            .collect();

        CircutLangScript{
            entry_point: entry_point_name.into(),
            gates,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Build
    // ─────────────────────────────────────────────────────────────────────────

    /// (Re-)compile the current editor graph into a runnable [`Simulation`].
    pub fn build_simulation_from_graph(&mut self) {
        
        let script = self.create_script_from_graph("entry_point");
        let entry_point_desc = editor_graph_to_desc(&self.graph);
        let port_to_wire_index = build_port_to_wire_index_map(&entry_point_desc);

        match Runtime::new_compile(script, Some(&self.external_gates)) {
            Ok(simulation) => {
                self.sim_runtime        = Some(simulation);
                self.port_to_wire_index = port_to_wire_index;
                self.simulation_error   = None;
            }
            Err(error_message) => {
                self.sim_runtime        = None;
                self.port_to_wire_index = HashMap::new();
                self.simulation_error   = Some(format!("{:?}", error_message));
            }
        }
        self.live_wire_signals = HashMap::new();
        self.output_states     = vec![false; self.graph.outputs.len()];
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Step
    // ─────────────────────────────────────────────────────────────────────────

    /// Inject the current input states, advance the simulation by one tick, and
    /// snapshot the resulting wire signals into `live_wire_signals`.
    pub fn step_simulation(&mut self) {
        let Some(sim_runtime) = &mut self.sim_runtime else { return };

        let mut external_node_library = ExternalNodeLibrary::default();

        match sim_runtime.run_one_tick_with_io(
            self.input_states.clone(),
            &mut external_node_library
        ) {
            Ok(output) => self.output_states = output,
            Err(err) => self.simulation_error = Some(format!("{:?}", err)),
        }

        self.live_wire_signals.clear();
        for &wire_index in self.port_to_wire_index.values() {
            let signal = sim_runtime.wire_value(WireId::new_unchecked(wire_index));
            self.live_wire_signals.insert(wire_index, signal);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  EditorGraph → GraphDesc translation
// ─────────────────────────────────────────────────────────────────────────────

pub fn editor_graph_to_desc(graph: &EditorGraph) -> GraphDesc {
    let input_count  = graph.inputs.len();
    let output_count = graph.outputs.len();
    GraphDesc {
        n_inputs:    input_count,
        n_outputs:   output_count,
        gates: graph.nodes.iter().map(|node| {
            let kind = match &node.kind {
                EditorNodeKind::Nand => GateKind::Nand,
                EditorNodeKind::SavedGate(name) => GateKind::SavedGate(name.clone()),
                EditorNodeKind::External(name) => GateKind::ExternalGate(name.clone()),
            };
            (node.input_count, node.output_count, kind)
        }).collect(),
        wires: graph.wires.iter().map(|wire| WireDesc {
            from: wire.from.clone(),
            to:   wire.to.clone(),
        }).collect(),
    }
}

pub fn build_port_to_wire_index_map(
    desc: &GraphDesc,
) -> HashMap<(usize, usize, bool), u32> {
    let mut port_to_wire: HashMap<(usize, usize, bool), u32> = HashMap::new();
    let mut next_wire_id: u32 = 0;

    for input_index in 0..desc.n_inputs {
        let node_id = GraphDesc::input_base() + input_index;
        port_to_wire.insert((node_id, 0, true), next_wire_id);
        next_wire_id += 1;
    }

    for (gate_slot, (_, gate_output_count, _)) in desc.gates.iter().enumerate() {
        let node_id = desc.gate_base() + gate_slot;
        for port_index in 0..*gate_output_count {
            port_to_wire.insert((node_id, port_index, true), next_wire_id);
            next_wire_id += 1;
        }
    }

    port_to_wire
}