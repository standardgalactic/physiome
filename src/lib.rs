//! physiome: an admissibility/repair-driven, pure-functional whole-body
//! physiology simulator with a domain-independent repair engine and a
//! declarative library of subsystem modules.

pub mod constraint;
pub mod repair;
pub mod specification;
pub mod state;
pub mod subsystems;

pub use constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
pub use repair::{
    step, step_until, step_until_hierarchical, Continuation, ContinuationEntry,
    HierarchicalContinuation, HierarchyLevel, Inputs, Perturbation, RepairEvent, RepairOp,
};
pub use specification::{
    validate_coupling_contracts, CouplingContract, ObservableSpec, RepairSpec,
    SubsystemSpecification,
};
pub use state::{all_violations, PhysiologicalState};

use subsystems::{
    cardiovascular, endocrine, gi, hematologic, hepatic, immune, integumentary, lymphatic,
    metabolic, microcirculation, musculoskeletal, nervous, renal, reproductive, respiratory,
    skeletal, thermal,
};

pub fn all_repair_ops() -> Vec<RepairOp> {
    vec![
        cardiovascular::baroreflex(),
        renal::raas(),
        hepatic::urea_cycle_upregulation(),
        hepatic::synthetic_upregulation(),
        gi::motility_correction(),
        nervous::autonomic_rebalance(),
        immune::immune_resolution(),
        immune::fever_response(),
        hematologic::coagulation_response(),
        hematologic::erythropoiesis(),
        endocrine::insulin_response(),
        metabolic::glucagon_response(),
        respiratory::ventilation_response(),
        thermal::thermoregulation(),
        microcirculation::perfusion_autoregulation(),
        microcirculation::capillary_barrier_repair(),
        lymphatic::lymph_pump_augmentation(),
        musculoskeletal::fatigue_recovery(),
        musculoskeletal::glycogen_repletion(),
        integumentary::barrier_repair(),
        integumentary::evaporative_cooling(),
        skeletal::calcium_buffering(),
        skeletal::marrow_support(),
        reproductive::hormone_normalization(),
        reproductive::placental_perfursion_support(),
    ]
}

pub struct AllContinuations {
    pub cardiovascular: cardiovascular::CardiovascularClock,
    pub renal: renal::RenalClock,
    pub hepatic: hepatic::HepaticClock,
    pub gi: gi::GiClock,
    pub nervous: nervous::NervousClock,
    pub immune: immune::ImmuneClock,
    pub hematologic: hematologic::HematologicClock,
    pub endocrine: endocrine::EndocrineClock,
    pub metabolic: metabolic::MetabolicClock,
    pub respiratory: respiratory::RespiratoryClock,
    pub thermal: thermal::ThermalClock,
    pub microcirculation: microcirculation::MicrocirculationClock,
    pub lymphatic: lymphatic::LymphaticClock,
    pub musculoskeletal: musculoskeletal::MusculoskeletalClock,
    pub integumentary: integumentary::IntegumentaryClock,
    pub skeletal: skeletal::SkeletalClock,
    pub reproductive: reproductive::ReproductiveClock,
}

impl AllContinuations {
    pub fn as_vec(&self) -> Vec<&dyn Continuation> {
        vec![
            &self.cardiovascular,
            &self.renal,
            &self.hepatic,
            &self.gi,
            &self.nervous,
            &self.immune,
            &self.hematologic,
            &self.endocrine,
            &self.metabolic,
            &self.respiratory,
            &self.thermal,
            &self.microcirculation,
            &self.lymphatic,
            &self.musculoskeletal,
            &self.integumentary,
            &self.skeletal,
            &self.reproductive,
        ]
    }
}

