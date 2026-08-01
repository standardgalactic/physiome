use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct CardiovascularState {
    pub mean_arterial_pressure: f64, // mmHg
    pub heart_rate: f64,             // bpm
    pub stroke_volume: f64,          // mL
    // cardiac_output is NOT stored: it's a pure function of heart_rate
    // and stroke_volume. Storing it separately would let the two
    // fall out of sync, so it's derived on demand instead.
}

impl CardiovascularState {
    pub fn cardiac_output(&self) -> f64 {
        self.heart_rate * self.stroke_volume / 1000.0
    }
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("mean_arterial_pressure", Constraint::new(70.0, 105.0)),
        ("heart_rate", Constraint::new(50.0, 100.0)),
    ])
}

impl ObservableBoundary for CardiovascularState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("mean_arterial_pressure", self.mean_arterial_pressure),
            ("heart_rate", self.heart_rate),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "cardiovascular"
    }
}

/// Baroreflex-style correction for low mean arterial pressure: raises
/// both chronotropy (heart rate) and contractility (stroke volume);
/// cardiac_output follows automatically since it's derived.
fn baroreflex_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.cardiovascular.heart_rate += 5.0;
    next.cardiovascular.stroke_volume += 2.0;
    next.cardiovascular.mean_arterial_pressure += 2.0;
    next
}

pub fn baroreflex() -> RepairOp {
    RepairOp {
        name: "baroreflex",
        applies_to: |v| v.subsystem == "cardiovascular" && v.variable == "mean_arterial_pressure",
        apply: baroreflex_apply,
    }
}

/// Cardiovascular reflexes are fast — re-evaluate every simulated second.
pub struct CardiovascularClock;

impl Continuation for CardiovascularClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        1.0
    }
    fn advance(&self, state: &PhysiologicalState, _dt: f64, _inputs: &Inputs) -> PhysiologicalState {
        // Passive drift toward baseline in the absence of a violation
        // triggering an active repair op; illustrative only.
        state.clone()
    }
}
