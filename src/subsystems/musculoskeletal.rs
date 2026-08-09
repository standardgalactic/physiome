use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct MusculoskeletalState {
    pub muscle_workload: f64,  // relative 0..1+
    pub fatigue_index: f64,    // relative 0..1+
    pub glycogen_reserve: f64, // arbitrary reserve units
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("muscle_workload", Constraint::new(0.0, 0.85)),
        ("fatigue_index", Constraint::new(0.0, 0.8)),
        ("glycogen_reserve", Constraint::new(45.0, 120.0)),
    ])
}

impl ObservableBoundary for MusculoskeletalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("muscle_workload", self.muscle_workload),
            ("fatigue_index", self.fatigue_index),
            ("glycogen_reserve", self.glycogen_reserve),
        ])
    }

    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }

    fn subsystem_name() -> &'static str {
        "musculoskeletal"
    }
}

fn fatigue_recovery_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.musculoskeletal.fatigue_index = (next.musculoskeletal.fatigue_index - 0.12).max(0.0);
    next.musculoskeletal.muscle_workload = (next.musculoskeletal.muscle_workload - 0.08).max(0.0);
    next.endocrine.cortisol += 0.2;
    next
}

fn glycogen_repletion_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.musculoskeletal.glycogen_reserve += 5.0;
    next.metabolic.blood_glucose -= 2.0;
    next
}

pub fn fatigue_recovery() -> RepairOp {
    RepairOp {
        name: "fatigue_recovery",
        applies_to: |v| v.subsystem == "musculoskeletal" && v.variable == "fatigue_index",
        apply: fatigue_recovery_apply,
        writes: &[
            "musculoskeletal.fatigue_index",
            "musculoskeletal.muscle_workload",
            "endocrine.cortisol",
        ],
    }
}

pub fn glycogen_repletion() -> RepairOp {
    RepairOp {
        name: "glycogen_repletion",
        applies_to: |v| v.subsystem == "musculoskeletal" && v.variable == "glycogen_reserve",
        apply: glycogen_repletion_apply,
        writes: &[
            "musculoskeletal.glycogen_reserve",
            "metabolic.blood_glucose",
        ],
    }
}

pub struct MusculoskeletalClock;

impl Continuation for MusculoskeletalClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        30.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let signal = inputs.exercise_intensity.clamp(0.0, 1.5);
        let noise = perturbation.sample(next.t + dt, "musculoskeletal") * 0.03;

        next.musculoskeletal.muscle_workload =
            (0.7 * next.musculoskeletal.muscle_workload + 0.3 * signal + noise).max(0.0);
        next.musculoskeletal.fatigue_index += 0.07 * signal;
        next.musculoskeletal.glycogen_reserve -= 2.5 * signal;

        if next.musculoskeletal.muscle_workload > 0.7 {
            next.cardiovascular.heart_rate += 1.2;
            next.respiratory.paco2 += 0.3;
        }
        next
    }
}

const OBSERVABLES: [ObservableSpec; 3] = [
    ObservableSpec {
        name: "muscle_workload",
        unit: "relative",
        admissible_lo: 0.0,
        admissible_hi: 0.85,
    },
    ObservableSpec {
        name: "fatigue_index",
        unit: "relative",
        admissible_lo: 0.0,
        admissible_hi: 0.8,
    },
    ObservableSpec {
        name: "glycogen_reserve",
        unit: "reserve-units",
        admissible_lo: 45.0,
        admissible_hi: 120.0,
    },
];

const REPAIRS: [RepairSpec; 2] = [
    RepairSpec {
        subsystem: "musculoskeletal",
        name: "fatigue_recovery",
        triggers: &["musculoskeletal.fatigue_index"],
        reads: &["musculoskeletal.fatigue_index"],
        writes: &[
            "musculoskeletal.fatigue_index",
            "musculoskeletal.muscle_workload",
            "endocrine.cortisol",
        ],
    },
    RepairSpec {
        subsystem: "musculoskeletal",
        name: "glycogen_repletion",
        triggers: &["musculoskeletal.glycogen_reserve"],
        reads: &["metabolic.blood_glucose"],
        writes: &[
            "musculoskeletal.glycogen_reserve",
            "metabolic.blood_glucose",
        ],
    },
];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "musculoskeletal",
        clock_interval_seconds: 30.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
