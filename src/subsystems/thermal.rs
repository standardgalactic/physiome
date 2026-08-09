use std::collections::HashMap;

use crate::constraint::{
    AdmissibilityBoundary, Constraint, ObservableBoundary, StatefulObservableBoundary, Violation,
};
use crate::repair::{Continuation, Inputs, Perturbation, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct ThermalState {
    pub core_temp: f64, // C
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([("core_temp", Constraint::new(36.5, 37.5))])
}

impl ObservableBoundary for ThermalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([("core_temp", self.core_temp)])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "thermal"
    }
}

impl StatefulObservableBoundary<PhysiologicalState> for ThermalState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([("core_temp", self.core_temp)])
    }

    fn boundary_for(state: &PhysiologicalState) -> AdmissibilityBoundary {
        let fever_shift = 0.3 * state.immune.cytokine_level;
        HashMap::from([(
            "core_temp",
            Constraint::new(36.5 + fever_shift, 37.5 + fever_shift),
        )])
    }

    fn subsystem_name() -> &'static str {
        "thermal"
    }
}

/// Thermoregulatory correction. Reads immune::cytokine_level as an
/// upstream input — elevated cytokines raise the effective set point
/// (fever), so the same "low core_temp" violation logic pushes toward
/// a higher admissible target during active inflammation.
fn thermoregulation_apply(state: &PhysiologicalState, v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    let fever_shift = 0.3 * next.immune.cytokine_level;
    if v.variable == "core_temp" && next.thermal.core_temp < 36.5 + fever_shift {
        next.thermal.core_temp += 0.1;
    } else if next.thermal.core_temp > 37.5 + fever_shift {
        next.thermal.core_temp -= 0.1;
    }
    next
}

pub fn thermoregulation() -> RepairOp {
    RepairOp {
        name: "thermoregulation",
        applies_to: |v| v.subsystem == "thermal" && v.variable == "core_temp",
        apply: thermoregulation_apply,
        writes: &["thermal.core_temp"],
    }
}

pub struct ThermalClock;

impl Continuation for ThermalClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        60.0 // 1 min
    }
    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        let mut next = state.clone();
        let ambient_pull = (inputs.ambient_temp - next.thermal.core_temp) * 0.0001 * dt;
        next.thermal.core_temp += ambient_pull;
        next
    }
}
