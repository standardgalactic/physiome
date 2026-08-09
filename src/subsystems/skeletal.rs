use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletalState {
    pub ionized_calcium: f64,   // mmol/L
    pub remodeling_signal: f64, // relative
    pub marrow_output: f64,     // relative
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("ionized_calcium", Constraint::new(1.10, 1.35)),
        ("remodeling_signal", Constraint::new(0.6, 1.4)),
        ("marrow_output", Constraint::new(0.8, 1.3)),
    ])
}

impl ObservableBoundary for SkeletalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("ionized_calcium", self.ionized_calcium),
            ("remodeling_signal", self.remodeling_signal),
            ("marrow_output", self.marrow_output),
        ])
    }

    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }

    fn subsystem_name() -> &'static str {
        "skeletal"
    }
}

fn calcium_buffering_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.skeletal.ionized_calcium += 0.03;
    next.endocrine.cortisol = (next.endocrine.cortisol - 0.2).max(5.0);
    next
}

fn marrow_support_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.skeletal.marrow_output += 0.03;
    next.hematologic.hemoglobin += 0.05;
    next
}

pub fn calcium_buffering() -> RepairOp {
    RepairOp {
        name: "calcium_buffering",
        applies_to: |v| v.subsystem == "skeletal" && v.variable == "ionized_calcium",
        apply: calcium_buffering_apply,
        writes: &["skeletal.ionized_calcium", "endocrine.cortisol"],
    }
}

pub fn marrow_support() -> RepairOp {
    RepairOp {
        name: "marrow_support",
        applies_to: |v| v.subsystem == "skeletal" && v.variable == "marrow_output",
        apply: marrow_support_apply,
        writes: &["skeletal.marrow_output", "hematologic.hemoglobin"],
    }
}

pub struct SkeletalClock;

impl Continuation for SkeletalClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        3600.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let load = inputs.exercise_intensity * 0.08;
        let endocrine_drive = (next.endocrine.aldosterone / 10.0) * 0.01;
        let stochastic = perturbation.sample(next.t, "skeletal") * 0.005;

        next.skeletal.remodeling_signal =
            (next.skeletal.remodeling_signal + load + endocrine_drive + stochastic).clamp(0.5, 1.6);
        next.skeletal.ionized_calcium = (next.skeletal.ionized_calcium
            - 0.005 * next.skeletal.remodeling_signal)
            .clamp(1.0, 1.4);
        next.skeletal.marrow_output = (next.skeletal.marrow_output
            + 0.01 * (1.2 - next.skeletal.remodeling_signal))
            .clamp(0.7, 1.4);
        next
    }
}

const OBSERVABLES: [ObservableSpec; 3] = [
    ObservableSpec {
        name: "ionized_calcium",
        unit: "mmol/L",
        admissible_lo: 1.10,
        admissible_hi: 1.35,
    },
    ObservableSpec {
        name: "remodeling_signal",
        unit: "relative",
        admissible_lo: 0.6,
        admissible_hi: 1.4,
    },
    ObservableSpec {
        name: "marrow_output",
        unit: "relative",
        admissible_lo: 0.8,
        admissible_hi: 1.3,
    },
];

const REPAIRS: [RepairSpec; 2] = [
    RepairSpec {
        subsystem: "skeletal",
        name: "calcium_buffering",
        triggers: &["skeletal.ionized_calcium"],
        reads: &["endocrine.aldosterone"],
        writes: &["skeletal.ionized_calcium", "endocrine.cortisol"],
    },
    RepairSpec {
        subsystem: "skeletal",
        name: "marrow_support",
        triggers: &["skeletal.marrow_output"],
        reads: &["hematologic.hemoglobin"],
        writes: &["skeletal.marrow_output", "hematologic.hemoglobin"],
    },
];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "skeletal",
        clock_interval_seconds: 3600.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
