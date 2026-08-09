use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct LymphaticState {
    pub lymph_flow: f64,          // L/day equivalent
    pub interstitial_volume: f64, // L
    pub edema_index: f64,         // relative
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("lymph_flow", Constraint::new(1.0, 4.0)),
        ("interstitial_volume", Constraint::new(11.0, 14.5)),
        ("edema_index", Constraint::new(0.0, 1.0)),
    ])
}

impl ObservableBoundary for LymphaticState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("lymph_flow", self.lymph_flow),
            ("interstitial_volume", self.interstitial_volume),
            ("edema_index", self.edema_index),
        ])
    }

    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }

    fn subsystem_name() -> &'static str {
        "lymphatic"
    }
}

fn lymph_pump_augmentation_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.lymphatic.lymph_flow += 0.3;
    next.lymphatic.interstitial_volume = (next.lymphatic.interstitial_volume - 0.1).max(11.0);
    next.lymphatic.edema_index = (next.lymphatic.edema_index - 0.08).max(0.0);
    next
}

pub fn lymph_pump_augmentation() -> RepairOp {
    RepairOp {
        name: "lymph_pump_augmentation",
        applies_to: |v| {
            v.subsystem == "lymphatic"
                && (v.variable == "edema_index" || v.variable == "interstitial_volume")
        },
        apply: lymph_pump_augmentation_apply,
        writes: &[
            "lymphatic.lymph_flow",
            "lymphatic.interstitial_volume",
            "lymphatic.edema_index",
        ],
    }
}

pub struct LymphaticClock;

impl Continuation for LymphaticClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        120.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let micro_leak = next.microcirculation.capillary_leak_index * 0.05;
        let thermal_drive = (next.thermal.core_temp - 37.0).max(0.0) * 0.03;
        let muscle_pump = 0.12 * inputs.exercise_intensity;
        let stochastic = perturbation.sample(next.t, "lymphatic") * 0.01;

        next.lymphatic.interstitial_volume += micro_leak + thermal_drive - muscle_pump;
        next.lymphatic.lymph_flow = (next.lymphatic.lymph_flow + muscle_pump + stochastic).max(0.8);
        next.lymphatic.edema_index = (next.lymphatic.interstitial_volume - 11.0) / 3.5;
        next
    }
}

const OBSERVABLES: [ObservableSpec; 3] = [
    ObservableSpec {
        name: "lymph_flow",
        unit: "L/day",
        admissible_lo: 1.0,
        admissible_hi: 4.0,
    },
    ObservableSpec {
        name: "interstitial_volume",
        unit: "L",
        admissible_lo: 11.0,
        admissible_hi: 14.5,
    },
    ObservableSpec {
        name: "edema_index",
        unit: "relative",
        admissible_lo: 0.0,
        admissible_hi: 1.0,
    },
];

const REPAIRS: [RepairSpec; 1] = [RepairSpec {
    subsystem: "lymphatic",
    name: "lymph_pump_augmentation",
    triggers: &["lymphatic.edema_index", "lymphatic.interstitial_volume"],
    reads: &[
        "microcirculation.capillary_leak_index",
        "musculoskeletal.muscle_workload",
    ],
    writes: &[
        "lymphatic.lymph_flow",
        "lymphatic.interstitial_volume",
        "lymphatic.edema_index",
    ],
}];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "lymphatic",
        clock_interval_seconds: 120.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
