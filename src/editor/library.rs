use std::collections::HashMap;

use bincode::Options;
use serde::{Deserialize, Serialize};

use super::app::App;
use super::graph::{EditorGraph, LibraryGate};

// ─────────────────────────────────────────────────────────────────────────────
//  Library Struct — Encapsulates all library data and internal graph updating
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Library {
    gates: HashMap<String, LibraryGate>,
}

impl Library {
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.gates.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&LibraryGate> {
        self.gates.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &LibraryGate)> {
        self.gates.iter()
    }

    pub fn sorted_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.gates.keys().cloned().collect();
        keys.sort();
        keys
    }

    // ── CRUD Operations ────────────────────────────────────────────────────────

    pub fn save(&mut self, gate: LibraryGate) {
        let name = gate.name.clone();
        let is_update = self.gates.contains_key(&name);
        
        self.gates.insert(name.clone(), gate);

        if is_update {
            let updated_gate = self.gates.get(&name).unwrap().clone();
            self.update_instances_in_internal_graphs(&name, &updated_gate);
        }
    }

    pub fn remove_gate(&mut self, name: &str) {
        self.gates.remove(name);
        // Update references inside the remaining library gates
        for gate in self.gates.values_mut() {
            gate.graph.remove_all_gates_of_library_name(name);
        }
    }

    pub fn rename_gate(&mut self, old_name: &str, new_name: &str) -> bool {
        if old_name == new_name || new_name.is_empty() || self.gates.contains_key(new_name) {
            return false;
        }

        // Update all SavedGate references in the internal graphs
        for gate in self.gates.values_mut() {
            gate.graph.rename_saved_gate_references(old_name, new_name);
        }

        // Swap the key in the HashMap
        if let Some(mut gate) = self.gates.remove(old_name) {
            gate.name = new_name.to_string();
            self.gates.insert(new_name.to_string(), gate);
            return true;
        }
        false
    }

    // ── Internal Helpers ───────────────────────────────────────────────────────

    fn update_instances_in_internal_graphs(&mut self, gate_name: &str, updated_library_gate: &LibraryGate) {
        for gate in self.gates.values_mut() {
            gate.graph.update_saved_gate_instances(gate_name, updated_library_gate);
        }
    }

    // ── Persistence ────────────────────────────────────────────────────────────

    pub fn save_to_file(&self) -> Result<(), String> {
        let file = std::fs::File::create("my_library.lbl")
            .map_err(|err| format!("Failed to open file on save: {}", err))?;
        bincode::config::DefaultOptions::new()
            .with_limit(10 * 1024 * 1024)
            .serialize_into(file, &self.gates)
            .map_err(|err| format!("Serialize on save failed: {}", err))
    }

    pub fn load_from_file() -> Result<Self, String> {
        let file = std::fs::File::open("my_library.lbl")
            .map_err(|err| format!("Failed to open file on load: {}", err))?;
        let gates: HashMap<String, LibraryGate> = bincode::config::DefaultOptions::new()
            .with_limit(10 * 1024 * 1024)
            .deserialize_from(file)
            .map_err(|err| format!("Deserialize on load failed: {}", err))?;
        Ok(Self { gates })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  App to Library Bridge Methods
// ─────────────────────────────────────────────────────────────────────────────

impl LibraryGate {
    fn from_editor(app: &App) -> Self {
        LibraryGate {
            name:         app.title.clone(),
            input_count:  app.graph.inputs.len(),
            output_count: app.graph.outputs.len(),
            graph:        app.graph.clone(),
        }
    }
}

impl App {
    /// Save the current canvas as a `LibraryGate`.
    pub fn save_current_graph_to_library(&mut self) {
        let new_gate = LibraryGate::from_editor(self);
        let gate_name = new_gate.name.clone();
        let is_update = self.library.contains(&gate_name);
        
        self.library.save(new_gate);

        if is_update {
            let updated_gate = self.library.get(&gate_name).unwrap().clone();
            self.graph.update_saved_gate_instances(&gate_name, &updated_gate);
        }
    }

    pub fn save_library_to_file(&mut self) {
        if let Err(error) = self.library.save_to_file() {
            self.simulation_error = Some(error);
        }
    }

    pub fn load_library_from_file(&mut self) {
        match Library::load_from_file() {
            Ok(library) => self.library = library,
            Err(error)  => self.simulation_error = Some(error),
        }
    }

    /// Load a saved gate back onto the canvas for editing.
    pub fn open_library_gate_for_editing(&mut self, gate_name: &str) {
        let gate = self.library.get(gate_name).cloned().unwrap();
        self.title              = gate.name;
        self.graph              = gate.graph;
        self.input_states       = vec![false; gate.input_count];
        self.output_states      = vec![false; gate.output_count];
        self.sim_runtime         = None;
        self.simulation_error   = None;
        self.simulation_running = false;
        self.pending_wire_start = None;
        self.dragging_gate      = None;
        self.live_wire_signals  = HashMap::new();
        self.port_to_wire_index = HashMap::new();
    }

    /// Delete a library gate by name.
    pub fn delete_library_gate(&mut self, deleted_name: &str) {
        self.graph.remove_all_gates_of_library_name(deleted_name);
        self.library.remove_gate(deleted_name);

        self.library_rename_index = None;
        self.library_rename_text.clear();
    }

    /// Rename a library gate and update all references across all graphs.
    pub fn rename_library_gate(&mut self, old_name: &str, new_name: &str) {
        if old_name == new_name || new_name.is_empty() || self.library.contains(new_name) {
            return;
        }

        self.graph.rename_saved_gate_references(old_name, new_name);
        
        if self.library.rename_gate(old_name, new_name) {
            // Update the visual labels/names on the nodes in the active graph
            let updated_gate = self.library.get(new_name).unwrap().clone();
            self.graph.update_saved_gate_instances(new_name, &updated_gate);
        }
    }
}