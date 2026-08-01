use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct HematologicState {
    pub hemoglobin: f64,       // g/dL
    pub platelet_count: f64,   // x10^3/uL
    pub coagulation_index: f64, // relative, 1.0 = normal INR-equivalent
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("hemoglobin", Constraint::new(12.0, 17.0)),
        ("platelet_count", Constraint::new(150.0, 400.0)),
        ("coagulation_index", Constraint::new(0.8, 1.2)),
    ])
}

impl ObservableBoundary for HematologicState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("hemoglobin", self.hemoglobin),
            ("platelet_count", self.platelet_count),
            ("coagulation_index", self.coagulation_index),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "hematologic"
    }
}

/// Coagulation cascade response to low coagulation_index. This op
/// reads hepatic::clotting_factors as an upstream admissibility input
/// — a concrete example of one subsystem's repair depending on
/// another's state, mediated only through the shared PhysiologicalState,
/// never through direct subsystem-to-subsystem calls.
fn coagulation_response_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    let hepatic_capacity = next.hepatic.clotting_factors;
    next.hematologic.coagulation_index += 0.05 * hepatic_capacity;
    next.hematologic.platelet_count += 5.0;
    next
}

pub fn coagulation_response() -> RepairOp {
    RepairOp {
        name: "coagulation_response",
        applies_to: |v| v.subsystem == "hematologic" && v.variable == "coagulation_index",
        apply: coagulation_response_apply,
    }
}

/// Erythropoiesis response to low hemoglobin — slow, EPO-mediated;
/// modeled here as a single illustrative increment per advance rather
/// than the multi-day real timescale, to keep the sketch runnable.
fn erythropoiesis_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.hematologic.hemoglobin += 0.1;
    next
}

pub fn erythropoiesis() -> RepairOp {
    RepairOp {
        name: "erythropoiesis",
        applies_to: |v| v.subsystem == "hematologic" && v.variable == "hemoglobin",
        apply: erythropoiesis_apply,
    }
}

/// Platelet/coagulation response is fast (minutes); hemoglobin
/// regulation is slow (days). Both are folded into one clock here for
/// simplicity — splitting them into HematologicFastClock /
/// HematologicSlowClock is a natural refinement once this compiles.
pub struct HematologicClock;

impl Continuation for HematologicClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        300.0 // 5 min
    }
    fn advance(&self, state: &PhysiologicalState, _dt: f64, inputs: &Inputs) -> PhysiologicalState {
        let mut next = state.clone();
        if inputs.hemorrhage_rate > 0.0 {
            next.hematologic.hemoglobin -= 0.02 * inputs.hemorrhage_rate;
            next.hematologic.platelet_count -= 1.0 * inputs.hemorrhage_rate;
        }
        next
    }
}
