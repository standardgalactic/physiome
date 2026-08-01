use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct HepaticState {
    pub bilirubin: f64,      // mg/dL
    pub albumin: f64,        // g/dL
    pub ammonia: f64,        // umol/L
    pub clotting_factors: f64, // relative synthesis rate, 1.0 = normal
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("bilirubin", Constraint::new(0.1, 1.2)),
        ("albumin", Constraint::new(3.5, 5.0)),
        ("ammonia", Constraint::new(11.0, 35.0)),
    ])
}

impl ObservableBoundary for HepaticState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("bilirubin", self.bilirubin),
            ("albumin", self.albumin),
            ("ammonia", self.ammonia),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "hepatic"
    }
}

/// Increases clearance capacity in response to elevated ammonia —
/// urea-cycle upregulation, illustrative magnitude.
fn urea_cycle_upregulation_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.hepatic.ammonia -= 4.0;
    next
}

pub fn urea_cycle_upregulation() -> RepairOp {
    RepairOp {
        name: "urea_cycle_upregulation",
        applies_to: |v| v.subsystem == "hepatic" && v.variable == "ammonia",
        apply: urea_cycle_upregulation_apply,
    }
}

/// Increases albumin/clotting-factor synthesis when albumin runs low —
/// coupled downstream to hematologic::coagulation via clotting_factors.
fn synthetic_upregulation_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.hepatic.albumin += 0.1;
    next.hepatic.clotting_factors = (next.hepatic.clotting_factors + 0.05).min(1.0);
    next
}

pub fn synthetic_upregulation() -> RepairOp {
    RepairOp {
        name: "synthetic_upregulation",
        applies_to: |v| v.subsystem == "hepatic" && v.variable == "albumin",
        apply: synthetic_upregulation_apply,
    }
}

/// Hepatic clearance and synthesis act over tens of minutes.
pub struct HepaticClock;

impl Continuation for HepaticClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        600.0 // 10 min
    }
    fn advance(&self, state: &PhysiologicalState, _dt: f64, _inputs: &Inputs) -> PhysiologicalState {
        state.clone()
    }
}
