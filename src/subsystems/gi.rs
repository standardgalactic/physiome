use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct GiState {
    pub gastric_ph: f64,
    pub motility_index: f64,  // relative rate, 1.0 = normal
    pub luminal_glucose: f64, // g, unabsorbed load in transit
    pub absorption_rate: f64, // g/min, current absorptive capacity
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("gastric_ph", Constraint::new(1.5, 3.5)),
        ("motility_index", Constraint::new(0.6, 1.4)),
    ])
}

impl ObservableBoundary for GiState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("gastric_ph", self.gastric_ph),
            ("motility_index", self.motility_index),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "gi"
    }
}

/// Motility correction — restores transit rate toward normal, which
/// indirectly regulates how fast luminal_glucose becomes available to
/// metabolic::blood_glucose.
fn motility_correction_apply(state: &PhysiologicalState, v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    if next.gi.motility_index < 1.0 {
        next.gi.motility_index += 0.1;
    } else {
        next.gi.motility_index -= 0.1;
    }
    let _ = v;
    next
}

pub fn motility_correction() -> RepairOp {
    RepairOp {
        name: "motility_correction",
        applies_to: |v| v.subsystem == "gi" && v.variable == "motility_index",
        apply: motility_correction_apply,
        writes: &["gi.motility_index"],
    }
}

/// GI transit and absorption act over minutes following a meal.
pub struct GiClock;

impl Continuation for GiClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        300.0 // 5 min
    }
    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        // A meal adds to the luminal load; absorption moves a portion
        // of it out each advance, feeding metabolic::blood_glucose.
        // The actual cross-subsystem write belongs in the repair layer
        // once a GI->metabolic RepairOp is added; this only tracks the
        // GI-local bookkeeping for now.
        next.gi.luminal_glucose += inputs.meal_glucose_load * (dt / 3600.0).min(1.0);
        let absorbed = (next.gi.absorption_rate * dt / 60.0).min(next.gi.luminal_glucose);
        next.gi.luminal_glucose -= absorbed;
        next
    }
}
