use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct EndocrineState {
    pub insulin: f64,     // uU/mL
    pub cortisol: f64,    // ug/dL
    pub aldosterone: f64, // ng/dL
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("insulin", Constraint::new(2.6, 24.9)),
        ("cortisol", Constraint::new(5.0, 23.0)),
    ])
}

impl ObservableBoundary for EndocrineState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([("insulin", self.insulin), ("cortisol", self.cortisol)])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "endocrine"
    }
}

fn insulin_response_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.endocrine.insulin += 1.0;
    next.metabolic.blood_glucose -= 5.0;
    next
}

pub fn insulin_response() -> RepairOp {
    RepairOp {
        name: "insulin_response",
        applies_to: |v| v.subsystem == "endocrine" && v.variable == "insulin",
        apply: insulin_response_apply,
        writes: &["endocrine.insulin", "metabolic.blood_glucose"],
    }
}

pub struct EndocrineClock;

impl Continuation for EndocrineClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        120.0 // 2 min
    }
    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        _inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        state.clone()
    }
}
