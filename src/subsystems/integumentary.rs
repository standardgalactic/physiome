use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct IntegumentaryState {
    pub barrier_integrity: f64, // relative
    pub skin_perfusion: f64,    // relative
    pub sweat_rate: f64,        // relative
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("barrier_integrity", Constraint::new(0.7, 1.0)),
        ("skin_perfusion", Constraint::new(0.3, 1.2)),
        ("sweat_rate", Constraint::new(0.0, 1.0)),
    ])
}

impl ObservableBoundary for IntegumentaryState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("barrier_integrity", self.barrier_integrity),
            ("skin_perfusion", self.skin_perfusion),
            ("sweat_rate", self.sweat_rate),
        ])
    }

    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }

    fn subsystem_name() -> &'static str {
        "integumentary"
    }
}

fn barrier_repair_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.integumentary.barrier_integrity = (next.integumentary.barrier_integrity + 0.05).min(1.0);
    next.immune.crp = (next.immune.crp - 0.2).max(0.0);
    next
}

fn evaporative_cooling_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.integumentary.sweat_rate = (next.integumentary.sweat_rate + 0.08).min(1.0);
    next.thermal.core_temp -= 0.05;
    next
}

pub fn barrier_repair() -> RepairOp {
    RepairOp {
        name: "barrier_repair",
        applies_to: |v| v.subsystem == "integumentary" && v.variable == "barrier_integrity",
        apply: barrier_repair_apply,
        writes: &["integumentary.barrier_integrity", "immune.crp"],
    }
}

pub fn evaporative_cooling() -> RepairOp {
    RepairOp {
        name: "evaporative_cooling",
        applies_to: |v| v.subsystem == "thermal" && v.variable == "core_temp",
        apply: evaporative_cooling_apply,
        writes: &["integumentary.sweat_rate", "thermal.core_temp"],
    }
}

pub struct IntegumentaryClock;

impl Continuation for IntegumentaryClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        45.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let heat = (inputs.ambient_temp - 22.0).max(0.0) * 0.01;
        let immune_cost = next.immune.cytokine_level * 0.02;
        let stochastic = perturbation.sample(next.t + dt, "integumentary") * 0.01;

        next.integumentary.skin_perfusion = (next.integumentary.skin_perfusion + heat
            - 0.03 * next.cardiovascular.mean_arterial_pressure / 100.0)
            .clamp(0.2, 1.5);
        next.integumentary.barrier_integrity =
            (next.integumentary.barrier_integrity - immune_cost + stochastic).clamp(0.6, 1.0);
        if inputs.ambient_temp > 28.0 {
            next.integumentary.sweat_rate = (next.integumentary.sweat_rate + 0.05).min(1.2);
            next.thermal.core_temp -= 0.01;
        }
        next
    }
}

const OBSERVABLES: [ObservableSpec; 3] = [
    ObservableSpec {
        name: "barrier_integrity",
        unit: "relative",
        admissible_lo: 0.7,
        admissible_hi: 1.0,
    },
    ObservableSpec {
        name: "skin_perfusion",
        unit: "relative",
        admissible_lo: 0.3,
        admissible_hi: 1.2,
    },
    ObservableSpec {
        name: "sweat_rate",
        unit: "relative",
        admissible_lo: 0.0,
        admissible_hi: 1.0,
    },
];

const REPAIRS: [RepairSpec; 2] = [
    RepairSpec {
        subsystem: "integumentary",
        name: "barrier_repair",
        triggers: &["integumentary.barrier_integrity"],
        reads: &["immune.cytokine_level"],
        writes: &["integumentary.barrier_integrity", "immune.crp"],
    },
    RepairSpec {
        subsystem: "integumentary",
        name: "evaporative_cooling",
        triggers: &["thermal.core_temp"],
        reads: &["thermal.core_temp"],
        writes: &["integumentary.sweat_rate", "thermal.core_temp"],
    },
];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "integumentary",
        clock_interval_seconds: 45.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
