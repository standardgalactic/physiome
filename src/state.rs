//! PhysiologicalState: struct-of-structs, one block per subsystem's
//! observables. Immutable by convention — transitions produce a new
//! PhysiologicalState rather than mutating an existing one.

use crate::constraint::{
    collect_violations, detect_violations_for, detect_violations_for_stateful, Violation,
};
use crate::subsystems::cardiovascular::CardiovascularState;
use crate::subsystems::endocrine::EndocrineState;
use crate::subsystems::gi::GiState;
use crate::subsystems::hematologic::HematologicState;
use crate::subsystems::hepatic::HepaticState;
use crate::subsystems::immune::ImmuneState;
use crate::subsystems::integumentary::IntegumentaryState;
use crate::subsystems::lymphatic::LymphaticState;
use crate::subsystems::metabolic::MetabolicState;
use crate::subsystems::microcirculation::MicrocirculationState;
use crate::subsystems::musculoskeletal::MusculoskeletalState;
use crate::subsystems::nervous::NervousState;
use crate::subsystems::renal::RenalState;
use crate::subsystems::reproductive::ReproductiveState;
use crate::subsystems::respiratory::RespiratoryState;
use crate::subsystems::skeletal::SkeletalState;
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
    pub microcirculation: MicrocirculationState,
    pub lymphatic: LymphaticState,
    pub musculoskeletal: MusculoskeletalState,
    pub integumentary: IntegumentaryState,
    pub skeletal: SkeletalState,
    pub reproductive: ReproductiveState,
    pub t: f64,
}

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
                functioning_mass: 1.0,
                perceived_perfusion_pressure: 90.0,
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
            microcirculation: MicrocirculationState {
                tissue_perfusion: 38.0,
                oxygen_extraction: 0.30,
                capillary_leak_index: 0.25,
            },
            lymphatic: LymphaticState {
                lymph_flow: 2.2,
                interstitial_volume: 12.2,
                edema_index: 0.35,
            },
            musculoskeletal: MusculoskeletalState {
                muscle_workload: 0.2,
                fatigue_index: 0.2,
                glycogen_reserve: 90.0,
            },
            integumentary: IntegumentaryState {
                barrier_integrity: 0.92,
                skin_perfusion: 0.7,
                sweat_rate: 0.2,
            },
            skeletal: SkeletalState {
                ionized_calcium: 1.22,
                remodeling_signal: 1.0,
                marrow_output: 1.0,
            },
            reproductive: ReproductiveState {
                sex_hormone_index: 1.0,
                cycle_phase: 0.5,
                placental_flow: 1.0,
            },
            t: 0.0,
        }
    }
}

pub fn all_violations(state: &PhysiologicalState) -> Vec<Violation> {
    collect_violations(vec![
        detect_violations_for(&state.cardiovascular),
        detect_violations_for_stateful(&state.renal, &state.renal),
        detect_violations_for(&state.hepatic),
        detect_violations_for(&state.gi),
        detect_violations_for(&state.nervous),
        detect_violations_for(&state.immune),
        detect_violations_for(&state.hematologic),
        detect_violations_for(&state.endocrine),
        detect_violations_for(&state.metabolic),
        detect_violations_for(&state.respiratory),
        detect_violations_for_stateful(&state.thermal, state),
        detect_violations_for(&state.microcirculation),
        detect_violations_for(&state.lymphatic),
        detect_violations_for(&state.musculoskeletal),
        detect_violations_for(&state.integumentary),
        detect_violations_for(&state.skeletal),
        detect_violations_for(&state.reproductive),
    ])
}
