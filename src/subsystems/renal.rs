use std::collections::HashMap;

use crate::constraint::{
    AdmissibilityBoundary, Constraint, ObservableBoundary, StatefulObservableBoundary, Violation,
};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::specification::{ObservableSpec, RepairSpec, SubsystemSpecification};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct RenalState {
    pub gfr: f64,           // mL/min
    pub plasma_sodium: f64, // mEq/L
    pub plasma_volume: f64, // L
    pub functioning_mass: f64, // 1.0 = two kidneys, 0.5 = unilateral nephrectomy
    pub perceived_perfusion_pressure: f64, // mmHg sensed at renal arterioles
}

pub fn boundary(functioning_mass: f64) -> AdmissibilityBoundary {
    let effective_mass = functioning_mass.clamp(0.0, 1.0);
    HashMap::from([
        (
            "gfr",
            Constraint::new(90.0 * effective_mass, 120.0 * effective_mass),
        ),
        ("plasma_sodium", Constraint::new(135.0, 145.0)),
    ])
}

impl ObservableBoundary for RenalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([("gfr", self.gfr), ("plasma_sodium", self.plasma_sodium)])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary(1.0)
    }
    fn subsystem_name() -> &'static str {
        "renal"
    }
}

impl StatefulObservableBoundary<RenalState> for RenalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([("gfr", self.gfr), ("plasma_sodium", self.plasma_sodium)])
    }

    fn boundary_for(state: &RenalState) -> AdmissibilityBoundary {
        boundary(state.functioning_mass)
    }

    fn subsystem_name() -> &'static str {
        "renal"
    }
}

/// RAAS-style correction for low GFR: retains sodium and volume to
/// support perfusion pressure. Illustrative magnitude.
fn raas_apply(state: &PhysiologicalState, _v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    let perfusion_deficit =
        ((90.0 - next.renal.perceived_perfusion_pressure).max(0.0) / 90.0).min(1.0);
    let aldosterone_drive = (next.endocrine.aldosterone / 10.0).clamp(0.2, 3.0);

    next.renal.plasma_sodium += (0.6 + 0.8 * perfusion_deficit) * aldosterone_drive;
    next.renal.plasma_volume += (0.02 + 0.04 * perfusion_deficit) * aldosterone_drive;
    next.renal.gfr += (1.5 + 2.0 * perfusion_deficit) * next.renal.functioning_mass.max(0.1);
    next.cardiovascular.mean_arterial_pressure += 0.3 * aldosterone_drive;
    next
}

pub fn raas() -> RepairOp {
    RepairOp {
        name: "raas",
        applies_to: |v| v.subsystem == "renal" && v.variable == "gfr",
        apply: raas_apply,
        writes: &[
            "renal.plasma_sodium",
            "renal.plasma_volume",
            "renal.gfr",
            "cardiovascular.mean_arterial_pressure",
        ],
    }
}

/// Renal regulation acts over tens of minutes to hours.
pub struct RenalClock;

impl Continuation for RenalClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        900.0 // 15 min
    }
    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let stenosis_multiplier = (1.0 - inputs.renal_artery_stenosis).clamp(0.05, 1.2);
        next.renal.perceived_perfusion_pressure =
            next.cardiovascular.mean_arterial_pressure * stenosis_multiplier;

        let perfusion_ratio = (next.renal.perceived_perfusion_pressure / 90.0).clamp(0.2, 1.2);
        let target_gfr = 100.0 * next.renal.functioning_mass * perfusion_ratio;
        let adaptation = (dt / 3600.0).clamp(0.0, 1.0);
        next.renal.gfr += (target_gfr - next.renal.gfr) * 0.25 * adaptation;

        next
    }
}

const OBSERVABLES: [ObservableSpec; 2] = [
    ObservableSpec {
        name: "gfr",
        unit: "mL/min",
        admissible_lo: 90.0,
        admissible_hi: 120.0,
    },
    ObservableSpec {
        name: "plasma_sodium",
        unit: "mEq/L",
        admissible_lo: 135.0,
        admissible_hi: 145.0,
    },
];

const REPAIRS: [RepairSpec; 1] = [RepairSpec {
    subsystem: "renal",
    name: "raas",
    triggers: &["renal.gfr"],
    reads: &[
        "renal.perceived_perfusion_pressure",
        "renal.functioning_mass",
        "endocrine.aldosterone",
    ],
    writes: &[
        "renal.plasma_sodium",
        "renal.plasma_volume",
        "renal.gfr",
        "cardiovascular.mean_arterial_pressure",
    ],
}];

pub fn specification() -> SubsystemSpecification {
    SubsystemSpecification {
        subsystem: "renal",
        clock_interval_seconds: 900.0,
        observables: &OBSERVABLES,
        repairs: &REPAIRS,
    }
}
