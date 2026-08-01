use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct MetabolicState {
    pub blood_glucose: f64, // mg/dL
    pub lactate: f64,       // mmol/L
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("blood_glucose", Constraint::new(70.0, 140.0)),
        ("lactate", Constraint::new(0.5, 2.2)),
    ])
}

impl ObservableBoundary for MetabolicState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("blood_glucose", self.blood_glucose),
            ("lactate", self.lactate),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "metabolic"
    }
}

fn glucagon_response_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.metabolic.blood_glucose += 8.0;
    next
}

pub fn glucagon_response() -> RepairOp {
    RepairOp {
        name: "glucagon_response",
        applies_to: |v| v.subsystem == "metabolic" && v.variable == "blood_glucose",
        apply: glucagon_response_apply,
    }
}

pub struct MetabolicClock;

impl Continuation for MetabolicClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        60.0 // 1 min
    }
    fn advance(&self, state: &PhysiologicalState, dt: f64, _inputs: &Inputs) -> PhysiologicalState {
        let mut next = state.clone();
        // GI absorption feeds metabolic glucose here rather than in
        // gi.rs, keeping the cross-subsystem write on the consuming
        // side; see gi::GiClock for the upstream bookkeeping.
        let absorbed = (next.gi.absorption_rate * dt / 60.0).min(next.gi.luminal_glucose);
        next.metabolic.blood_glucose += absorbed * 4.0; // illustrative g -> mg/dL scaling
        next
    }
}
