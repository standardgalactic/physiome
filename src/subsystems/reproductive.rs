use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct ReproductiveState {
    pub sex_hormone_index: f64, // relative
    pub cycle_phase: f64,       // normalized 0..1
    pub placental_flow: f64,    // relative
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("sex_hormone_index", Constraint::new(0.5, 1.6)),
        ("cycle_phase", Constraint::new(0.0, 1.0)),
        ("placental_flow", Constraint::new(0.6, 1.4)),
    ])
}

impl ObservableBoundary for ReproductiveState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("sex_hormone_index", self.sex_hormone_index),
            ("cycle_phase", self.cycle_phase),
            ("placental_flow", self.placental_flow),
        ])
    }

    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }

    fn subsystem_name() -> &'static str {
        "reproductive"
    }
}

fn hormone_normalization_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.reproductive.sex_hormone_index = 0.8 * next.reproductive.sex_hormone_index + 0.2 * 1.0;
    next.endocrine.cortisol = (next.endocrine.cortisol - 0.1).max(5.0);
    next
}

fn placental_perfusion_support_apply(
    state: &PhysiologicalState,
    _v: &Violation,
) -> PhysiologicalState {
    let mut next = state.clone();
    next.reproductive.placental_flow += 0.05;
    next.cardiovascular.mean_arterial_pressure += 0.2;
    next
}

pub fn hormone_normalization() -> RepairOp {
    RepairOp {
        name: "hormone_normalization",
        applies_to: |v| v.subsystem == "reproductive" && v.variable == "sex_hormone_index",
        apply: hormone_normalization_apply,
        writes: &["reproductive.sex_hormone_index", "endocrine.cortisol"],
    }
}

pub fn placental_perfusion_support() -> RepairOp {
    RepairOp {
        name: "placental_perfusion_support",
        applies_to: |v| v.subsystem == "reproductive" && v.variable == "placental_flow",
        apply: placental_perfusion_support_apply,
        writes: &[
            "reproductive.placental_flow",
            "cardiovascular.mean_arterial_pressure",
        ],
    }
}

#[deprecated(note = "use placental_perfusion_support")]
pub fn placental_perfursion_support() -> RepairOp {
    placental_perfusion_support()
}

pub struct ReproductiveClock;

impl Continuation for ReproductiveClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        43200.0 // 12h
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        _inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let cyclical =
            ((next.reproductive.cycle_phase * std::f64::consts::TAU).sin() + 1.0) * 0.3 + 0.7;
        let stochastic = perturbation.sample(next.t, "reproductive") * 0.02;

        next.reproductive.cycle_phase = (next.reproductive.cycle_phase + (1.0 / 56.0)).fract();
        next.reproductive.sex_hormone_index = (cyclical + stochastic).clamp(0.4, 1.7);
        next.reproductive.placental_flow = (1.0 + 0.1 * stochastic).clamp(0.6, 1.4);
        next
    }
}

const OBSERVABLES: [ObservableSpec; 3] = [
    ObservableSpec {
        name: "sex_hormone_index",
        unit: "relative",
        admissible_lo: 0.5,
        admissible_hi: 1.6,
    },
    ObservableSpec {
        name: "cycle_phase",
        unit: "normalized",
        admissible_lo: 0.0,
        admissible_hi: 1.0,
    },
    ObservableSpec {
        name: "placental_flow",
        unit: "relative",
        admissible_lo: 0.6,
        admissible_hi: 1.4,
    },
];

const REPAIRS: [RepairSpec; 2] = [
    RepairSpec {
        subsystem: "reproductive",
        name: "hormone_normalization",
        triggers: &["reproductive.sex_hormone_index"],
        reads: &["endocrine.cortisol"],
        writes: &["reproductive.sex_hormone_index", "endocrine.cortisol"],
    },
    RepairSpec {
        subsystem: "reproductive",
        name: "placental_perfusion_support",
        triggers: &["reproductive.placental_flow"],
        reads: &["cardiovascular.mean_arterial_pressure"],
        writes: &[
            "reproductive.placental_flow",
            "cardiovascular.mean_arterial_pressure",
        ],
    },
];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "reproductive",
        clock_interval_seconds: 43200.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
