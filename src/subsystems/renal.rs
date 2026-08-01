use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct RenalState {
    pub gfr: f64,           // mL/min
    pub plasma_sodium: f64, // mEq/L
    pub plasma_volume: f64, // L
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("gfr", Constraint::new(90.0, 120.0)),
        ("plasma_sodium", Constraint::new(135.0, 145.0)),
    ])
}

impl ObservableBoundary for RenalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("gfr", self.gfr),
            ("plasma_sodium", self.plasma_sodium),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "renal"
    }
}

/// RAAS-style correction for low GFR: retains sodium and volume to
/// support perfusion pressure. Illustrative magnitude.
fn raas_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.renal.plasma_sodium += 1.0;
    next.renal.plasma_volume += 0.05;
    next.renal.gfr += 3.0;
    next
}

pub fn raas() -> RepairOp {
    RepairOp {
        name: "raas",
        applies_to: |v| v.subsystem == "renal" && v.variable == "gfr",
        apply: raas_apply,
    }
}

/// Renal regulation acts over tens of minutes to hours.
pub struct RenalClock;

impl Continuation for RenalClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        900.0 // 15 min
    }
    fn advance(&self, state: &PhysiologicalState, _dt: f64, _inputs: &Inputs) -> PhysiologicalState {
        state.clone()
    }
}
