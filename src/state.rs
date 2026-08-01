//! PhysiologicalState: struct-of-structs, one block per subsystem's
//! observables. Immutable by convention — transitions produce a new
//! PhysiologicalState rather than mutating an existing one.

use crate::constraint::{detect_violations_for, Violation};
use crate::subsystems::cardiovascular::CardiovascularState;
use crate::subsystems::endocrine::EndocrineState;
use crate::subsystems::gi::GiState;
use crate::subsystems::hematologic::HematologicState;
use crate::subsystems::hepatic::HepaticState;
use crate::subsystems::immune::ImmuneState;
use crate::subsystems::metabolic::MetabolicState;
use crate::subsystems::nervous::NervousState;
use crate::subsystems::renal::RenalState;
use crate::subsystems::respiratory::RespiratoryState;
use crate::subsystems::thermal::ThermalState;

#[derive(Clone, Debug, PartialEq)]
pub struct PhysiologicalState {
    pub cardiovascular: CardiovascularState,
    pub renal: RenalState,
    pub hepatic: HepaticState,
    pub gi: GiState,
    pub nervous: NervousState,
    pub immune: ImmuneState,
    pub hematologic: HematologicState,
    pub endocrine: EndocrineState,
    pub metabolic: MetabolicState,
    pub respiratory: RespiratoryState,
    pub thermal: ThermalState,
    /// Simulation time, in seconds, since t0.
    pub t: f64,
}

/// A normal, admissible resting state across all eleven subsystems —
/// the starting point for scenarios and tests.
impl PhysiologicalState {
    pub fn baseline() -> Self {
        PhysiologicalState {
            cardiovascular: CardiovascularState {
                mean_arterial_pressure: 90.0,
                heart_rate: 70.0,
                stroke_volume: 70.0,
            },
            renal: RenalState {
                gfr: 100.0,
                plasma_sodium: 140.0,
                plasma_volume: 5.0,
            },
            hepatic: HepaticState {
                bilirubin: 0.6,
                albumin: 4.2,
                ammonia: 20.0,
                clotting_factors: 1.0,
            },
            gi: GiState {
                gastric_ph: 2.0,
                motility_index: 1.0,
                luminal_glucose: 0.0,
                absorption_rate: 0.5,
            },
            nervous: NervousState {
                sympathetic_tone: 0.3,
                parasympathetic_tone: 0.4,
                baroreceptor_gain: 1.0,
            },
            immune: ImmuneState {
                wbc_count: 7.0,
                cytokine_level: 0.05,
                crp: 1.0,
            },
            hematologic: HematologicState {
                hemoglobin: 14.5,
                platelet_count: 250.0,
                coagulation_index: 1.0,
            },
            endocrine: EndocrineState {
                insulin: 10.0,
                cortisol: 12.0,
                aldosterone: 10.0,
            },
            metabolic: MetabolicState {
                blood_glucose: 90.0,
                lactate: 1.0,
            },
            respiratory: RespiratoryState {
                blood_ph: 7.40,
                paco2: 40.0,
                pao2: 95.0,
            },
            thermal: ThermalState { core_temp: 37.0 },
            t: 0.0,
        }
    }
}

/// Every subsystem's violations, collected in one call. This is the
/// piece of wiring that has to know about all eleven subsystems by
/// name — everything else in the engine (constraint.rs, repair.rs)
/// stays domain-independent. Pass this as the `violations_after`
/// closure to `repair::step_until`.
pub fn all_violations(state: &PhysiologicalState) -> Vec<Violation> {
    let mut v = Vec::new();
    v.extend(detect_violations_for(&state.cardiovascular));
    v.extend(detect_violations_for(&state.renal));
    v.extend(detect_violations_for(&state.hepatic));
    v.extend(detect_violations_for(&state.gi));
    v.extend(detect_violations_for(&state.nervous));
    v.extend(detect_violations_for(&state.immune));
    v.extend(detect_violations_for(&state.hematologic));
    v.extend(detect_violations_for(&state.endocrine));
    v.extend(detect_violations_for(&state.metabolic));
    v.extend(detect_violations_for(&state.respiratory));
    v.extend(detect_violations_for(&state.thermal));
    v
}
