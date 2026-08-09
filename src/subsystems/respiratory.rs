use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct RespiratoryState {
    pub blood_ph: f64,
    pub paco2: f64, // mmHg
    pub pao2: f64,  // mmHg
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("blood_ph", Constraint::new(7.35, 7.45)),
        ("paco2", Constraint::new(35.0, 45.0)),
        ("pao2", Constraint::new(75.0, 100.0)),
    ])
}

impl ObservableBoundary for RespiratoryState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("blood_ph", self.blood_ph),
            ("paco2", self.paco2),
            ("pao2", self.pao2),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "respiratory"
    }
}

fn ventilation_response_apply(state: &PhysiologicalState, v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    if v.variable == "paco2" && next.respiratory.paco2 > 45.0 {
        next.respiratory.paco2 -= 3.0;
        next.respiratory.blood_ph += 0.02;
    }
    next
}

pub fn ventilation_response() -> RepairOp {
    RepairOp {
        name: "ventilation_response",
        applies_to: |v| v.subsystem == "respiratory" && v.variable == "paco2",
        apply: ventilation_response_apply,
        writes: &["respiratory.paco2", "respiratory.blood_ph"],
    }
}

/// Ventilatory adjustments are the fastest subsystem in the model —
/// breath-by-breath in reality, modeled here at a coarse sub-second grain.
pub struct RespiratoryClock;

impl Continuation for RespiratoryClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        4.0
    }
    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        next.respiratory.paco2 += 0.5 * inputs.exercise_intensity;
        next
    }
}
