use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct NervousState {
    pub sympathetic_tone: f64,     // 0.0-1.0
    pub parasympathetic_tone: f64, // 0.0-1.0
    pub baroreceptor_gain: f64,    // sensitivity multiplier, 1.0 = normal
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("sympathetic_tone", Constraint::new(0.1, 0.7)),
        ("parasympathetic_tone", Constraint::new(0.1, 0.7)),
    ])
}

impl ObservableBoundary for NervousState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("sympathetic_tone", self.sympathetic_tone),
            ("parasympathetic_tone", self.parasympathetic_tone),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "nervous"
    }
}

/// Autonomic rebalancing — this is the subsystem most other repair ops
/// (baroreflex, thermoregulation) implicitly assume is available; here
/// it's made explicit as its own admissibility-governed state rather
/// than an unmodeled global.
fn autonomic_rebalance_apply(state: &PhysiologicalState, v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    if v.variable == "sympathetic_tone" && next.nervous.sympathetic_tone > 0.7 {
        next.nervous.sympathetic_tone -= 0.05;
        next.nervous.parasympathetic_tone += 0.03;
    }
    next
}

pub fn autonomic_rebalance() -> RepairOp {
    RepairOp {
        name: "autonomic_rebalance",
        applies_to: |v| v.subsystem == "nervous",
        apply: autonomic_rebalance_apply,
    }
}

/// Autonomic tone shifts fast — sub-second in reality; modeled here at
/// a coarser but still fast grain relative to hormonal subsystems.
pub struct NervousClock;

impl Continuation for NervousClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        2.0
    }
    fn advance(&self, state: &PhysiologicalState, _dt: f64, inputs: &Inputs) -> PhysiologicalState {
        let mut next = state.clone();
        // Exercise intensity is the clearest direct driver of
        // sympathetic tone among the current Inputs.
        next.nervous.sympathetic_tone =
            (next.nervous.sympathetic_tone + 0.1 * inputs.exercise_intensity).min(1.0);
        next
    }
}
