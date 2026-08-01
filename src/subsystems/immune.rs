use std::collections::HashMap;

use crate::constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
use crate::repair::{Continuation, Inputs, RepairOp};
use crate::state::PhysiologicalState;

#[derive(Clone, Debug, PartialEq)]
pub struct ImmuneState {
    pub wbc_count: f64,       // cells/uL, thousands
    pub cytokine_level: f64,  // relative, 0.0 = baseline
    pub crp: f64,             // mg/L, C-reactive protein
}

pub fn boundary() -> AdmissibilityBoundary {
    HashMap::from([
        ("wbc_count", Constraint::new(4.5, 11.0)),
        ("crp", Constraint::new(0.0, 10.0)),
        // Above baseline: cytokine signaling has moved from "present"
        // to "driving a systemic response." This is what lets fever
        // response trigger off cytokine_level as its own admissibility
        // violation, rather than off some ad hoc threshold check.
        ("cytokine_level", Constraint::new(0.0, 0.3)),
    ])
}

impl ObservableBoundary for ImmuneState {
    fn observables(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("wbc_count", self.wbc_count),
            ("crp", self.crp),
            ("cytokine_level", self.cytokine_level),
        ])
    }
    fn boundary() -> AdmissibilityBoundary {
        boundary()
    }
    fn subsystem_name() -> &'static str {
        "immune"
    }
}

/// Immune resolution — the body's actual corrective response to
/// elevated inflammatory markers: pulls wbc_count, cytokine_level, and
/// crp back toward admissible. This replaces an earlier "leukocytosis"
/// op that was wired backwards — it only ever increased wbc_count, so
/// it never resolved the violation it was supposedly fixing, and fired
/// every tick forever. The actual pathogen-driven rise now lives only
/// in ImmuneClock::advance, below; this op is the counter-force.
fn immune_resolution_apply(state: &PhysiologicalState, v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    match v.variable {
        "wbc_count" => next.immune.wbc_count -= 1.0,
        "cytokine_level" => next.immune.cytokine_level = (next.immune.cytokine_level - 0.04).max(0.0),
        "crp" => next.immune.crp -= 1.0,
        _ => {}
    }
    next
}

pub fn immune_resolution() -> RepairOp {
    RepairOp {
        name: "immune_resolution",
        applies_to: |v| {
            v.subsystem == "immune"
                && (v.variable == "wbc_count" || v.variable == "cytokine_level" || v.variable == "crp")
        },
        apply: immune_resolution_apply,
    }
}

/// Fever response — triggers off an elevated cytokine_level violation
/// (immune's own admissibility boundary), but writes to
/// thermal::core_temp. The trigger is immune's, the effect is
/// thermal's, and the two never call each other directly — everything
/// passes through the shared PhysiologicalState. Clamped at 41.5C: a
/// hard physiological ceiling regardless of how far out of range the
/// triggering cytokine violation is, rather than trusting severity to
/// stay small.
fn fever_response_apply(state: &PhysiologicalState, v: &Violation) -> PhysiologicalState {
    let mut next = state.clone();
    next.thermal.core_temp = (next.thermal.core_temp + 0.15 * v.severity).min(41.5);
    next
}

pub fn fever_response() -> RepairOp {
    RepairOp {
        name: "fever_response",
        applies_to: |v| v.subsystem == "immune" && v.variable == "cytokine_level",
        apply: fever_response_apply,
    }
}

/// Innate immune response mobilizes within minutes; adaptive response
/// (not modeled yet) would run on a much slower, day-scale clock.
pub struct ImmuneClock;

impl Continuation for ImmuneClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        300.0 // 5 min
    }
    fn advance(&self, state: &PhysiologicalState, _dt: f64, inputs: &Inputs) -> PhysiologicalState {
        let mut next = state.clone();
        // The disease process: pathogen_load continuously drives all
        // three inflammatory markers up. immune_resolution (above) is
        // the only counter-force — the dynamic equilibrium between
        // this and resolution is what determines whether the immune
        // response stabilizes or the infection outpaces it.
        next.immune.wbc_count += 0.5 * inputs.pathogen_load;
        next.immune.cytokine_level += 0.02 * inputs.pathogen_load;
        next.immune.crp += 0.4 * inputs.pathogen_load;
        next
    }
}
