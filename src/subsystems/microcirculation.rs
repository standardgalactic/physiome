use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct MicrocirculationState {
    pub tissue_perfusion: f64,     // mL/min/100g
    pub oxygen_extraction: f64,    // fraction
    pub capillary_leak_index: f64, // relative
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("tissue_perfusion", Constraint::new(25.0, 55.0)),
        ("oxygen_extraction", Constraint::new(0.20, 0.45)),
        ("capillary_leak_index", Constraint::new(0.0, 1.2)),
    ])
}

impl ObservableBoundary for MicrocirculationState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("tissue_perfusion", self.tissue_perfusion),
            ("oxygen_extraction", self.oxygen_extraction),
            ("capillary_leak_index", self.capillary_leak_index),
        ])
    }

    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }

    fn subsystem_name() -> &'static str {
        "microcirculation"
    }
}

fn perfusion_autoregulation_apply(
    state: &PhysiologicalState,
    _v: &Violation,
) -> PhysiologicalState {
    let mut next = state.clone();
    if next.microcirculation.tissue_perfusion < 25.0 {
        next.microcirculation.tissue_perfusion += 2.5;
        next.microcirculation.oxygen_extraction =
            (next.microcirculation.oxygen_extraction + 0.01).min(0.45);
    } else {
        next.microcirculation.tissue_perfusion -= 1.5;
    }
    next
}

fn capillary_barrier_repair_apply(
    state: &PhysiologicalState,
    _v: &Violation,
) -> PhysiologicalState {
    let mut next = state.clone();
    next.microcirculation.capillary_leak_index =
        (next.microcirculation.capillary_leak_index - 0.06).max(0.0);
    next.lymphatic.interstitial_volume = (next.lymphatic.interstitial_volume - 0.04).max(11.0);
    next
}

pub fn perfusion_autoregulation() -> RepairOp {
    RepairOp {
        name: "perfusion_autoregulation",
        applies_to: |v| v.subsystem == "microcirculation" && v.variable == "tissue_perfusion",
        apply: perfusion_autoregulation_apply,
        writes: &[
            "microcirculation.tissue_perfusion",
            "microcirculation.oxygen_extraction",
        ],
    }
}

pub fn capillary_barrier_repair() -> RepairOp {
    RepairOp {
        name: "capillary_barrier_repair",
        applies_to: |v| v.subsystem == "microcirculation" && v.variable == "capillary_leak_index",
        apply: capillary_barrier_repair_apply,
        writes: &[
            "microcirculation.capillary_leak_index",
            "lymphatic.interstitial_volume",
        ],
    }
}

pub struct MicrocirculationClock;

impl Continuation for MicrocirculationClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        8.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let pressure_drive = ((next.cardiovascular.mean_arterial_pressure - 90.0) / 90.0) * 8.0;
        let oxygen_supply = ((next.respiratory.pao2 - 90.0) / 90.0) * 0.08;
        let thermal_stress = (next.thermal.core_temp - 37.0).max(0.0) * 0.06;
        let effort_tax = inputs.exercise_intensity * 0.08;
        let noise = perturbation.sample(next.t + dt, "microcirculation") * 0.02;

        next.microcirculation.tissue_perfusion += pressure_drive + (0.01 * dt) + noise;
        next.microcirculation.oxygen_extraction =
            (next.microcirculation.oxygen_extraction + effort_tax - oxygen_supply).clamp(0.15, 0.6);
        next.microcirculation.capillary_leak_index =
            (next.microcirculation.capillary_leak_index + thermal_stress).max(0.0);

        if next.microcirculation.oxygen_extraction > 0.45 {
            next.metabolic.lactate += 0.03;
        }
        next
    }
}

const OBSERVABLES: [ObservableSpec; 3] = [
    ObservableSpec {
        name: "tissue_perfusion",
        unit: "mL/min/100g",
        admissible_lo: 25.0,
        admissible_hi: 55.0,
    },
    ObservableSpec {
        name: "oxygen_extraction",
        unit: "fraction",
        admissible_lo: 0.20,
        admissible_hi: 0.45,
    },
    ObservableSpec {
        name: "capillary_leak_index",
        unit: "relative",
        admissible_lo: 0.0,
        admissible_hi: 1.2,
    },
];

const REPAIRS: [RepairSpec; 2] = [
    RepairSpec {
        subsystem: "microcirculation",
        name: "perfusion_autoregulation",
        triggers: &["microcirculation.tissue_perfusion"],
        reads: &["cardiovascular.mean_arterial_pressure"],
        writes: &[
            "microcirculation.tissue_perfusion",
            "microcirculation.oxygen_extraction",
        ],
    },
    RepairSpec {
        subsystem: "microcirculation",
        name: "capillary_barrier_repair",
        triggers: &["microcirculation.capillary_leak_index"],
        reads: &["microcirculation.capillary_leak_index"],
        writes: &[
            "microcirculation.capillary_leak_index",
            "lymphatic.interstitial_volume",
        ],
    },
];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "microcirculation",
        clock_interval_seconds: 8.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