pub fn all_continuations() -> AllContinuations {
    AllContinuations {
        cardiovascular: cardiovascular::CardiovascularClock,
        renal: renal::RenalClock,
        hepatic: hepatic::HepaticClock,
        gi: gi::GiClock,
        nervous: nervous::NervousClock,
        immune: immune::ImmuneClock,
        hematologic: hematologic::HematologicClock,
        endocrine: endocrine::EndocrineClock,
        metabolic: metabolic::MetabolicClock,
        respiratory: respiratory::RespiratoryClock,
        thermal: thermal::ThermalClock,
        microcirculation: microcirculation::MicrocirculationClock,
        lymphatic: lymphatic::LymphaticClock,
        musculoskeletal: musculoskeletal::MusculoskeletalClock,
        integumentary: integumentary::IntegumentaryClock,
        skeletal: skeletal::SkeletalClock,
        reproductive: reproductive::ReproductiveClock,
    }
}

pub fn all_subsystem_specifications() -> Vec<SubsystemSpecification> {
    vec![
        microcirculation::specification(),
        lymphatic::specification(),
        musculoskeletal::specification(),
        integumentary::specification(),
        skeletal::specification(),
        reproductive::specification(),
    ]
}

pub fn all_coupling_contracts() -> Vec<CouplingContract> {
    vec![
        CouplingContract {
            subsystem: "microcirculation",
            allowed_reads: &[
                "cardiovascular.mean_arterial_pressure",
                "respiratory.pao2",
                "thermal.core_temp",
                "microcirculation.capillary_leak_index",
            ],
            allowed_writes: &[
                "microcirculation.tissue_perfusion",
                "microcirculation.oxygen_extraction",
                "microcirculation.capillary_leak_index",
                "lymphatic.interstitial_volume",
                "metabolic.lactate",
            ],
        },
        CouplingContract {
            subsystem: "lymphatic",
            allowed_reads: &[
                "microcirculation.capillary_leak_index",
                "musculoskeletal.muscle_workload",
            ],
            allowed_writes: &[
                "lymphatic.lymph_flow",
                "lymphatic.interstitial_volume",
                "lymphatic.edema_index",
            ],
        },
        CouplingContract {
            subsystem: "musculoskeletal",
            allowed_reads: &["metabolic.blood_glucose"],
            allowed_writes: &[
                "musculoskeletal.muscle_workload",
                "musculoskeletal.fatigue_index",
                "musculoskeletal.glycogen_reserve",
                "endocrine.cortisol",
                "metabolic.blood_glucose",
                "cardiovascular.heart_rate",
                "respiratory.paco2",
            ],
        },
        CouplingContract {
            subsystem: "integumentary",
            allowed_reads: &["immune.cytokine_level", "thermal.core_temp"],
            allowed_writes: &[
                "integumentary.barrier_integrity",
                "integumentary.skin_perfusion",
                "integumentary.sweat_rate",
                "immune.crp",
                "thermal.core_temp",
            ],
        },
        CouplingContract {
            subsystem: "skeletal",
            allowed_reads: &["endocrine.aldosterone", "hematologic.hemoglobin"],
            allowed_writes: &[
                "skeletal.ionized_calcium",
                "skeletal.remodeling_signal",
                "skeletal.marrow_output",
                "endocrine.cortisol",
                "hematologic.hemoglobin",
            ],
        },
        CouplingContract {
            subsystem: "reproductive",
            allowed_reads: &[
                "endocrine.cortisol",
                "cardiovascular.mean_arterial_pressure",
            ],
            allowed_writes: &[
                "reproductive.sex_hormone_index",
                "reproductive.cycle_phase",
                "reproductive.placental_flow",
                "endocrine.cortisol",
                "cardiovascular.mean_arterial_pressure",
            ],
        },
    ]
}

pub fn validate_new_subsystem_specs() -> Result<(), String> {
    let repairs: Vec<RepairSpec> = all_subsystem_specifications()
        .into_iter()
        .flat_map(|s| s.repairs.iter().cloned())
        .collect();
    validate_coupling_contracts(&all_coupling_contracts(), &repairs)
}
